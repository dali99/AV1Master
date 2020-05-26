#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use serde::{Serialize, Deserialize};

use rocket::State;
use rocket::response::status::NotFound;
use rocket::Data;

use rocket_contrib::json::Json;
use serde_json::Value;
use serde_json::json;
use rocket_contrib::uuid::Uuid;
use rocket_contrib::serve::StaticFiles;

use std::sync::Mutex;
use std::collections::HashMap;
use std::path::Path;

mod workunit;
use workunit::WUnit;
use workunit::EStatus;

const VERSION: &str = "0.12.0";

#[derive(Default, Debug)]
struct SharedState {
    jobs: Mutex<HashMap<uuid::Uuid, WUnit>>
}

#[get("/")]
fn index() -> String {
format!("Wecome to the AV1Master Server version {version}\n
This currently requires a distro with CAP_SYS_USER_NS enabled and correct permissions
curl -L {baseurl}/av1client > av1client && chmod +x ./av1client && ./av1client {baseurl}", baseurl="https://av1.dodsorf.as", version=VERSION)
}

#[get("/version")]
fn version() -> &'static str {
    VERSION
}

#[get("/stats")]
fn get_stats(shared: State<SharedState>) -> Json<Value> {

    #[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
    struct Stats {
        queued: u32,
        progress: u32,
        completed: u32,
        cancelled: u32,
        error: u32,
        length: u32
    };

    let list = shared.jobs.lock().unwrap().clone();
    let mut stats: Stats = Stats {
        queued: 0,
        progress: 0,
        completed: 0,
        cancelled: 0,
        error: 0,
        length: 999
    };

    for job in list.values() {
        match &job.status {
            EStatus::Queued => stats.queued += 1,
            EStatus::Reserved(a) => stats.progress += 1,
            EStatus::Completed(a) => stats.completed += 1,
            EStatus::Cancelled => stats.cancelled += 1,
            EStatus::Error(a) => stats.error += 1
        };
    }

    Json(serde_json::to_value(&stats).unwrap())
}

#[get("/get_jobs")]
fn get_jobs(shared: State<SharedState>) -> Json<Value> {
    let list = shared.jobs.lock().unwrap().clone();
    Json(serde_json::to_value(&list).unwrap())
}

#[get("/request_job")]
fn request_job(shared: State<SharedState>) -> Result<Json<Value>, NotFound<String>> {
    let mut list: Vec<WUnit> = shared.jobs.lock().unwrap()
        .values()
        .filter(|x| x.status == EStatus::Queued).cloned()
        .collect();

    list.sort_by(|a, b| (b.description.priority, b.description.length).cmp(&(a.description.priority, a.description.length)));

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


pub struct RealIP(std::net::IpAddr);

impl<'a, 'r> rocket::request::FromRequest<'a, 'r> for RealIP {
    type Error = ();
    fn from_request(request: &'a rocket::Request<'r>) -> rocket::request::Outcome<Self, Self::Error> {
        match request.client_ip() {
            Some(ip) => rocket::Outcome::Success(RealIP(ip)),
            None => rocket::Outcome::Failure((rocket::http::Status::from_code(401).unwrap(), ()))
        }
    }
}

#[get("/edit_status/<id>/<status>")]
fn edit_status(id: Uuid, status: String, shared: State<SharedState>, remote_addr: RealIP) -> Result<String, Box<std::error::Error>> {
    let mut list = shared.jobs.lock().unwrap();
    let job = list.get_mut(&id).ok_or("what")?;
    let status = match status.as_str() {
        "queued" => Ok(EStatus::Queued),
        "reserved" => Ok(EStatus::Reserved(remote_addr.0.to_string())),
        "completed" => Ok(EStatus::Completed(remote_addr.0.to_string())),
        "error" => Ok(EStatus::Error(remote_addr.0.to_string())),
        _ => Err("Not a valid status, valid statuses are queued, reserved, completed, and error")
    }?;

    job.status = status;

    Ok("Status changed".to_string())
}



#[post("/upload/<id>", data = "<video>")]
fn upload(id: Uuid, video: Data, shared: State<SharedState>) -> Result<String, std::io::Error> {
    if shared.jobs.lock().unwrap().contains_key(&id) == false {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Job not found"))
    }
    else {
        let list = shared.jobs.lock().unwrap();
        let job = list.get(&id).unwrap();

	let folder = format!("results/{jobset}", jobset = job.jobset);
        let filename = format!("{folder}/{name}.{id}.webm", folder = folder, name = job.description.file_name, id = id);

        let url = format!("{host}/{id}\n", host = "https://av1.dodsorf.as", id = id);
	std::fs::create_dir_all(&folder)?;
        video.stream_to_file(Path::new(&filename))?;
        Ok(url)
    }
}


#[post("/add_job/<jobset>", format = "json", data = "<message>")]
fn add_job(message: Json<workunit::WDesc>, jobset: String, shared: State<SharedState>) {
    let job = message.into_inner();
    let id = uuid::Uuid::new_v4();
    shared.jobs.lock().unwrap().insert(id, WUnit::new(id, jobset, job));
}

fn main() {
        rocket::ignite()
        .manage(SharedState::default())
        .mount("/", StaticFiles::from("src/static")) // switch to templates or something cause this is dumb
        .mount("/", routes![index, version, get_jobs, get_job, request_job, edit_status, add_job, upload])
        .mount("/", routes![test_job])
        .launch();
}


#[get("/test_job/<jobset>")]
fn test_job(jobset: String, shared: State<SharedState>) {
    let id = uuid::Uuid::new_v4();
    shared.jobs.lock().unwrap().insert(id, WUnit::new(id, jobset, workunit::WDesc::new("https://pomf.dodsorf.as/f/g91y5j.mkv", "014", None, 90, (540, 960), None)));
}