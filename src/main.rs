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
r#"
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