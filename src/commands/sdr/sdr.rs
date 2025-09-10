/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(dead_code)]

use crate::debug3; // 添加debug3宏导入
use crate::debug5;
use crate::error::IpmiError;
use crate::helper::*;
use crate::ipmi::intf::*;
use crate::ipmi::ipmi::*; // 添加debug5宏导入

use crate::commands::sdr::iter::SdrIterator;
use crate::commands::sdr::sdradd::*;
use crate::commands::sdr::*;
use crate::commands::sensor::sensor::*;

use unpack::RAWDATA;

static mut USE_BUILT_IN: bool = false; /* Uses DeviceSDRs instead of SDRR */
static mut SDR_MAX_READ_LEN: i32 = 0;
static mut SDRIANA: i64 = 0;

const GET_SDR_RESERVE_REPO: u8 = 0x22;

// SDR Commands
pub const GET_SDR: u8 = 0x23;
pub const GET_SDR_REPO_INFO: u8 = 0x20;
pub const GET_SDR_ALLOC_INFO: u8 = 0x21;

// Sensor Status Flags
pub const READING_UNAVAILABLE: u8 = 0x20;
pub const SCANNING_DISABLED: u8 = 0x40;
pub const EVENT_MSG_DISABLED: u8 = 0x80;

// SDR Sensor Status Bits
pub const SDR_SENSOR_STAT_LO_NC: u8 = 1 << 0;
pub const SDR_SENSOR_STAT_LO_CR: u8 = 1 << 1;
pub const SDR_SENSOR_STAT_LO_NR: u8 = 1 << 2;
pub const SDR_SENSOR_STAT_HI_NC: u8 = 1 << 3;
pub const SDR_SENSOR_STAT_HI_CR: u8 = 1 << 4;
pub const SDR_SENSOR_STAT_HI_NR: u8 = 1 << 5;

// Device SDR Commands
pub const GET_DEVICE_SDR_INFO: u8 = 0x20;
pub const GET_DEVICE_SDR: u8 = 0x21;
pub const GET_SENSOR_FACTORS: u8 = 0x23;
pub const SET_SENSOR_HYSTERESIS: u8 = 0x24;
pub const GET_SENSOR_HYSTERESIS: u8 = 0x25;
pub const SET_SENSOR_THRESHOLDS: u8 = 0x26;
pub const GET_SENSOR_THRESHOLDS: u8 = 0x27;
pub const SET_SENSOR_EVENT_ENABLE: u8 = 0x28;
pub const GET_SENSOR_EVENT_ENABLE: u8 = 0x29;
pub const GET_SENSOR_EVENT_STATUS: u8 = 0x2b;
pub const GET_SENSOR_READING: u8 = 0x2d;
pub const GET_SENSOR_TYPE: u8 = 0x2f;

pub const IPM_DEV_DEVICE_ID_SDR_MASK: u8 = 0x80; // 1 = provides SDRs
pub const IPM_DEV_DEVICE_ID_REV_MASK: u8 = 0x0F; // BCD-encoded

pub const IPM_DEV_FWREV1_AVAIL_MASK: u8 = 0x80; // 0 = normal operation
pub const IPM_DEV_FWREV1_MAJOR_MASK: u8 = 0x7F; // Major rev, BCD-encoded

pub const IPM_DEV_IPMI_VER_MAJOR_MASK: u8 = 0x0F; // Major rev, BCD-encoded
pub const IPM_DEV_IPMI_VER_MINOR_MASK: u8 = 0xF0; // Minor rev, BCD-encoded
pub const IPM_DEV_IPMI_VER_MINOR_SHIFT: u8 = 4; // Minor rev shift

pub const IPM_DEV_MANUFACTURER_ID_RESERVED: u32 = 0x0FFFFF;
pub const IPM_DEV_ADTL_SUPPORT_BITS: u8 = 8;

// 单位描述数组 (UNIT_TYPE_MAX = 92)
const UNIT_DESC: [&str; 93] = [
    "unspecified",         // 0
    "degrees C",           // 1
    "degrees F",           // 2
    "degrees K",           // 3
    "Volts",               // 4
    "Amps",                // 5
    "Watts",               // 6
    "Joules",              // 7
    "Coulombs",            // 8
    "VA",                  // 9
    "Nits",                // 10
    "lumen",               // 11
    "lux",                 // 12
    "Candela",             // 13
    "kPa",                 // 14
    "PSI",                 // 15
    "Newton",              // 16
    "CFM",                 // 17
    "RPM",                 // 18
    "Hz",                  // 19
    "microsecond",         // 20
    "millisecond",         // 21
    "second",              // 22
    "minute",              // 23
    "hour",                // 24
    "day",                 // 25
    "week",                // 26
    "mil",                 // 27
    "inches",              // 28
    "feet",                // 29
    "cu in",               // 30
    "cu feet",             // 31
    "mm",                  // 32
    "cm",                  // 33
    "m",                   // 34
    "cu cm",               // 35
    "cu m",                // 36
    "liters",              // 37
    "fluid ounce",         // 38
    "radians",             // 39
    "steradians",          // 40
    "revolutions",         // 41
    "cycles",              // 42
    "gravities",           // 43
    "ounce",               // 44
    "pound",               // 45
    "ft-lb",               // 46
    "oz-in",               // 47
    "gauss",               // 48
    "gilberts",            // 49
    "henry",               // 50
    "millihenry",          // 51
    "farad",               // 52
    "microfarad",          // 53
    "ohms",                // 54
    "siemens",             // 55
    "mole",                // 56
    "becquerel",           // 57
    "PPM",                 // 58
    "reserved",            // 59
    "Decibels",            // 60
    "DbA",                 // 61
    "DbC",                 // 62
    "gray",                // 63
    "sievert",             // 64
    "color temp deg K",    // 65
    "bit",                 // 66
    "kilobit",             // 67
    "megabit",             // 68
    "gigabit",             // 69
    "byte",                // 70
    "kilobyte",            // 71
    "megabyte",            // 72
    "gigabyte",            // 73
    "word",                // 74
    "dword",               // 75
    "qword",               // 76
    "line",                // 77
    "hit",                 // 78
    "miss",                // 79
    "retry",               // 80
    "reset",               // 81
    "overflow",            // 82
    "underrun",            // 83
    "collision",           // 84
    "packets",             // 85
    "messages",            // 86
    "characters",          // 87
    "error",               // 88
    "correctable error",   // 89
    "uncorrectable error", //90
    "fatal error",         // 91
    "grams",               // 92
];

