/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(dead_code)]

use super::auth::IpmiAuth;
use super::rmcp::{RmcpHeader, RMCP_UDP_PORT};
use crate::error::{IpmiError, IpmiResult};
/// IPMI LAN Interface Implementation
///
/// Based on reference/ipmitool-c/src/plugins/lan/lan.c
/// Implements IPMI v1.5 LAN interface over UDP/RMCP
use crate::ipmi::intf::{IpmiContext, IpmiIntf};
use crate::ipmi::ipmi::{IpmiRq, IpmiRs, IpmiRsPayload, IpmiV2Payload, IPMI_BUF_SIZE};

use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::Duration;

// IPMI LAN Constants
const IPMI_LAN_TIMEOUT: u64 = 2; // seconds
const IPMI_LAN_RETRY: u32 = 4;
const IPMI_LAN_CHANNEL_E: u8 = 0x0e;

// IPMI LAN size limits (from C reference)
const IPMI_LAN_MAX_REQUEST_SIZE: u16 = 38; // 45 - 7
const IPMI_LAN_MAX_RESPONSE_SIZE: u16 = 34; // 42 - 8

// IPMI Commands used by LAN interface
const IPMI_GET_CHANNEL_AUTH_CAP: u8 = 0x38;
const IPMI_GET_SESSION_CHALLENGE: u8 = 0x39;
const IPMI_ACTIVATE_SESSION: u8 = 0x3a;
const IPMI_SET_SESSION_PRIVILEGE: u8 = 0x3b;
const IPMI_CLOSE_SESSION: u8 = 0x3c;

/// IPMI LAN Session Information
#[derive(Debug, Clone, Default)]
pub struct IpmiLanSession {
    pub active: bool,
    pub session_id: u32,
    pub in_seq: u32,
    pub out_seq: u32,
    pub auth: IpmiAuth,
    pub challenge: [u8; 16],
    pub timeout: u64,
    pub privilege_level: u8,
}

impl IpmiLanSession {
    pub fn new() -> Self {
        Self {
            timeout: IPMI_LAN_TIMEOUT,
            ..Default::default()
        }
    }
}

/// IPMI LAN Interface
pub struct IpmiLanIntf {
    pub context: IpmiContext,
    pub socket: Option<UdpSocket>,
    pub target_addr: SocketAddr,
    pub session: IpmiLanSession,
    pub hostname: String,
    pub username: String,
    pub password: String,
    pub privilege_level: u8,
    pub timeout: u64,
    pub retry_count: u32,
}

impl IpmiLanIntf {
    /// Create new LAN interface
    pub fn new(hostname: String) -> Result<Self, String> {
        let target_addr = format!("{}:{}", hostname, RMCP_UDP_PORT)
            .to_socket_addrs()
            .map_err(|e| format!("Failed to resolve hostname {}: {}", hostname, e))?
            .next()
            .ok_or_else(|| format!("No address found for hostname: {}", hostname))?;

        Ok(Self {
            context: IpmiContext::default(),
            socket: None,
            target_addr,
            session: IpmiLanSession::new(),
            hostname,
            username: String::new(),
            password: String::new(),
            privilege_level: 2, // USER level
            timeout: IPMI_LAN_TIMEOUT,
            retry_count: IPMI_LAN_RETRY,
        })
    }

    /// Set authentication credentials
    pub fn set_credentials(&mut self, username: String, password: String) {
        self.username = username;
        self.password = password;
    }

    /// Set privilege level
    pub fn set_privilege_level(&mut self, level: u8) {
        self.privilege_level = level;
    }

