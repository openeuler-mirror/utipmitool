/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(unexpected_cfgs)]
//use super::ipmi_oem::*;
use super::ipmi::*;
use crate::error::IpmiResult;
use crate::ipmi::context::IpmiContext as NewIpmiContext;

/*
 * An enumeration that describes every possible session state for
 * an IPMIv2 / RMCP+ session.
 */
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LanplusSessionState {
    Presession = 0,
    OpenSessionSent,
    OpenSessionReceived,
    Rakp1Sent,
    Rakp2Received,
    Rakp3Sent,
    Active,
    CloseSent,
}

pub const IPMI_DEFAULT_PAYLOAD_SIZE: u16 = 25;

pub const IPMI_AUTHCODE_BUFFER_SIZE: usize = 20;
pub const IPMI_SIK_BUFFER_SIZE: usize = IPMI_MAX_MD_SIZE;
pub const IPMI_KG_BUFFER_SIZE: usize = 21; // key plus null byte

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CipherSuiteIds {
    IpmiLanplusCipherSuite0 = 0,
    IpmiLanplusCipherSuite1 = 1,
    IpmiLanplusCipherSuite2 = 2,
    IpmiLanplusCipherSuite3 = 3,
    IpmiLanplusCipherSuite4 = 4,
    IpmiLanplusCipherSuite5 = 5,
    IpmiLanplusCipherSuite6 = 6,
    IpmiLanplusCipherSuite7 = 7,
    IpmiLanplusCipherSuite8 = 8,
    IpmiLanplusCipherSuite9 = 9,
    IpmiLanplusCipherSuite10 = 10,
    IpmiLanplusCipherSuite11 = 11,
    IpmiLanplusCipherSuite12 = 12,
    IpmiLanplusCipherSuite13 = 13,
    IpmiLanplusCipherSuite14 = 14,
    #[cfg(feature = "crypto-sha256")]
    IpmiLanplusCipherSuite15 = 15,
    #[cfg(feature = "crypto-sha256")]
    IpmiLanplusCipherSuite16 = 16,
    #[cfg(feature = "crypto-sha256")]
    IpmiLanplusCipherSuite17 = 17,
    IpmiLanplusCipherSuiteReserved = 0xff,
}

#[derive(Debug, Clone)]
pub struct CipherSuiteInfo {
    pub cipher_suite_id: CipherSuiteIds,
    pub auth_alg: u8,
    pub integrity_alg: u8,
    pub crypt_alg: u8,
    pub iana: u32,
}

#[derive(Clone)]
pub struct IpmiSessionParams {
    pub hostname: String,
    pub username: [u8; 17],
    pub authcode_set: [u8; IPMI_AUTHCODE_BUFFER_SIZE + 1],
    pub authtype_set: u8,
    pub privlvl: u8,
    pub cipher_suite_id: CipherSuiteIds,
    pub sol_escape_char: char,
    pub password: i32,
    pub port: i32,
    pub retry: i32,
    pub timeout: u32,
    pub kg: [u8; IPMI_KG_BUFFER_SIZE],
    pub lookupbit: u8,
}
impl Default for IpmiSessionParams {
    fn default() -> Self {
        IpmiSessionParams {
            hostname: String::new(),
            username: [0u8; 17],
            authcode_set: [0u8; IPMI_AUTHCODE_BUFFER_SIZE + 1],
            authtype_set: 0,
            privlvl: 0,
            cipher_suite_id: CipherSuiteIds::IpmiLanplusCipherSuite0,
            sol_escape_char: '\0',
            password: 0,
            port: 0,
            retry: 0,
            timeout: 0,
            kg: [0u8; IPMI_KG_BUFFER_SIZE],
            lookupbit: 0,
        }
    }
}

pub const IPMI_AUTHSTATUS_PER_MSG_DISABLED: u8 = 0x10;
pub const IPMI_AUTHSTATUS_PER_USER_DISABLED: u8 = 0x08;
pub const IPMI_AUTHSTATUS_NONNULL_USERS_ENABLED: u8 = 0x04;
pub const IPMI_AUTHSTATUS_NULL_USERS_ENABLED: u8 = 0x02;
pub const IPMI_AUTHSTATUS_ANONYMOUS_USERS_ENABLED: u8 = 0x01;

pub struct IpmiSession {
    pub active: i32,
    pub session_id: u32,
    pub in_seq: u32,
    pub out_seq: u32,