// 传感器类型描述数组 (共44个元素)
const SENSOR_TYPE_DESC: [&str; 45] = [
    "reserved",                 // 0
    "Temperature",              // 1
    "Voltage",                  // 2
    "Current",                  // 3
    "Fan",                      // 4
    "Physical Security",        // 5
    "Platform Security",        // 6
    "Processor",                // 7
    "Power Supply",             // 8
    "Power Unit",               // 9
    "Cooling Device",           // 10
    "Other",                    // 11
    "Memory",                   // 12
    "Drive Slot / Bay",         // 13
    "POST Memory Resize",       // 14
    "System Firmwares",         // 15
    "Event Logging Disabled",   //16
    "Watchdog1",                // 17
    "System Event",             // 18
    "Critical Interrupt",       // 19
    "Button",                   // 20
    "Module / Board",           // 21
    "Microcontroller",          // 22
    "Add-in Card",              // 23
    "Chassis",                  // 24
    "Chip Set",                 // 25
    "Other FRU",                // 26
    "Cable / Interconnect",     // 27
    "Terminator",               // 28
    "System Boot Initiated",    //29
    "Boot Error",               // 30
    "OS Boot",                  // 31
    "OS Critical Stop",         // 32
    "Slot / Connector",         // 33
    "System ACPI Power State",  //34
    "Watchdog2",                //35
    "Platform Alert",           //36
    "Entity Presence",          //37
    "Monitor ASIC",             //38
    "LAN",                      //39
    "Management Subsys Health", //40
    "Battery",                  //41
    "Session Audit",            //42
    "Version Change",           //43
    "FRU State",                //44
];

// Helper functions
pub fn ipm_dev_ipmi_version_major(x: u8) -> u8 {
    x & IPM_DEV_IPMI_VER_MAJOR_MASK
}

pub fn ipm_dev_ipmi_version_minor(x: u8) -> u8 {
    (x & IPM_DEV_IPMI_VER_MINOR_MASK) >> IPM_DEV_IPMI_VER_MINOR_SHIFT
}

pub fn ipm_dev_manufacturer_id(x: &[u8; 3]) -> u32 {
    ipmi24toh(x)
}

// Helper functions
pub fn is_reading_unavailable(val: u8) -> bool {
    (val & READING_UNAVAILABLE) != 0
}

pub fn is_scanning_disabled(val: u8) -> bool {
    (val & SCANNING_DISABLED) != 0
}

pub fn is_event_msg_disabled(val: u8) -> bool {
    (val & EVENT_MSG_DISABLED) == 0
}

//impl IpmiSdrInterface for Box<dyn IpmiIntf> {

//这些方法都要访问openintf的数据成员
//他们是公共方法，业务相关
pub fn ipmi_sdr_add_from_sensors(intf: &mut dyn IpmiIntf, maxslot: i32) -> bool {
    // Clear SDR repository
    if !ipmi_sdr_repo_clear(intf) {
        println!("Cannot erase SDRR. Give up.");
        return false;
    }

    //TODO target_addr是成员变量
    let myaddr = intf.context().target_addr();

    // First fill the SDRR from local built-in sensors
    let mut rc = sdr_copy_to_sdrr(intf, true, myaddr, myaddr);

    // Now fill the SDRR with remote sensors
    if maxslot != 0 {
        let mut slave_addr = 0xB0;
        for _ in 0..maxslot {
            // Hole in the PICMG 2.9 mapping
            if slave_addr == 0xC2 {
                slave_addr += 2;
            }
            if !sdr_copy_to_sdrr(intf, false, slave_addr, myaddr) {
                rc = false;
            }
        }
    }
    rc
}

fn sdr_copy_to_sdrr(
    intf: &mut dyn IpmiIntf,
    use_builtin: bool,
    from_addr: u32,
    _to_addr: u32,
) -> bool {
    // Set target address for reading
    intf.context().set_target_addr(from_addr);

    debug3!("Load SDRs from 0x{:x}", from_addr);

    // Create SDR iterator
    //let intf_box: Box<dyn IpmiIntf> = Box::new(intf);
    // Box<dyn IpmiIntf>
    //let mut sdr_iter: SdrIterator = SdrIterator::new(intf, use_builtin).unwrap();

    // Collect records
    //let mut records = Vec::new();
    let records = {
        let mut sdr_iter = SdrIterator::new(intf, use_builtin).unwrap();
        match sdr_iter.sdrr_get_records() {
            Ok(r) => r,
            Err(_) => return false,
        }
    }; // sdr_iter在这里被自动释放

    // Write records to destination SDR Repository

    for record in records {
        if !ipmi_sdr_add_record(intf, &record) {
            println!(
                "Cannot add SDR ID 0x{:04x} to repository...",
                record.header.id
            );
            return false;
        }
    }

    true
}

