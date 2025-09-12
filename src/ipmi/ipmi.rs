/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use ipmi_macros::DataAccess;
use std::convert::TryFrom;
//ipmi.h,->base，还是common
// Constants
pub const IPMI_BUF_SIZE: usize = 1024;
pub const IPMI_MAX_MD_SIZE: usize = 0x20;

// IPMI Payload Types
pub const IPMI_PAYLOAD_TYPE_IPMI: u8 = 0x00;
pub const IPMI_PAYLOAD_TYPE_SOL: u8 = 0x01;
pub const IPMI_PAYLOAD_TYPE_OEM: u8 = 0x02;
pub const IPMI_PAYLOAD_TYPE_RMCP_OPEN_REQUEST: u8 = 0x10;
pub const IPMI_PAYLOAD_TYPE_RMCP_OPEN_RESPONSE: u8 = 0x11;
pub const IPMI_PAYLOAD_TYPE_RAKP_1: u8 = 0x12;
pub const IPMI_PAYLOAD_TYPE_RAKP_2: u8 = 0x13;
pub const IPMI_PAYLOAD_TYPE_RAKP_3: u8 = 0x14;
pub const IPMI_PAYLOAD_TYPE_RAKP_4: u8 = 0x15;

/*
extern "Rust" {
    #[no_mangle] // 固定符号名称
    #[link_name = "verbose"]
    static mut verbose: i32;

    #[no_mangle]
    #[link_name = "csv_output"]
    static mut csv_output: i32;
}
*/

#[derive(DataAccess)]
#[repr(C)]
pub struct IpmiMessage {
    pub netfn_lun: u8, // 6 bits+2 bits
    pub cmd: u8,
    pub target_cmd: u8,
    pub data_len: u16,
    pub data: *mut u8,
}

#[derive(Default)]
#[repr(C)]
pub struct IpmiRq {
    pub msg: IpmiMessage,
}

impl Default for IpmiMessage {
    fn default() -> Self {
        Self {
            netfn_lun: 0,
            cmd: 0,
            target_cmd: 0,
            data_len: 0,
            data: std::ptr::null_mut(),
        }
    }
}

use std::fmt;
impl fmt::Debug for IpmiMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IpmiMessage")
            .field("netfn", &self.netfn())
            .field("lun", &self.lun())
            .field("cmd", &self.cmd)
            .field("target_cmd", &self.target_cmd)
            .field("data_len", &self.data_len)
            .field("data", &self.data()) // 将指针转换为整数显示
            .finish()
    }
}

// 提供位域访问方法
impl IpmiMessage {
    pub fn new(netfn: u8, cmd: u8) -> Self {
        Self {
            netfn_lun: netfn << 2,
            cmd,
            target_cmd: 0,
            data_len: 0,
            data: std::ptr::null_mut(),
        }
    }
    pub fn netfn(&self) -> u8 {
        self.netfn_lun >> 2
    }

    pub fn lun(&self) -> u8 {
        self.netfn_lun & 0b11
    }

    pub fn netfn_mut(&mut self, val: u8) {
        self.netfn_lun = (val << 2) | (self.netfn_lun & 0b11);
    }

    pub fn lun_mut(&mut self, val: u8) {
        self.netfn_lun = (self.netfn_lun & 0b11111100) | (val & 0b11);
    }
}
//#[derive(Default)]
pub struct SolPacket {
    pub data: [u8; IPMI_BUF_SIZE],
    pub character_count: u16,
    pub packet_sequence_number: u8,
    pub acked_packet_number: u8,
    pub accepted_character_count: u8,
    pub is_nack: bool,
    pub assert_ring_wor: bool,
    pub generate_break: bool,
    pub deassert_cts: bool,
    pub deassert_dcd_dsr: bool,
    pub flush_inbound: bool,
    pub flush_outbound: bool,
}

pub enum IpmiPayloadData {
    IpmiRequest { rq_seq: u8, request: Box<IpmiRq> },
    IpmiResponse { rs_seq: u8, response: Box<IpmiRs> },
    OpenSessionRequest { request: Vec<u8> },
    Rakp1Message { message: Vec<u8> },
    Rakp2Message { message: Vec<u8> },
    Rakp3Message { message: Vec<u8> },
    Rakp4Message { message: Vec<u8> },
    SolPacket(Box<SolPacket>),
}

