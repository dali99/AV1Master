use serde::{Serialize, Deserialize};
use uuid::Uuid;

use std::net::IpAddr;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct WUnit {
    pub id: Uuid,
    pub description: WDesc,
    pub status: EStatus
}
impl WUnit {
    pub fn new(id: Uuid, description: WDesc) -> Self {
        WUnit {
            id: id,
            description: description,
            status: EStatus::Queued
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct WDesc {
    pub file_url: String,
    pub file_name: String,
    pub priority: u16,
    pub length: u32,
    pub resolution: (u16, u16),
    pub options: EOptions
}
impl WDesc {
    pub fn new(file_url: &str, file_name: &str, priority: Option<u16>, length: u32, resolution: (u16, u16), options: Option<EOptions>) -> Self {
        WDesc {
            file_url: file_url.to_string(),
            file_name: file_name.to_string(),
            priority: priority.unwrap_or(0),
            length: length,
            resolution: resolution,
            options: options.unwrap_or(EOptions::default()),
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EOptions {
    pub ffmpeg: String,
    pub aomenc: String,
    pub two_pass: bool,
    pub pix_fmt: EPixFmt
}
impl Default for EOptions {
    fn default() -> Self {
        EOptions{
            ffmpeg: String::default(),
            aomenc: "--lag-in-frames=25 --tile-columns=0 --tile-rows=0 --enable-fwd-kf=1 --bit-depth=10 --cpu-used=3 --cq-level=30 --end-usage=q".to_string(),
            two_pass: false,
            pix_fmt: EPixFmt::I422
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum EStatus {
    Queued,
    Reserved(String),
    Completed(String),
    Cancelled,
    Error(String)
}
impl Default for EStatus {
    fn default() -> Self {
        EStatus::Queued
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum EPixFmt {
    YV12,
    I420,
    I422,
    I444
}