    /// Send IPMI command before session is established
    fn send_command_pre_session(&self, req: &IpmiRq) -> Result<IpmiRs, String> {
        let socket = self
            .socket
            .as_ref()
            .ok_or_else(|| "Socket not initialized".to_string())?;

        // Build RMCP + IPMI packet for pre-session command
        let packet = self.build_pre_session_packet(req)?;

        // Send with retry logic
        for attempt in 0..self.retry_count {
            // Send packet
            socket
                .send_to(&packet, self.target_addr)
                .map_err(|e| format!("Failed to send packet: {}", e))?;

            // Wait for response
            socket
                .set_read_timeout(Some(Duration::from_secs(self.timeout)))
                .map_err(|e| format!("Failed to set socket timeout: {}", e))?;

            let mut buffer = [0u8; 1024];
            match socket.recv_from(&mut buffer) {
                Ok((len, addr)) => {
                    if addr == self.target_addr {
                        return self.parse_response(&buffer[..len]);
                    }
                }
                Err(e) if attempt < self.retry_count - 1 => {
                    log::error!("Attempt {} failed: {}, retrying...", attempt + 1, e);
                    continue;
                }
                Err(e) => return Err(format!("Failed to receive response: {}", e)),
            }
        }

        Err("Failed to get response after all retries".to_string())
    }

    /// Build RMCP packet for pre-session commands
    fn build_pre_session_packet(&self, req: &IpmiRq) -> Result<Vec<u8>, String> {
        let mut packet = Vec::new();

        // RMCP header
        let rmcp = RmcpHeader::new_ipmi(0xff); // No sequence for pre-session
        packet.extend_from_slice(&rmcp.to_bytes());

        // IPMI session header (all zeros for pre-session)
        packet.extend_from_slice(&[0u8; 9]); // Auth type + seq + session ID + auth code placeholder

        // IPMI message
        packet.push(0x20); // rsSA (BMC slave address)
        packet.push(req.msg.netfn() | (req.msg.lun() << 2));
        packet.push(0); // Checksum placeholder
        packet.push(0x81); // rqSA (software ID)
        packet.push(req.msg.lun() << 2); // rqSEQ + rqLUN
        packet.push(req.msg.cmd);

        // Add request data
        if req.msg.data_len > 0 {
            let data =
                unsafe { std::slice::from_raw_parts(req.msg.data, req.msg.data_len as usize) };
            packet.extend_from_slice(data);
        }

        // Calculate and update checksums
        self.update_checksums(&mut packet);

        Ok(packet)
    }

    /// Update IPMI message checksums
    fn update_checksums(&self, packet: &mut Vec<u8>) {
        if packet.len() < 16 {
            return;
        }

        // First checksum (header checksum)
        let hdr_start = 13; // After RMCP + session header
        let hdr_sum = (0x100u16 - (packet[hdr_start] as u16 + packet[hdr_start + 1] as u16)) as u8;
        packet[hdr_start + 2] = hdr_sum;

        // Second checksum (message checksum)
        if packet.len() > 16 {
            let msg_start = 16;
            let mut sum = 0u8;
            for item in packet.iter().skip(msg_start) {
                sum = sum.wrapping_add(*item);
            }
            let msg_sum = (0x100u16 as u8).wrapping_sub(sum);
            packet.push(msg_sum);
        }
    }

    /// Parse IPMI response packet
    fn parse_response(&self, data: &[u8]) -> Result<IpmiRs, String> {
        if data.len() < 16 {
            return Err("Response packet too short".to_string());
        }

        // Parse RMCP header
        let rmcp = RmcpHeader::from_bytes(&data[0..4])?;
        if !rmcp.is_ipmi() {
            return Err("Not an IPMI response".to_string());
        }

        // Skip session header (9 bytes) for pre-session commands
        let msg_start = 13;
        if data.len() < msg_start + 7 {
            return Err("IPMI message too short".to_string());
        }

        // Parse IPMI message
        let _rs_sa = data[msg_start];
        let netfn_lun = data[msg_start + 1];
        let _checksum1 = data[msg_start + 2];
        let _rq_sa = data[msg_start + 3];
        let _seq_lun = data[msg_start + 4];
        let cmd = data[msg_start + 5];
        let ccode = data[msg_start + 6];

        // Extract response data
        let data_start = msg_start + 7;
        let data_len = if data.len() > data_start + 1 {
            data.len() - data_start - 1 // Subtract final checksum
        } else {
            0
        };

        let mut rsp = IpmiRs {
            ccode,
            data: [0; IPMI_BUF_SIZE],
            data_len: data_len as i32,
            msg: Default::default(),
            session: Default::default(),
            payload: IpmiRsPayload::IpmiResponse {
                rq_addr: 0x81,
                netfn: netfn_lun >> 2,
                rq_lun: 0,
                rs_addr: 0x20,
                rq_seq: 0,
                rs_lun: netfn_lun & 0x03,
                cmd,
            },
        };

        if data_len > 0 {
            // Copy response data to the data array
            rsp.data[..data_len].copy_from_slice(&data[data_start..data_start + data_len]);
        }

        Ok(rsp)
    }
}

