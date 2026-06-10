use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub use libc::{LOG_CONS, LOG_ERR, LOG_LOCAL4, LOG_NOTICE, LOG_WARNING};

fn cstring_sanitize(input: &str) -> CString {
    match CString::new(input) {
        Ok(cstr) => cstr,
        Err(_) => {
            // Strip interior NULs to keep syslog formatting sane.
            let sanitized: String = input.chars().filter(|&ch| ch != '\0').collect();
            CString::new(sanitized).unwrap_or_else(|_| CString::new("<invalid>").unwrap())
        }
    }
}

pub fn openlog(name: &CStr, option: i32, facility: i32) {
    unsafe {
        libc::openlog(name.as_ptr(), option, facility);
    }
}

pub fn closelog() {
    unsafe {
        libc::closelog();
    }
}

pub fn syslog_msg(level: i32, msg: &str) {
    let msg = cstring_sanitize(msg);
    unsafe {
        libc::syslog(level, "%s\0".as_ptr() as *const c_char, msg.as_ptr());
    }
}

pub fn syslog_msg2(level: i32, msg: &str, msg2: &str) {
    let msg = cstring_sanitize(msg);
    let msg2 = cstring_sanitize(msg2);
    unsafe {
        libc::syslog(
            level,
            "%s: %s\0".as_ptr() as *const c_char,
            msg.as_ptr(),
            msg2.as_ptr(),
        );
    }
}

pub fn strerror(errno: i32) -> String {
    let ptr = unsafe { libc::strerror(errno) };
    if ptr.is_null() {
        return format!("errno {}", errno);
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}