fn ipmi_sdr_repo_clear(intf: &mut dyn IpmiIntf) -> bool {
    let reserve_id = match ipmi_sdr_get_reservation(intf, false) {
        Some(id) => id,
        None => {
            println!("Unable to get SDR reservation ID");
            return false;
        }
    };

    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_STORAGE);
    req.msg.cmd = 0x27;
    req.msg.data_len = 6;
    //let mut msg_data = Vec::with_capacity(6);
    let mut msg_data = vec![
        (reserve_id & 0xFF) as u8,
        (reserve_id >> 8) as u8,
        b'C',
        b'L',
        b'R',
        0xAA,
        0,
        0,
    ];
    req.msg.data = msg_data.as_mut_ptr();

    for _ in 0..5 {
        let rsp = match intf.sendrecv(&req) {
            Some(r) => r,
            None => {
                println!("Unable to clear SDRR");
                return false;
            }
        };

        if rsp.ccode != 0 {
            println!(
                "Unable to clear SDRR: {}",
                IpmiError::CompletionCode(rsp.ccode)
            );
            return false;
        }

        if (rsp.data[0] & 1) == 1 {
            println!("SDRR successfully erased");
            return true;
        }

        println!("Wait for SDRR erasure completed...");
        msg_data[5] = 0;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    false
}

pub fn ipmi_sdr_get_reservation(intf: &mut dyn IpmiIntf, use_builtin: bool) -> Option<u16> {
    // Create request
    let mut req = IpmiRq::default();
    if use_builtin {
        req.msg.netfn_mut(IPMI_NETFN_SE);
    } else {
        req.msg.netfn_mut(IPMI_NETFN_STORAGE);
    }

    req.msg.cmd = GET_SDR_RESERVE_REPO;
    // Send request and get response
    let rsp = match intf.sendrecv(&req) {
        Some(response) => response,
        None => return None,
    };

    // Check completion code
    if rsp.ccode != 0 {
        return None;
    }

    // Extract reserve_id from response data
    if rsp.data.len() < 2 {
        return None;
    }

    let reserve_id = u16::from_le_bytes([rsp.data[0], rsp.data[1]]);
    //log::debug!("SDR reservation ID {:04x}", reserve_id);
    debug3!("SDR reservation ID {:04x}", reserve_id);
    Some(reserve_id)
}

#[repr(C)]
#[derive(Debug, Default, AsBytes)]
pub struct SdrGetRq {
    pub reserve_id: u16, // reservation ID
    pub id: u16,         // record ID
    pub offset: u8,      // offset into SDR
    pub length: u8,      // length to read
}

#[derive(Debug, Default, AsBytes)]
#[repr(C)]
pub struct SdrGetRs {
    pub next: u16,               // next record id
    pub header: SdrRecordHeader, // record header
}

#[repr(C)]
#[derive(Debug, Default, AsBytes, Clone)]
pub struct SdrRecordHeader {
    pub id: u16,         // record ID
    pub version: u8,     // SDR version (51h)
    pub record_type: u8, // record type
    pub length: u8,      // remaining record bytes
}

pub const SDR_RECORD_TYPE_FULL_SENSOR: u8 = 0x01;
pub const SDR_RECORD_TYPE_COMPACT_SENSOR: u8 = 0x02;
pub const SDR_RECORD_TYPE_EVENTONLY_SENSOR: u8 = 0x03;
pub const SDR_RECORD_TYPE_ENTITY_ASSOC: u8 = 0x08;
pub const SDR_RECORD_TYPE_DEVICE_ENTITY_ASSOC: u8 = 0x09;
pub const SDR_RECORD_TYPE_GENERIC_DEVICE_LOCATOR: u8 = 0x10;
pub const SDR_RECORD_TYPE_FRU_DEVICE_LOCATOR: u8 = 0x11;
pub const SDR_RECORD_TYPE_MC_DEVICE_LOCATOR: u8 = 0x12;
pub const SDR_RECORD_TYPE_MC_CONFIRMATION: u8 = 0x13;
pub const SDR_RECORD_TYPE_BMC_MSG_CHANNEL_INFO: u8 = 0x14;
pub const SDR_RECORD_TYPE_OEM: u8 = 0xc0;

fn tos32(val: i32, bits: i32) -> i32 {
    if val & (1 << (bits - 1)) != 0 {
        -(val & (1 << (bits - 1))) | val
    } else {
        val
    }
}

fn bswap_16(x: u16) -> u16 {
    ((x & 0xff00) >> 8) | ((x & 0x00ff) << 8)
}

fn bswap_32(x: u32) -> u32 {
    ((x & 0xff000000) >> 24)
        | ((x & 0x00ff0000) >> 8)
        | ((x & 0x0000ff00) << 8)
        | ((x & 0x000000ff) << 24)
}

fn to_tol(mtol: u16) -> u16 {
    bswap_16(mtol) & 0x3f
}

fn to_m(mtol: u16) -> i16 {
    tos32(
        ((bswap_16(mtol) & 0xff00) >> 8 | (bswap_16(mtol) & 0xc0) << 2) as i32,
        10,
    ) as i16
}

fn to_b(bacc: u32) -> i32 {
    tos32(
        ((bswap_32(bacc) & 0xff000000) >> 24 | (bswap_32(bacc) & 0xc00000) >> 14) as i32,
        10,
    )
}

fn to_acc(bacc: u32) -> u32 {
    ((bswap_32(bacc) & 0x3f0000) >> 16) | ((bswap_32(bacc) & 0xf000) >> 6)
}

fn to_acc_exp(bacc: u32) -> u32 {
    (bswap_32(bacc) & 0xc00) >> 10
}

fn to_r_exp(bacc: u32) -> i32 {
    tos32(((bswap_32(bacc) & 0xf0) >> 4) as i32, 4)
}

fn to_b_exp(bacc: u32) -> i32 {
    tos32((bswap_32(bacc) & 0xf) as i32, 4)
}

#[derive(Debug, Default, AsBytes)]
#[repr(C)]
pub struct SdrRepoInfoRs {
    pub version: u8,      // SDR version (51h)
    pub count: u16,       // number of records
    pub free: u16,        // free space in SDR
    pub add_stamp: u32,   // last add timestamp
    pub erase_stamp: u32, // last del timestamp
    pub op_support: u8,   // supported operations
}

