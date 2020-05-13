#! /usr/bin/env nix-shell
#! nix-shell -i bash -p bash curl jq

base_url="http://localhost:8000"

curl "$base_url"/version
curl "$base_url"

curl "$base_url"/get_jobs | jq

curl "$base_url"/add_job/a -X POST -H "Content-Type: application/json" -d \
'
    {
        "file_url": "https://pomf.dodsorf.as/f/g91y5j.mkv",
        "file_name": "014",
        "priority": 15,
        "length": 90,
        "resolution": [540, 960],
        "options": {
            "FFMPEG": {
                "two_pass": true,
                "crf": 45,
                "b_v": "0",
                "lag_in_frames": 35,
                "gop": null,
                "pix_fmt": "yuv420p10le",
                "tiles": "1x1",
                "speed": 4
            }
        }
    }
'

curl "$base_url"/request_job | jq
job=`curl "$base_url"/request_job | jq -r .id`

curl "$base_url"/get_job/"$job" | jq

curl "$base_url"/edit_status/"$job"/reserved
curl "$base_url"/edit_status/"$job"/queued

curl "$base_url"/get_job/"$job" | jq

curl "$base_url"/request_job | jq