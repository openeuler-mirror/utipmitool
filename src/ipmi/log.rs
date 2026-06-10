/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

use std::ffi::CString;
use std::io::{self, Write};
use std::sync::Mutex;

use utipmi_sys::syslog as sys;

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
            sys::syslog_msg(level, &logmsg);
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

        let errno = io::Error::last_os_error().raw_os_error().unwrap_or(0);
        let err_msg = sys::strerror(errno);

        if logpriv.daemon {
            sys::syslog_msg2(level, &logmsg, &err_msg);
        } else {
            eprintln!("{}: {}", logmsg, err_msg);
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
        level: verbose + sys::LOG_NOTICE,
    });

    if isdaemon {
        sys::openlog(&log_name, sys::LOG_CONS, sys::LOG_LOCAL4);
    }
}

pub fn log_halt() {
    let mut log_priv = LOG_PRIV.lock().unwrap();

    if let Some(logpriv) = log_priv.take() {
        if logpriv.daemon {
            sys::closelog();
        }
    }
}

pub fn log_level_set(verbose: i32) {
    let mut log_priv = LOG_PRIV.lock().unwrap();

    if let Some(ref mut logpriv) = *log_priv {
        logpriv.level = verbose + sys::LOG_NOTICE;
    }
}
