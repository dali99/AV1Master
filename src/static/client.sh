#! /usr/bin/env nix-shell
#! nix-shell -i bash -p bash curl jq libaom ffmpeg-full

set -euo pipefail
IFS=$'\n\t'

base_url="$1"
version="0.4.0"

while true; do
    sleep 30
    set +e
    upsteam_version=`curl -s "$base_url"/version`
    retval=$?
    set -e
    if [ $retval -ne 0 ]; then
        echo "Is the Job Server Down?"
        continue
    fi
    if [[ $version != $upsteam_version ]]; then
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
    echo "$job" | jq

    job_id=`echo "$job" | jq -r .id`

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

    source=`echo $job | jq -r .description.file_url`
    sourceext=${source##*.}
    echo "Downloading source file: $source"
    
    source=`echo $job | jq -r .description.file_url`

    name=`echo $job | jq -r .description.file_name`
    input="$name.$job_id.$sourceext"

    set +e
    curl "$source" -o "$input"
    retval=$?
    set -e
    if [ $retval -ne 0 ]; then
        echo "Could not Download file!"
        curl -s -L "$base_url"/edit_status/"$job_id"/error || true
        continue
    fi

    echo ""

    echo "Starting Encode"

    height=`echo $job | jq -r .description.resolution[0]`
    width=`echo $job | jq -r .description.resolution[1]`

    echo $job | jq

    aomenco=`echo $job | jq -r .description.options.aomenc`
    aomenco=${aomenco//[^a-zA-Z0-9_\- =]/}
    ffmpego=`echo $job | jq -r .description.options.ffmpeg`
    ffmpego=${ffmpego//[^a-zA-Z0-9_\- =:]/}

    two_pass=`echo $job | jq -r .description.options.two_pass`

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

        echo "Deleting Source and Temporary files"
        rm "$input" "$input".fpf

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

        echo "Deleting Source"
        rm "$input"
    fi

    set +e
    curl -s -L "$base_url"/edit_status/"$job_id"/completed
    set -e

    
    echo "Uploading file!"

    set +e
    curl --data-binary @"$input".out.webm "$base_url"/upload/"$job_id"
    set -e
    retval=$?
    echo ""
    if [ $retval -ne 0 ]; then
        echo "Couldn't upload file!"
        continue
    else
        echo "Upload finished, deleting result!"
        rm "$input".out.webm
    fi


done