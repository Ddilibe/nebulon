// src/stroage/mod.rs

pub mod storage;
pub mod drivers;
pub mod volumes;
pub mod images;
pub mod snapshot;


use std::path::PathBuf;

use crate::PROGRAM_ROOT;

pub struct StorageConfig {
    pub driver: String,
    pub root: PathBuf,
    pub volumes_dir: PathBuf,
    pub images_dir: PathBuf,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            driver:"overlayfs".to_string(),
            root: PROGRAM_ROOT.to_path_buf().clone(),
            volumes_dir: PROGRAM_ROOT.join("volumes"),
            images_dir: PROGRAM_ROOT.join("images"),
        }
    }
}


pub struct StorageManager {
    config: StorageConfig,
    volume: volumes::VolumeManager,
    image_manager: images::ImageManager,
    driver: Box<dyn drivers::StorageDriver>,
}