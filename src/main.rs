#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
use rocket::State;
use rocket::response::status::NotFound;

use rocket_contrib::json::Json;
use serde_json::Value;

use std::path::PathBuf;
use std::sync::Mutex;

mod workunit;
use workunit::WUnit;
use workunit::EStatus;

const VERSION: &str = "0.1.0";

#[derive(Default, Debug)]
struct SharedState {
    list: Mutex<Vec<WUnit>>
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
    let list = shared.list.lock().unwrap();
    Json(serde_json::to_value(&list[..]).unwrap())
}

#[get("/request_job")]
fn request_job(shared: State<SharedState>) -> Result<Json<Value>, NotFound<String>> {
    let mut list = shared.list.lock().unwrap().clone();

    list.retain(|x| x.status == EStatus::Queued);
    list.sort_by(|a, b| b.length.cmp(&a.length));

    let job = list.get(0);

    Ok(Json(serde_json::to_value(&job).unwrap()))
}

#[get("/get_job/<id>")]
fn get_job(id: u32, shared: State<SharedState>) -> Result<Json<Value>, NotFound<String>> {
    let list = shared.list.lock().unwrap().clone();

    let job = list.into_iter().find(|x| x.id == id).ok_or(NotFound(format!("Job not Found: {id}", id = id)));

    match job {
        Ok(j) => Ok(Json(serde_json::to_value(&j).unwrap())),
        Err(e) => Err(e)
    }
}

#[post("/add_job")]
fn add_job(shared: State<SharedState>) -> Result<String, std::io::Error> {
    shared.list.lock().unwrap().push(WUnit::new(2, "iduno", None, 10, workunit::EOptions::default()));
    Ok(format!("{:#?}", shared))
}

fn main() {
        rocket::ignite()
        .manage(SharedState::default())
        .mount("/", routes![index, version, get_jobs, get_job, request_job, add_job])
        .launch();
}