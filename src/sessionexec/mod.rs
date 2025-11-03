use std::{error::Error, process::Command};

use cstr::CStr;

pub(crate) mod cstr;
pub mod execve;
pub mod gamescope;
pub mod plasma;
pub mod runner;

pub(crate) fn find_program_path(program: &str) -> Result<String, Box<dyn Error>> {
    let output = Command::new("which").arg(program).output()?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(path)
    } else {
        Err(format!("Program '{}' not found in PATH", program).into())
    }
}

pub(crate) fn execve_wrapper(
    prog: &CStr,
    argv_data: &Vec<CStr>,
    envp_data: &Vec<CStr>,
) -> Result<(), Box<dyn std::error::Error>> {
    let prog = prog.inner();

    let argv = argv_data
        .iter()
        .map(|e| e.inner())
        .chain(std::iter::once(std::ptr::null()))
        .collect::<Vec<*const libc::c_char>>();

    let envp = envp_data
        .iter()
        .map(|e| e.inner())
        .chain(std::iter::once(std::ptr::null()))
        .collect::<Vec<*const libc::c_char>>();

    let execve_err = unsafe { libc::execve(prog, argv.as_ptr(), envp.as_ptr()) };

    if execve_err == -1 {
        return Err(format!("execve failed: {}", std::io::Error::last_os_error()).into());
    }

    unreachable!()
}

pub(crate) fn get_unix_username(uid: u32) -> Option<String> {
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
                let ptr = passwd.pw_name as *const _;
                let username = std::ffi::CStr::from_ptr(ptr).to_str().unwrap().to_owned();
                Some(username)
            }
            _ => None,
        }
    }
}
