// src/runtime/container.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub command: Vec<String>,
    pub args: Vec<String>,
    pub env_vars: Vec<String>,
    pub working_dirs: PathBuf,
    pub hostname: String,
    pub rootfs: PathBuf,
    pub uid: u32,
    pub gid: u32,
    pub volumes: Vec<VolumeMount>,
    pub storage_driver: String,
}

impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            command: vec!["/bin/sh".to_string()],
            args: vec![],
            env_vars: vec!["PATH=/usr/bin:/bin".to_string()],
            working_dirs: PathBuf::from("/"),
            hostname: "nb-container".to_string(),
            rootfs: PathBuf::from("/var/lib/nb/rootfs"),
            uid: 0,
            gid: 0,
            volumes: vec![],
            storage_driver: "/".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Container {
    pub id: String,
    pub pid: i32,
    pub status: ContainerStatus,
    pub config: ContainerConfig,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum ContainerStatus {
    Created,
    Running,
    Stopped,
    Exited,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub source: String,
    pub target: PathBuf,
    pub read_only: bool,
}
