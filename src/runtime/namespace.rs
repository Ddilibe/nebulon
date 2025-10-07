// src/runtime/namespace.rs
use anyhow::{Result, anyhow};
// use nix::libc::uname;
use nix::sched::{CloneFlags, unshare};
use nix::unistd::{Gid, Uid, setgid, sethostname, setuid};

pub struct Namespaces;

impl Namespaces {
    pub fn unshare_all() -> Result<()> {
        let flags = CloneFlags::CLONE_NEWNS // Mount Namespace
        | CloneFlags::CLONE_NEWUTS  // UTS name space (hostname)
        | CloneFlags::CLONE_NEWPID  // PID namespace
        | CloneFlags::CLONE_NEWNET  // newtwork namespace
        | CloneFlags::CLONE_NEWIPC // IPC namespace
        | CloneFlags::CLONE_NEWUSER; // User namespace

        unshare(flags)
            .map_err(|e| anyhow!("Failed to unshare namespaces: {}", e))
            .unwrap();
        Ok(())
    }

    pub fn set_hosename(hostname: &str) -> Result<()> {
        sethostname(hostname).map_err(|e| anyhow!("Failed to set HostName: {}", e)).unwrap();
        Ok(())
    }

    pub fn drop_privileges(uid: u32, gid: u32) -> Result<()> {
        setgid(Gid::from_raw(gid))
            .map_err(|e| anyhow!("Failed to set GID: {}", e))
            .unwrap();
        setuid(Uid::from_raw(uid))
            .map_err(|e| anyhow!("Failed to set UID: {}", e))
            .unwrap();
        Ok(())
    }
}
