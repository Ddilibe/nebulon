// src/storage/drivers.rs

use anyhow::{Result, anyhow};
use copy_dir::copy_dir;
use nix::mount::{MsFlags, mount, umount};
use std::{
    collections::HashMap,
    fs::{create_dir_all, read_dir, remove_dir_all},
    path::{Path, PathBuf},
};

pub trait StorageDriver: Send + Sync {
    fn name(&self) -> &str;

    // Snapshot operation
    fn create_snapshot(&self, id: &str, source: &Path) -> Result<PathBuf>;
    fn remove_snapshot(&self, id: &str) -> Result<()>;

    // Mount Operations
    fn mount(&self, source: &Path, target: &Path, option: &HashMap<String, String>) -> Result<()>;
    fn unmount(&self, target: &Path) -> Result<()>;

    // Query operation
    fn exists(&self, id: &str) -> bool;
    fn list_snapshots(&self) -> Result<Vec<String>>;
}

pub struct OverlayFSDriver {
    root: PathBuf,
    work_dir: PathBuf,
}

impl OverlayFSDriver {
    pub fn new(root: PathBuf) -> Result<Self> {
        let work_dir = root.join("work");
        create_dir_all(&work_dir).map_err(|e| {
            anyhow!(
                "Failed to create directory {:?} for OverlayDriver: {}",
                work_dir,
                e
            )
        })?;
        Ok(Self { root, work_dir })
    }

    fn create_dirs(&self, lower: &Path, upper: &Path, work: &Path, merged: &Path) -> Result<()> {
        for dir in &[lower, upper, work, merged] {
            if !dir.exists() {
                create_dir_all(dir)?;
            }
        }
        Ok(())
    }
}

impl StorageDriver for OverlayFSDriver {
    fn name(&self) -> &str {
        "overlayfs"
    }

    fn create_snapshot(&self, id: &str, source: &Path) -> Result<PathBuf> {
        let snapshot_dir = self.root.join("snapshots").join(id);
        let lower_dir = snapshot_dir.join("lower");
        let upper_dir = snapshot_dir.join("upper");
        let work_dir = snapshot_dir.join("work");
        let merged_dir = snapshot_dir.join("merged");

        self.create_dirs(&lower_dir, &upper_dir, &work_dir, &merged_dir)?;

        if source.exists() {
            copy_dir(source, &lower_dir).unwrap();
        }

        let options = format!(
            "lowerdir={},upperdir={},workdir={}",
            lower_dir.display(),
            upper_dir.display(),
            work_dir.display()
        );

        mount(
            Some("overlay"),
            &merged_dir,
            Some("overlay"),
            MsFlags::empty(),
            Some(options.as_str()),
        )?;

        Ok(merged_dir)
    }

    fn remove_snapshot(&self, id: &str) -> Result<()> {
        let snapshot_dir = self.root.join("snapshots").join(id);
        let merged_dir = snapshot_dir.join("merged");

        if merged_dir.exists() {
            umount(&merged_dir).unwrap();
        }

        if snapshot_dir.exists() {
            remove_dir_all(&snapshot_dir).unwrap();
        }

        Ok(())
    }

    fn mount(&self, source: &Path, target: &Path, option: &HashMap<String, String>) -> Result<()> {
        if !target.exists() {
            create_dir_all(target)?;
        }
        mount(
            Some(source),
            target,
            None::<&str>,
            MsFlags::MS_BIND,
            None::<&str>,
        )
        .unwrap();

        Ok(())
    }

    fn unmount(&self, target: &Path) -> Result<()> {
        umount(target).unwrap();
        Ok(())
    }

    fn exists(&self, id: &str) -> bool {
        self.root.join("snapshots").join(id).exists()
    }

    fn list_snapshots(&self) -> Result<Vec<String>> {
        let snapshot_dir = self.root.join("snapshots");

        if !snapshot_dir.exists() {
            return Ok(Vec::new());
        }

        let mut snapshots = Vec::new();
        for entry in read_dir(snapshot_dir).unwrap() {
            let entry = entry?;
            if entry.file_type().unwrap().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    snapshots.push(name.to_string());
                }
            }
        }

        Ok(snapshots)
    }
}

pub struct AufsDriver {
    root: PathBuf,
}

impl AufsDriver {
    pub fn new(root: PathBuf) -> Result<Self> {
        create_dir_all(&root)?;
        Ok(Self { root })
    }
}

impl StorageDriver for AufsDriver {
    fn name(&self) -> &str {
        "aufs"
    }

    fn create_snapshot(&self, id: &str, source: &Path) -> Result<PathBuf> {
        let snapshot_dir = self.root.join("snapshots").join(id);
        let merged_dir = snapshot_dir.join("merged");

        create_dir_all(&merged_dir).unwrap();

        if source.exists() {
            copy_dir(source, &merged_dir).unwrap();
        }
        Ok(merged_dir)
    }

    fn remove_snapshot(&self, id: &str) -> Result<()> {
        let snapshot_dir = self.root.join("snapshots").join(id);

        if snapshot_dir.exists() {
            remove_dir_all(&snapshot_dir)?;
        }
        Ok(())
    }

    fn mount(&self, source: &Path, target: &Path, option: &HashMap<String, String>) -> Result<()> {
        mount(
            Some(source),
            target,
            None::<&str>,
            MsFlags::MS_BIND,
            None::<&str>,
        )
        .unwrap();
        Ok(())
    }

    fn unmount(&self, target: &Path) -> Result<()> {
        umount(target).unwrap();
        Ok(())
    }

    fn exists(&self, id: &str) -> bool {
        self.root.join("snapshots").join(id).exists()
    }

    fn list_snapshots(&self) -> Result<Vec<String>> {
        let snapshot_dir = self.root.join("snapshots");

        if !snapshot_dir.exists() {
            return Ok(Vec::new());
        }

        let mut snapshots = Vec::new();
        for entry in read_dir(snapshot_dir).unwrap() {
            let entry = entry?;
            if entry.file_type().unwrap().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    snapshots.push(name.to_string());
                }
            }
        }

        Ok(snapshots)
    }
}

pub enum DriverType {
    OverlayFS,
    Aufs,
}

pub fn create_driver(driver_type: DriverType, root: PathBuf) -> Result<Box<dyn StorageDriver>> {
    match driver_type {
        DriverType::OverlayFS => Ok(Box::new(OverlayFSDriver::new(root).unwrap())),
        DriverType::Aufs => Ok(Box::new(AufsDriver::new(root).unwrap())),
    }
}
