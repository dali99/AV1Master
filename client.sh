#! /usr/bin/env nix-shell
#! nix-shell -i bash -p bash curl jq libaom ffmpeg-full

set -euo pipefail
IFS=$'\n\t'

base_url="$1"
version="0.2.0"

while true; do
    sleep 30
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

    aomenco=`echo $job | jq -r .description.options.aomenc`
    aomenco=${aomenco//[^a-zA-Z0-9_\- =]/}
    ffmpego=`echo $job | jq -r .description.options.ffmpeg`
    ffmpego=${ffmpego//[^a-zA-Z0-9_\- =:]/}

    two_pass=`echo $job | jq -r .description.options.two_pass`

    echo $two_pass

    if [[ $two_pass = true ]]; then
        eval 'ffmpeg -nostats -hide_banner -loglevel warning \
        -i "'$input'" '$ffmpego' -pix_fmt yuv444p -f yuv4mpegpipe - | aomenc - --i444 '$aomenco' \
        --pass=1 --passes=2 --fpf="'$input'.fpf" --webm -o "'$input'.out.webm"'

        eval 'ffmpeg -nostats -hide_banner -loglevel warning \
            -i "'$input'" '$ffmpego' -pix_fmt yuv444p -f yuv4mpegpipe - | aomenc - --i444 '$aomenco' \
            --pass=2 --passes=2 --fpf="'$input.fpf'" --webm -o "'$input'.out.webm"'
    else
        eval 'ffmpeg -nostats -hide_banner -loglevel warning \
            -i "'$input'" '$ffmpego' -pix_fmt yuv444p -f yuv4mpegpipe - | aomenc - --i444 '$aomenco' \
            --passes=1 --fpf="'$input.fpf'" --webm -o "'$input.out.webm'"'
    fi

    curl -s -L "$base_url"/edit_status/"$job_id"/completed

done