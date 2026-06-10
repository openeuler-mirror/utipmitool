/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

#![allow(dead_code)]

use std::collections::HashMap;
use std::fmt;
use std::sync::OnceLock;

// 定义值-字符串映射类型
type ValStrMap = HashMap<u8, &'static str>;

// 获取 completion code 映射表的函数（使用 OnceLock 实现单例）
fn get_completion_code_map() -> &'static ValStrMap {
    static MAP: OnceLock<ValStrMap> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert(0x00, "Command completed normally");
        m.insert(0xc0, "Node busy");
        m.insert(0xc1, "Invalid command");
        m.insert(0xc2, "Invalid command on LUN");
        m.insert(0xc3, "Timeout");
        m.insert(0xc4, "Out of space");
        m.insert(0xc5, "Reservation cancelled or invalid");
        m.insert(0xc6, "Request data truncated");
        m.insert(0xc7, "Request data length invalid");
        m.insert(0xc8, "Request data field length limit exceeded");
        m.insert(0xc9, "Parameter out of range");
        m.insert(0xca, "Cannot return number of requested data bytes");
        m.insert(0xcb, "Requested sensor, data, or record not found");
        m.insert(0xcc, "Invalid data field in request");
        m.insert(0xcd, "Command illegal for specified sensor or record type");
        m.insert(0xce, "Command response could not be provided");
        m.insert(0xcf, "Cannot execute duplicated request");
        m.insert(0xd0, "SDR Repository in update mode");
        m.insert(0xd1, "Device firmware in update mode");
        m.insert(0xd2, "BMC initialization in progress");
        m.insert(0xd3, "Destination unavailable");
        m.insert(0xd4, "Insufficient privilege level");
        m.insert(0xd5, "Command not supported in present state");
        m.insert(0xd6, "Cannot execute command, command disabled");
        m.insert(0xff, "Unspecified error");
        m
    })
}

// 获取 OEM 信息映射表的函数
fn get_oem_info_map() -> &'static HashMap<u32, &'static str> {
    static MAP: OnceLock<HashMap<u32, &'static str>> = OnceLock::new();
    MAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert(0x000c29, "VMware, Inc.");
        m.insert(0x0002a0, "Intel Corporation");
        m
    })
}

// 辅助函数：将 completion code 转换为描述字符串
pub fn completion_code_to_string(code: u8) -> &'static str {
    get_completion_code_map()
        .get(&code)
        .copied()
        .unwrap_or("Unknown completion code")
}

// 辅助函数：将 OEM ID 转换为字符串
pub fn oem_id_to_string(oem_id: u32) -> &'static str {
    get_oem_info_map()
        .get(&oem_id)
        .copied()
        .unwrap_or("Unknown OEM")
}

pub type IpmiResult<T> = Result<T, IpmiError>;

/// IPMI specific error types
#[derive(Debug, Clone)]
pub enum IpmiError {
    /// Interface error with message
    Interface(String),
    /// IPMI completion code error
    CompletionCode(u8),
    /// Timeout error
    Timeout,
    /// Invalid data error
    InvalidData(String),
    /// Network error
    Network(String),
    /// Authentication error
    Authentication(String),
    /// Session error
    Session(String),
    /// Command not supported
    NotSupported(String),
    /// Generic error
    Generic(String),
    /// Response error (no response received)
    ResponseError,
    /// System error (file I/O, kernel interactions)
    System(String),
    /// 返回一个数值
    Value(i8),
}

impl fmt::Display for IpmiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IpmiError::Interface(msg) => write!(f, "Interface error: {}", msg),
            IpmiError::CompletionCode(code) => {
                let description = completion_code_to_string(*code);
                if description == "Unknown completion code" {
                    write!(f, "Completion code error: 0x{:02x}", code)
                } else {
                    write!(f, "{}", description)
                }
            }
            IpmiError::Timeout => write!(f, "Operation timed out"),
            IpmiError::InvalidData(msg) => write!(f, "{}", msg),
            IpmiError::Network(msg) => write!(f, "Network error: {}", msg),
            IpmiError::Authentication(msg) => write!(f, "Authentication error: {}", msg),
            IpmiError::Session(msg) => write!(f, "Session error: {}", msg),
            IpmiError::NotSupported(msg) => write!(f, "Command not supported: {}", msg),
            IpmiError::Generic(msg) => write!(f, "Generic error: {}", msg),
            IpmiError::ResponseError => write!(f, "Response error: No response received"),
            IpmiError::System(msg) => write!(f, "{}", msg),
            IpmiError::Value(v) => write!(f, "{}", v),
        }
    }
}

impl std::error::Error for IpmiError {}

// 从 std::io::Error 转换
impl From<std::io::Error> for IpmiError {
    fn from(error: std::io::Error) -> Self {
        IpmiError::System(error.to_string())
    }
}

// 从 nix::Error 转换（如果使用nix crate）
impl From<nix::Error> for IpmiError {
    fn from(error: nix::Error) -> Self {
        IpmiError::System(error.to_string())
    }
}

/// Helper function to convert legacy i32 error codes to IpmiError
pub fn i32_to_ipmi_error(code: i32, context: &str) -> IpmiError {
    match code {
        0 => panic!("Code 0 should not be converted to error"),
        -1 => IpmiError::Interface(format!("{}: operation failed", context)),
        -2 => IpmiError::InvalidData(format!("{}: invalid parameter", context)),
        -3 => IpmiError::ResponseError,
        -4 => IpmiError::Timeout,
        -5 => IpmiError::Authentication(format!("{}: insufficient privileges", context)),
        _ => IpmiError::System(format!("{}: error code {}", context, code)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_code_display() {
        let error = IpmiError::CompletionCode(0x00);
        assert_eq!(format!("{}", error), "Command completed normally");

        let error = IpmiError::CompletionCode(0xc1);
        assert_eq!(format!("{}", error), "Invalid command");

        let error = IpmiError::CompletionCode(0x99); // Unknown code
        assert_eq!(format!("{}", error), "Completion code error: 0x99");
    }

    #[test]
    fn test_oem_id_to_string() {
        assert_eq!(oem_id_to_string(0x000c29), "VMware, Inc.");
        assert_eq!(oem_id_to_string(0x0002a0), "Intel Corporation");
        assert_eq!(oem_id_to_string(0x123456), "Unknown OEM");
    }
}
