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
use std::net::SocketAddr;

mod workunit;
use workunit::WUnit;
use workunit::EStatus;

const VERSION: &str = "0.2.0";

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

base_url="$1"
version="0.2.0"

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
"#
}

#[get("/version")]
fn version() -> &'static str {
    VERSION
}

#[get("/get_jobs")]
fn get_jobs(shared: State<SharedState>) -> Json<Value> {
    let list = shared.jobs.lock().unwrap().clone();

    println!("{:#?}", list);

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
fn edit_status(id: Uuid, status: String, shared: State<SharedState>, remote_addr: SocketAddr) -> Result<String, Box<std::error::Error>> {
    let mut list = shared.jobs.lock().unwrap();
    let job = list.get_mut(&id).ok_or("what")?;
    let status = match status.as_str() {
        "queued" => Ok(EStatus::Queued),
        "reserved" => Ok(EStatus::Reserved(remote_addr.to_string())),
        "completed" => Ok(EStatus::Completed(remote_addr.to_string())),
        "error" => Ok(EStatus::Error(remote_addr.to_string())),
        _ => Err("Not a valid status, valid statuses are queued, reserved, completed, and error")
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