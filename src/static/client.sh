#! /usr/bin/env bash

set -euo pipefail
IFS=$'\n\t'

base_url="https://av1.dodsorf.as"
version="0.9.0"

while true; do
    sleep 30
    set +e
    upstream_version=`curl -L -f -s "$base_url"/version`
    retval=$?
    set -e
    if [ $retval -ne 0 ]; then
        echo "Error: $retval"
        echo ""
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

    pix_fmt=`echo $job | jq -r .description.options.pix_fmt`
    if [[ $pix_fmt = "YV12" ]]; then
        ffpix="yuv12p"
        aompix="--yv12"
    elif [[ $pix_fmt = "I420" ]]; then
        ffpix="yuv420p"
        aompix="--i420"
    elif [[ $pix_fmt = "I422" ]]; then
        ffpix="yuv422p"
        aompix="--i422"
    elif [[ $pix_fmt = "I444" ]]; then
        ffpix="yuv444p"
        aompix="--i444"
    fi



    fps=`echo $job | jq -r .description.options.fps`
    if [[ $fps = "null" ]]; then
        fffps=""
        aomfps=""
    else
        fpsrate=`echo $fps | jq -r '.[0]'`
        fpsscale=`echo $fps | jq -r '.[1]'`
        fpsv="$fpsrate/$fpsscale"
        fffps="fps=fps=$fpsv -r $fpsv"
        aomfps="--fps=$fpsv"
    fi


    two_pass=`echo $job | jq -r .description.options.two_pass`

    if [[ $two_pass = true ]]; then
        set +e
        echo 'ffmpeg -nostats -hide_banner -loglevel warning \
        -i "'$input'" '$ffmpego' -vf scale='$width':'$height','$fffps' -pix_fmt '$ffpix' -f yuv4mpegpipe - | aomenc - '$aomfps' '$aompix' '$aomenco' \
        --pass=1 --passes=2 --fpf="'$input'.fpf" -o "'$input'.out.ivf"'

        retval=$?
        if [ $retval -ne 0 ]; then
            echo "Error running encode pass 1"
            curl -s -L "$base_url"/edit_status/"$job_id"/error || true
            echo ""
            continue
        fi

        echo 'ffmpeg -nostats -hide_banner -loglevel warning \
        -i "'$input'" '$ffmpego' -vf scale='$width':'$height','$fffps' -pix_fmt '$ffpix' -f yuv4mpegpipe - | aomenc - '$aomfps' '$aompix' '$aomenco' \
        --pass=2 --passes=2 --fpf="'$input'.fpf" -o "'$input'.out.ivf"'

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
        -i "'$input'" '$ffmpego' -vf scale='$width':'$height','$fffps' -pix_fmt '$ffpix' -f yuv4mpegpipe - | aomenc - '$aomfps' '$aompix' '$aomenco' \
        --passes=1 --fpf="'$input'.fpf" -o "'$input'.out.ivf"'

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
    curl --data-binary @"$input".out.ivf "$base_url"/upload/"$job_id"
    set -e
    retval=$?
    echo ""
    if [ $retval -ne 0 ]; then
        echo "Couldn't upload file!"
        continue
    else
        echo "Upload finished, deleting result!"
        rm "$input".out.ivf
        echo ""
    fi
done