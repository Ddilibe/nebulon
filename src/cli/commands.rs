// src/cli/commands.rs
use clap::{Parser, Subcommand};
use std::{env::current_dir, path::PathBuf};

// use crate::runtime::container::VolumeMount;
fn get_current_dir() -> String {
    let current_dir = current_dir().unwrap();
    let dir_name_os_str = current_dir.file_name();
    let dir_name = dir_name_os_str
        .and_then(|name| name.to_str())
        .map(|s| s.to_owned())
        .unwrap();
    return dir_name;
}

#[derive(Parser)]
#[command(name = "Nebulon")]
#[command(about = "Nebulon Container Runtime", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Run {
        container_id: String,
    },

    Create {
        command: String,

        args: Vec<String>,

        #[arg(short, long)]
        rootfs: Option<PathBuf>,

        #[arg(short, long, default_value = "/")]
        workdir: PathBuf,

        #[arg(short, long)]
        env: Vec<String>,

        #[arg(long, default_value = "orca-container")]
        hostname: String,

        #[arg(short, long, default_value = "0")]
        uid: u32,

        #[arg(short, long, default_value = "0")]
        gid: u32,

        #[arg(short, long, default_value = "[]")]
        volumes: Vec<String>,

        #[arg(short, long, default_value = "")]
        storage_driver: String,
    },

    Ps,

    Stop {
        #[arg(short, long, default_value_t= get_current_dir())]
        container_id: String,
    },

    Init,

    Volume {
        #[command(subcommand)]
        command: VolumeCommands,
    },

    Image {
        #[command(subcommand)]
        command: ImageCommands,
    },
}

#[derive(Subcommand)]
pub enum VolumeCommands {
    Create {
        name: String,

        #[arg(short, long)]
        driver: Option<String>,
    },
    Ls,
    Rm {
        name: String,
    },
}

#[derive(Subcommand)]
pub enum ImageCommands {
    Import {
        path: PathBuf,
        name: String,
        tag: String,
    },
    Ls,
    Rm {
        name: String,
        tag: String,
    },
}
