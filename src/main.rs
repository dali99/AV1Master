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
    "Welcome to the AV1 Encoder Master Server"
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
        .values().cloned()
        .filter(|x| x.status == EStatus::Queued)
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

#[post("/add_job", format = "json", data = "<message>")]
fn add_job(message: Json<workunit::WDesc>, shared: State<SharedState>) -> Result<String, String> {
    println!("{:#?}", message);
    let job = message.into_inner();

    let id = uuid::Uuid::new_v4();

    shared.jobs.lock().unwrap().insert(id, WUnit::new(id, job));
    Ok(format!("{:#?}", shared))
}

fn main() {
        rocket::ignite()
        .manage(SharedState::default())
        .mount("/", routes![index, version, get_jobs, get_job, request_job, add_job])
        .launch();
}