#[derive(Debug, AsBytes)]
#[repr(C)]
pub struct SdrRecordCompactSensor {
    pub cmn: SdrRecordCommonSensor,
    pub share: SdrCompactShare,
    pub threshold: SdrCompactThreshold,
    pub reserved: [u8; 3],
    pub oem: u8,
    pub id_code: u8,
    pub id_string: [u8; 16],
}

#[derive(Debug, RAWDATA)]
#[repr(C)]
pub struct SdrRecordFullSensor {
    pub cmn: SdrRecordCommonSensor,
    pub linearization: u8,
    pub mtol: u16,        // M, tolerance
    pub bacc: u32,        // accuracy, B, Bexp, Rexp
    pub analog_flag: u8,  // bit flags for nominal/normal values
    pub nominal_read: u8, // nominal reading, raw value
    pub normal_max: u8,   // normal maximum, raw value
    pub normal_min: u8,   // normal minimum, raw value
    pub sensor_max: u8,   // sensor maximum, raw value
    pub sensor_min: u8,   // sensor minimum, raw value
    pub threshold: SdrThresholds,
    pub reserved: [u8; 2],
    pub oem: u8,             // reserved for OEM use
    pub id_code: u8,         // sensor ID string type/length code
    pub id_string: [u8; 16], // sensor ID string bytes
}

// impl ::unpack::RawSize  for RawSdrRecordFullSensor {

// }

impl SdrRecordFullSensor {
    pub fn from_le_bytes(data: &[u8]) -> Result<Self, &'static str> {
        // 检查最小长度 (不包括id_string的16字节)
        const MIN_SIZE: usize = <SdrRecordFullSensor as ::unpack::RawSize>::RAW_SIZE - 16;
        if data.len() < MIN_SIZE {
            return Err("Input data too short for SdrRecordFullSensor");
        }

        let mut offset = 0;

        // 反序列化固定长度部分
        let cmn = SdrRecordCommonSensor::from_le_bytes(&data[offset..])?;
        offset += <SdrRecordCommonSensor as ::unpack::RawSize>::RAW_SIZE;

        let linearization = data[offset];
        offset += 1;

        let mtol = u16::from_le_bytes([data[offset], data[offset + 1]]);
        offset += 2;

        let bacc = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        offset += 4;

        let analog_flag = data[offset];
        offset += 1;

        let nominal_read = data[offset];
        offset += 1;

        let normal_max = data[offset];
        offset += 1;

        let normal_min = data[offset];
        offset += 1;

        let sensor_max = data[offset];
        offset += 1;

        let sensor_min = data[offset];
        offset += 1;

        let threshold = SdrThresholds::from_le_bytes(&data[offset..])?;
        offset += <SdrThresholds as ::unpack::RawSize>::RAW_SIZE;

        let reserved = [data[offset], data[offset + 1]];
        offset += 2;

        let oem = data[offset];
        offset += 1;

        let id_code = data[offset];
        offset += 1;

        // 处理id_string (可能不完整)
        let id_len = (id_code & 0x1F) as usize; // 取低5位作为长度
        let mut id_string = [0u8; 16];

        // 检查剩余数据是否足够
        let remaining_data = if offset <= data.len() {
            &data[offset..]
        } else {
            &[]
        };

        // 复制有效数据部分
        let copy_len = id_len.min(16).min(remaining_data.len());
        if copy_len > 0 {
            id_string[..copy_len].copy_from_slice(&remaining_data[..copy_len]);
        }

        Ok(Self {
            cmn,
            linearization,
            mtol,
            bacc,
            analog_flag,
            nominal_read,
            normal_max,
            normal_min,
            sensor_max,
            sensor_min,
            threshold,
            reserved,
            oem,
            id_code,
            id_string,
        })
    }

    /// 阈值字段格式化方法
    pub fn print_thresh_setting(
        &self,
        thresh_is_avail: bool,
        setting: u8,
        field_sep: &str,
    ) -> String {
        //先输出field_sep
        if !thresh_is_avail {
            return format!("{field_sep}na"); //使用字符串输出
        }

        if !self.cmn.are_discrete() {
            let value = self.sdr_convert_sensor_reading(setting);
            format!("{field_sep}{value:.3}")
        } else {
            format!("{field_sep}0x{:x}", setting) //使用16进制输出
        }
    }

    pub fn sdr_convert_sensor_reading(&self, val: u8) -> f64 {
        let m = to_m(self.mtol) as i32;
        let b = to_b(self.bacc);
        let k1 = to_b_exp(self.bacc);
        let k2 = to_r_exp(self.bacc);

        let mut result = match self.cmn.unit.analog() {
            0 => {
                ((m as f64 * val as f64) + (b as f64 * 10f64.powf(k1 as f64)))
                    * 10f64.powf(k2 as f64)
            }
            1 => {
                let mut v = val;
                if v & 0x80 != 0 {
                    v = v.wrapping_add(1);
                }
                ((m as f64 * v as i8 as f64) + (b as f64 * 10f64.powf(k1 as f64)))
                    * 10f64.powf(k2 as f64)
            }
            2 => {
                ((m as f64 * val as i8 as f64) + (b as f64 * 10f64.powf(k1 as f64)))
                    * 10f64.powf(k2 as f64)
            }
            _ => return 0.0, // Not an analog sensor
        };

        match self.linearization & 0x7f {
            SDR_SENSOR_L_LN => result = result.ln(),
            SDR_SENSOR_L_LOG10 => result = result.log10(),
            SDR_SENSOR_L_LOG2 => result = result.log2(),
            SDR_SENSOR_L_E => result = result.exp(),
            SDR_SENSOR_L_EXP10 => result = 10f64.powf(result),
            SDR_SENSOR_L_EXP2 => result = 2f64.powf(result),
            SDR_SENSOR_L_1_X => result = result.powf(-1.0),
            SDR_SENSOR_L_SQR => result = result.powf(2.0),
            SDR_SENSOR_L_CUBE => result = result.powf(3.0),
            SDR_SENSOR_L_SQRT => result = result.sqrt(),
            SDR_SENSOR_L_CUBERT => result = result.cbrt(),
            SDR_SENSOR_L_LINEAR => {}
            _ => {}
        }

        result
    }
}

