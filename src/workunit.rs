use std::path::PathBuf;

#[derive(Default, Debug)]
pub struct WUnit {
    file_name: PathBuf,
    priority: u16,
    length: u32,
    options: EOptions,
    status: EStatus
}


#[derive(Debug)]
struct EOptions {
    mode: EMode,
    resolution: Resolution,
    color_depth: EColorDepth,
    enable_fwd_keyframe: bool,
    kf_min_dist: Option<u16>,
    kf_max_dist: Option<u16>,
    two_pass: bool,
    speed: u8
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

#[derive(Debug)]
enum EMode {
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

#[derive(Default, Debug)]
struct Resolution {
    width: Option<u16>,
    height: Option<u16>
}

#[derive(Debug)]
enum EColorDepth {
    Eight = 8,
    Ten = 10,
    Twelve = 12
}
impl Default for EColorDepth {
    fn default() -> Self {
        EColorDepth::Ten
    }
}

#[derive(Debug)]
enum EStatus {
    Queued,
    Reserved,
    Completed
}
impl Default for EStatus {
    fn default() -> Self {
        EStatus::Queued
    }
}