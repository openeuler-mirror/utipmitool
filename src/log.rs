/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use log::{Log, Level, Metadata, Record, SetLoggerError};
use syslog::{Facility, Formatter3164};
use std::sync::{Arc, Mutex};
use std::fmt;

lazy_static::lazy_static! {
    static ref LOGGER: Logger = Logger::new();
}

pub struct Logger {
    inner: Arc<Mutex<LoggerInner>>,
}

struct LoggerInner {
    name: String,
    daemon: bool,
    syslog: Option<syslog::Logger<syslog::LoggerBackend, Formatter3164>>,
    level: log::LevelFilter,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(LoggerInner {
                name: "ipmitool".into(),
                daemon: false,
                syslog: None,
                level: log::LevelFilter::Info,
            })),
        }
    }

    pub fn setup(&self, name: &str, daemon: bool, verbose: u8) -> Result<(), SetLoggerError> {
        let mut inner = self.inner.lock().unwrap();
        
        // 修复3：添加日志系统初始化
        if daemon {
            let formatter = Formatter3164 {
                facility: Facility::LOG_LOCAL4,
                hostname: None,
                process: name.into(),
                pid: std::process::id(),
            };
            // 修复4：处理可能的错误而不是panic
            inner.syslog = Some(
                syslog::unix(formatter)
                    .map_err(|e| SetLoggerError::new(std::io::ErrorKind::Other, e))?
            );
        }

        log::set_logger(&*LOGGER)?;
        log::set_max_level(inner.level);
        Ok(())
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.inner.lock().unwrap().level
    }

    fn log(&self, record: &Record) {
        let inner = self.inner.lock().unwrap();  // 改为不可变借用
        if !self.enabled(record.metadata()) {
            return;
        }

        let message = format!("{}", record.args());
        
        if inner.daemon {
            if let Some(syslog) = &inner.syslog {
                // 修复1：使用正确的syslog等级映射
                let severity = match record.level() {
                    Level::Error => syslog::Severity::LOG_ERR,
                    Level::Warn => syslog::Severity::LOG_WARNING,
                    Level::Info => syslog::Severity::LOG_INFO,
                    Level::Debug => syslog::Severity::LOG_DEBUG,
                    Level::Trace => syslog::Severity::LOG_DEBUG,
                };
                syslog.send(severity, message).ok();
            }
        } else {
            // 修复2：添加颜色和完整格式
            eprintln!("[{}] {} - {}", 
                record.level(), 
                record.target(), 
                message
            );
        }
    }

    fn flush(&self) {}
}

// 错误处理扩展
pub trait LoggableError {
    fn log_error(self, context: &str) -> Self;
}

impl<T, E: fmt::Display> LoggableError for Result<T, E> {
    fn log_error(self, context: &str) -> Self {
        if let Err(ref e) = self {
            log::error!("{}: {}", context, e);
        }
        self
    }
}