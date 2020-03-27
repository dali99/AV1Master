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
		"options": {
		    "mode": { "VBR": 33 },
		    "color_depth": 10,
		    "enable_fwd_keyframe": true,
		    "two_pass": true,
		    "speed": 0,
		    "resolution": {
		        "width": 960,
		        "height": 540
		    },
		    "kf_min_dist": 9999,
		    "kf_max_dist": 9999
		}
	    }
	' 
done

