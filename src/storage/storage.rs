use anyhow::Ok;
// src/storage.rs
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{OnceLock, RwLock};
use sysinfo::System;

use crate::PROGRAM_NAME;
use crate::runtime::container::Container;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemData {
    pub system_name: String,
    pub os_version: String,
    pub total_memory: u64,
    pub containers: Vec<Container>,
}

fn get_datafile() -> String {
    format!("/var/lib/{}/metadata.json", *PROGRAM_NAME)
}

static SYSTEM_DATA_LOCK: OnceLock<RwLock<SystemData>> = OnceLock::new();

pub fn init() -> Result<()> {
    if Path::new(get_datafile().as_str()).exists() {
        log::info!("System file already initialized.");
        let data = read_from_disk()?;
        SYSTEM_DATA_LOCK.set(RwLock::new(data)).ok();
        return Ok(());
    }

    let mut sys = System::new_all();
    sys.refresh_all();

    let data = SystemData {
        system_name: sysinfo::System::name().unwrap_or("Unknown".into()),
        os_version: sysinfo::System::os_version().unwrap_or("Unknown".into()),
        total_memory: sys.total_memory(),
        containers: vec![],
    };

    let json = serde_json::to_string_pretty(&data)?;
    let mut file = File::create(get_datafile().as_str())?;
    file.write_all(json.as_bytes())?;

    SYSTEM_DATA_LOCK.set(RwLock::new(data)).ok();
    log::info!("System initialized and locked: {}", get_datafile().as_str());
    Ok(())
}

fn read_from_disk() -> Result<SystemData> {
    let mut file = File::open(get_datafile().as_str())?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let data: SystemData = serde_json::from_str(&content)?;
    Ok(data)
}

fn write_to_disk(data: &SystemData) -> Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    fs::write(get_datafile().as_str(), json)?;
    Ok(())
}

pub fn add_container(container: Container) -> Result<()> {
    let lock = SYSTEM_DATA_LOCK
        .get()
        .expect("System not initialized")
        .write()
        .unwrap();

    let mut data = lock.clone();

    data.containers.push(container);
    write_to_disk(&data)?;
    drop(lock);

    SYSTEM_DATA_LOCK.get().unwrap().write().unwrap().containers = data.containers.clone();

    println!("Container added successfully.");
    Ok(())
}

pub fn get_container_ids() -> Result<Vec<String>> {
    let lock = SYSTEM_DATA_LOCK
        .get()
        .expect("System not initialized")
        .read()
        .unwrap();
    let ids = lock.containers.iter().map(|c| c.id.clone()).collect();
    Ok(ids)
}

pub fn delete_container(id: &str) -> Result<()> {
    let mut lock = SYSTEM_DATA_LOCK
        .get()
        .expect("System not initialized")
        .write()
        .unwrap();

    let initial_len = lock.containers.len();
    lock.containers.retain(|c| c.id != id);

    if lock.containers.len() < initial_len {
        write_to_disk(&lock)?;
        println!("Container {} deleted successfully.", id);
    } else {
        println!("Container with ID {} not found.", id);
    }

    Ok(())
}

pub fn get_container(id: &str) -> Result<Container> {
    let mut lock = SYSTEM_DATA_LOCK
        .get()
        .expect("System not initialized")
        .write()
        .unwrap();
    lock.containers.retain(|e| e.id == id);

    if lock.containers.len() > 0 {
        log::info!("The number of containers are {}", lock.containers.len());
        return Ok(lock.containers.get(0).unwrap().clone());
    }
    Err(anyhow!("Container does not exists"))
}
