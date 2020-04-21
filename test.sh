#! /usr/bin/env nix-shell
#! nix-shell -i bash -p bash curl jq

base_url="http://localhost:8000"

curl "$base_url"/version
curl "$base_url"

curl "$base_url"/get_jobs | jq

curl "$base_url"/add_job -X POST -H "Content-Type: application/json" -d \
'
    {
        "file_url": "https://pomf.dodsorf.as/f/vz9dtl.mkv",
        "file_name": "014",
        "priority": 0,
        "length": 20,
        "resolution": [540, 960],
        "options": {
            "aomenc": "--lag-in-frames=25 --tile-columns=0 --tile-rows=0 --enable-fwd-kf=1 --bit-depth=10 --cpu-used=0 --end-usage=vbr --target-bitrate=60 --kf-min-dist=9999 --kf-max-dist=9999",
            "ffmpeg": "",
            "two_pass": true,
            "pix_fmt": "YV12"
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