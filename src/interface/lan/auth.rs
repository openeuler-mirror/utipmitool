/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
/// IPMI Authentication Implementation
///
/// Based on reference/ipmitool-c/src/plugins/lan/auth.c and auth.h
use std::collections::HashMap;

// IPMI Authentication Types
pub const IPMI_SESSION_AUTHTYPE_NONE: u8 = 0x00;
pub const IPMI_SESSION_AUTHTYPE_MD2: u8 = 0x01;
pub const IPMI_SESSION_AUTHTYPE_MD5: u8 = 0x02;
pub const IPMI_SESSION_AUTHTYPE_PASSWORD: u8 = 0x04;
pub const IPMI_SESSION_AUTHTYPE_OEM: u8 = 0x05;

// Authentication algorithm lengths
pub const IPMI_AUTHCODE_MD2: usize = 16;
pub const IPMI_AUTHCODE_MD5: usize = 16;
pub const IPMI_AUTHCODE_PASSWORD: usize = 16;

/// IPMI Authentication Information
#[derive(Debug, Clone, Default)]
pub struct IpmiAuth {
    pub authtype: u8,
    pub authcode: [u8; 16],
    pub password: String,
}

impl IpmiAuth {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_with_password(authtype: u8, password: String) -> Self {
        Self {
            authtype,
            password,
            authcode: [0; 16],
        }
    }

    /// Get authentication type name
    pub fn get_auth_type_name(&self) -> &'static str {
        match self.authtype {
            IPMI_SESSION_AUTHTYPE_NONE => "NONE",
            IPMI_SESSION_AUTHTYPE_MD2 => "MD2",
            IPMI_SESSION_AUTHTYPE_MD5 => "MD5",
            IPMI_SESSION_AUTHTYPE_PASSWORD => "PASSWORD",
            IPMI_SESSION_AUTHTYPE_OEM => "OEM",
            _ => "UNKNOWN",
        }
    }

    /// Calculate authentication code for IPMI message
    pub fn calculate_authcode(
        &mut self,
        session_id: u32,
        session_seq: u32,
        password: &[u8],
        msg_data: &[u8],
    ) -> Result<[u8; 16], String> {
        match self.authtype {
            IPMI_SESSION_AUTHTYPE_NONE => Ok([0; 16]),
            IPMI_SESSION_AUTHTYPE_MD2 => {
                self.calculate_md2_authcode(session_id, session_seq, password, msg_data)
            }
            IPMI_SESSION_AUTHTYPE_MD5 => {
                self.calculate_md5_authcode(session_id, session_seq, password, msg_data)
            }
            IPMI_SESSION_AUTHTYPE_PASSWORD => self.calculate_password_authcode(password),
            _ => Err(format!(
                "Unsupported authentication type: {}",
                self.authtype
            )),
        }
    }

    /// Calculate MD5 authentication code
    fn calculate_md5_authcode(
        &self,
        _session_id: u32,
        _session_seq: u32,
        _password: &[u8],
        _msg_data: &[u8],
    ) -> Result<[u8; 16], String> {
        // TODO: Implement proper MD5 authentication code calculation
        // For now, return zeros to get the code compiling
        Ok([0u8; 16])
    }

    /// Calculate MD2 authentication code (simplified implementation)
    fn calculate_md2_authcode(
        &self,
        _session_id: u32,
        _session_seq: u32,
        _password: &[u8],
        _msg_data: &[u8],
    ) -> Result<[u8; 16], String> {
        // MD2 is deprecated and rarely used, return error for now
        Err("MD2 authentication not implemented".to_string())
    }

    /// Calculate password authentication code (straight password)
    fn calculate_password_authcode(&self, password: &[u8]) -> Result<[u8; 16], String> {
        let mut authcode = [0u8; 16];
        let pwd_len = std::cmp::min(password.len(), 16);
        authcode[..pwd_len].copy_from_slice(&password[..pwd_len]);
        Ok(authcode)
    }

    /// Verify authentication code
    pub fn verify_authcode(
        &self,
        expected: &[u8; 16],
        session_id: u32,
        session_seq: u32,
        password: &[u8],
        msg_data: &[u8],
    ) -> Result<bool, String> {
        let mut auth_copy = self.clone();
        let calculated =
            auth_copy.calculate_authcode(session_id, session_seq, password, msg_data)?;
        Ok(calculated == *expected)
    }
}

/// Parse authentication capabilities from BMC response
pub fn parse_auth_capabilities(data: &[u8]) -> Result<HashMap<String, u8>, String> {
    if data.len() < 8 {
        return Err("Authentication capabilities response too short".to_string());
    }

    let mut caps = HashMap::new();

    // Byte 0: Completion code (should be 0x00)
    if data[0] != 0x00 {
        return Err(format!(
            "Authentication capabilities failed: 0x{:02x}",
            data[0]
        ));
    }

    // Byte 1: Channel number
    caps.insert("channel".to_string(), data[1] & 0x0f);

    // Byte 2: Authentication type support
    caps.insert("auth_support".to_string(), data[2]);

    // Byte 3: Authentication status
    caps.insert("auth_status".to_string(), data[3]);

    // Byte 4: Channel privilege level
    caps.insert("privilege_level".to_string(), data[4] & 0x0f);

    // Bytes 5-7: OEM info
    let oem_id = ((data[7] as u32) << 16) | ((data[6] as u32) << 8) | (data[5] as u32);
    caps.insert("oem_id".to_string(), oem_id as u8);

    Ok(caps)
}

/// Check if authentication type is supported
pub fn is_auth_type_supported(auth_support: u8, auth_type: u8) -> bool {
    match auth_type {
        IPMI_SESSION_AUTHTYPE_NONE => (auth_support & 0x01) != 0,
        IPMI_SESSION_AUTHTYPE_MD2 => (auth_support & 0x02) != 0,
        IPMI_SESSION_AUTHTYPE_MD5 => (auth_support & 0x04) != 0,
        IPMI_SESSION_AUTHTYPE_PASSWORD => (auth_support & 0x10) != 0,
        IPMI_SESSION_AUTHTYPE_OEM => (auth_support & 0x20) != 0,
        _ => false,
    }
}

/// Get available authentication types as string
pub fn get_auth_types_string(auth_support: u8) -> String {
    let mut types = Vec::new();

    if is_auth_type_supported(auth_support, IPMI_SESSION_AUTHTYPE_NONE) {
        types.push("NONE");
    }
    if is_auth_type_supported(auth_support, IPMI_SESSION_AUTHTYPE_MD2) {
        types.push("MD2");
    }
    if is_auth_type_supported(auth_support, IPMI_SESSION_AUTHTYPE_MD5) {
        types.push("MD5");
    }
    if is_auth_type_supported(auth_support, IPMI_SESSION_AUTHTYPE_PASSWORD) {
        types.push("PASSWORD");
    }
    if is_auth_type_supported(auth_support, IPMI_SESSION_AUTHTYPE_OEM) {
        types.push("OEM");
    }

    types.join(" ")
}
