/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
/// RMCP Protocol Implementation
///
/// Remote Management Control Protocol - used for IPMI over LAN
/// Based on reference/ipmitool-c/src/plugins/lan/rmcp.h

// RMCP Constants
pub const RMCP_VERSION_1: u8 = 0x06;
pub const RMCP_UDP_PORT: u16 = 623; // 0x26f
pub const RMCP_UDP_SECURE_PORT: u16 = 664; // 0x298

// RMCP Type flags
pub const RMCP_TYPE_MASK: u8 = 0x80;
pub const RMCP_TYPE_NORM: u8 = 0x00;
pub const RMCP_TYPE_ACK: u8 = 0x01;

// RMCP Classes
pub const RMCP_CLASS_MASK: u8 = 0x1f;
pub const RMCP_CLASS_ASF: u8 = 0x06;
pub const RMCP_CLASS_IPMI: u8 = 0x07;
pub const RMCP_CLASS_OEM: u8 = 0x08;

/// RMCP Header Structure
/// Matches C struct rmcp_hdr exactly
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct RmcpHeader {
    pub version: u8,
    pub reserved: u8,
    pub sequence: u8,
    pub class: u8,
}

impl Default for RmcpHeader {
    fn default() -> Self {
        Self {
            version: RMCP_VERSION_1,
            reserved: 0,
            sequence: 0,
            class: RMCP_CLASS_IPMI,
        }
    }
}

impl RmcpHeader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_ipmi(sequence: u8) -> Self {
        Self {
            version: RMCP_VERSION_1,
            reserved: 0,
            sequence,
            class: RMCP_CLASS_IPMI,
        }
    }

    pub fn new_asf(sequence: u8) -> Self {
        Self {
            version: RMCP_VERSION_1,
            reserved: 0,
            sequence,
            class: RMCP_CLASS_ASF,
        }
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        [self.version, self.reserved, self.sequence, self.class]
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 4 {
            return Err("RMCP header too short".to_string());
        }

        Ok(Self {
            version: data[0],
            reserved: data[1],
            sequence: data[2],
            class: data[3],
        })
    }

    pub fn is_ack(&self) -> bool {
        (self.class & RMCP_TYPE_ACK) != 0
    }

    pub fn is_ipmi(&self) -> bool {
        (self.class & RMCP_CLASS_MASK) == RMCP_CLASS_IPMI
    }

    pub fn is_asf(&self) -> bool {
        (self.class & RMCP_CLASS_MASK) == RMCP_CLASS_ASF
    }
}

/// ASF Header Structure for ping/pong messages
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct AsfHeader {
    pub iana: u32,    // Intel assigned number
    pub msg_type: u8, // Message type
    pub msg_tag: u8,  // Message tag
    pub reserved: u8, // Reserved
    pub data_len: u8, // Data length
}

impl Default for AsfHeader {
    fn default() -> Self {
        Self {
            iana: 0x000011be, // Intel IANA
            msg_type: 0,
            msg_tag: 0,
            reserved: 0,
            data_len: 0,
        }
    }
}

impl AsfHeader {
    pub fn new_ping(tag: u8) -> Self {
        Self {
            iana: 0x000011be,
            msg_type: 0x80, // Ping
            msg_tag: tag,
            reserved: 0,
            data_len: 0,
        }
    }

    pub fn to_bytes(&self) -> [u8; 8] {
        let mut bytes = [0u8; 8];
        bytes[0..4].copy_from_slice(&self.iana.to_le_bytes());
        bytes[4] = self.msg_type;
        bytes[5] = self.msg_tag;
        bytes[6] = self.reserved;
        bytes[7] = self.data_len;
        bytes
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 8 {
            return Err("ASF header too short".to_string());
        }

        let iana = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

        Ok(Self {
            iana,
            msg_type: data[4],
            msg_tag: data[5],
            reserved: data[6],
            data_len: data[7],
        })
    }

    pub fn is_pong(&self) -> bool {
        self.msg_type == 0x81
    }
}

/// RMCP Pong Response Structure
#[repr(C, packed)]
#[derive(Debug, Clone)]
pub struct RmcpPong {
    pub rmcp: RmcpHeader,
    pub asf: AsfHeader,
    pub iana: u32,
    pub oem: u32,
    pub sup_entities: u8,
    pub sup_interact: u8,
    pub reserved: [u8; 6],
}

impl RmcpPong {
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 26 {
            return Err("RMCP pong response too short".to_string());
        }

        let rmcp = RmcpHeader::from_bytes(&data[0..4])?;
        let asf = AsfHeader::from_bytes(&data[4..12])?;

        let iana = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        let oem = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        let sup_entities = data[20];
        let sup_interact = data[21];
        let mut reserved = [0u8; 6];
        reserved.copy_from_slice(&data[22..28]);

        Ok(Self {
            rmcp,
            asf,
            iana,
            oem,
            sup_entities,
            sup_interact,
            reserved,
        })
    }
}

/// Handle RMCP protocol messages
pub fn handle_rmcp_message(data: &[u8]) -> Result<(), String> {
    if data.len() < 4 {
        return Err("RMCP message too short".to_string());
    }

    let header = RmcpHeader::from_bytes(data)?;

    match header.class & RMCP_CLASS_MASK {
        RMCP_CLASS_IPMI => {
            // Handle IPMI message
            Ok(())
        }
        RMCP_CLASS_ASF => {
            // Handle ASF message (ping/pong)
            if data.len() >= 12 {
                let asf = AsfHeader::from_bytes(&data[4..])?;
                if asf.is_pong() {
                    // Handle pong response
                }
            }
            Ok(())
        }
        _ => Err(format!(
            "Unknown RMCP class: {}",
            header.class & RMCP_CLASS_MASK
        )),
    }
}

/// Create RMCP ping packet
pub fn create_ping_packet(sequence: u8, tag: u8) -> Vec<u8> {
    let mut packet = Vec::new();

    // RMCP header
    let rmcp = RmcpHeader::new_asf(sequence);
    packet.extend_from_slice(&rmcp.to_bytes());

    // ASF header
    let asf = AsfHeader::new_ping(tag);
    packet.extend_from_slice(&asf.to_bytes());

    packet
}
