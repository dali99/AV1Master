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
fn getJobs(shared: State<SharedState>) -> Json<Value> {
//    let shared_data: &SharedState = shared.inner();
    let list = shared.list.lock().unwrap();

    println!("get jobs blah");
    // println!("{:#?}", Json(list));

    Json(serde_json::to_value(&list[..]).unwrap())
}

#[get("/get_job/<id>")]
fn getJob(id: usize, shared: State<SharedState>) -> Result<String, NotFound<String>> {
    let shared_data: &SharedState = shared.inner();
    let list = shared_data.list.lock().unwrap();

    let job = list.get(id).ok_or(NotFound(format!("Job not Found: {id}", id = id)));

    match job {
        Ok(j) => Ok(format!("{:#?}", j)),
        Err(e) => Err(e)
    }
}

#[get("/add_job")]
fn addJob(shared: State<SharedState>) -> Result<String, std::io::Error> {
    let shared_data: &SharedState = shared.inner();

    shared_data.list.lock().unwrap().push(WUnit::default());
    Ok(format!("{:#?}", shared_data))
}

fn main() {
    rocket::ignite()
        .manage(SharedState::default())
        .mount("/", routes![index, version, getJobs, getJob, addJob])
        .launch();
}