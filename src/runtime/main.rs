use crate::runtime::username;
// src/runtime/main.rs
use crate::PROGRAM_CMD;
use crate::runtime::cgroups::CgroupManager;
use crate::runtime::container::{Container, ContainerConfig, ContainerStatus, VolumeMount};
use crate::runtime::filesystem::Filesystem;
use crate::runtime::namespace::Namespaces;
use anyhow::{Result, anyhow};
use nix::mount::{MsFlags, mount};
use nix::sys::signal::Signal::SIGTERM;
use nix::sys::signal::kill;
use nix::unistd::{ForkResult, Pid, chdir, fork};
use std::ffi::CString;
use std::fs::create_dir_all;
use std::path::Path;
use uuid::Uuid;

pub struct Runtime;

impl Runtime {
    pub fn create_container(config: ContainerConfig) -> Result<Container> {
        let container_id = format!(
            "{}-{}",
            *PROGRAM_CMD,
            Uuid::new_v4().to_string()[..8].to_string()
        );

        let container = Container {
            id: container_id.clone(),
            pid: 0,
            status: ContainerStatus::Created,
            config,
        };
        log::info!("Created container: {}", container_id);
        log::info!("Container name is {:?}", container);
        Ok(container)
    }

    pub fn start_container(container: &mut Container) -> Result<()> {
        log::info!("Starting container: {}", container.id);
        username();

        Filesystem::create_rootfs(&container.config.rootfs).unwrap();

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                container.pid = child.as_raw();
                container.status = ContainerStatus::Running;

                let cgroup_manager = CgroupManager::new(&container.id).unwrap();

                cgroup_manager.add_process(child.as_raw()).unwrap();
                // cgroup_manager.set_memory_limit(512).unwrap();
                username();
                // cgroup_manager.set_cpu_quota(100).unwrap();

                log::info!("Container {} started with PID: {}", container.id, child);
            }
            Ok(ForkResult::Child) => {
                Self::container_process(container).unwrap();
                std::process::exit(0);
            }
            Err(e) => return Err(anyhow!("Failed to fork process: {}", e)),
        }

        Ok(())
    }

    fn container_process(container: &Container) -> Result<()> {
        Namespaces::unshare_all().unwrap();
        Namespaces::set_hosename(&container.config.hostname).unwrap();

        Filesystem::setup_rootfs(&container.config.rootfs).unwrap();

        if container.config.uid != 0 {
            Namespaces::drop_privileges(container.config.uid, container.config.gid).unwrap();
        }

        chdir(&container.config.working_dirs)
            .map_err(|e| anyhow!("Failed to change directory: {}", e))
            .unwrap();

        for volume_mount in &container.config.volumes {
            Self::mount_volume(volume_mount, &container.id)?;
        }

        let _command = CString::new(container.config.command[0].as_bytes())
            .map_err(|e| anyhow!("invalid command: {}", e))
            .unwrap();

        let _args: Vec<CString> = container
            .config
            .command
            .iter()
            .chain(container.config.args.iter())
            .map(|arg| CString::new(arg.as_bytes()).unwrap())
            .collect();

        let _env: Vec<CString> = container
            .config
            .env_vars
            .iter()
            .map(|env| CString::new(env.as_bytes()).unwrap())
            .collect();

        // execve(&command, &args, &env)
        // .map_err(|e| anyhow!("Failed to execute command: {}", e))
        // .unwrap();

        Ok(())
    }

    pub fn stop_container(container: &mut Container) -> Result<()> {
        if container.status != ContainerStatus::Running {
            return Err(anyhow!("Container is not running"));
        }

        log::info!("Stopping container: {}", container.id);

        kill(Pid::from_raw(container.pid), SIGTERM)
            .map_err(|e| anyhow!("Failed to send SIFTERM to container: {}", e))
            .unwrap();

        container.status = ContainerStatus::Stopped;

        let cgroup_manager = CgroupManager::new(&container.id).unwrap();
        cgroup_manager.cleanup().unwrap();

        Ok(())
    }

    fn mount_volume(volume_mount: &VolumeMount, _container_id: &str) -> Result<()> {
        let target_path = Path::new(&volume_mount.target);

        if !target_path.exists() {
            create_dir_all(target_path)?;
        }

        if volume_mount.source.starts_with('/') {
            mount(
                Some(Path::new(&volume_mount.source)),
                target_path,
                None::<&str>,
                MsFlags::MS_BIND,
                None::<&str>,
            )?;
        } else {
            todo!();
        }
        if volume_mount.read_only {
            mount(
                Some(target_path),
                target_path,
                None::<&str>,
                MsFlags::MS_REMOUNT | MsFlags::MS_RDONLY,
                None::<&str>,
            )?;
        }
        Ok(())
    }
}
