/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

// logger.rs
use chrono::Local;
use std::fmt;
use std::io::{self, Write};
use std::sync::Mutex;
use std::sync::OnceLock;

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Error = 0,
    Warning = 1,
    Notice = 2,
    Info = 3,
    Debug = 4,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            LogLevel::Notice => "NOTICE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARN",
            LogLevel::Error => "ERROR",
        };
        write!(f, "{}", s)
    }
}

/// ANSI 颜色代码
#[allow(dead_code)]
mod ansi_colors {
    pub const RESET: &str = "\x1b[0m";
    pub const BLACK: &str = "\x1b[30m";
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";

    // 亮色
    pub const BRIGHT_BLACK: &str = "\x1b[90m";
    pub const BRIGHT_RED: &str = "\x1b[91m";
    pub const BRIGHT_GREEN: &str = "\x1b[92m";
    pub const BRIGHT_YELLOW: &str = "\x1b[93m";
    pub const BRIGHT_BLUE: &str = "\x1b[94m";
    pub const BRIGHT_MAGENTA: &str = "\x1b[95m";
    pub const BRIGHT_CYAN: &str = "\x1b[96m";
    pub const BRIGHT_WHITE: &str = "\x1b[97m";
}

/// 日志级别颜色映射
struct LevelColors {
    notice: &'static str,
    debug: &'static str,
    info: &'static str,
    warning: &'static str,
    error: &'static str,
}

impl Default for LevelColors {
    fn default() -> Self {
        Self {
            notice: ansi_colors::WHITE,
            debug: ansi_colors::CYAN,
            info: ansi_colors::GREEN,
            warning: ansi_colors::YELLOW,
            error: ansi_colors::RED,
        }
    }
}

impl LevelColors {
    fn get(&self, level: LogLevel) -> &'static str {
        match level {
            LogLevel::Notice => self.notice,
            LogLevel::Debug => self.debug,
            LogLevel::Info => self.info,
            LogLevel::Warning => self.warning,
            LogLevel::Error => self.error,
        }
    }
}
/// 日志配置
pub struct LogConfig {
    pub level: LogLevel,
    pub use_colors: bool,
    pub use_timestamps: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            use_colors: true,
            use_timestamps: false,
        }
    }
}

/// 内部日志器
struct Logger {
    config: Mutex<LogConfig>,
    colors: LevelColors, // 颜色配置
}

impl Logger {
    fn new(level: LogLevel) -> Self {
        Self {
            config: Mutex::new(LogConfig {
                level,
                use_colors: true,
                use_timestamps: false,
            }),
            colors: LevelColors::default(),
        }
    }

    fn set_config(&self, config: LogConfig) {
        let mut cfg = self.config.lock().unwrap();
        *cfg = config;
    }

    fn get_level(&self) -> LogLevel {
        self.config.lock().unwrap().level
    }

    fn color_enabled(&self) -> bool {
        self.config.lock().unwrap().use_colors
    }

    fn timestamps_enabled(&self) -> bool {
        self.config.lock().unwrap().use_timestamps
    }

    fn log(&self, level: LogLevel, message: fmt::Arguments) {
        if level > self.get_level() {
            return;
        }

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        // 需要输出行号，取消下面注释
        // let (file, line) = get_caller_location();

        // let output = format!("{} {:<5} [{}:{}] {}", timestamp, level, file, line, message);
        // 改变输出格式，调整output变量即可
        let output = if self.timestamps_enabled() {
            format!("{} {}", timestamp, message)
        } else {
            format!("{}", message)
        };
        if self.color_enabled() {
            let color = self.colors.get(level);
            let colored = format!("{}{}{}", color, output, ansi_colors::RESET);

            let _ = match level {
                LogLevel::Error => writeln!(io::stderr(), "{}", colored),
                _ => writeln!(io::stdout(), "{}", colored),
            };
        } else {
            let _ = match level {
                LogLevel::Error => writeln!(io::stderr(), "{}", output),
                _ => writeln!(io::stdout(), "{}", output),
            };
        }
    }
}

/// 需要输出行号，取消注释，获取调用位置（文件名和行号）
// fn get_caller_location() -> (&'static str, u32) {
//     let location = std::panic::Location::caller();
//     (location.file(), location.line())
// }

// 全局静态日志器
static LOGGER: OnceLock<Logger> = OnceLock::new();

// 设置 v的级别，通过
fn get_logger() -> &'static Logger {
    LOGGER.get_or_init(|| Logger::new(LogLevel::Notice))
}

/// 初始化日志系统
pub fn init_logger(config: LogConfig) {
    get_logger().set_config(config);
}

/// 设置日志级别
pub fn set_log_level(level: u8) {
    let level = match level + 2 {
        0 => LogLevel::Error,
        1 => LogLevel::Warning,
        2 => LogLevel::Notice,
        3 => LogLevel::Info,
        4 => LogLevel::Debug,
        _ => LogLevel::Debug,
    };
    let mut config = get_logger().config.lock().unwrap();
    config.level = level;
}

/// 启用/禁用颜色输出
pub fn set_color_enabled(enabled: bool) {
    let mut config = get_logger().config.lock().unwrap();
    config.use_colors = enabled;
}

// 内部函数，供宏调用
#[doc(hidden)]
pub fn _log(level: LogLevel, args: fmt::Arguments) {
    get_logger().log(level, args);
}

// 宏定义：方便在任何地方调用
#[macro_export]
macro_rules! log_notice {
    ($($arg:tt)*) => {
        $crate::logger::_log($crate::logger::LogLevel::Notice, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::logger::_log($crate::logger::LogLevel::Debug, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::logger::_log($crate::logger::LogLevel::Info, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::logger::_log($crate::logger::LogLevel::Warning, format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::logger::_log($crate::logger::LogLevel::Error, format_args!($($arg)*))
    };
}