impl SdrRecordCompactSensor {
    // pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
    //     if bytes.len() < std::mem::size_of::<Self>() {
    //         return None;
    //     }

    //     // Safety: The struct is #[repr(packed)] and we've verified the length
    //     unsafe {
    //         let ptr = bytes.as_ptr() as *const Self;
    //         Some(ptr.read_unaligned())
    //     }
    // }
}

#[repr(C)]
#[derive(Debug, Default, AsBytes)]
pub struct SdrCompactShare {
    bits: u16,
    //pub count_mod_type: u8,     // bits 0-3: count, bits 4-5: mod_type, bits 6-7: reserved
    //pub mod_offset_entity: u8,  // bits 0-6: mod_offset, bit 7: entity_inst
}
impl SdrCompactShare {
    // count: bits[0..3] (4 bits)
    pub fn count(&self) -> u8 {
        (self.bits & 0x000F) as u8
    }

    pub fn count_mut(&mut self, val: u8) {
        self.bits = (self.bits & !0x000F) | (val as u16 & 0x000F);
    }

    // mod_type: bits[4..5] (2 bits)
    pub fn mod_type(&self) -> u8 {
        ((self.bits >> 4) & 0x03) as u8
    }

    pub fn mod_type_mut(&mut self, val: u8) {
        self.bits = (self.bits & !0x0030) | ((val as u16 & 0x03) << 4);
    }

    // mod_offset: bits[8..14] (7 bits)
    pub fn mod_offset(&self) -> u8 {
        ((self.bits >> 8) & 0x7F) as u8
    }

    pub fn mod_offset_mut(&mut self, val: u8) {
        self.bits = (self.bits & !0x7F00) | ((val as u16 & 0x7F) << 8);
    }

    // entity_inst: bit[15] (1 bit)
    pub fn entity_inst(&self) -> bool {
        (self.bits >> 15) & 0x01 != 0
    }

    pub fn entity_inst_mut(&mut self, val: bool) {
        self.bits = (self.bits & !0x8000) | ((val as u16) << 15);
    }
}
#[repr(C)] //#[derive(Debug, Default, AsBytes)]
#[derive(Debug, Default, AsBytes)]
pub struct SdrCompactThreshold {
    pub hysteresis: SdrHysteresis,
}

#[repr(C)]
#[derive(Debug, Default, AsBytes)]
pub struct SdrHysteresis {
    pub positive: u8,
    pub negative: u8,
}

#[derive(Debug, Default)]
pub struct SensorReading {
    pub s_id: [u8; 17], // name of the sensor
    // pub full: *mut SdrRecordFullSensor,
    // pub compact: *mut SdrRecordCompactSensor,
    pub full: Option<Box<SdrRecordFullSensor>>,
    pub compact: Option<Box<SdrRecordCompactSensor>>,
    pub s_reading_valid: bool,       // read value validity
    pub s_scanning_disabled: bool,   // read of value disabled
    pub s_reading_unavailable: bool, // read value unavailable
    pub s_reading: u8,               // value which was read
    pub s_data2: u8,                 // data2 value read
    pub s_data3: u8,                 // data3 value read
    pub s_has_analog_value: bool,    // sensor has analog value
    pub s_a_val: f64,                // read value converted to analog
    pub s_a_str: String,             // analog value as a string
    pub s_a_units: String,           // analog value units string
}

impl SensorReading {
    pub fn new() -> Self {
        Self {
            s_id: [0u8; 17],
            full: None,
            compact: None,
            s_reading: 0,
            s_data2: 0,
            s_data3: 0,
            s_reading_valid: false,
            s_reading_unavailable: false,
            s_scanning_disabled: false,
            s_has_analog_value: false,
            s_a_val: 0.0,
            s_a_str: String::new(),
            s_a_units: String::new(),
        }
    }
}

//0x4,0x2d
pub fn ipmi_sdr_get_sensor_reading_ipmb(
    //mut intf: Box<dyn IpmiIntf>,
    intf: &mut dyn IpmiIntf, // 更通用的方式
    mut sensor: u8,
    target: u8,
    lun: u8,
    channel: u8,
) -> Option<IpmiRs> {
    let mut bridged_request = false;
    let mut save_addr = 0;
    let mut save_channel = 0;

    // Handle bridged requests
    if intf.context().bridge_to_sensor(target, channel) {
        bridged_request = true;
        save_addr = intf.context().target_addr();
        save_channel = intf.context().target_channel();
        debug3!(
            "Bridge to Sensor Intf my/{:#x} tgt/{:#x}:{:#x} Sdr tgt/{:#x}:{:#x}",
            intf.context().my_addr(),
            save_addr,
            save_channel,
            target,
            channel
        );
        // bridged_request = true;
        // save_addr = intf.context().target_addr();
        // save_channel = intf.context().target_channel();

        intf.context().set_target_addr(target as u32);
        intf.context().set_target_channel(channel);
    }

    // Build request
    let mut req: IpmiRq = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_SE);
    req.msg.lun_mut(lun);
    req.msg.cmd = GET_SENSOR_READING;
    //req.msg.data = sensor.as_ptr();//&mut sensor as *mut u8;
    req.msg.data = &mut sensor as *mut u8;
    req.msg.data_len = 1;
    // Send request and get response
    let rsp = intf.sendrecv(&req)?;

    // Restore original addressing if bridged
    if bridged_request {
        intf.context().set_target_addr(save_addr);
        intf.context().set_target_channel(save_channel);
    }

    Some(rsp)
}

