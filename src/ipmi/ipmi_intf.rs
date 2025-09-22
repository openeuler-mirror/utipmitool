/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */


//use super::ipmi_oem::*;
use super::ipmi::*;
use crate::ipmi::ipmi_constants::*;
use std::{error::Error, thread::sleep};

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


//#[derive(Default)]
pub struct IpmiIntf{ //<'a> 
    pub name: String,// &'a str,
    pub desc: String, //&'a str,
    pub devfile: Option<String>,
    pub fd: i32,
    pub opened: i32,
    pub abort: i32,
    pub noanswer: i32,
    pub picmg_avail: i32,
    pub vita_avail: i32,
    pub manufacturer_id: IPMI_OEM,
    pub ai_family: i32,

    pub ssn_params: IpmiSessionParams,
    pub session: Option<Box<IpmiSession>>,
    //pub oem: Option<Box<IpmiOemHandle>>,
    //pub cmdlist: Option<Box<IpmiCmd>>,
    pub target_ipmb_addr: u8,
    pub my_addr: u32,
    pub target_addr: u32,
    pub target_lun: u8,
    pub target_channel: u8,
    pub transit_addr: u32,
    pub transit_channel: u8,
    pub max_request_data_size: u16,
    pub max_response_data_size: u16,

    pub devnum: u8,

    // pub setup: Option<fn(&mut IpmiIntf) -> i32>,
    // pub open: Option<fn(&mut IpmiIntf) -> i32>,
    // pub close: Option<fn(&mut IpmiIntf)>,
    // pub sendrecv: Option<fn(&mut IpmiIntf, &IpmiRequest) -> Option<Box<IpmiResponse>>>,
    // pub recv_sol: Option<fn(&mut IpmiIntf) -> Option<Box<IpmiResponse>>>,
    // pub send_sol: Option<fn(&mut IpmiIntf, &IpmiV2Payload) -> Option<Box<IpmiResponse>>>,
    // pub keepalive: Option<fn(&mut IpmiIntf) -> i32>,
    // pub set_my_addr: Option<fn(&mut IpmiIntf, u8) -> i32>,
    // pub set_max_request_data_size: Option<fn(&mut IpmiIntf, u16)>,
    // pub set_max_response_data_size: Option<fn(&mut IpmiIntf, u16)>,

    pub curr_seq: u8,
    //接口对象
    pub transport: Box<dyn Transport>,
}

impl IpmiIntf {

    // pub fn new() -> Self{
       
    // }
    // pub fn new(transport: Box<dyn Transport>) -> Self {
        
    // }

    pub fn new(transport: Box<dyn Transport>) -> Self {
        IpmiIntf{
            name: String::new(),
            desc: String::new(),
            devfile: None,
            fd: 0,
            opened: 0,
            abort: 0,
            noanswer: 0,
            picmg_avail: 0,
            vita_avail: 0,
            manufacturer_id: IPMI_OEM::Unknown,
            ai_family: 0,
            ssn_params: IpmiSessionParams::default(),
            session: None,
            //oem: None,
            //cmdlist: None,
            target_ipmb_addr: 0,
            my_addr: 0,
            target_addr: 0,
            target_lun: 0,
            target_channel: 0,
            transit_addr: 0,
            transit_channel: 0,
            max_request_data_size: 0,
            max_response_data_size: 0,
            devnum: 0,
            curr_seq: 0,
            transport: transport,
        }
    }

    pub fn setup(&mut self) -> i32{
       self.transport.setup()
    }
    pub fn open(&mut self) -> i32{
        self.transport.open()
    }
    pub fn close(&mut self){
        self.transport.close()
    }
    pub fn sendrecv(&mut self, req: &IpmiRq) -> Option<IpmiRs>{
        self.transport.sendrecv(req)
    }
    pub fn set_my_addr(&mut self, addr: u8) -> i32{
        self.transport.set_my_addr(addr)
    }
}


pub trait Transport {
    fn setup(&mut self) -> i32;
    fn open(&mut self) -> i32;
    fn close(&mut self);
    
    fn sendrecv(&mut self, req: &IpmiRq) -> Option<IpmiRs>;
    fn send_sol(&mut self, payload: &IpmiV2Payload) -> Option<IpmiRs>;
    fn recv_sol(&mut self) -> Option<IpmiRs>;
    fn keepalive(&mut self) -> i32;

    fn set_my_addr(&mut self, addr: u8) -> i32;
    fn set_max_request_size(&mut self, size: u16);
    fn set_max_response_size(&mut self, size: u16);
}

impl IpmiIntf {
    pub fn session_set_hostname(&mut self, hostname: String) {
        self.ssn_params.hostname = hostname;
    }

    pub fn session_set_username(&mut self, username: String) {
        self.ssn_params.username.fill(0);
        let len = username.len().min(16);
        self.ssn_params.username[..len].copy_from_slice(&username.as_bytes()[..len]);
        
    }

