// src/storage/volumes.rs

use std::{
    collections::HashMap,
    fs::{create_dir_all, read_dir, read_to_string, remove_dir_all, write},
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use nix::mount::{MsFlags, mount, umount};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string_pretty};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub mountpoint: PathBuf,
    pub labels: HashMap<String, String>,
    pub options: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeConfig {
    pub name: String,
    pub driver: String,
    pub labels: HashMap<String, String>,
    pub options: HashMap<String, String>,
}

pub struct VolumeManager {
    volumes_dir: PathBuf,
    volumes: HashMap<String, Volume>,
}

impl VolumeManager {
    pub fn new(volumes_dir: PathBuf) -> Result<Self> {
        create_dir_all(&volumes_dir)?;

        let mut manager = Self {
            volumes_dir,
            volumes: HashMap::new(),
        };
        manager.load_volumes();
        Ok(manager)
    }

    fn load_volumes(&mut self) -> Result<()> {
        if !self.volumes_dir.exists() {
            return Ok(());
        }
        for entry in read_dir(&self.volumes_dir)? {
            let entry = entry?;
            let volume_path = entry.path();

            if volume_path.is_dir() {
                let volume_file = volume_path.join("volume.json");
                if volume_file.exists() {
                    let volume_data = read_to_string(volume_file)?;
                    let volume: Volume = from_str(&volume_data)?;
                    self.volumes.insert(volume.id.clone(), volume);
                }
            }
        }
        Ok(())
    }

    pub fn create(&mut self, config: VolumeConfig) -> Result<Volume> {
        if self.volumes.values().any(|v| v.name == config.name) {
            return Err(anyhow!("Volume with name {} already exists", config.name));
        }

        let volume_id = Uuid::new_v4().to_string();
        let volume_dir = self.volumes_dir.join(&volume_id);
        let mount_point = volume_dir.join("data");

        create_dir_all(&mount_point)?;

        let volume = Volume {
            id: volume_id.clone(),
            name: config.name,
            driver: config.driver,
            mountpoint: mount_point,
            labels: config.labels,
            options: config.options,
            created_at: Utc::now(),
        };

        self.save_volume(&volume)?;
        self.volumes.insert(volume_id, volume.clone());

        Ok(volume)
    }

    pub fn remove(&mut self, name: &str) -> Result<()> {
        let volume_id = self
            .volumes
            .iter()
            .find(|(_, v)| v.name == name)
            .map(|(id, _)| id.clone())
            .ok_or_else(|| anyhow!("Volume '{}' not found", name))?;

        let _ = self
            .volumes
            .get(&volume_id)
            .ok_or_else(|| anyhow!("Volume not found"))?;

        let volume_dir = self.volumes_dir.join(&volume_id);
        if volume_dir.exists() {
            remove_dir_all(volume_dir);
        }

        self.volumes.remove(&volume_id);

        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&Volume> {
        self.volumes.values().find(|v| v.name == name)
    }

    pub fn list(&self) -> Vec<&Volume> {
        self.volumes.values().collect()
    }

    pub fn mount(&self, name: &str, target: &Path) -> Result<()> {
        let volume = self
            .get(name)
            .ok_or_else(|| anyhow!("Volume '{}' not found", name))?;

        if !target.exists() {
            create_dir_all(target)?;
        }
        mount(
            Some(&volume.mountpoint),
            target,
            None::<&str>,
            MsFlags::MS_BIND,
            None::<&str>,
        )?;

        Ok(())
    }

    pub fn unmount(&self, target: &Path) -> Result<()> {
        umount(target)?;
        Ok(())
    }

    fn save_volume(&self, volume: &Volume) -> Result<()> {
        let volume_dir = self.volumes_dir.join(&volume.id);
        create_dir_all(&volume_dir)?;

        let volume_file = volume_dir.join("volume.json");
        let volume_data = to_string_pretty(volume)?;
        write(volume_file, volume_data)?;

        Ok(())
    }
}
