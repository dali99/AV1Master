#! /usr/bin/env nix-shell
#! nix-shell -i bash -p bash curl jq

base_url="http://localhost:8000"

curl "$base_url"/version
curl "$base_url"

curl "$base_url"/get_jobs | jq

curl "$base_url"/add_job -X POST -H "Content-Type: application/json" -d \
'
    {
        "file_url": "https://pomf.dodsorf.as/f/38ez7v.mkv",
        "file_name": "014",
        "priority": 0,
        "length": 15,
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

curl "$base_url"/request_job | jq
job=`curl "$base_url"/request_job | jq -r .id`

curl "$base_url"/get_job/"$job" | jq

curl "$base_url"/edit_status/"$job"/reserved
curl "$base_url"/edit_status/"$job"/queued

curl "$base_url"/get_job/"$job" | jq

curl "$base_url"/request_job | jq