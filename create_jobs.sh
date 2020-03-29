#! /usr/bin/env nix-shell
#! nix-shell -i bash -p curl bash ffmpeg-full

set -euo pipefail
IFS=$'\n\t'

base_url="$2"

upload() { for f; do echo $(curl -#Sf -F "files[]=@$f" https://pomf.dodsorf.as/upload\?output\=text); done }

files=`find $1 -name "*.mkv" -type f`

for file in $files; do
	file_url=`upload $file`
	file_name=`basename $file .mkv`
	length=`ffprobe -v error -count_frames -select_streams v:0 -show_entries stream=nb_read_frames -of default=nokey=1:noprint_wrappers=1 $file`

	curl "$base_url"/add_job -X POST -H "Content-Type: application/json" -d \
	'
    {
        "file_url": "'$file_url'",
        "file_name": "'$file_name'",
        "priority": 0,
        "length": '$length',
        "resolution": [1080, 1920],
        "options": {
            "aomenc": "--lag-in-frames=25 --tile-columns=0 --tile-rows=0 --enable-fwd-kf=1 --bit-depth=10 --cpu-used=0 --end-usage=vbr --target-bitrate=60 --kf-min-dist=9999 --kf-max-dist=9999",
            "ffmpeg": "",
            "two_pass": true
        }
    }
	' 
done

