/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use std::ffi::CString;
use std::io::{self, Write};
use std::ptr;
use std::sync::Mutex;
use libc::{closelog, openlog, strerror, syslog, LOG_CONS, LOG_LOCAL4, LOG_NOTICE, LOG_WARNING, LOG_ERR};
use std::os::raw::c_char;

const LOG_NAME_DEFAULT: &str = "ipmitool";
const LOG_MSG_LENGTH: usize = 1024;

struct LogPriv {
    name: CString,
    daemon: bool,
    level: i32,
}

//LogPriv全局指针
lazy_static::lazy_static! {
    static ref LOG_PRIV: Mutex<Option<LogPriv>> = Mutex::new(None);
}

fn log_reinit() {
    log_init(None, false, 0);
}

pub fn lprintf(level: i32, format: &str, args: std::fmt::Arguments) {
    let mut log_priv = LOG_PRIV.lock().unwrap();

    if log_priv.is_none() {
        log_reinit();
    }

    if let Some(ref logpriv) = *log_priv {
        if logpriv.level < level {
            return;
        }

        let mut logmsg = String::with_capacity(LOG_MSG_LENGTH);
        logmsg.push_str(&format!("{}", args));

        if logpriv.daemon {
            unsafe {
                syslog(level, "%s\0".as_ptr() as *const c_char, logmsg.as_ptr() as *const c_char);
            }
        } else {
            eprintln!("{}", logmsg);
        }
    }
}

pub fn lperror(level: i32, format: &str, args: std::fmt::Arguments) {
    let mut log_priv = LOG_PRIV.lock().unwrap();

    if log_priv.is_none() {
        log_reinit();
    }

    if let Some(ref logpriv) = *log_priv {
        if logpriv.level < level {
            return;
        }

        let mut logmsg = String::with_capacity(LOG_MSG_LENGTH);
        logmsg.push_str(&format!("{}", args));

        let err_msg = unsafe { CString::from_raw(strerror(io::Error::last_os_error().raw_os_error().unwrap())) };

        if logpriv.daemon {
            unsafe {
                syslog(
                    level,
                    "%s: %s\0".as_ptr() as *const c_char,
                    logmsg.as_ptr() as *const c_char,
                    err_msg.as_ptr(),
                );
            }
        } else {
            eprintln!("{}: {}", logmsg, err_msg.to_string_lossy());
        }
    }
}

pub fn log_init(name: Option<&str>, isdaemon: bool, verbose: i32) {
    let mut log_priv = LOG_PRIV.lock().unwrap();

    if log_priv.is_some() {
        return;
    }

    let log_name = CString::new(name.unwrap_or(LOG_NAME_DEFAULT)).unwrap();

    *log_priv = Some(LogPriv {
        name: log_name.clone(),
        daemon: isdaemon,
        level: verbose + LOG_NOTICE,
    });

    if isdaemon {
        unsafe {
            openlog(log_name.as_ptr(), LOG_CONS, LOG_LOCAL4);
        }
    }
}

pub fn log_halt() {
    let mut log_priv = LOG_PRIV.lock().unwrap();

    if let Some(logpriv) = log_priv.take() {
        if logpriv.daemon {
            unsafe {
                closelog();
            }
        }
    }
}

pub fn log_level_set(verbose: i32) {
    let mut log_priv = LOG_PRIV.lock().unwrap();

    if let Some(ref mut logpriv) = *log_priv {
        logpriv.level = verbose + LOG_NOTICE;
    }
}
