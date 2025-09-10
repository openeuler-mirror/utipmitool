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
/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(dead_code)]
#![allow(clippy::if_same_then_else)]

use crate::commands::mc::BMC_GET_DEVICE_ID;
use crate::commands::sel::define::*;
use crate::commands::sel::describe::*;
use crate::commands::sel::entry::get_sensor_name_fast;
use crate::commands::sel::entry::try_next_entry_id;
use crate::commands::sel::entry::SelEntry;
use crate::error::IpmiError;
use crate::ipmi::intf::*;
use crate::ipmi::ipmi::IPMI_OEM;
use crate::ipmi::ipmi::*;

use crate::ipmi::oem::ipmi_get_oem;
use crate::ipmi::oem::ipmi_get_oem_id;
use crate::ipmi::picmg::picmg_discover;
use crate::ipmi::strings::IPMI_GENERIC_SENSOR_TYPE_VALS;
use crate::ipmi::strings::IPMI_OEM_SENSOR_TYPE_VALS;
use crate::ipmi::time::ipmi_timestamp_date;
use crate::ipmi::time::ipmi_timestamp_time;
use crate::ipmi::vita::vita_discover;

use std::error::Error;
use std::fmt::Write;

pub const ALL_OFFSETS_SPECIFIED: u8 = 0xff;

pub const IPMI_CMD_GET_SEL_INFO: u8 = 0x40;
pub const IPMI_CMD_GET_SEL_ALLOC_INFO: u8 = 0x41;
pub const IPMI_CMD_RESERVE_SEL: u8 = 0x42;
pub const IPMI_CMD_GET_SEL_ENTRY: u8 = 0x43;
pub const IPMI_CMD_ADD_SEL_ENTRY: u8 = 0x44;
pub const IPMI_CMD_PARTIAL_ADD_SEL_ENTRY: u8 = 0x45;
pub const IPMI_CMD_DELETE_SEL_ENTRY: u8 = 0x46;
pub const IPMI_CMD_CLEAR_SEL: u8 = 0x47;
pub const IPMI_CMD_GET_SEL_TIME: u8 = 0x48;
pub const IPMI_CMD_SET_SEL_TIME: u8 = 0x49;
pub const IPMI_CMD_GET_AUX_LOG_STATUS: u8 = 0x5A;
pub const IPMI_CMD_SET_AUX_LOG_STATUS: u8 = 0x5B;

// event_data[0] bit masks
pub const DATA_BYTE2_SPECIFIED_MASK: u8 = 0xc0;
pub const DATA_BYTE3_SPECIFIED_MASK: u8 = 0x30;
pub const EVENT_OFFSET_MASK: u8 = 0x0f;

// Dell specific OEM Byte in Byte 2/3 Mask
pub const OEM_CODE_IN_BYTE2: u8 = 0x80;
pub const OEM_CODE_IN_BYTE3: u8 = 0x20;

// MASK MACROS
pub const MASK_LOWER_NIBBLE: u8 = 0x0F;
pub const MASK_HIGHER_NIBBLE: u8 = 0xF0;

// Sensor type Macros
pub const SENSOR_TYPE_MEMORY: u8 = 0x0C;
pub const SENSOR_TYPE_CRIT_INTR: u8 = 0x13;
pub const SENSOR_TYPE_EVT_LOG: u8 = 0x10;
pub const SENSOR_TYPE_SYS_EVENT: u8 = 0x12;
pub const SENSOR_TYPE_PROCESSOR: u8 = 0x07;
pub const SENSOR_TYPE_OEM_SEC_EVENT: u8 = 0xC1;
pub const SENSOR_TYPE_VER_CHANGE: u8 = 0x2B;
pub const SENSOR_TYPE_FRM_PROG: u8 = 0x0F;
pub const SENSOR_TYPE_WTDOG: u8 = 0x23;
pub const SENSOR_TYPE_OEM_NFATAL_ERROR: u8 = 0xC2;
pub const SENSOR_TYPE_OEM_FATAL_ERROR: u8 = 0xC3;
pub const SENSOR_TYPE_TXT_CMD_ERROR: u8 = 0x20;
pub const SENSOR_TYPE_SUPERMICRO_OEM: u8 = 0xD0;

// End of Macro for DELL Specific
pub const SEL_OEM_TS_DATA_LEN: usize = 6;
pub const SEL_OEM_NOTS_DATA_LEN: usize = 13;

#[derive(Debug, PartialEq)]
enum RecordType {
    Standard,            // 0x00-0xBF
    KernelPanic,         // 0xF0
    OemWithTimestamp,    // [0xC0-0xE0)DF
    OemWithoutTimestamp, // [0xE0-0xFF]
}