//读取到SensorReading
pub fn ipmi_sdr_read_sensor_value(
    //intf: Box<dyn IpmiIntf>,
    intf: &mut dyn IpmiIntf, // 更通用的方式
    sensor_raw: &[u8],
    sdr_record_type: u8,
    _precision: i32,
) -> Option<SensorReading> {
    let sensor = match SdrRecordCommonSensor::from_le_bytes(sensor_raw) {
        Ok(s) => s,
        Err(e) => {
            debug5!("Failed to parse sensor record: {}", e);
            return None;
        }
    };

    let mut sr: SensorReading = SensorReading::new();

    match sdr_record_type {
        //full=59bytes,实际返回52,可能最后id_string的16个字节没有被填充
        SDR_RECORD_TYPE_FULL_SENSOR => match SdrRecordFullSensor::from_le_bytes(sensor_raw) {
            Ok(s) => {
                let mut idlen = (s.id_code & 0x1f) as usize;

                // 如果 id_code 长度为 0，尝试从 id_string 中找到实际长度
                if idlen == 0 {
                    // 查找 id_string 中的有效字符长度（到第一个 null 字符或非打印字符）
                    for i in 0..16 {
                        if s.id_string[i] == 0 || s.id_string[i] < 0x20 || s.id_string[i] > 0x7e {
                            idlen = i;
                            break;
                        }
                        if i == 15 {
                            // 检查是否所有字符都是可打印字符
                            if s.id_string.iter().all(|&b| (0x20..=0x7e).contains(&b)) {
                                idlen = 16;
                            } else {
                                idlen = 0; // 如果有非可打印字符，认为名称无效
                            }
                        }
                    }
                }

                idlen = idlen.min(sr.s_id.len() - 1).min(16);
                if idlen > 0 {
                    sr.s_id[..idlen].copy_from_slice(&s.id_string[..idlen]);
                }
                sr.full = Some(Box::new(s));
            }
            Err(e) => {
                debug5!("Failed to parse full sensor record: {}", e);
                return None;
            }
        },
        SDR_RECORD_TYPE_COMPACT_SENSOR => match SdrRecordCompactSensor::from_le_bytes(sensor_raw) {
            Ok(s) => {
                let mut idlen = (s.id_code & 0x1f) as usize;

                // 如果 id_code 长度为 0，尝试从 id_string 中找到实际长度
                if idlen == 0 {
                    // 查找 id_string 中的有效字符长度（到第一个 null 字符或非打印字符）
                    for i in 0..16 {
                        if s.id_string[i] == 0 || s.id_string[i] < 0x20 || s.id_string[i] > 0x7e {
                            idlen = i;
                            break;
                        }
                        if i == 15 {
                            // 检查是否所有字符都是可打印字符
                            if s.id_string.iter().all(|&b| (0x20..=0x7e).contains(&b)) {
                                idlen = 16;
                            } else {
                                idlen = 0; // 如果有非可打印字符，认为名称无效
                            }
                        }
                    }
                }

                idlen = idlen.min(sr.s_id.len() - 1).min(16);
                if idlen > 0 {
                    sr.s_id[..idlen].copy_from_slice(&s.id_string[..idlen]);
                }
                sr.compact = Some(Box::new(s));
            }
            Err(e) => {
                debug5!("Failed to parse compact sensor record: {}", e);
                return None;
            }
        },
        _ => return None,
    }

    //rintln!("Sensor ID: {:#?}", sensor);
    // Get current reading via IPMI interface
    //29, c0, c0
    let rsp = match ipmi_sdr_get_sensor_reading_ipmb(
        intf,
        sensor.keys.sensor_num, //sensor
        sensor.keys.owner_id,   //target
        sensor.keys.lun(),      //lun
        sensor.keys.channel(),  //channel
    ) {
        Some(r) => r,
        None => {
            debug5!(
                "Error reading sensor {:?} (#{})",
                sr.s_id,
                sensor.keys.sensor_num
            );
            return Some(sr);
        }
    };

    // sr.s_a_val   = 0.0;	/* init analog value to a floating point 0 */
    // sr.s_a_str[0] = '\0';	/* no converted analog value string */
    // sr.s_a_units = "";	/* no converted analog units units */
    if rsp.ccode != 0 {
        // 记录错误信息，但对于某些错误码仍然继续处理
        debug5!(
            "Sensor {:?} (#{:02x}) reading has ccode 0x{:02x}",
            String::from_utf8_lossy(&sr.s_id).trim_matches('\0'),
            sensor.keys.sensor_num,
            rsp.ccode
        );

        // 只对严重错误直接返回，其他情况继续处理响应数据
        match rsp.ccode {
            0xc1 | 0xc0 | 0xc3 => {
                // 严重错误：Invalid command, Node busy, Invalid data field
                debug5!(
                    "Sensor {:?} (#{:02x}) has serious error 0x{:02x}, skipping",
                    String::from_utf8_lossy(&sr.s_id).trim_matches('\0'),
                    sensor.keys.sensor_num,
                    rsp.ccode
                );
                sr.s_reading_valid = false;
                sr.s_reading_unavailable = true;
                sr.s_scanning_disabled = true;
                return Some(sr);
            }
            _ => {
                // 其他错误码（包括 0xcb, 0xcd）：记录但继续处理数据
                debug5!(
                    "Sensor {:?} (#{:02x}) has non-fatal error 0x{:02x}, continuing",
                    String::from_utf8_lossy(&sr.s_id).trim_matches('\0'),
                    sensor.keys.sensor_num,
                    rsp.ccode
                );
            }
        }
    }

    //失败
    if rsp.data_len < 2 {
        debug5!(
            "Sensor {:?} response length {} < 2, but may still have reading in data[0]",
            String::from_utf8_lossy(&sr.s_id).trim_matches('\0'),
            rsp.data_len
        );
        // 即使长度小于2，如果有 data[0] 也尝试读取
        if rsp.data_len >= 1 {
            sr.s_reading_valid = true;
            sr.s_reading = rsp.data[0];
            debug5!(
                "Sensor {:?} using data[0]={} despite short response",
                String::from_utf8_lossy(&sr.s_id).trim_matches('\0'),
                sr.s_reading
            );
        } else {
            debug5!(
                "Sensor {:?} has no data, returning with invalid reading",
                String::from_utf8_lossy(&sr.s_id).trim_matches('\0')
            );
            return Some(sr);
        }
    }

    if is_reading_unavailable(rsp.data[1]) {
        sr.s_reading_unavailable = true;
        debug5!(
            "Sensor {:?} reading unavailable (data[1]=0x{:02x}), but continuing",
            String::from_utf8_lossy(&sr.s_id).trim_matches('\0'),
            rsp.data[1]
        );
    }

    if is_scanning_disabled(rsp.data[1]) {
        sr.s_scanning_disabled = true;
        debug5!(
            "Sensor {:?} (#{:02x}) scanning disabled (data[1]=0x{:02x}), but continuing",
            String::from_utf8_lossy(&sr.s_id).trim_matches('\0'),
            sensor.keys.sensor_num,
            rsp.data[1]
        );
        // 不再立即返回，继续处理可能的有效数据
    }

    // 尝试读取传感器数据，即使标记为不可用也尝试处理
    if rsp.data_len >= 1 {
        let raw_reading = rsp.data[0];

        // 检查是否为无效读数（常见的无效值）
        let is_invalid_reading = match raw_reading {
            0xff => true, // 常见的无效值标识
            0x00 => {
                // 对于某些传感器，0x00 可能表示无效或断开状态
                // 需要结合状态位判断
                //rsp.data_len >= 2 && (sr.s_reading_unavailable || sr.s_scanning_disabled)

                //bgz 上一行注释，添加下面内容：
                // 对于离散传感器，0x00是有效值
                if sensor.is_threshold_sensor() {
                    // 只有模拟传感器才检查状态位
                    rsp.data_len >= 2 && (sr.s_reading_unavailable || sr.s_scanning_disabled)
                } else {
                    false // 离散传感器0x00为有效值
                }
            }
            _ => false,
        };

        if is_invalid_reading {
            debug5!(
                "Sensor {:?} (#{:02x}) has invalid reading 0x{:02x}, marking as unavailable",
                String::from_utf8_lossy(&sr.s_id).trim_matches('\0'),
                sensor.keys.sensor_num,
                raw_reading
            );
            sr.s_reading_valid = false;
            sr.s_reading_unavailable = true;
        } else {
            sr.s_reading_valid = true;
            sr.s_reading = raw_reading;
            debug5!(
                "Sensor {:?} (#{:02x}) reading set: {} (data[0]=0x{:02x}) valid={} unavailable={} disabled={}",
                String::from_utf8_lossy(&sr.s_id).trim_matches('\0'),
                sensor.keys.sensor_num,
                sr.s_reading,
                raw_reading,
                sr.s_reading_valid,
                sr.s_reading_unavailable,
                sr.s_scanning_disabled
            );
        }
    } else {
        debug5!(
            "Sensor {:?} (#{:02x}) no reading data available",
            String::from_utf8_lossy(&sr.s_id).trim_matches('\0'),
            sensor.keys.sensor_num
        );
    }

    if rsp.data_len > 2 {
        sr.s_data2 = rsp.data[2];
    }
    if rsp.data_len > 3 {
        sr.s_data3 = rsp.data[3];
    }

    // TODO: Implement analog reading conversion similar to C version
    // This would require porting additional support functions
    if sdr_sensor_has_analog_reading(intf, &mut sr) {
        sr.s_has_analog_value = true;
        debug5!(
            "Sensor {:?} has analog reading, converting value {}",
            String::from_utf8_lossy(&sr.s_id).trim_matches('\0'),
            sr.s_reading
        );

        if let Some(ref full) = sr.full {
            if sr.s_reading_valid {
                sr.s_a_val = full.sdr_convert_sensor_reading(sr.s_reading);
                debug5!(
                    "Sensor {:?} converted value: {} -> {}",
                    String::from_utf8_lossy(&sr.s_id).trim_matches('\0'),
                    sr.s_reading,
                    sr.s_a_val
                );
            }
            sr.s_a_units = ipmi_sdr_get_unit_string(
                full.cmn.unit.pct(),
                full.cmn.unit.modifier(),
                full.cmn.unit.unit_type.base,
                full.cmn.unit.unit_type.modifier,
            );
            debug5!(
                "Sensor {:?} units: {}",
                String::from_utf8_lossy(&sr.s_id).trim_matches('\0'),
                sr.s_a_units
            );
        }
    } else {
        debug5!(
            "Sensor {:?} does not have analog reading",
            String::from_utf8_lossy(&sr.s_id).trim_matches('\0')
        );
    }
    Some(sr)
}

