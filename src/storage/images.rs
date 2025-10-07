// src/storage/images.rs

use std::{
    collections::HashMap,
    fs::{File, create_dir_all, read_to_string, remove_dir_all, write},
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::Result;
use chrono::{DateTime, Utc};
use copy_dir::copy_dir;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string_pretty};
use sha2::{Digest, Sha256};
use tar::Archive;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub id: String,
    pub name: String,
    pub tag: String,
    pub digest: String,
    pub layers: Vec<String>,
    pub size: u64,
    pub created_at: DateTime<Utc>,
    pub config: ImageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    pub entrypoint: Vec<String>,
    pub cmd: Vec<String>,
    pub env: Vec<String>,
    pub working_dir: String,
    pub user: String,
    pub labels: HashMap<String, String>,
}

pub struct ImageManager {
    images_dir: PathBuf,
    images: HashMap<String, Image>,
}

impl Default for ImageConfig {
    fn default() -> Self {
        Self {
            entrypoint: vec![],
            cmd: vec!["/bin/sh".to_string()],
            env: vec!["PATH+/usr/bin:/bin".to_string()],
            working_dir: "/".to_string(),
            user: "root".to_string(),
            labels: HashMap::new(),
        }
    }
}

impl ImageManager {
    pub fn new(images_dir: PathBuf) -> Result<Self> {
        create_dir_all(&images_dir)?;

        let mut manager = Self {
            images_dir,
            images: HashMap::new(),
        };

        manager.load_images()?;
        Ok(manager)
    }

    fn load_images(&mut self) -> Result<()> {
        let images_file = self.images_dir.join("images.json");
        if images_file.exists() {
            let images_data = read_to_string(images_file)?;
            let images: Vec<Image> = from_str(&images_data)?;
            for image in images {
                self.images.insert(image.id.clone(), image);
            }
        }
        Ok(())
    }

    fn save_images(&self) -> Result<()> {
        let images_file = self.images_dir.join("images.json");
        let images_data = to_string_pretty(&self.images.values().collect::<Vec<&Image>>())?;

        write(images_file, images_data)?;
        Ok(())
    }

    pub fn import_from_tar(&mut self, tar_path: &Path, name: &str, tag: &str) -> Result<Image> {
        let file = File::open(tar_path)?;
        let mut archive = Archive::new(file);

        let image_id = Self::generate_image_id(tar_path)?;
        let image_dir = self.images_dir.join(&image_id);
        create_dir_all(&image_dir)?;

        let layers_dir = image_dir.join("layers");
        create_dir_all(&layers_dir)?;

        archive.unpack(&layers_dir)?;

        let size = Self::calculate_directory_size(&layers_dir)?;

        let image = Image {
            id: image_id.clone(),
            name: name.to_string(),
            tag: tag.to_string(),
            digest: Self::calculate_digest(tar_path)?,
            layers: vec![layers_dir.to_string_lossy().to_string()],
            size,
            created_at: Utc::now(),
            config: ImageConfig::default(),
        };

        self.images.insert(image_id, image.clone());
        self.save_images()?;

        Ok(image)
    }

    pub fn remove(&mut self, image_id: &str) -> Result<()> {
        if let Some(image) = self.images.remove(image_id) {
            let image_dir = self.images_dir.join(image_id);
            if image_dir.exists() {
                remove_dir_all(image_dir)?;
            }
            self.save_images()?;
        }
        return Ok(());
    }

    pub fn get(&self, name: &str, tag: &str) -> Option<&Image> {
        self.images
            .values()
            .find(|img| img.name == name && img.tag == tag)
    }

    pub fn list(&self) -> Vec<&Image> {
        self.images.values().collect()
    }

    pub fn prepare_rootfs(&self, image: &Image, target: &Path) -> Result<()> {
        for layer in &image.layers {
            let layer_path = Path::new(layer);
            if layer_path.exists() {
                Self::copy_directory(layer_path, target)?;
            }
        }
        Ok(())
    }

    fn generate_image_id(tar_path: &Path) -> Result<String> {
        let mut file = File::open(tar_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 1204];

        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn calculate_digest(path: &Path) -> Result<String> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 1204];

        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        Ok(format!("sha256:{:x}", hasher.finalize()))
    }

    fn copy_directory(src: &Path, dst: &Path) -> Result<()> {
        copy_dir(src, dst)?;
        Ok(())
    }

    fn calculate_directory_size(path: &Path) -> Result<u64> {
        let total_size = 0;

        Ok(total_size)
    }
}
