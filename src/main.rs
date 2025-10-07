// src/main.rs
mod cli;
mod runtime;
pub mod storage;

use std::fs::create_dir_all;
use std::path::{Path, PathBuf};

use crate::cli::commands::{Cli, Commands};
use crate::runtime::container::ContainerConfig;
use crate::runtime::main::Runtime;
use crate::storage::storage::{
    add_container, get_container, get_container_ids, init,
};
use anyhow::anyhow;
use chrono::{DateTime, Datelike, Local};
use clap::Parser;
use log::{LevelFilter, info};
use log4rs::Config;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use nix::sys::wait::{WaitPidFlag, waitpid};
use nix::unistd::Pid;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref PROGRAM_NAME: String = String::from("Nebulon").to_lowercase();
    pub static ref PROGRAM_CMD: String = String::from("nb");
    pub static ref PROGRAM_START_TIME: DateTime<Local> = Local::now();
    pub static ref LOG_FILE_PATH: String = format!(
        "logs/{}_{}_{}_{}.log",
        PROGRAM_CMD.as_str(),
        PROGRAM_START_TIME.year(),
        PROGRAM_START_TIME.month(),
        PROGRAM_START_TIME.day()
    );
    pub static ref PROGRAM_ROOT: PathBuf = PathBuf::from("/var/lib/nebulon/");
}

fn setup_logging() {
    let file_name = LOG_FILE_PATH.as_str();
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{h({d(%Y-%m-%d %H:%M:%S)(utc)} - {l}: {m}{n})}",
        )))
        .build(file_name)
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config).unwrap();
}

fn main() {
    // env_logger::init();

    let cli = Cli::parse();
    let root = "/var/lib/nebulon/rootfs";
    setup_logging();

    let path = Path::new(root);
    match create_dir_all(path) {
        Ok(_) => {
            println!("created {:?}", root);
            log::info!("created {:?}", root);
        }
        Err(err) => println!("Error creating directory: {}\n", err),
    };
    init().unwrap();

    match cli.command {
        Commands::Init => {
            info!("Calling the init command");
        }
        Commands::Run { container_id } => {
            if get_container_ids().unwrap().contains(&container_id) {
                let mut container = get_container(&container_id).unwrap();
                Runtime::start_container(&mut container).unwrap();
            }
        }
        Commands::Create {
            command,
            args,
            rootfs,
            workdir,
            env,
            hostname,
            uid,
            gid,
            volumes,
            storage_driver,
        } => {
            let config = ContainerConfig {
                command: vec![command],
                args,
                env_vars: env,
                working_dirs: workdir,
                hostname: hostname,
                rootfs: rootfs.unwrap_or_else(|| root.into()),
                uid,
                gid,
                volumes: vec![],
                storage_driver: storage_driver,
            };

            let container = Runtime::create_container(config).unwrap();

            log::info!(
                "Pid before container creation is {:?}",
                Pid::from_raw(container.pid)
            );
            waitpid(Pid::from_raw(container.pid), Some(WaitPidFlag::empty()))
                .map_err(|e| anyhow!("Failed to initiate container: {}", e))
                .unwrap();
            add_container(container).unwrap();
            // unwrap();
        }
        Commands::Ps => {
            println!("Listing containers");
            for (num, con) in get_container_ids().unwrap().iter().enumerate() {
                println!("{}. {}", num + 1, con);
            }
        }
        Commands::Stop { container_id } => {
            println!("Stoping container: {}", container_id);
            if get_container_ids().unwrap().contains(&container_id) {
                let mut container = get_container(&container_id).unwrap();
                Runtime::stop_container(&mut container).unwrap();
            }
        }
        Commands::Volume { command } => match command {
            cli::commands::VolumeCommands::Create { name, driver } => {
                log::info!("Volume of {} with driver {:?} is created", name, driver)
            }
            cli::commands::VolumeCommands::Ls => {
                log::info!("Listing out the exiting volumes")
            }
            cli::commands::VolumeCommands::Rm { name } => {
                log::info!("Removing the volume: {}", name)
            }
        },
        Commands::Image { command } => match command {
            cli::commands::ImageCommands::Import { path, name, tag } => {
                log::info!(
                    "Importing the image with name {} and tag {} on {:?} path",
                    name,
                    tag,
                    path
                )
            }
            cli::commands::ImageCommands::Ls => {
                log::info!("Listing out the images")
            }
            cli::commands::ImageCommands::Rm { name, tag } => {
                log::info!("Removing image {} with tag {}", name, tag)
            }
        },
    }
}
