use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct WUnit {
    pub id: u32,
    pub file_url: String,
    pub priority: u16,
    pub length: u32,
    pub options: EOptions,
    pub status: EStatus
}
impl WUnit {
    pub fn new(id: u32, file_url: &str, priority: Option<u16>, length: u32, options: EOptions) -> Self {
        WUnit {
            id: id,
            file_url: file_url.to_string(),
            priority: priority.unwrap_or(0),
            length: length,
            options: options,
            status: EStatus::default()
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EOptions {
    pub mode: EMode,
    pub resolution: Resolution,
    pub color_depth: EColorDepth,
    pub enable_fwd_keyframe: bool,
    pub kf_min_dist: Option<u16>,
    pub kf_max_dist: Option<u16>,
    pub two_pass: bool,
    pub speed: u8
}
impl Default for EOptions {
    fn default() -> Self {
        EOptions{
            mode: EMode::default(),
            resolution: Resolution::default(),
            color_depth: EColorDepth::default(),
            enable_fwd_keyframe: true,
            kf_min_dist: Option::default(),
            kf_max_dist: Option::default(),
            two_pass: false,
            speed: 3
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum EMode {
    // Quality (CRF), Constrained Quality, Variable Bitrate, Constant Bitrate
    Q(u8),
    CQ(u8),
    VBR(u32),
    CBR(u32)
}
impl Default for EMode {
    fn default() -> Self {
        EMode::Q(30)
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Resolution {
    pub width: Option<u16>,
    pub height: Option<u16>
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum EColorDepth {
    Eight = 8,
    Ten = 10,
    Twelve = 12
}
impl Default for EColorDepth {
    fn default() -> Self {
        EColorDepth::Ten
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum EStatus {
    Queued,
    Reserved(Client),
    Completed
}
impl Default for EStatus {
    fn default() -> Self {
        EStatus::Queued
    }
}

type Client = String;