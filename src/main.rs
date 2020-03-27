#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
use rocket::State;
use rocket::response::status::NotFound;

use rocket_contrib::json::Json;
use serde_json::Value;
use serde_json::json;
use serde::{Serialize, Deserialize};
use rocket_contrib::uuid::Uuid;

use std::sync::Mutex;
use std::collections::HashMap;

mod workunit;
use workunit::WUnit;
use workunit::EStatus;

const VERSION: &str = "0.1.0";

#[derive(Default, Debug)]
struct SharedState {
    jobs: Mutex<HashMap<uuid::Uuid, WUnit>>
}

#[get("/")]
fn index() -> &'static str {
r#"#! /usr/bin/env nix-shell
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
"#
}

#[get("/version")]
fn version() -> &'static str {
    "0.1.0"
}

#[get("/get_jobs")]
fn get_jobs(shared: State<SharedState>) -> Json<Value> {
    let list = shared.jobs.lock().unwrap().clone();

    println!("{:#?}", list);

    //Json(json!("god hlep me"))
    Json(serde_json::to_value(&list).unwrap())
}

#[get("/request_job")]
fn request_job(shared: State<SharedState>) -> Result<Json<Value>, NotFound<String>> {
    let mut list: Vec<WUnit> = shared.jobs.lock().unwrap()
        .values()
        .filter(|x| x.status == EStatus::Queued).cloned()
        .collect();

    list.sort_by(|a, b| b.description.length.cmp(&a.description.length));

    let job = list.get(0);

    Ok(Json(serde_json::to_value(&job).unwrap()))
}

#[get("/get_job/<id>")]
fn get_job(id: Uuid, shared: State<SharedState>) -> Result<Json<Value>, NotFound<String>> {
    let list = shared.jobs.lock().unwrap();

    let job = list.get(&id).ok_or(NotFound(format!("Job not Found: {id}", id = id)));

    match job {
        Ok(j) => Ok(Json(serde_json::to_value(&j).unwrap())),
        Err(e) => Err(e)
    }
}

#[get("/edit_status/<id>/<status>")]
fn edit_status(id: Uuid, status: String, shared: State<SharedState>) -> Result<String, Box<std::error::Error>> {
    let mut list = shared.jobs.lock().unwrap();
    let job = list.get_mut(&id).ok_or("what")?;
    let status = match status.as_str() {
        "queued" => Ok(EStatus::Queued),
        "reserved" => Ok(EStatus::Reserved),
        "completed" => Ok(EStatus::Completed),
        _ => Err("Not a valid status, valid statuses are queued, reserved, completed")
    }?;

    job.status = status;

    Ok("Status changed".to_string())
}

#[post("/add_job", format = "json", data = "<message>")]
fn add_job(message: Json<workunit::WDesc>, shared: State<SharedState>) {
    println!("{:#?}", message);
    let job = message.into_inner();

    let id = uuid::Uuid::new_v4();

    shared.jobs.lock().unwrap().insert(id, WUnit::new(id, job));
}

fn main() {
        rocket::ignite()
        .manage(SharedState::default())
        .mount("/", routes![index, version, get_jobs, get_job, request_job, edit_status, add_job])
        .launch();
}