impl RecordType {
    fn from(record_type: u8) -> Self {
        match record_type {
            0xF0 => Self::KernelPanic,
            0xC0..=0xDF => Self::OemWithTimestamp,
            0xE0..=0xFF => Self::OemWithoutTimestamp,
            _ => Self::Standard,
        }
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct SelEventRecord {
    pub record_id: u16,    //2
    pub record_type: u8,   //1
    pub sel_type: SelType, //13
}

impl SelEventRecord {
    // 0xc0,0xff会终止后续逻辑
    fn get_type(&self) -> RecordType {
        match self.record_type {
            0xF0 => RecordType::KernelPanic,
            0xC0..=0xDF => RecordType::OemWithTimestamp,
            0xE0..=0xFF => RecordType::OemWithoutTimestamp,
            _ => RecordType::Standard,
        }
    }

    fn type_string(&self) -> String {
        match self.get_type() {
            RecordType::KernelPanic => "Kernel Panic".to_string(),
            RecordType::OemWithTimestamp => format!("OEM record {:02X}", self.record_type),
            RecordType::OemWithoutTimestamp => format!("OEM record {:02X}", self.record_type),
            RecordType::Standard => "Standard".to_string(),
        }
    }

    fn oem_defined(&self) -> String {
        match self.get_type() {
            //RecordType::KernelPanic => "Kernel Panic".to_string(),
            RecordType::OemWithTimestamp => format!("OEM record {:?}", unsafe {
                self.sel_type.oem_ts_type.oem_defined
            }),
            RecordType::OemWithoutTimestamp => format!("OEM record {:?}", unsafe {
                self.sel_type.oem_nots_type.oem_defined
            }),
            //RecordType::Standard=> "Standard".to_string(),
            _ => "".to_string(),
        }
    }
    //fn ipmi_sel_oem_message(&self){} 都没有见过输出
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union SelType {
    pub standard_type: StandardSpecSelRec, //4+2+4+3
    pub oem_ts_type: OemTsSpecSelRec,      // 4+3+6
    pub oem_nots_type: OemNotsSpecSelRec,  //13
}

impl Default for SelType {
    fn default() -> Self {
        SelType {
            standard_type: StandardSpecSelRec::default(),
        }
    }
}

//用于格式化输出
pub trait SelRecordFormatter {
    fn output_fields(&self) -> Vec<String>;

    fn format_csv(&self, csv: bool) -> String {
        if csv {
            self.output_fields().join(",")
        } else {
            self.output_fields().join("|")
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct StandardSpecSelRec {
    pub timestamp: u32,
    pub gen_id: u16,
    pub evm_rev: u8,
    pub sensor_type: u8,
    pub sensor_num: u8,
    pub event_flag: u8, // 7 bits event_type, 1 bit event_dir (LSB)
    pub event_data: [u8; 3],
}
impl StandardSpecSelRec {
    pub fn event_type(&self) -> u8 {
        //self.event_flag >> 1
        self.event_flag & 0x7F
    }
    pub fn event_dir(&self) -> bool {
        //self.event_flag & 0x01 != 0
        (self.event_flag >> 7) & 0x01 != 0
    }

    #[inline]
    pub fn data(&self) -> (u8, u8, u8) {
        (self.event_data[0], self.event_data[1], self.event_data[2])
    }
    pub fn from(data: &[u8]) -> Self {
        //self.record_type in 0x00 .. 0xC0
        //13字节
        if data.len() < 13 {
            panic!("Data length is too short for StandardSpecSelRec");
        }
        Self {
            timestamp: u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            gen_id: u16::from_le_bytes([data[4], data[5]]),
            evm_rev: data[6],
            sensor_type: data[7],
            sensor_num: data[8],
            //type=data[9] & 0x7f;dir=data[9] & 0x80) >> 7;
            event_flag: data[9], // 7 bits event_type, 1 bit event_dir (LSB)
            event_data: [data[10], data[11], data[12]],
        }
    }

    pub fn format_output(&self, out: &mut SelDisplayData) {
        out.date = self.output_fields().first().cloned();
        out.time = self.output_fields().get(1).cloned();
    }
}
// 为StandardSpecSelRec实现trait
impl SelRecordFormatter for StandardSpecSelRec {
    //timestamp
    fn output_fields(&self) -> Vec<String> {
        let mut output_fields: Vec<String> = Vec::new();
        if self.timestamp < 0x20000000 {
            output_fields.push("Pre-Init".to_string());
            output_fields.push(format!("{:010}", self.timestamp));
        } else {
            output_fields.push(ipmi_timestamp_date(self.timestamp, true));
            output_fields.push(ipmi_timestamp_time(self.timestamp, true));
        }
        output_fields
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct OemTsSpecSelRec {
    pub timestamp: u32,
    pub manf_id: [u8; 3],
    pub oem_defined: [u8; SEL_OEM_TS_DATA_LEN],
}
impl OemTsSpecSelRec {
    pub fn from(data: &[u8]) -> Self {
        //self.record_type in  0xC0.. 0xE0
        //13字节
        if data.len() < 13 {
            panic!("Data length is too short for StandardSpecSelRec");
        }
        let _data = data[8..(8 + SEL_OEM_TS_DATA_LEN)]
            .try_into()
            .unwrap_or([0; SEL_OEM_TS_DATA_LEN]);
        Self {
            timestamp: u32::from_le_bytes([data[1], data[2], data[3], data[4]]),
            manf_id: [data[7], data[6], data[5]],
            oem_defined: _data,
        }
    }
    pub fn format_output(&self, out: &mut SelDisplayData) {
        out.date = self.output_fields().first().cloned();
        out.time = self.output_fields().get(1).cloned();
        out.oem_id = self.output_fields().get(2).cloned();
        out.oem_data = self.output_fields().get(3).cloned();
    }
}

// 为OemTsSpecSelRec实现trait
impl SelRecordFormatter for OemTsSpecSelRec {
    //timestamp|"OEM record %02x"|manf_id|tsdata|
    fn output_fields(&self) -> Vec<String> {
        let mut output_fields: Vec<String> = Vec::new();
        if self.timestamp < 0x20000000 {
            output_fields.push("Pre-Init".to_string());
            output_fields.push(format!("{:010}", self.timestamp));
        } else {
            output_fields.push(ipmi_timestamp_date(self.timestamp, true));
            output_fields.push(ipmi_timestamp_time(self.timestamp, true));
        }
        output_fields.push(format!(
            "{:02X}{:02X}{:02X}",
            self.manf_id[0], self.manf_id[1], self.manf_id[2]
        ));

        let mut oem_data = String::with_capacity(SEL_OEM_TS_DATA_LEN * 2);
        self.oem_defined
            .iter()
            .for_each(|byte| write!(&mut oem_data, "{:02X}", byte).unwrap());
        output_fields.push(oem_data);
        output_fields
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct OemNotsSpecSelRec {
    pub oem_defined: [u8; SEL_OEM_NOTS_DATA_LEN],
}

impl OemNotsSpecSelRec {
    pub fn from(data: &[u8]) -> Self {
        //self.record_type in   0xE0,0xFF
        //13字节
        if data.len() < 13 {
            panic!("Data length is too short for StandardSpecSelRec");
        }

        let _data = data[5..(5 + SEL_OEM_NOTS_DATA_LEN)]
            .try_into()
            .unwrap_or([0; SEL_OEM_NOTS_DATA_LEN]);
        Self { oem_defined: _data }
    }
    pub fn format_output(&self, out: &mut SelDisplayData) {
        out.oem_data = self.output_fields().first().cloned();
    }
}

// 为OemNotsSpecSelRec实现trait
impl SelRecordFormatter for OemNotsSpecSelRec {
    //|"OEM record %02x"|tsdata|
    fn output_fields(&self) -> Vec<String> {
        let mut oem_data = String::with_capacity(SEL_OEM_NOTS_DATA_LEN * 2);
        self.oem_defined
            .iter()
            .for_each(|byte| write!(&mut oem_data, "{:02X}", byte).unwrap());
        vec![oem_data]
    }
}

pub fn ipmi_sel_list(
    intf: &mut dyn IpmiIntf,
    order: Option<String>,
    count: Option<usize>,
    extend: bool,
) -> Result<(), Box<dyn Error>> {
    // 处理负数参数的情况
    let (sign, actual_count) = match (&order, count) {
        (Some(order_str), Some(n)) => match order_str.as_str() {
            "last" => (-1, n as i32),
            "first" => (1, n as i32),
            _ => return Err("Invalid order, use 'first' or 'last'".into()),
        },
        (None, Some(n)) => (1, n as i32), // 默认为 first
        (Some(order_str), None) => match order_str.as_str() {
            "last" => (-1, 0),
            "first" => (1, 0),
            _ => return Err("Invalid order, use 'first' or 'last'".into()),
        },
        (None, None) => (1, 0), // 显示所有
    };

    let count = actual_count * sign;
    ipmi_sel_savelist_entries(intf, count, None, extend)
}

// 性能优化：SEL处理上下文，缓存OEM信息和设备信息
thread_local! {
    static OEM_CACHE: std::cell::RefCell<Option<IPMI_OEM>> = const { std::cell::RefCell::new(None) };
    static DEVICE_INFO_CACHE: std::cell::RefCell<Option<Vec<u8>>> = const { std::cell::RefCell::new(None) };
}

// 性能优化：使用缓存的OEM ID获取函数
#[inline]
pub fn get_cached_oem_id(_intf: &mut dyn IpmiIntf) -> IPMI_OEM {
    OEM_CACHE.with(|cache| cache.borrow().unwrap_or(IPMI_OEM::Unknown))
}

// 性能优化：缓存设备信息，避免重复网络调用
pub fn get_cached_device_info(intf: &mut dyn IpmiIntf) -> Option<Vec<u8>> {
    DEVICE_INFO_CACHE.with(|cache| {
        if cache.borrow().is_none() {
            // 第一次调用，获取设备信息并缓存
            let mut req = IpmiRq::default();
            req.msg.netfn_mut(IPMI_NETFN_APP);
            req.msg.lun_mut(0);
            req.msg.cmd = BMC_GET_DEVICE_ID;
            req.msg.data = std::ptr::null_mut();
            req.msg.data_len = 0;

            if let Some(rsp) = intf.sendrecv(&req) {
                if rsp.ccode == 0 {
                    *cache.borrow_mut() = Some(rsp.data.to_vec());
                }
            }
        }
        cache.borrow().clone()
    })
}

// 性能优化：直接输出SEL条目，避免复杂的格式化
#[inline]
fn print_sel_entry_fast(
    intf: &mut dyn IpmiIntf,
    entry: &SelEntry,
    extend: bool,
    sdr_cache: &std::collections::HashMap<(u16, u8, u8), crate::commands::sdr::sdradd::SdrRecord>,
) {
    // 直接格式化输出，避免中间对象分配
    // 格式化record ID：右对齐，保持与ipmitool一致的3位空格填充
    print!("{:>4x} | ", entry.record_id);

    if entry.record_type == 0xf0 {
        // Linux kernel panic记录 - 优先处理，因为0xf0 >= 0xC0
        // 智能尝试多个字节范围来找到最完整的panic消息
        let possible_ranges = [
            (0, 13), // 完整数据
            (0, 8),  // 前8字节
            (2, 13), // 跳过前2字节
            (5, 13), // 跳过前5字节（原始方案）
            (0, 16), // 如果数据更长
        ];

        let mut best_panic = String::new();
        let mut best_len = 0;

        for &(start, end) in &possible_ranges {
            if entry.data.len() >= end && start < end {
                let panic_bytes = &entry.data[start..end.min(entry.data.len())];
                let panic_str = String::from_utf8_lossy(panic_bytes);
                let panic_clean = panic_str.trim_end_matches('\0').trim();

                // 选择最长且包含可打印字符的字符串
                if panic_clean.len() > best_len
                    && panic_clean
                        .chars()
                        .any(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
                {
                    best_panic = panic_clean.to_string();
                    best_len = panic_clean.len();
                }
            }
        }

        if best_panic.is_empty() {
            println!("Linux kernel panic: Unknown");
        } else {
            println!("Linux kernel panic: {}", best_panic);
        }
        return;
    }

    if entry.record_type >= 0xC0 && entry.record_type != 0xf0 {
        // OEM记录处理
        match entry.record_type {
            0xC0..0xE0 => {
                // OEM带时间戳记录
                let oem_ts = OemTsSpecSelRec::from(&entry.data);
                if oem_ts.timestamp < 0x20000000 {
                    print!(" Pre-Init  |{:010}| ", oem_ts.timestamp);
                } else {
                    let date_str = ipmi_timestamp_date(oem_ts.timestamp, true);
                    let time_str = ipmi_timestamp_time(oem_ts.timestamp, true);
                    print!("{} | {} | ", date_str, time_str);
                }
                println!("OEM record {:02X}", entry.record_type);
            }
            0xE0..=0xFF => {
                // OEM不带时间戳记录
                println!("OEM record {:02X}", entry.record_type);
            }
            _ => {
                println!("Unknown OEM record {:02X}", entry.record_type);
            }
        }
        return;
    }

    // 标准记录处理
    let standard = StandardSpecSelRec::from(&entry.data);

    // 时间戳处理（使用原有的时间格式化函数）
    if standard.timestamp < 0x20000000 {
        print!(" Pre-Init  |{:010}| ", standard.timestamp);
    } else {
        let date_str = ipmi_timestamp_date(standard.timestamp, true);
        let time_str = ipmi_timestamp_time(standard.timestamp, true);
        print!("{} | {} | ", date_str, time_str);
    }

    // 传感器信息
    let sensor_type_name = ipmi_get_sensor_type(intf, standard.sensor_type);
    if extend {
        if let Some(name) = get_sensor_name_fast(standard.sensor_type, standard.sensor_num) {
            print!("{} {} | ", sensor_type_name, name);
        } else {
            // 修复：只有当sensor_num不为0时才显示#0x{:02x}，与ipmitool保持一致
            if standard.sensor_num != 0 {
                print!("{} #0x{:02x} | ", sensor_type_name, standard.sensor_num);
            } else {
                print!("{} | ", sensor_type_name);
            }
        }
    } else {
        // 修复：只有当sensor_num不为0时才显示#0x{:02x}，与ipmitool保持一致
        if standard.sensor_num != 0 {
            print!("{} #0x{:02x} | ", sensor_type_name, standard.sensor_num);
        } else {
            print!("{} | ", sensor_type_name);
        }
    }

    // 事件描述（简化）
    let rec = SelEventRecord {
        record_id: entry.record_id,
        record_type: entry.record_type,
        sel_type: SelType {
            standard_type: standard,
        },
    };

    // 事件描述和状态
    if let Some(desc) = ipmi_get_event_desc(intf, &rec) {
        if standard.event_dir() {
            print!("{} | Deasserted", desc);
        } else {
            print!("{} | Asserted", desc);
        }
    } else if standard.event_dir() {
        print!("Unknown event | Deasserted");
    } else {
        print!("Unknown event | Asserted");
    }

    // 阈值信息（仅在extend模式且为特定传感器时）
    if extend && standard.event_type() == 1 && matches!(standard.sensor_type, 0x01 | 0x02 | 0x04) {
        let (data1, data2, data3) = standard.data();
        if (data1 >> 6) & 3 == 1 && (data1 >> 4) & 3 == 1 {
            // 按照ipmitool的方法：使用SDR记录进行转换
            let (reading, threshold) = if let Some(sdr_record) =
                sdr_cache.get(&(standard.gen_id, standard.sensor_num, standard.sensor_type))
            {
                // 从缓存中找到SDR记录，使用SDR转换 - 与ipmitool完全一致
                if let Ok(full_sensor) =
                    crate::commands::sdr::sdr::SdrRecordFullSensor::from_le_bytes(&sdr_record.raw)
                {
                    let reading_val = full_sensor.sdr_convert_sensor_reading(data2);
                    let threshold_val = full_sensor.sdr_convert_sensor_reading(data3);

                    // SDR转换成功

                    (reading_val, threshold_val)
                } else {
                    // SDR解析失败，使用原始值
                    (data2 as f64, data3 as f64)
                }
            } else {
                // 没有在缓存中找到SDR记录，使用原始值
                (data2 as f64, data3 as f64)
            };
            let comparison = if (data1 & 0xf) % 2 == 1 { ">" } else { "<" };
            let unit = match standard.sensor_type {
                0x01 => "degrees C",
                0x02 => "Volts",
                0x04 => "RPM",
                _ => "",
            };

            if standard.sensor_type == 0x02 {
                // 电压传感器 - 与ipmitool一致的2位小数格式
                let reading_str = format!("{:.2}", reading);
                let threshold_str = format!("{:.2}", threshold);
                print!(
                    " | Reading {} {} Threshold {} {}",
                    reading_str, comparison, threshold_str, unit
                );
            } else {
                // 其他传感器使用整数显示
                print!(
                    " | Reading {:.0} {} Threshold {:.0} {}",
                    reading, comparison, threshold, unit
                );
            }
        }
    }

    println!(); // 换行
}

pub fn ipmi_sel_savelist_entries(
    intf: &mut dyn IpmiIntf,
    count: i32,
    _savefile: Option<&str>,
    extend: bool,
) -> Result<(), Box<dyn Error>> {
    // 性能优化：预先缓存OEM信息，避免每个条目都进行网络调用
    OEM_CACHE.with(|cache| {
        if cache.borrow().is_none() {
            let oem_id = ipmi_get_oem_id(intf);
            *cache.borrow_mut() = Some(oem_id);
        }
    });

    // 性能优化：只在extend模式（elist命令）时才加载SDR缓存
    let sdr_cache = if extend {
        use crate::commands::sdr::iter::SdrIterator;
        use std::collections::HashMap;

        let mut cache: HashMap<(u16, u8, u8), crate::commands::sdr::sdradd::SdrRecord> =
            HashMap::new();

        // 加载所有SDR记录到缓存中
        if let Some(mut sdr_iter) = SdrIterator::new(intf, false) {
            if let Ok(records) = sdr_iter.sdrr_get_records() {
                for record in records {
                    // 解析SDR记录以获取key信息
                    if let Ok(common) =
                        crate::commands::sdr::SdrRecordCommonSensor::from_le_bytes(&record.raw)
                    {
                        // 使用多种gen_id匹配策略，确保与ipmi_sdr_find_sdr_bynumtype保持一致
                        let owner_id = common.keys.owner_id as u16;

                        // 创建多个可能的gen_id匹配项
                        let gen_ids = vec![
                            owner_id,                 // 直接匹配
                            owner_id | 0x0020,        // 与0x0020合并
                            (owner_id << 8) | 0x0020, // 高位匹配
                            0x0020,                   // 默认值
                        ];

                        for gen_id in gen_ids {
                            let key = (gen_id, common.keys.sensor_num, common.sensor.sensor_type);
                            cache.insert(key, record.clone());
                        }
                    }
                }
            }
        }
        cache
    } else {
        // list命令不需要SDR缓存，创建空缓存
        std::collections::HashMap::new()
    };

    let _evt = SelEventRecord::default();

    // 步骤1: 获取SEL信息
    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_STORAGE);
    req.msg.cmd = IPMI_CMD_GET_SEL_INFO;

    let rsp = match intf.sendrecv(&req) {
        Some(rsp) => rsp,
        None => return Err("Get SEL Info command failed".into()),
    };

    if rsp.ccode != 0 {
        return Err(format!(
            "Get SEL Info command failed: {}",
            IpmiError::CompletionCode(rsp.ccode)
        )
        .into());
    }

    if rsp.data[1] == 0 && rsp.data[2] == 0 {
        println!("SEL has no entries");
        return Ok(());
    }

    // 步骤2: 保留SEL资源 (关键性能优化!)
    let mut reserve_req = IpmiRq::default();
    reserve_req.msg.netfn_mut(IPMI_NETFN_STORAGE);
    reserve_req.msg.cmd = 0x42; // IPMI_CMD_RESERVE_SEL
    reserve_req.msg.data_len = 0;

    if let Some(reserve_rsp) = intf.sendrecv(&reserve_req) {
        if reserve_rsp.ccode != 0 {
            // Reserve SEL command failed, continue without reservation
        }
    } else {
        // Reserve SEL command failed, continue without reservation
    }

    let mut next_id: u16 = 0;
    let mut _curr_id: u16; // 使用下划线前缀表示可能未使用的变量
    let mut n = 0;

    let _entry = SelEntry::default();
    // 跳过前count的条目
    if count < 0 {
        let entries = u16::from_le_bytes([rsp.data[1], rsp.data[2]]);
        let mut count = count;
        if -count > entries as i32 {
            count = -(entries as i32);
        }
        // 返回next_id和evt,这里值需要id,不需要evt
        for _ in 0..(entries as i32 + count) {
            next_id = try_next_entry_id(intf, next_id, None::<fn(&mut dyn IpmiIntf, &SelEntry)>)?;
            if next_id == 0 {
                break;
            }
        }
    }

    // let mut file: Option<std::fs::File> = if let Some(path) = savefile {
    //     match std::fs::File::create(path) {
    //         Ok(f) => Some(f),
    //         Err(e) => {
    //             eprintln!("Failed to open file {}: {}", path, e);
    //             return Err(e.into());
    //         }
    //     }
    // } else {
    //     None
    // };

    while next_id != 0xffff {
        _curr_id = next_id;
        // 移除调试输出以提高性能
        // eprintln!("SEL Next ID: {:04x}", _curr_id);

        next_id = try_next_entry_id(
            intf,
            next_id,
            Some(|intf: &mut dyn IpmiIntf, entry: &SelEntry| {
                // 性能优化：直接输出，避免复杂的格式化
                print_sel_entry_fast(intf, entry, extend, &sdr_cache);
            }),
        )?;
        if next_id == 0 {
            break;
        }

        // if crate::VERBOSE.load(std::sync::atomic::Ordering::Relaxed) > 0 {
        //     crate::ipmi_sel_print_std_entry_verbose(intf, &evt);
        // } else {
        //     crate::ipmi_sel_print_std_entry(intf, &evt);
        // }

        //ipmi_sel_print_std_entry(intf, &mut evt);

        // if let Some(ref mut fp) = file {
        //     if binary {
        //         let buf = evt.as_bytes();
        //         fp.write_all(&buf[..16])?;
        //     } else {
        //         crate::ipmi_sel_print_event_file(intf, &evt, fp)?;
        //     }
        // }

        n += 1;
        if n == count {
            break;
        }
    }
    Ok(())
}

//填充SelEventRecord
pub fn ipmi_sel_get_std_entry(
    intf: &mut Box<dyn IpmiIntf>,
    id: u16,
    evt: &mut SelEventRecord,
) -> Result<u16, Box<dyn Error>> {
    // 准备请求数据
    let msg_data = [
        0x00, // no reserve id, not partial get
        0x00,
        (id & 0xff) as u8,
        (id >> 8) as u8,
        0x00, // offset
        0xff, // length
    ];

    // 构建请求
    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_STORAGE);
    req.msg.cmd = IPMI_CMD_GET_SEL_ENTRY;
    req.msg.data = msg_data.as_ptr() as *mut u8;
    req.msg.data_len = msg_data.len() as u16;

    // 发送请求并获取响应
    let rsp = match intf.sendrecv(&req) {
        Some(rsp) => rsp,
        None => return Err(format!("Get SEL Entry {:x} command failed", id).into()),
    };

    if rsp.ccode != 0 {
        return Err(format!(
            "Get SEL Entry {:x} command failed: {}",
            id,
            IpmiError::CompletionCode(rsp.ccode)
        )
        .into());
    }

    // 获取下一个条目ID
    let next = u16::from_le_bytes([rsp.data[0], rsp.data[1]]);

    // 移除调试输出以提高性能
    // eprintln!("SEL Entry: {:?}", &rsp.data[2..(rsp.data_len as usize) - 2]);
    // 如果有evt参数，填充事件记录

    *evt = SelEventRecord::default(); // 每次都要清空evt
    evt.record_id = u16::from_le_bytes([rsp.data[2], rsp.data[3]]);
    evt.record_type = rsp.data[4];

    match evt.record_type {
        //<0xC0
        0x00..0xC0 => {
            // 标准类型记录
            evt.sel_type.standard_type = StandardSpecSelRec {
                timestamp: u32::from_le_bytes([rsp.data[5], rsp.data[6], rsp.data[7], rsp.data[8]]),
                gen_id: u16::from_le_bytes([rsp.data[9], rsp.data[10]]),
                evm_rev: rsp.data[11],
                sensor_type: rsp.data[12],
                sensor_num: rsp.data[13],
                //data[14] & 0x7f;data[14] & 0x80) >> 7;
                event_flag: rsp.data[14], // 7 bits event_type, 1 bit event_dir (LSB)
                event_data: [rsp.data[15], rsp.data[16], rsp.data[17]],
            };
        }
        //[0xC0-0xE0)
        0xC0..0xE0 => {
            // OEM带时间戳记录
            evt.sel_type.oem_ts_type = OemTsSpecSelRec {
                timestamp: u32::from_le_bytes([rsp.data[5], rsp.data[6], rsp.data[7], rsp.data[8]]),
                manf_id: [rsp.data[11], rsp.data[10], rsp.data[9]],
                oem_defined: {
                    let mut data = [0u8; SEL_OEM_TS_DATA_LEN];
                    for (i, item) in data.iter_mut().enumerate().take(SEL_OEM_TS_DATA_LEN) {
                        if let Some(&byte) = rsp.data.get(12 + i) {
                            *item = byte;
                        }
                    }
                    data //12-16部分数据
                },
            };
        }
        //>= 0xE0,0xFF
        _ => {
            // OEM不带时间戳记录
            evt.sel_type.oem_nots_type = OemNotsSpecSelRec {
                oem_defined: {
                    let mut data = [0u8; SEL_OEM_NOTS_DATA_LEN];
                    for (i, item) in data.iter_mut().enumerate().take(SEL_OEM_NOTS_DATA_LEN) {
                        if let Some(&byte) = rsp.data.get(5 + i) {
                            *item = byte;
                        }
                    }
                    data // 5-18
                },
            };
        }
    }

    Ok(next)
}

// 类型定义
#[derive(Debug, Default)]
pub struct SelDisplayData {
    pub record_id: String,
    pub date: Option<String>,
    pub time: Option<String>,
    pub record_type: Option<String>,
    pub sensor_info: Option<String>,
    pub event_desc: Option<String>,
    pub event_state: Option<String>,
    pub threshold_info: Option<String>,
    pub additional_info: Option<String>,
    pub oem_id: Option<String>,
    pub oem_data: Option<String>,
    pub is_kernel_panic: bool,
    pub kernel_panic_msg: Option<String>,
}

impl SelDisplayData {
    pub fn format_csv(&self, csv: bool) -> String {
        // 严格按照 ipmitool 的输出格式
        let mut fields = Vec::new();

        // Record ID - 保持十六进制格式，右对齐，匹配 ipmitool
        // ipmitool 显示的是十六进制格式，如：b, c, d, e, f, 10, 11 等
        let record_id_hex = if let Ok(hex_val) = u32::from_str_radix(&self.record_id, 16) {
            format!("{:>4x}", hex_val) // 十六进制，4位右对齐
        } else {
            format!("{:>4}", self.record_id)
        };
        fields.push(record_id_hex);

        // Date - MM/DD/YYYY 格式
        if let Some(date) = &self.date {
            if date == "Pre-Init" {
                // Pre-Init记录：前后加空格以匹配ipmitool格式
                fields.push("  Pre-Init  ".to_string());
            } else {
                // 将 MM/DD/YY 转换为 MM/DD/YYYY 格式
                if date.len() == 8 && date.contains("/") {
                    let parts: Vec<&str> = date.split("/").collect();
                    if parts.len() == 3 {
                        let year = if parts[2].len() == 2 {
                            let yy: i32 = parts[2].parse().unwrap_or(0);
                            if yy >= 70 {
                                1900 + yy
                            } else {
                                2000 + yy
                            }
                        } else {
                            parts[2].parse().unwrap_or(2023)
                        };
                        fields.push(format!("{}/{}/{}", parts[0], parts[1], year));
                    } else {
                        fields.push(date.clone());
                    }
                } else {
                    fields.push(date.clone());
                }
            }
        } else {
            fields.push(String::new());
        }

        // Time - HH:MM:SS 格式（去掉时区信息）- 性能优化
        if let Some(time) = &self.time {
            let is_preinit_time = self.date.as_ref().map_or(false, |d| d == "Pre-Init");
            if is_preinit_time {
                fields.push(time.to_string());
            } else {
                // 优化：大部分时间字符串不包含时区，先快速检查
                if time.len() <= 8 || (!time.contains('+') && !time.contains('-')) {
                    fields.push(time.to_string());
                } else {
                    // 只有包含时区信息的时间才做分割处理
                    let time_only = if let Some(pos) = time.find('+') {
                        &time[..pos]
                    } else if let Some(pos) = time.find('-') {
                        &time[..pos]
                    } else {
                        time
                    }
                    .trim();
                    fields.push(time_only.to_string());
                }
            }
        } else {
            fields.push(String::new());
        }

        // Sensor Info - 性能优化：避免不必要的字符串操作
        if let Some(sensor_info) = &self.sensor_info {
            // 大部分sensor_info已经是正确格式，只有少数需要替换
            if sensor_info.contains(" #") && !sensor_info.contains(" #0x") {
                fields.push(sensor_info.replace(" #", " #0x"));
            } else {
                fields.push(sensor_info.clone());
            }
        } else {
            fields.push(String::new());
        }

        // Event Description
        if let Some(desc) = &self.event_desc {
            // 特殊处理：修复Linux kernel panic消息的截断问题
            let fixed_desc = fix_kernel_panic_message(desc);
            fields.push(fixed_desc);
        } else {
            fields.push(String::new());
        }

        // Event State
        if let Some(state) = &self.event_state {
            fields.push(state.clone());
        } else {
            fields.push(String::new());
        }

        // Threshold Info (仅在elist模式且有阈值信息时添加)
        if let Some(threshold_info) = &self.threshold_info {
            fields.push(threshold_info.clone());
        }

        // 特殊处理：Pre-Init记录使用特殊格式
        let is_preinit = self.date.as_ref().map_or(false, |d| d == "Pre-Init");
        if is_preinit && !csv {
            // Pre-Init记录：自定义格式以匹配ipmitool
            // ipmitool格式：22 |  Pre-Init  |0378893105| System ACPI Power State #0x25 | S4/S5: soft-off | Asserted
            return format!(
                "{} |{}|{}| {} | {} | {}",
                fields[0], // record_id
                fields[1], // "  Pre-Init  " (已包含前后空格)
                fields[2], // timestamp without spaces
                fields[3], // sensor_info
                fields[4], // event_desc
                fields[5]  // event_state
            );
        }

        // 特殊处理：对于没有时间戳的OEM记录，使用简化格式
        if self.date.is_none()
            && self.time.is_none()
            && self.sensor_info.is_none()
            && self.event_desc.is_some()
            && self.event_state.is_none()
        {
            // OEM记录格式：只有记录ID和事件描述
            let separator = if csv { "," } else { " | " };
            return format!("{}{}{}", fields[0], separator, fields[4]);
        }

        // 使用制表符进行列对齐，匹配 ipmitool 格式
        let separator = if csv { "," } else { " | " };
        fields.join(separator)
    }
}

// 定义表指针
pub fn ipmi_get_first_event_sensor_type(
    intf: &mut dyn IpmiIntf,
    sensor_type: u8,
    event_type: u8,
) -> Option<&'static IpmiEventSensorType> {
    let code = if event_type == 0x6f {
        sensor_type
    } else {
        event_type
    };

    // 构建要搜索的表序列
    static EMPTY_EVENT_TYPES: [IpmiEventSensorType; 0] = []; // 构造空表
    let tables: [&[IpmiEventSensorType]; 2] = if event_type == 0x6f {
        if (0xC0..0xF0).contains(&sensor_type) && ipmi_get_oem(intf) == IPMI_OEM::Kontron {
            [OEM_KONTRON_EVENT_TYPES, &EMPTY_EVENT_TYPES]
        } else if vita_discover(intf) != 0 {
            //intf.vita_avail()
            [VITA_SENSOR_EVENT_TYPES, SENSOR_SPECIFIC_EVENT_TYPES]
        } else {
            [SENSOR_SPECIFIC_EVENT_TYPES, &EMPTY_EVENT_TYPES]
        }
    } else {
        [GENERIC_EVENT_TYPES, &EMPTY_EVENT_TYPES]
    };

    // 使用迭代器链式查找
    tables
        .iter()
        .flat_map(|&table| table.iter())
        .find(|evt| !evt.desc.is_empty() && evt.code == code)
}

pub struct EventSensorTypeIter {
    tables: [&'static [IpmiEventSensorType]; 2], // 最多两个表
    current_table_idx: usize,
    current_index: usize,
    target_code: u8,
    found: bool,
}

impl EventSensorTypeIter {
    /// 创建新的迭代器
    pub fn new(intf: &mut dyn IpmiIntf, sensor_type: u8, event_type: u8) -> Self {
        let target_code = if event_type == 0x6f {
            sensor_type
        } else {
            event_type
        };

        // 构造表序列
        let tables = if event_type == 0x6f {
            if (0xC0..0xF0).contains(&sensor_type) && ipmi_get_oem(intf) == IPMI_OEM::Kontron {
                // Kontron OEM表
                [OEM_KONTRON_EVENT_TYPES, &[]]
            } else if vita_discover(intf) != 0 {
                // VITA表 + 通用传感器表
                [VITA_SENSOR_EVENT_TYPES, SENSOR_SPECIFIC_EVENT_TYPES]
            } else {
                // 仅通用传感器表
                [SENSOR_SPECIFIC_EVENT_TYPES, &[]]
            }
        } else {
            // 通用事件类型表
            [GENERIC_EVENT_TYPES, &[]]
        };

        Self {
            tables,
            current_table_idx: 0,
            current_index: 0,
            target_code,
            found: false,
        }
    }
}

impl Iterator for EventSensorTypeIter {
    type Item = &'static IpmiEventSensorType;

    fn next(&mut self) -> Option<Self::Item> {
        // 遍历所有表
        while self.current_table_idx < self.tables.len() {
            let table = self.tables[self.current_table_idx];

            // 遍历当前表
            while self.current_index < table.len() {
                let evt = &table[self.current_index];
                self.current_index += 1;

                // 跳过表结束标记（空描述符）
                if evt.desc.is_empty() {
                    break;
                }

                // 检查是否匹配目标代码
                if evt.code == self.target_code {
                    // 返回匹配的条目，下次还从当前这个表查询。
                    // 原来的逻辑不在找另外一个表。
                    self.found = true;

                    return Some(evt);
                }
            }
            if self.found {
                break;
            }
            // 第一个表没找到，移动到下一个表
            self.current_table_idx += 1;
            self.current_index = 0;
        }

        // 所有表都遍历完毕
        None
    }
}

// ipmi_quantaoem.c
pub fn ipmi_get_oem_desc(intf: &mut dyn IpmiIntf, rec: &SelEventRecord) -> Option<String> {
    match get_cached_oem_id(intf) {
        IPMI_OEM::Viking => get_viking_evt_desc(intf, rec),
        IPMI_OEM::Kontron => get_kontron_evt_desc(rec),
        IPMI_OEM::Dell => get_dell_evt_desc(intf, rec),
        IPMI_OEM::Supermicro | IPMI_OEM::Supermicro47488 => get_supermicro_evt_desc(intf, rec),
        IPMI_OEM::Quanta => oem_qct_get_evt_desc(intf, rec),
        _ => None,
    }
}

//传入的rec无法通过迭代器匹配到数据
pub fn ipmi_get_event_desc(intf: &mut dyn IpmiIntf, rec: &SelEventRecord) -> Option<String> {
    let std: &StandardSpecSelRec = unsafe { &rec.sel_type.standard_type };
    let mut sfx: Option<String> = None;

    // OEM event type range
    if (std.event_type() >= 0x70) && (std.event_type() < 0x7F) {
        return ipmi_get_oem_desc(intf, rec);
    } else if std.event_type() == 0x6f {
        if std.sensor_type >= 0xC0 && std.sensor_type < 0xF0 {
            match get_cached_oem_id(intf) {
                IPMI_OEM::Kontron => {
                    #[allow(clippy::cast_enum_truncation)]
                    {
                        // Use OEM type supplied description for Kontron sensor
                    }
                }
                IPMI_OEM::Dell => {
                    if (std.event_data[0] & DATA_BYTE2_SPECIFIED_MASK) == OEM_CODE_IN_BYTE2
                        || (std.event_data[0] & DATA_BYTE3_SPECIFIED_MASK) == OEM_CODE_IN_BYTE3
                    {
                        sfx = ipmi_get_oem_desc(intf, rec);
                    }
                }
                IPMI_OEM::Supermicro | IPMI_OEM::Supermicro47488 | IPMI_OEM::Quanta => {
                    sfx = ipmi_get_oem_desc(intf, rec);
                }
                _ => {
                    // Use standard type supplied description for OEM sensor
                }
            }
        } else {
            match get_cached_oem_id(intf) {
                IPMI_OEM::Supermicro | IPMI_OEM::Supermicro47488 | IPMI_OEM::Quanta => {
                    sfx = ipmi_get_oem_desc(intf, rec);
                }
                _ => {}
            } // ipmi_get_event_desc
        }
        // DELL OEM special handling
        if ipmi_get_oem(intf) == IPMI_OEM::Dell {
            if (std.event_data[0] & DATA_BYTE2_SPECIFIED_MASK) == OEM_CODE_IN_BYTE2
                || (std.event_data[0] & DATA_BYTE3_SPECIFIED_MASK) == OEM_CODE_IN_BYTE3
            {
                sfx = ipmi_get_oem_desc(intf, rec);
            } else if std.event_data[0] == SENSOR_TYPE_OEM_SEC_EVENT && std.sensor_num == 0x23 {
                sfx = ipmi_get_oem_desc(intf, rec);
            }
        }
    }

    let offset = std.event_data[0] & 0xf;

    //原来是for循环遍历，找到且if条件满足的就返回
    //现在在ipmi_get_first_event_sensor_type内直接找，找到后if满足就返回

    let it = EventSensorTypeIter::new(intf, std.sensor_type, std.event_type());
    //获取Failure detected ()
    //没找到合适的e，返回None
    for e in it {
        let data_match = (e.data == ALL_OFFSETS_SPECIFIED)
            || ((std.event_data[0] & DATA_BYTE2_SPECIFIED_MASK) != 0
                && e.data == std.event_data[1]);
        if e.offset == offset && !e.desc.is_empty() && data_match {
            let desc = if let Some(sfx_str) = sfx {
                format!("{} ({})", e.desc, sfx_str)
            } else {
                e.desc.to_string()
            };
            return Some(desc);
        }
    }

    // OEM secondary events fallback
    if sfx.is_some() && std.event_type() == 0x6F {
        let mut flag = 0u8;
        match std.sensor_type {
            SENSOR_TYPE_FRM_PROG => {
                if offset == 0x0F {
                    flag = 0x01;
                }
            }
            SENSOR_TYPE_OEM_SEC_EVENT => {
                if offset == 0x01 || offset == 0x02 || offset == 0x03 {
                    flag = 0x01;
                }
            }
            SENSOR_TYPE_OEM_NFATAL_ERROR => {
                if offset == 0x00 || offset == 0x02 {
                    flag = 0x01;
                }
            }
            SENSOR_TYPE_OEM_FATAL_ERROR => {
                if offset == 0x01 {
                    flag = 0x01;
                }
            }
            SENSOR_TYPE_SUPERMICRO_OEM => {
                flag = 0x02;
            }
            _ => {}
        }
        if flag != 0 {
            let sfx_str = sfx.unwrap();
            if flag == 0x02 {
                return Some(sfx_str);
            } else {
                return Some(format!("({})", sfx_str));
            }
        }
    }
    None
}

pub fn ipmi_get_next_event_sensor_type(
    evt: &'static IpmiEventSensorType,
) -> Option<&'static IpmiEventSensorType> {
    // 假设所有事件类型都存储在一个静态数组中
    // 这里需要根据实际存储方式实现
    // 示例实现：
    let all_events = [
        &GENERIC_EVENT_TYPES,
        &SENSOR_SPECIFIC_EVENT_TYPES,
        &VITA_SENSOR_EVENT_TYPES,
        &OEM_KONTRON_EVENT_TYPES,
    ];

    all_events
        .iter()
        .flat_map(|&table| table.iter())
        .skip_while(|e| *e as *const _ != evt as *const _)
        .skip(1)
        .find(|e| e.code == evt.code)
}
// Assume SENSOR_TYPE_MAX and ipmi_generic_sensor_type_vals are defined elsewhere
pub fn ipmi_get_generic_sensor_type(code: u8) -> Option<&'static str> {
    if (code as usize) < IPMI_GENERIC_SENSOR_TYPE_VALS.len() {
        Some(IPMI_GENERIC_SENSOR_TYPE_VALS[code as usize])
    } else {
        None
    }
}

// Assume OemValStr and IPMI_OEM are defined elsewhere
pub fn ipmi_get_oem_sensor_type(intf: &mut dyn IpmiIntf, code: u8) -> Option<&'static str> {
    let iana = ipmi_get_oem(intf) as u32;
    let mut found: Option<&'static str> = None;

    for v in IPMI_OEM_SENSOR_TYPE_VALS.iter() {
        // if v.str_val.is_none() {
        //     break;
        // }
        if v.oem == iana && v.code == code {
            return Some(v.desc);
        }
        if (picmg_discover(intf) == 0 && v.oem == IPMI_OEM::PICMG as u32 && v.code == code)
            || (vita_discover(intf) != 0 && v.oem == IPMI_OEM::VITA as u32 && v.code == code)
        {
            found = Some(v.desc);
        }
    }
    found
}

pub fn ipmi_get_sensor_type(intf: &mut dyn IpmiIntf, code: u8) -> &'static str {
    let type_str = if code >= 0xC0 {
        ipmi_get_oem_sensor_type(intf, code)
    } else {
        ipmi_get_generic_sensor_type(code)
    };
    type_str.unwrap_or("Unknown")
}

/// 修复Linux kernel panic消息的截断问题
fn fix_kernel_panic_message(desc: &str) -> String {
    // 只修复明确的kernel panic截断模式，避免误匹配
    match desc.trim() {
        "s: Fatal" => "Linux kernel panic: Oops: Fatal".to_string(),
        "ception" => "Linux kernel panic:  exception".to_string(),
        "al excep" => "Linux kernel panic: Fatal excep".to_string(),
        "n" => "Linux kernel panic: tion".to_string(),
        _ => {
            // 不对其他文本做任何修改，避免误匹配
            desc.to_string()
        }
    }
}
