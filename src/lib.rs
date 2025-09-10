/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

//pub mod commands;
pub mod debug_control;
pub mod error;
pub mod helper;
//pub mod interface;
//pub mod ipmi;
pub mod logging;

/*
// 在 src/lib.rs 中添加或确认存在
use std::sync::atomic::AtomicUsize;
pub static VERBOSE_LEVEL: AtomicUsize = AtomicUsize::new(0);

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if $crate::VERBOSE_LEVEL.load(std::sync::atomic::Ordering::Relaxed) > 0
           && !$crate::debug_control::should_skip_debug() {
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! debug1 {
    ($($arg:tt)*) => {
        if $crate::VERBOSE_LEVEL.load(std::sync::atomic::Ordering::Relaxed) >= 1
           && !$crate::debug_control::should_skip_debug() {
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! debug2 {
    ($($arg:tt)*) => {
        if $crate::VERBOSE_LEVEL.load(std::sync::atomic::Ordering::Relaxed) >= 2
           && !$crate::debug_control::should_skip_debug() {
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! debug3 {
    ($($arg:tt)*) => {
        if $crate::VERBOSE_LEVEL.load(std::sync::atomic::Ordering::Relaxed) >= 3
           && !$crate::debug_control::should_skip_debug() {
            println!($($arg)*);
        }
    };
}

// 在其他debug宏定义旁边添加debug4!宏定义
#[macro_export]
macro_rules! debug4 {
    ($($arg:tt)*) => {
        if $crate::VERBOSE_LEVEL.load(std::sync::atomic::Ordering::Relaxed) >= 4
           && !$crate::debug_control::should_skip_debug() {
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! debug5 {
    ($($arg:tt)*) => {
        if $crate::VERBOSE_LEVEL.load(std::sync::atomic::Ordering::Relaxed) >= 5
           && !$crate::debug_control::should_skip_debug() {
            println!($($arg)*);
        }
    };
}

// use std::fmt::{Display, Formatter, Result};

// pub trait HexDisplay {
//     fn to_hex_string(&self, sep: Option<&str>) -> String;
// }

// impl HexDisplay for [u8] {
//     fn to_hex_string(&self, sep: Option<&str>) -> String {
//         let sep = sep.unwrap_or("");
//         let mut output = String::with_capacity(self.len() * (2 + sep.len()));

//         for (i, byte) in self.iter().enumerate() {
//             if i != 0 {
//                 output.push_str(sep);
//             }
//             output.push_str(&format!("{:02x}", byte));
//         }

//         output
//     }
// }

// impl Display for dyn HexDisplay {
//     fn fmt(&self, f: &mut Formatter<'_>) -> Result {
//         write!(f, "{}", self.to_hex_string(None))
//     }
// }
//
//
//
*/