pub struct IpmiV2Payload {
    pub payload_length: u16,
    pub payload_type: u8,
    pub payload: IpmiPayloadData,
}

// pub struct IpmiRqEntry {
//     pub req: IpmiRq,
//     pub intf: Option<*mut IpmiIntf>,
//     pub rq_seq: u8,
//     pub data: Option<*mut u8>,    // 为了兼容DataAccess，除非已经实现的函数要使用户指定的字段名。
//     pub data_len: i32,
//     pub bridging_level: i32,
//     pub next: Option<Box<IpmiRqEntry>>,
// }

//#[derive(Default)]
pub struct IpmiRs {
    pub ccode: u8,
    pub data: [u8; IPMI_BUF_SIZE],
    pub data_len: i32,

    pub msg: IpmiRsMsg,
    pub session: IpmiSession,
    pub payload: IpmiRsPayload,
}

impl IpmiRs {
    #[inline]
    pub fn fail(&self) -> bool {
        self.ccode != 0
    }
    #[inline]
    pub fn ok(&self) -> bool {
        self.ccode == 0
    }

    pub fn shift_data(&mut self, i: usize, n: usize) {
        // 参数合法性检查
        if i + n > self.data.len() {
            panic!("Range exceeds array length");
        }
        // 使用 copy_within 将 data[i..i+n] 复制到 data[0..n]
        self.data.copy_within(i..i + n, 0);

        // 使用 fill 方法将 data[n..] 填充为 0
        self.data[n..].fill(0);
    }
}

#[derive(Default)]
pub struct IpmiRsMsg {
    pub netfn: u8,
    pub cmd: u8,
    pub seq: u8,
    pub lun: u8,
}

#[derive(Default)]
pub struct IpmiSession {
    pub authtype: u8,
    pub seq: u32,
    pub id: u32,
    pub b_encrypted: u8,     // IPMI v2 only
    pub b_authenticated: u8, // IPMI v2 only
    pub payloadtype: u8,     // IPMI v2 only
    pub msglen: u16,         // Total length of payload or IPMI message
}

pub enum IpmiRsPayload {
    IpmiResponse {
        rq_addr: u8,
        netfn: u8,
        rq_lun: u8,
        rs_addr: u8,
        rq_seq: u8,
        rs_lun: u8,
        cmd: u8,
    },
    OpenSessionResponse {
        message_tag: u8,
        rakp_return_code: u8,
        max_priv_level: u8,
        console_id: u32,
        bmc_id: u32,
        auth_alg: u8,
        integrity_alg: u8,
        crypt_alg: u8,
    },
    Rakp2Message {
        message_tag: u8,
        rakp_return_code: u8,
        console_id: u32,
        bmc_rand: [u8; 16],
        bmc_guid: [u8; 16],
        key_exchange_auth_code: [u8; IPMI_MAX_MD_SIZE],
    },
    Rakp4Message {
        message_tag: u8,
        rakp_return_code: u8,
        console_id: u32,
        integrity_check_value: [u8; IPMI_MAX_MD_SIZE],
    },
    SolPacket {
        packet_sequence_number: u8,
        acked_packet_number: u8,
        accepted_character_count: u8,
        is_nack: u8,
        transfer_unavailable: u8,
        sol_inactive: u8,
        transmit_overrun: u8,
        break_detected: u8,
    },
}

// Network Function Codes
pub const IPMI_NETFN_CHASSIS: u8 = 0x0;
pub const IPMI_NETFN_BRIDGE: u8 = 0x2;
pub const IPMI_NETFN_SE: u8 = 0x4;
pub const IPMI_NETFN_APP: u8 = 0x6;
pub const IPMI_NETFN_FIRMWARE: u8 = 0x8;
pub const IPMI_NETFN_STORAGE: u8 = 0xa;
pub const IPMI_NETFN_TRANSPORT: u8 = 0xc;
pub const IPMI_NETFN_PICMG: u8 = 0x2C;
pub const IPMI_NETFN_DCGRP: u8 = 0x2C;
pub const IPMI_NETFN_OEM: u8 = 0x2E;
pub const IPMI_NETFN_ISOL: u8 = 0x34;
pub const IPMI_NETFN_TSOL: u8 = 0x30;

pub const IPMI_BMC_SLAVE_ADDR: u32 = 0x20;
pub const IPMI_REMOTE_SWID: u8 = 0x81;

