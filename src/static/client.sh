#! /usr/bin/env nix-shell
#! nix-shell -i bash -p bash curl jq libaom ffmpeg-full

set -euo pipefail
IFS=$'\n\t'

base_url="$1"
version="0.2.0"

while true; do
    sleep 30
    set +e
    upstream_version=`curl -s "$base_url"/version`
    retval=$?
    set -e
    if [ $retval -ne 0 ]; then
        echo "Is the Job Server Down?"
        continue
    fi
    if [[ $version != $upstream_version ]]; then
        echo "Wrong version: client version is $version, while job server requires $upstream_version"
        break
    fi

    set +e
    job=`curl -s "$base_url"/request_job | jq`
    retval=$?
    set -e
    if [[ $job = "null" ]] || [ $retval -ne 0 ]; then
        echo "No Jobs Available ¯\_(ツ)_/¯"
        continue
    fi

    echo "Got new job!"
    printf "%s\n" "$job" | jq

    job_id=`printf "%s\n" "$job" | jq -r .id`

    echo "Reserving Job"
    set +e
    curl -s "$base_url"/edit_status/"$job_id"/reserved
    retval=$?
    set -e
    if [ $retval -ne 0 ]; then
        echo "Is the Job Server Down?"
        continue
    fi
    echo "Reserved!"

    source=`printf "%s\n" "$job" | jq -r .description.file_url`
    sourceext=${source##*.}
    echo "Downloading source file: $source"
    
    source=`printf "%s\n" "$job" | jq -r .description.file_url`

    name=`printf "%s\n" "$job" | jq -r .description.file_name`
    input="$name.$job_id.$sourceext"

    set +e
    curl "$source" -o "$input"
    retval=$?
    set -e
    if [ $retval -ne 0 ]; then
        echo "Could not Download file!"
        curl -s -L "$base_url"/edit_status/"$job_id"/error || true
        echo ""
        continue
    fi

    echo ""

    echo "Starting Encode"

    height=`printf "%s\n" $job | jq -r .description.resolution[0]`
    width=`printf "%s\n" $job | jq -r .description.resolution[1]`

    printf "%s\n" "$job" | jq

    aomenco=`printf "%s\n" "$job" | jq -r .description.options.aomenc`
    aomenco=${aomenco//[^a-zA-Z0-9_\- =]/}
    ffmpego=`printf "%s\n" "$job" | jq -r .description.options.ffmpeg`
    ffmpego=${ffmpego//[^a-zA-Z0-9_\- =:]/}

    two_pass=`printf "%s\n" "$job" | jq -r .description.options.two_pass`

    if [[ $two_pass = true ]]; then
        set +e
        eval 'ffmpeg -nostats -hide_banner -loglevel warning \
        -i "'$input'" '$ffmpego' -vf scale='$height':'$width' -pix_fmt yuv422p -f yuv4mpegpipe - | aomenc - --i422 '$aomenco' \
        --pass=1 --passes=2 --fpf="'$input'.fpf" --webm -o "'$input'.out.webm"'

        retval=$?
        if [ $retval -ne 0 ]; then
            echo "Error running encode pass 1"
            curl -s -L "$base_url"/edit_status/"$job_id"/error || true
            echo ""
            continue
        fi

        eval 'ffmpeg -nostats -hide_banner -loglevel warning \
        -i "'$input'" '$ffmpego' -vf scale='$height':'$width' -pix_fmt yuv422p -f yuv4mpegpipe - | aomenc - --i422 '$aomenco' \
        --pass=2 --passes=2 --fpf="'$input'.fpf" --webm -o "'$input'.out.webm"'

        retval=$?
        if [ $retval -ne 0 ]; then
            echo "Error running encode pass 2"
            curl -s -L "$base_url"/edit_status/"$job_id"/error || true
            echo ""
            continue
        fi
        set -e

    else
        set +e
        eval 'ffmpeg -nostats -hide_banner -loglevel warning \
        -i "'$input'" '$ffmpego' -vf scale='$height':'$width' -pix_fmt yuv422p -f yuv4mpegpipe - | aomenc - --i422 '$aomenco' \
        --passes=1 --fpf="'$input'.fpf" --webm -o "'$input'.out.webm"'

        retval=$?
        if [ $retval -ne 0 ]; then
            echo "Error running encode"
            curl -s -L "$base_url"/edit_status/"$job_id"/error || true
            echo ""
            continue
        fi
        set -e
    fi

    set +e
    curl -s -L "$base_url"/edit_status/"$job_id"/completed
    set -e

done