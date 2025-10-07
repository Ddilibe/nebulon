//src/runtime/mod.rs

use nix::unistd::{Uid, User};

pub mod cgroups;
pub mod filesystem;
pub mod namespace;

pub mod container;
pub mod main;

fn get_username() -> Option<String> {
    let uid = Uid::current();
    User::from_uid(uid).ok().flatten().map(|u| u.name)
}

pub fn username() {
    match get_username() {
        Some(user) => println!("Current user: {}", user),
        None => println!("Unable to get username"),
    }
}
