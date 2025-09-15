/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(dead_code)]

use chrono::{Local, TimeZone, Utc};
// 定义IPMI特殊时间常量
const IPMI_TIME_UNSPECIFIED: u32 = 0xFFFFFFFF;
const IPMI_TIME_INIT_DONE: u32 = 0x20000000;
const SECONDS_A_DAY: u32 = 24 * 60 * 60;

/// 检查是否为特殊时间戳(系统启动后的时间)
fn is_special_timestamp(ts: u32) -> bool {
    ts < IPMI_TIME_INIT_DONE
}

/// 检查时间戳是否有效
fn is_valid_timestamp(ts: u32) -> bool {
    ts != IPMI_TIME_UNSPECIFIED
}

/// IPMI时间戳格式化(类似C的ipmi_timestamp_numeric)
/// 严格匹配ipmitool的时间格式：MM/DD/YYYY HH:MM:SS（无时区信息）
pub fn ipmi_timestamp_numeric(stamp: u32) -> String {
    if !is_valid_timestamp(stamp) {
        return "Unspecified".to_string();
    }

    if is_special_timestamp(stamp) {
        if stamp < SECONDS_A_DAY {
            return format!("S+ {}", format_duration(stamp));
        } else {
            let years = stamp / (365 * SECONDS_A_DAY);
            let days = (stamp % (365 * SECONDS_A_DAY)) / SECONDS_A_DAY;
            let remaining_secs = stamp % SECONDS_A_DAY;
            return format!("S+ {}/{} {}", years, days, format_time(remaining_secs));
        }
    }

    // 正常时间戳处理 - 使用UTC时间
    Utc.timestamp_opt(stamp as i64, 0)
        .unwrap()
        .format("%m/%d/%Y %H:%M:%S")
        .to_string()
}

/// 辅助函数: 格式化秒数为HH:MM:SS
fn format_time(seconds: u32) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

/// 辅助函数: 格式化持续时间为HH:MM:SS
fn format_duration(seconds: u32) -> String {
    format_time(seconds)
}

/// 模拟C中的time_in_utc全局变量
/// ipmitool默认使用gmtime()，即UTC时间，我们需要匹配这个行为
fn time_in_utc() -> bool {
    // ipmitool在SEL时间戳处理中使用gmtime()，即UTC时间
    // 为了与ipmitool保持一致，我们也应该使用UTC时间
    true // 默认使用UTC时间以匹配ipmitool
}

// 检查是否是特殊时间戳
fn ipmi_timestamp_is_special(stamp: u32) -> bool {
    stamp < IPMI_TIME_INIT_DONE
}

// 检查时间戳是否有效
fn ipmi_timestamp_is_valid(stamp: u32) -> bool {
    stamp != IPMI_TIME_UNSPECIFIED
}

// Rust实现ipmi_timestamp_date
pub fn ipmi_timestamp_date_old(stamp: u32, time_in_utc: bool) -> String {
    if !ipmi_timestamp_is_valid(stamp) {
        return "Unspecified".to_string();
    }

    if ipmi_timestamp_is_special(stamp) {
        return format_timestamp(stamp, time_in_utc, "S+ %y/%j");
    }

    format_timestamp(stamp, time_in_utc, "%x")
}

// Rust实现ipmi_timestamp_time
// 强制使用UTC时间，忽略time_in_utc参数
pub fn ipmi_timestamp_time(stamp: u32, _time_in_utc: bool) -> String {
    if !ipmi_timestamp_is_valid(stamp) {
        return "Unspecified".to_string();
    }

    // 强制使用UTC时间
    format_timestamp(stamp, true, "%H:%M:%S")
}

// 辅助函数：格式化时间戳
fn format_timestamp_old(stamp: u32, time_in_utc: bool, fmt: &str) -> String {
    let seconds = stamp as i64;

    if time_in_utc || ipmi_timestamp_is_special(stamp) {
        if let Some(dt) = Utc.timestamp_opt(seconds, 0).single() {
            dt.format(fmt).to_string()
        } else {
            "Invalid timestamp".to_string()
        }
    } else if let Some(dt) = Local.timestamp_opt(seconds, 0).single() {
        dt.format(fmt).to_string()
    } else {
        "Invalid timestamp".to_string()
    }
}

// 修复后的format_timestamp函数 - 修复UTC时间被转换为本地时间的BUG
fn format_timestamp(stamp: u32, time_in_utc: bool, fmt: &str) -> String {
    let seconds = stamp as i64;

    if time_in_utc || ipmi_timestamp_is_special(stamp) {
        // 使用UTC时间
        if let Some(dt) = Utc.timestamp_opt(seconds, 0).single() {
            dt.format(fmt).to_string()
        } else {
            "Invalid timestamp".to_string()
        }
    } else {
        // 使用本地时间
        if let Some(dt) = Local.timestamp_opt(seconds, 0).single() {
            dt.format(fmt).to_string()
        } else {
            "Invalid timestamp".to_string()
        }
    }
}

// 优化后的ipmi_timestamp_date函数
// 强制使用UTC时间，忽略time_in_utc参数
pub fn ipmi_timestamp_date(stamp: u32, _time_in_utc: bool) -> String {
    match (
        ipmi_timestamp_is_valid(stamp),
        ipmi_timestamp_is_special(stamp),
    ) {
        (false, _) => "Unspecified".to_string(),
        (true, true) => format_timestamp(stamp, true, "S+ %y/%j"),
        (true, false) => format_timestamp(stamp, true, "%m/%d/%Y"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_special_timestamps() {
        assert_eq!(ipmi_timestamp_numeric(3600), "S+ 01:00:00");
        assert_eq!(ipmi_timestamp_numeric(86400), "S+ 1/0 00:00:00");
        assert_eq!(ipmi_timestamp_numeric(90061), "S+ 1/1 01:01:01");
    }

    #[test]
    fn test_invalid_timestamp() {
        assert_eq!(ipmi_timestamp_numeric(IPMI_TIME_UNSPECIFIED), "Unspecified");
    }
}
