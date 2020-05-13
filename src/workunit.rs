use serde::{Serialize, Deserialize};
use uuid::Uuid;

use std::net::IpAddr;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct WUnit {
    pub id: Uuid,
    pub jobset: String,
    pub description: WDesc,
    pub status: EStatus
}
impl WUnit {
    pub fn new(id: Uuid, _jobset: String, description: WDesc) -> Self {
        let jobset: String = _jobset.chars()
            .map(|x| match x {
                '.' => '_',
                '/' => '-',
                _ => x
            }).collect();
        WUnit {
            id: id,
            jobset: jobset,
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
pub enum EOptions {
    AOMENC(AomencO),
    FFMPEG(FffmpegO)
}
impl Default for EOptions {
    fn default() -> Self {
        let default = AomencO::default();
        EOptions::AOMENC(default)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AomencO {
    pub ffmpeg: String,
    pub aomenc: String,
    pub two_pass: bool,
    pub pix_fmt: EPixFmt,
    pub fps: (u16, u16)
}
impl Default for AomencO {
    fn default() -> Self {
        AomencO{
            ffmpeg: String::default(),
            aomenc: "--lag-in-frames=35 --tile-columns=0 --tile-rows=0 --enable-fwd-kf=1 --bit-depth=10 --cpu-used=4 --cq-level=30 --end-usage=q".to_string(),
            two_pass: true,
            pix_fmt: EPixFmt::I422,
            fps: (25, 1)
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FffmpegO {
    pub two_pass: bool,
    pub crf: u8,
    pub b_v: String,
    pub lag_in_frames: u8,
    pub gop: Option<u16>,
    pub pix_fmt: String,
    pub tiles: String,
    pub speed: u8
}
impl Default for FffmpegO {
    fn default() -> Self {
        FffmpegO {
            two_pass: true,
            crf: 30,
            b_v: "0".to_string(),
            lag_in_frames: 35,
            gop: Option::None,
            pix_fmt: "yuv420p10le".to_string(),
            tiles: "1x1".to_string(),
            speed: 4
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