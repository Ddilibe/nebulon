// src/storage/snapshot.rs

use anyhow::Result;
use copy_dir::copy_dir;
use std::{
    collections::HashMap,
    fs::{create_dir_all, remove_dir_all},
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub id: String,
    pub parent: Option<String>,
    pub rootfs: PathBuf,
    pub active: bool,
}

pub struct SnapshotManager {
    snapshots_dir: PathBuf,
    snapshots: HashMap<String, Snapshot>,
}

impl SnapshotManager {
    pub fn new(snapshots_dir: PathBuf) -> Result<Self> {
        create_dir_all(&snapshots_dir)?;

        let mut manager = Self {
            snapshots_dir,
            snapshots: HashMap::new(),
        };

        manager.load_snapshots()?;
        Ok(manager)
    }

    fn load_snapshots(&mut self) -> Result<()> {
        todo!();
        Ok(())
    }

    pub fn create(&mut self, parent: Option<&str>) -> Result<Snapshot> {
        let snapshot_id = Uuid::new_v4().to_string();
        let snapshot_dir = self.snapshots_dir.join(&snapshot_id);
        create_dir_all(&snapshot_dir)?;

        let rootfs = snapshot_dir.join("rootfs");
        create_dir_all(&rootfs)?;

        if let Some(parent_id) = parent {
            if let Some(parent_snapshot) = self.snapshots.get(parent_id) {
                Self::copy_directory(&parent_snapshot.rootfs, &rootfs)?;
            }
        }

        let snapshot = Snapshot {
            id: snapshot_id.clone(),
            parent: parent.map(|s| s.to_string()),
            rootfs,
            active: false,
        };
        self.snapshots.insert(snapshot_id, snapshot.clone());

        Ok(snapshot)
    }

    pub fn remove(&mut self, snapshot_id: &str) -> Result<()> {
        if let Some(snapshot) = self.snapshots.remove(snapshot_id) {
            if snapshot.rootfs.exists() {
                remove_dir_all(snapshot.rootfs)?;
            }
        }
        Ok(())
    }

    pub fn get(&self, snapshot_id: &str) -> Option<&Snapshot> {
        self.snapshots.get(snapshot_id)
    }

    fn copy_directory(src: &Path, dst: &Path) -> Result<()> {
        copy_dir(src, dst);
        Ok(())
    }
}
