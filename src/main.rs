#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use std::path::PathBuf;

mod workunit;
use workunit::WUnit;

#[get("/")]
fn index() -> &'static str {
    "Welcome to the AV1 Encoder Master Server"
}

#[get("/get_work/<max_length>")]
fn getJobs(max_length: u32, ) -> Result<String, std::io::Error> {
    let mut work = WUnit::default();
    Ok(format!("{:#?}", work))
}

fn main() {
    rocket::ignite().mount("/", routes![index, getJobs]).launch();
}