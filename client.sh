#! /usr/bin/env nix-shell
#! nix-shell -i bash -p bash curl jq libaom ffmpeg-full

set -euo pipefail
IFS=$'\n\t'

base_url="http://localhost:8000"
version="0.1.0"

while true; do
    sleep 1
    upsteam_version=`curl -s "$base_url"/version`
    if [[ $version != $upsteam_version ]]; then
        break
    fi

    job=`curl -s "$base_url"/request_job | jq`
    if [[ $job = "null" ]]; then
        echo "No Jobs Available ¯\_(ツ)_/¯"
        continue
    fi

    echo "Got new job!"
    echo "$job" | jq

    job_id=`echo "$job" | jq -r .id`

    echo "Reserving Job"
    curl -s "$base_url"/edit_status/"$job_id"/reserved
    echo ""

    source=`echo $job | jq -r .description.file_url`
    sourceext=${source##*.}
    echo "Downloading source file: $source"
    
    source=`echo $job | jq -r .description.file_url`

    name=`echo $job | jq -r .description.file_name`
    input="$name.$job_id.$sourceext"

    curl "$source" -o "$input"
    echo ""

    echo "Starting Encode"

    target_bitrate=`echo $job | jq -r .description.options.mode.VBR`
    width=`echo $job | jq -r .description.options.resolution.width`
    height=`echo $job | jq -r .description.options.resolution.height`
    color_depth=`echo $job | jq -r .description.options.color_depth`
    kf_min_dist=`echo $job | jq -r .description.options.kf_min_dist`
    kf_max_dist=`echo $job | jq -r .description.options.kf_max_dist`

    speed=`echo $job | jq -r .description.options.speed`

    ffmpeg -i "$input" -vf scale=$width:$height -f yuv4mpegpipe - | aomenc - --lag-in-frames=25 --tile-columns=0 --tile-rows=0 --enable-fwd-kf=1 \
        --target-bitrate=$target_bitrate --width="$width" --height="$height" --bit-depth=$color_depth --kf-min-dist=$kf_min_dist --kf-max-dist=$kf_min_dist \
        --cpu-used=$speed \
        --pass=1 --passes=2 --fpf="$input.$target_bitrate.$width.$height.$color_depth.fpf" --webm -o "$input.$target_bitrate.$width.$height.$color_depth.webm"

    ffmpeg -i "$input" -vf scale=$width:$height -f yuv4mpegpipe - | aomenc - --lag-in-frames=25 --tile-columns=0 --tile-rows=0 --enable-fwd-kf=1 \
        --target-bitrate=$target_bitrate --width="$width" --height="$height" --bit-depth=$color_depth --kf-min-dist=$kf_min_dist --kf-max-dist=$kf_min_dist \
        --cpu-used=$speed \
        --pass=2 --passes=2 --fpf="$input.$target_bitrate.$width.$height.$color_depth.fpf" --webm -o "$input.$target_bitrate.$width.$height.$color_depth.webm"

done