    pub authcode: [u8; IPMI_AUTHCODE_BUFFER_SIZE + 1],
    pub challenge: [u8; 16],
    pub authtype: u8,
    pub authstatus: u8,
    pub authextra: u8,
    pub timeout: u32,

    pub addr: std::net::SocketAddr,
    pub addrlen: u32,

    // IPMI v2 / RMCP+ session specific data
    pub v2_data: IpmiV2Data,

    // Serial Over Lan session specific data
    pub sol_data: SolData,
}

pub struct IpmiV2Data {
    pub session_state: LanplusSessionState,

    // Agreed upon algorithms for the session
    pub requested_auth_alg: u8,
    pub requested_integrity_alg: u8,
    pub requested_crypt_alg: u8,
    pub auth_alg: u8,
    pub integrity_alg: u8,
    pub crypt_alg: u8,
    pub max_priv_level: u8,

    pub console_id: u32,
    pub bmc_id: u32,

    // RAKP message values
    pub console_rand: [u8; 16],
    pub bmc_rand: [u8; 16],
    pub bmc_guid: [u8; 16],
    pub requested_role: u8,
    pub rakp2_return_code: u8,

    pub sik: [u8; IPMI_SIK_BUFFER_SIZE],
    pub sik_len: u8,
    pub kg: [u8; IPMI_KG_BUFFER_SIZE],
    pub k1: [u8; IPMI_MAX_MD_SIZE],
    pub k1_len: u8,
    pub k2: [u8; IPMI_MAX_MD_SIZE],
    pub k2_len: u8,
}

pub struct SolData {
    pub max_inbound_payload_size: u16,
    pub max_outbound_payload_size: u16,
    pub port: u16,
    pub sequence_number: u8,
    pub last_received_sequence_number: u8,
    pub last_received_byte_count: u8,
    //pub sol_input_handler: Option<fn(&IpmiRS)>, // 函数指针
}
// pub struct IpmiCmd {
//     pub func: Option<fn(&mut dyn IpmiIntf, i32, Vec<String>) -> i32>,
//     pub name: String,
//     pub desc: String,
// }

pub struct IpmiIntfSupport {
    pub name: String,
    pub supported: i32,
}

// pub struct IpmiContext {
//     pub target_ipmb_addr: u8,
//     pub my_addr: u32,
//     pub target_addr: u32,
//     pub target_lun: u8,
//     pub target_channel: u8,
//     pub transit_addr: u32,
//     pub transit_channel: u8,
//     pub max_request_data_size: u16,
//     pub max_response_data_size: u16,
// }

// ====================
// Legacy compatibility for IpmiContext
// 为了保持向后兼容，我们创建一个类型别名和转换层
// ====================

pub type IpmiContext = NewIpmiContext;

pub trait IpmiIntf {
    //IpmiInterface
    fn context(&mut self) -> &mut IpmiContext;

    fn setup(&mut self) -> IpmiResult<()>;
    fn open(&mut self) -> IpmiResult<()>;
    fn close(&mut self);

    fn sendrecv(&mut self, req: &IpmiRq) -> Option<IpmiRs>;
    fn send_sol(&mut self, payload: &IpmiV2Payload) -> Option<IpmiRs>;
    fn recv_sol(&mut self) -> Option<IpmiRs>;
    fn keepalive(&mut self) -> IpmiResult<()>;

    fn set_my_addr(&mut self, addr: u8) -> IpmiResult<()>;

    //lan和lanplus才有的接口，默认实现
    fn set_max_request_size(&mut self, _size: u16) {}
    fn set_max_response_size(&mut self, _size: u16) {}

    // fn get_max_request_data_size(&self) -> u16;
    // fn get_max_response_data_size(&self) -> u16;

    // fn bridge_to_sensor(&mut self, addr: u8, chan: u8) -> bool;

    // fn target_ipmb_addr(&self) -> u8;

    // fn target_addr(&self) -> u32;
    // fn target_channel(&self) -> u8;
    // fn set_target_addr(&mut self, addr: u32);
    // fn set_target_channel(&mut self, channel: u8);
    // fn manufacturer_id(&mut self) -> IPMI_OEM;
}

// 扩展 trait，包含泛型方法
pub trait IpmiIntfExt: IpmiIntf {
    fn with_context<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut IpmiContext) -> R,
    {
        let ctx = self.context();
        f(ctx)
    }
}
// 为所有实现了 IpmiIntf 的类型自动实现扩展 trait
impl<T: IpmiIntf> IpmiIntfExt for T {}