pub fn sdr_sensor_has_analog_reading(intf: &mut dyn IpmiIntf, sr: &mut SensorReading) -> bool {
    // Compact sensors can't return analog values
    if sr.full.is_none() {
        return false;
    }
    let full = match &mut sr.full {
        Some(full_sensor) => &mut **full_sensor,
        None => return false,
    };

    let sensor = &full.cmn;
    if sensor.are_discrete() {
        return false; // Sensor specified as not having Analog Units
    }

    if !sensor.is_threshold_sensor() {
        if sensor.unit.pct()
            || sensor.unit.modifier() != 0
            || sensor.unit.unit_type.base != 0
            || sensor.unit.unit_type.modifier != 0
        {
            // Allow analog readings for sensors with valid units
            // Original code only allowed this for HP systems, but many other systems
            // also support analog readings for non-threshold sensors
            return true;
        } else {
            return false;
        }
    }

    // Handle linearization
    if full.linearization >= SDR_SENSOR_L_NONLINEAR && full.linearization <= 0x7F {
        // TODO: Implement get_sensor_reading_factors
        if !ipmi_sensor_get_sensor_reading_factors(intf, full, sr.s_reading) {
            sr.s_reading_valid = false;
            return false;
        }
    }
    true
}

///4,0x27
pub fn ipmi_sdr_get_sensor_thresholds(
    //mut intf:Box<dyn IpmiIntf>,
    intf: &mut dyn IpmiIntf, // 更通用的方式
    sensor: u8,
    target: u8,
    lun: u8,
    channel: u8,
) -> Option<IpmiRs> {
    let mut bridged_request = false;
    let mut save_addr = 0;
    let mut save_channel = 0;

    // Handle bridged requests

    if intf.context().bridge_to_sensor(target, channel) {
        bridged_request = true;
        save_addr = intf.context().target_addr();
        save_channel = intf.context().target_channel();
        intf.context().set_target_addr(target as u32);
        intf.context().set_target_channel(channel);
    }

    let mut data = [sensor];

    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_SE);
    req.msg.lun_mut(lun);
    req.msg.cmd = GET_SENSOR_THRESHOLDS;
    req.msg.data = data.as_mut_ptr();
    req.msg.data_len = data.len() as u16;

    // Send request and get response
    let rsp = intf.sendrecv(&req)?;

    // Restore original addressing if bridged
    if bridged_request {
        intf.context().set_target_addr(save_addr);
        intf.context().set_target_channel(save_channel);
    }

    Some(rsp)
}

