/*
    login-ng A greeter written in rust that also supports autologin with systemd-homed
    Copyright (C) 2024-2025  Denis Benato

    This program is free software; you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation; either version 2 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License along
    with this program; if not, write to the Free Software Foundation, Inc.,
    51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
*/

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use sessionrunner::dbus::SessionManagerDBus;
use sessionrunner::desc::NodeServiceDescriptor;
use sessionrunner::errors::SessionManagerError;
use sessionrunner::manager::SessionManager;
use sessionrunner::node::{SessionNode, SessionNodeRestart, SessionNodeType};
use sessionrunner::signal::Signal;
use std::time::{SystemTime, UNIX_EPOCH};
use zbus::connection;

pub(crate) fn get_home_dir(uid: u32) -> Option<String> {
    unsafe {
        let mut result = std::ptr::null_mut();
        let amt = match libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) {
            n if n < 0 => 512 as usize,
            n => n as usize,
        };
        let mut buf = Vec::with_capacity(amt);
        let mut passwd: libc::passwd = std::mem::zeroed();

        match libc::getpwuid_r(
            uid,
            &mut passwd,
            buf.as_mut_ptr(),
            buf.capacity() as libc::size_t,
            &mut result,
        ) {
            0 if !result.is_null() => {
                let ptr = passwd.pw_dir as *const _;
                let username = std::ffi::CStr::from_ptr(ptr).to_str().unwrap().to_owned();
                Some(username)
            }
            _ => None,
        }
    }
}

pub(crate) fn get_shell(uid: u32) -> Option<String> {
    unsafe {
        let mut result = std::ptr::null_mut();
        let amt = match libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) {
            n if n < 0 => 512 as usize,
            n => n as usize,
        };
        let mut buf = Vec::with_capacity(amt);
        let mut passwd: libc::passwd = std::mem::zeroed();

        match libc::getpwuid_r(
            uid,
            &mut passwd,
            buf.as_mut_ptr(),
            buf.capacity() as libc::size_t,
            &mut result,
        ) {
            0 if !result.is_null() => {
                let ptr = passwd.pw_shell as *const _;
                let username = std::ffi::CStr::from_ptr(ptr).to_str().unwrap().to_owned();
                Some(username)
            }
            _ => None,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), SessionManagerError> {
    let user_homedir = PathBuf::from(
        get_home_dir(unsafe { libc::getuid() }).expect("Failed to get user information"),
    );
    let load_directories = vec![
        user_homedir.join(".config").join("sessionrunner"),
        PathBuf::from("/etc/sessionrunner/"),
        PathBuf::from("/usr/lib/sessionrunner/"),
    ];

    let default_service_name = String::from("default.service");

    let mut nodes = HashMap::new();
    match NodeServiceDescriptor::load_tree(
        &mut nodes,
        &default_service_name,
        load_directories.as_slice(),
    )
    .await
    {
        Ok(_) => {}
        Err(err) => match err {
            sessionrunner::errors::NodeLoadingError::IOError(err) => {
                eprintln!("File error: {err}");
                std::process::exit(-1)
            }
            sessionrunner::errors::NodeLoadingError::FileNotFound(filename) => {
                // if the default target is missing use the default user shell
                if filename == default_service_name {
                    let shell = get_shell(unsafe { libc::getuid() })
                        .expect("Failed to get user information");

                    eprintln!(
                        "Definition for {default_service_name} not found: using shell {shell}"
                    );

                    nodes = HashMap::from([(
                        default_service_name.clone(),
                        Arc::new(SessionNode::new(
                            default_service_name.clone(),
                            SessionNodeType::Service,
                            None,
                            shell.clone(),
                            vec![],
                            Signal::SIGTERM,
                            SessionNodeRestart::no_restart(),
                            Vec::new(),
                            HashMap::new(),
                        )),
                    )])
                } else {
                    eprintln!("Dependency not found: {filename}");
                    std::process::exit(-1)
                }
            }
            sessionrunner::errors::NodeLoadingError::CyclicDependency(filename) => {
                eprintln!("Cycle for target: {filename}");
                std::process::exit(-1)
            }
            sessionrunner::errors::NodeLoadingError::JSONError(err) => {
                eprintln!("JSON deserialization error: {err}");
                std::process::exit(-1)
            }
            sessionrunner::errors::NodeLoadingError::InvalidKind(err) => {
                eprintln!("JSON syntax error: unrecognised kind value {err}");
                std::process::exit(-1)
            }
        },
    };

    // the XDG_RUNTIME_DIR is required for generating the default dbus socket path
    // and also the runtime directory (hopefully /tmp mounted) to keep track of services
    let xdg_runtime_dir = PathBuf::from(std::env::var("XDG_RUNTIME_DIR").unwrap());

    let manager_runtime_path = xdg_runtime_dir.join(format!(
        "{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    ));

    std::fs::create_dir(manager_runtime_path.clone()).unwrap();

    let manager = Arc::new(SessionManager::new(nodes));

    let dbus_manager = connection::Builder::session()
        .map_err(SessionManagerError::ZbusError)?
        .name("org.neroreflex.sessionrunner")
        .map_err(SessionManagerError::ZbusError)?
        .serve_at(
            "/org/neroreflex/sessionrunner",
            SessionManagerDBus::new(manager.clone()),
        )
        .map_err(SessionManagerError::ZbusError)?
        .build()
        .await
        .map_err(SessionManagerError::ZbusError)?;

    println!("Running the session manager");

    manager.run(&default_service_name).await?;

    drop(dbus_manager);

    Ok(())
}