impl IpmiIntf for IpmiLanIntf {
    fn context(&mut self) -> &mut IpmiContext {
        &mut self.context
    }

    fn setup(&mut self) -> IpmiResult<()> {
        // 设置 LAN 接口的默认参数
        self.context.protocol.max_request_data_size = IPMI_LAN_MAX_REQUEST_SIZE;
        self.context.protocol.max_response_data_size = IPMI_LAN_MAX_RESPONSE_SIZE;

        Ok(())
    }

    fn open(&mut self) -> IpmiResult<()> {
        if self.socket.is_some() {
            return Ok(()); // 已经打开
        }

        // 创建 UDP socket
        let local_addr: SocketAddr = "0.0.0.0:0"
            .parse()
            .map_err(|e| IpmiError::System(format!("Invalid local address: {}", e)))?;

        let socket = UdpSocket::bind(local_addr)
            .map_err(|e| IpmiError::Network(format!("Failed to bind UDP socket: {}", e)))?;

        // 连接到远程地址
        if !self.hostname.is_empty() {
            let remote_addr = format!("{}:{}", self.hostname, RMCP_UDP_PORT);
            let remote: SocketAddr = remote_addr.parse().map_err(|e| {
                IpmiError::Network(format!("Invalid remote address {}: {}", remote_addr, e))
            })?;

            socket.connect(remote).map_err(|e| {
                IpmiError::Network(format!("Failed to connect to {}: {}", remote_addr, e))
            })?;
        }

        self.socket = Some(socket);
        Ok(())
    }

    fn close(&mut self) {
        // 关闭会话（如果有活动会话）
        if self.session.session_id != 0 {
            // TODO: 发送关闭会话命令
            self.session.session_id = 0;
        }

        // 关闭 socket
        self.socket = None;
    }

    fn sendrecv(&mut self, req: &IpmiRq) -> Option<IpmiRs> {
        // Use pre-session method for now (simplified implementation)
        match self.send_command_pre_session(req) {
            Ok(rsp) => Some(rsp),
            Err(e) => {
                log::error!("Failed to send command: {}", e);
                None
            }
        }
    }

    fn send_sol(&mut self, _payload: &IpmiV2Payload) -> Option<IpmiRs> {
        // SOL not supported in IPMI v1.5 LAN
        None
    }

    fn recv_sol(&mut self) -> Option<IpmiRs> {
        // SOL not supported in IPMI v1.5 LAN
        None
    }

    fn keepalive(&mut self) -> IpmiResult<()> {
        if self.session.session_id == 0 {
            return Ok(()); // 没有活动会话
        }

        // TODO: 发送 keep-alive 消息以维持会话
        // 通常发送 Get Channel Authentication Capabilities 命令
        Ok(())
    }

    fn set_my_addr(&mut self, addr: u8) -> IpmiResult<()> {
        self.context.base.my_addr = addr as u32;
        Ok(())
    }

    fn set_max_request_size(&mut self, size: u16) {
        self.context.protocol.max_request_data_size =
            std::cmp::min(size, IPMI_LAN_MAX_REQUEST_SIZE);
    }

    fn set_max_response_size(&mut self, size: u16) {
        self.context.protocol.max_response_data_size =
            std::cmp::min(size, IPMI_LAN_MAX_RESPONSE_SIZE);
    }
}
