use std::path::PathBuf;

use lazy_static::lazy_static;

// root directory from perspective of pico8 process
pub const DRIVE_DIR: &'static str = "drive";

#[cfg(target_os = "linux")]
pub const IN_PIPE: &'static str = "in_pipe";

#[cfg(target_os = "linux")]
pub const OUT_PIPE: &'static str = "out_pipe";

#[cfg(target_os = "windows")]
pub const IN_PIPE: &'static str = r"\\.\pipe\in_pipe";

#[cfg(target_os = "windows")]
pub const OUT_PIPE: &'static str = r"\\.\pipe\out_pipe";

lazy_static! {
    pub static ref EXE_DIR: PathBuf = PathBuf::from("drive/exe");
    pub static ref CART_DIR: PathBuf = PathBuf::from("drive/carts");
    pub static ref GAMES_DIR: PathBuf = PathBuf::from("drive/carts/games");
    pub static ref MUSIC_DIR: PathBuf = PathBuf::from("drive/carts/music");
    pub static ref LABEL_DIR: PathBuf = PathBuf::from("drive/carts/labels");
    pub static ref METADATA_DIR: PathBuf = PathBuf::from("drive/carts/metadata");
}

// path of png files generated by pico8
pub const RAW_SCREENSHOT_PATH: &'static str = "drive/screenshots";

// path of scaled cart screenshots
pub const SCREENSHOT_PATH: &'static str = "drive/carts/screenshots";
