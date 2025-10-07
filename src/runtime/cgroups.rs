// src/runtime/cgroups.rs
use anyhow::{Result, anyhow};
use nix::sys::statfs::{CGROUP_SUPER_MAGIC, CGROUP2_SUPER_MAGIC, statfs};
use std::fs::{self, File, create_dir_all};
use std::io::Write;
use std::path::PathBuf;

use crate::PROGRAM_NAME;

pub struct CgroupManager {
    cgroup_path: PathBuf,
}

impl CgroupManager {
    pub fn new(container_id: &str) -> Result<Self> {
        let cgroup_path =
            PathBuf::from(format!("/sys/fs/cgroup/{}", *PROGRAM_NAME)).join(container_id);

        if !cgroup_path.exists() {
            create_dir_all(&cgroup_path)
                .map_err(|e| anyhow!("Failed to create cgroup directory: {}", e))
                .unwrap();
        }

        Ok(Self { cgroup_path })
    }

    pub fn set_memory_limit(&self, limit_mb: usize) -> Result<()> {
        
        let version = self.detect_cgroup_version().unwrap();
        let path = if version == 2 {
            "memory.max"
        } else {
            "memory.limit_in_bytes"
        };
        let memory_path = self.cgroup_path.join(path);
        let mut file = File::create(memory_path)
            .map_err(|e| anyhow!("Failed to create memory limit file: {}", e))
            .unwrap();

        let limit_bytes = limit_mb * 1024 * 1024;
        write!(file, "{}", limit_bytes)
            .map_err(|e| anyhow!("Failed to write CPU quota: {}", e))
            .unwrap();

        Ok(())
    }

    fn detect_cgroup_version(&self) -> Result<u8> {
        let stat = statfs("/sys/fs/cgroup").unwrap();

        if stat.filesystem_type() == CGROUP2_SUPER_MAGIC {
            Ok(2)
        } else if stat.filesystem_type() == CGROUP_SUPER_MAGIC {
            Ok(1)
        } else {
            Err(anyhow!("Unknown cgroup filetype system"))
        }
    }

    pub fn set_cpu_quota(&self, quota_percent: u32) -> Result<()> {
        let cpu_quota_path = self.cgroup_path.join("cpu.max");
        let mut file = File::create(cpu_quota_path)
            .map_err(|e| anyhow!("Failed to create CPU quota file: {}", e))
            .unwrap();

        // Calculate quota (percentage of CPU time)
        let quota = (quota_percent * 1000) as u32; // Convert to millicores
        write!(file, "{} 100000", quota)
            .map_err(|e| anyhow!("Failed to write CPU quota: {}", e))
            .unwrap();

        Ok(())
    }

    pub fn add_process(&self, pid: i32) -> Result<()> {
        let procs_path = self.cgroup_path.join("cgroup.procs");
        let mut file = File::create(procs_path)
            .map_err(|e| anyhow!("Failed to create cgroup procs file: {}", e))
            .unwrap();

        write!(file, "{}", pid)
            .map_err(|e| anyhow!("Failed to add process to cgroup: {}", e))
            .unwrap();

        Ok(())
    }

    pub fn cleanup(&self) -> Result<()> {
        if self.cgroup_path.exists() {
            fs::remove_dir(&self.cgroup_path)
                .map_err(|e| anyhow!("Failed to remove cgroup: {}", e))
                .unwrap();
        }
        Ok(())
    }
}