macro_rules! define_ipmi_oem {
    ($($name:ident = $value:expr),+ $(,)?) => {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
        pub enum IPMI_OEM {
            #[default]
            $(
                $name = $value,
            )+
        }

        impl TryFrom<u32> for IPMI_OEM {
            type Error = ();

            fn try_from(value: u32) -> Result<Self, Self::Error> {
                match value {
                    $(
                        $value => Ok(IPMI_OEM::$name),
                    )+
                    _ => Err(())
                }
            }
        }
    };
}

//#[derive(Clone, Copy,Debug, PartialEq, Eq)]
//pub enum IPMI_OEM
define_ipmi_oem! {
    Unknown = 0,
    Debug = 0xFFFFFE,
    Reserved = 0x0FFFFF,
    IBM2 = 2,
    HP = 11,
    Sun = 42,
    Nokia = 94,
    Bull = 107,
    Hitachi116 = 116,
    NEC = 119,
    Toshiba = 186,
    Ericsson = 193,
    Intel = 343,
    Tatung = 373,
    Hitachi399 = 399,
    Dell = 674,
    LMC = 2168,
    Radisys = 4337,
    Broadcom = 4413,
    IBM4769 = 4769,
    Magnum = 5593,
    Tyan = 6653,
    Quanta = 7244,
    Viking = 9237,
    Advantech = 10297,
    FujitsuSiemens = 10368,
    Avocent = 10418,
    Peppercon = 10437,
    Supermicro = 10876,
    OSA = 11102,
    Google = 11129,
    PICMG = 12634,
    Raritan = 13742,
    Kontron = 15000,
    PPS = 16394,
    IBM20301 = 20301,
    AMI = 20974,
    Adlink24339 = 24339,
    NokiaSolutionsAndNetworks = 28458,
    VITA = 33196,
    Supermicro47488 = 47488,
    YADRO = 49769,
}

//extern const struct valstr completion_code_vals[];

/*
 * CC
 * See IPMI specification table 5-2 Generic Completion Codes
 */
pub const IPMI_CC_OK: u8 = 0x00;
pub const IPMI_CC_NODE_BUSY: u8 = 0xc0;
pub const IPMI_CC_INV_CMD: u8 = 0xc1;
pub const IPMI_CC_INV_CMD_FOR_LUN: u8 = 0xc2;
pub const IPMI_CC_TIMEOUT: u8 = 0xc3;
pub const IPMI_CC_OUT_OF_SPACE: u8 = 0xc4;
pub const IPMI_CC_RES_CANCELED: u8 = 0xc5;
pub const IPMI_CC_REQ_DATA_TRUNC: u8 = 0xc6;
pub const IPMI_CC_REQ_DATA_INV_LENGTH: u8 = 0xc7;
pub const IPMI_CC_REQ_DATA_FIELD_EXCEED: u8 = 0xc8;
pub const IPMI_CC_PARAM_OUT_OF_RANGE: u8 = 0xc9;
pub const IPMI_CC_CANT_RET_NUM_REQ_BYTES: u8 = 0xca;
pub const IPMI_CC_REQ_DATA_NOT_PRESENT: u8 = 0xcb;
pub const IPMI_CC_INV_DATA_FIELD_IN_REQ: u8 = 0xcc;
pub const IPMI_CC_ILL_SENSOR_OR_RECORD: u8 = 0xcd;
pub const IPMI_CC_RESP_COULD_NOT_BE_PRV: u8 = 0xce;
pub const IPMI_CC_CANT_RESP_DUPLI_REQ: u8 = 0xcf;
pub const IPMI_CC_CANT_RESP_SDRR_UPDATE: u8 = 0xd0;
pub const IPMI_CC_CANT_RESP_FIRM_UPDATE: u8 = 0xd1;
pub const IPMI_CC_CANT_RESP_BMC_INIT: u8 = 0xd2;
pub const IPMI_CC_DESTINATION_UNAVAILABLE: u8 = 0xd3;
pub const IPMI_CC_INSUFFICIENT_PRIVILEGES: u8 = 0xd4;
pub const IPMI_CC_NOT_SUPPORTED_PRESENT_STATE: u8 = 0xd5;
pub const IPMI_CC_ILLEGAL_COMMAND_DISABLED: u8 = 0xd6;
pub const IPMI_CC_UNSPECIFIED_ERROR: u8 = 0xff;
