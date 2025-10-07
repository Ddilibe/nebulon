// src/runtime/filesystem.rs

use anyhow::{Result, anyhow};
use nix::mount::{MsFlags, mount};
use nix::unistd::{chdir, pivot_root};
use std::fs::create_dir_all;
use std::path::{Path};

pub struct Filesystem;

impl Filesystem {
    pub fn setup_rootfs(rootfs_path: &Path) -> Result<()> {
        if !rootfs_path.exists() {
            return Err(anyhow!("Rootfs path does not exists: {:?}", rootfs_path));
        }

        let proc_path = rootfs_path.join("proc");
        if proc_path.exists() {
            mount(
                Some("proc"),
                &proc_path,
                Some("proc"),
                MsFlags::empty(),
                None::<&str>,
            )
            .map_err(|e| anyhow!("Failed to mount proc: {}", e))
            .unwrap();
        }
        chdir(rootfs_path)
            .map_err(|e| anyhow!("Failed to create rootfs directory: {}", e))
            .unwrap();

        Ok(())
    }

    pub fn create_rootfs(base_path: &Path) -> Result<()> {
        if !base_path.exists() {
            create_dir_all(base_path)
                .map_err(|e| anyhow!("Failed to create rootfs directory: {}", e))
                .unwrap();

            let dirs = vec!["bin", "dev", "etc", "proc", "sys", "tmp", "usr", "var"];
            let _ = dirs.iter().map(|f| {
                create_dir_all(base_path.join(f.to_string().as_str()))
                    .map_err(|e| anyhow!("Failed to create directory {}: {}", f, e))
                    .unwrap()
            });
        }

        Ok(())
    }

    pub fn _pivot_root(new_root: &Path) -> Result<()> {
        let put_old = new_root.join(".pivot_root");

        pivot_root(new_root, &put_old)
            .map_err(|e| anyhow!("Failed to pivot root: {}", e))
            .unwrap();

        chdir("/")
            .map_err(|e| anyhow!("Failed to change to new root: {}", e))
            .unwrap();

        Ok(())
    }
}