pub fn ipmi_sdr_get_unit_string(pct: bool, relation: u8, base: u8, modifier: u8) -> String {
    //const UNIT_TYPE_LONGEST_NAME: usize = 15; // 根据实际单位字符串最大长度调整

    // 处理基础单位
    let basestr = UNIT_DESC.get(base as usize).copied().unwrap_or("invalid");

    // 处理修饰符单位
    let modstr = UNIT_DESC
        .get(modifier as usize)
        .copied()
        .unwrap_or("invalid");

    // 百分比前缀处理
    let pct_prefix = if pct { "% " } else { "" };

    match relation {
        SDR_UNIT_MOD_MUL => format!("{}{}*{}", pct_prefix, basestr, modstr),
        SDR_UNIT_MOD_DIV => format!("{}{}/{}", pct_prefix, basestr, modstr),
        SDR_UNIT_MOD_NONE => {
            if base == 0 && pct {
                "percent".to_string()
            } else {
                format!("{}{}", pct_prefix, basestr)
            }
        }
        _ => {
            if base == 0 && pct {
                "percent".to_string()
            } else {
                format!("{}{}", pct_prefix, basestr)
            }
        }
    }
}

//根据三项匹配获取Sensor信息
pub fn ipmi_sdr_find_sdr_bynumtype(
    intf: &mut dyn IpmiIntf,
    gen_id: u16,
    num: u8,
    sensor_type: u8,
) -> Option<SdrRecord> {
    // 初始化SDR迭代器
    let mut sdr_iter = match SdrIterator::new(intf, false) {
        Some(iter) => iter,
        None => {
            eprintln!("Unable to open SDR for reading");
            return None;
        }
    };

    //之前使用sdr_list_head全局变量缓存数据优化查找,
    //如果查找到了，直接返回,没有则遍历，然后将遍历的结果保存在sdr_list_head全局变量中
    //现在直接遍历SDR记录
    let records = match sdr_iter.sdrr_get_records() {
        Ok(records) => records,
        Err(e) => {
            log::error!("Failed to get SDR records: {}", e);
            return None;
        }
    };

    // 查找匹配的记录，find返回低一个匹配项，闭包要求返回一个布尔值，true表示匹配
    records.into_iter().find(|record| {
        match record.header.record_type {
            SDR_RECORD_TYPE_FULL_SENSOR | SDR_RECORD_TYPE_COMPACT_SENSOR => {
                let common = match SdrRecordCommonSensor::from_le_bytes(&record.raw) {
                    Ok(s) => s,
                    Err(_) => return false,
                };

                // 多种匹配策略：解决LoongArch等架构的gen_id匹配问题
                let owner_id_matches = common.keys.owner_id == (gen_id & 0x00ff) as u8        // 原有低字节匹配
                    || common.keys.owner_id == ((gen_id >> 8) & 0xff) as u8  // 高字节匹配
                    || common.keys.owner_id as u16 == gen_id                 // 完整匹配
                    || (gen_id == 0x0020 && common.keys.owner_id == 0x20); // 默认值匹配

                common.keys.sensor_num == num
                    && owner_id_matches
                    && common.sensor.sensor_type == sensor_type
            }
            SDR_RECORD_TYPE_EVENTONLY_SENSOR => {
                let eventonly = match SdrRecordEventonlySensor::from_le_bytes(&record.raw) {
                    Ok(s) => s,
                    Err(_) => return false,
                };
                // 多种匹配策略：解决LoongArch等架构的gen_id匹配问题
                let owner_id_matches = eventonly.keys.owner_id == (gen_id & 0x00ff) as u8        // 原有低字节匹配
                    || eventonly.keys.owner_id == ((gen_id >> 8) & 0xff) as u8  // 高字节匹配
                    || eventonly.keys.owner_id as u16 == gen_id                 // 完整匹配
                    || (gen_id == 0x0020 && eventonly.keys.owner_id == 0x20); // 默认值匹配

                eventonly.keys.sensor_num == num
                    && owner_id_matches
                    && eventonly.sensor_type == sensor_type
            }
            //之前添加缓存的代码不要了
            _ => false,
        }
    })
}