    pub fn session_set_password(&mut self, password: Option<&str>) {
        self.ssn_params.authcode_set.fill(0);
        if let Some(pass) = password {
            self.ssn_params.password = 1;
            let len = pass.len().min(IPMI_AUTHCODE_BUFFER_SIZE);
            self.ssn_params.authcode_set[..len].copy_from_slice(&pass.as_bytes()[..len]); 
        } else {
            self.ssn_params.password = 0;
        }
    }

    pub fn session_set_privlvl(&mut self, level: u8) {
        self.ssn_params.privlvl = level;
    }

    pub fn session_set_lookupbit(&mut self, lookupbit: u8) {
        self.ssn_params.lookupbit = lookupbit;  
    }

    pub fn session_set_cipher_suite_id(&mut self, cipher_suite_id: CipherSuiteIds) {
        self.ssn_params.cipher_suite_id = cipher_suite_id;
    }

    pub fn session_set_sol_escape_char(&mut self, sol_escape_char: char) {
        self.ssn_params.sol_escape_char = sol_escape_char;
    }

    pub fn session_set_kgkey(&mut self, kgkey: &[u8]) {
        self.ssn_params.kg[..IPMI_KG_BUFFER_SIZE].copy_from_slice(kgkey);
    }

    pub fn session_set_port(&mut self, port: i32) {
        self.ssn_params.port = port;
    }

    pub fn session_set_authtype(&mut self, authtype: u8) {
        if authtype == IPMI_SESSION_AUTHTYPE_NONE {
            self.ssn_params.authcode_set.fill(0);
            self.ssn_params.password = 0;
        }
        self.ssn_params.authtype_set = authtype;
    }

    pub fn session_set_timeout(&mut self, timeout: u32) {
        self.ssn_params.timeout = timeout;
    }

    pub fn session_set_retry(&mut self, retry: i32) {
        self.ssn_params.retry = retry;
    }

    pub fn session_cleanup(&mut self) {
        self.session = None;
    }

    pub fn cleanup(&mut self) {
        // TODO: ipmi_sdr_list_empty();
        if self.session.is_none() {
            return;
        }
        self.session = None;
    }
    pub fn get_max_request_data_size(&self) -> u16 {
        let mut size = self.max_request_data_size as i16;
        let bridging_level = self.get_bridging_level();

        // check if request size is not specified
        if size == 0 {
            // See comment in original C code for rationale
            size = IPMI_DEFAULT_PAYLOAD_SIZE as i16;

            // check if message is forwarded
            if bridging_level > 0 {
                // add Send Message request size
                size += 8;
            }
        }

        // check if message is forwarded
        if bridging_level > 0 {
            // subtract send message request size
            size -= 8;

            // Check that forwarded request size is not greater than the default payload size
            if size > IPMI_DEFAULT_PAYLOAD_SIZE as i16 {
                size = IPMI_DEFAULT_PAYLOAD_SIZE as i16;
            }

            // check for double bridging
            if bridging_level == 2 {
                // subtract inner send message request size
                size -= 8;
            }
        }

        // check for underflow
        if size < 0 {
            return 0;
        }

        size as u16
    }

    pub fn get_max_response_data_size(&self) -> u16 {
        let mut size = self.max_response_data_size as i16;
        let bridging_level = self.get_bridging_level();

        // check if response size is not specified
        if size == 0 {
            // See comment in original C code for rationale
            size = IPMI_DEFAULT_PAYLOAD_SIZE as i16; // response length with subtracted header and checksum byte

            // check if message is forwarded
            if bridging_level > 0 {
                // add Send Message header size
                size += 7;
            }
        }

        // check if message is forwarded
        if bridging_level > 0 {
            // subtract the internal message header size
            size -= 8;

            // Check that forwarded response is not greater than the default payload size
            if size > IPMI_DEFAULT_PAYLOAD_SIZE as i16 {
                size = IPMI_DEFAULT_PAYLOAD_SIZE as i16;
            }

            // check for double bridging
            if bridging_level == 2 {
                // subtract inner send message header size
                size -= 8;
            }
        }

        // check for underflow
        if size < 0 {
            return 0;
        }

        size as u16
    }
  
    pub fn get_bridging_level(&self) -> u8 {
        if self.target_addr > 0 && self.target_addr != self.my_addr {
            if self.transit_addr > 0 && 
               (self.transit_addr != self.target_addr || 
                self.transit_channel != self.target_channel) {
                2
            } else {
                1
            }
        } else {
            0
        }
    }

    pub fn set_max_request_data_size(&mut self, size: u16) {
        if size < IPMI_DEFAULT_PAYLOAD_SIZE {
            // TODO: Log error
            return;
        }
        // TODO: Call transport trait method if available
        self.max_request_data_size = size;
    }

    pub fn set_max_response_data_size(&mut self, size: u16) {
        if size < IPMI_DEFAULT_PAYLOAD_SIZE - 1 {
            // TODO: Log error
            return;
        }
        // TODO: Call transport trait method if available
        self.max_response_data_size = size;
    }
}
