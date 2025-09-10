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
#![allow(clippy::module_inception)]

pub mod iter;
pub mod sdr;
pub mod sdradd;
pub mod types;

use ipmi_macros::AsBytes;

use unpack::RAWDATA;

use crate::commands::sdr::sdr::SdrRecordHeader;
use crate::commands::sdr::types::{get_sdr_record_type_name, SdrRepositoryInfo};
use crate::error::IpmiResult;
use crate::ipmi::context::OutputContext;
use crate::ipmi::ipmi::IpmiRq;
use crate::{debug2, debug3};
use clap::Subcommand;
use std::error::Error;

#[derive(Subcommand, Debug)]
pub enum SdrCommand {
    /// Display information about the SDR repository
    Info,
    /// List SDR entries (standard format)
    List {
        /// Record type filter
        #[arg(value_enum)]
        record_type: Option<SdrRecordType>,
    },
    /// List SDR entries (extended format with sensor number and entity info)
    Elist {
        /// Record type filter
        #[arg(value_enum)]
        record_type: Option<SdrRecordType>,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum SdrRecordType {
    /// All SDR Records
    All,
    /// Full Sensor Record
    Full,
    /// Compact Sensor Record
    Compact,
    /// Event-Only Sensor Record
    Event,
    /// Management Controller Locator Record
    Mcloc,
    /// FRU Locator Record
    Fru,
    /// Generic Device Locator Record
    Generic,
}

pub fn ipmi_sdr_main(
    command: SdrCommand,
    mut intf: Box<dyn crate::ipmi::intf::IpmiIntf>,
) -> Result<(), Box<dyn Error>> {
    match command {
        SdrCommand::Info => ipmi_sdr_info(intf),
        SdrCommand::List { record_type } => {
            // 使用builder模式设置标准格式
            intf.context().output = OutputContext::default().with_extended(false);
            let type_filter = match record_type {
                None => 0xfe,                     // Default: all except OEM
                Some(SdrRecordType::All) => 0xff, // All records including OEM
                Some(SdrRecordType::Full) => {
                    crate::commands::sdr::types::SDR_RECORD_TYPE_FULL_SENSOR
                }
                Some(SdrRecordType::Compact) => {
                    crate::commands::sdr::types::SDR_RECORD_TYPE_COMPACT_SENSOR
                }
                Some(SdrRecordType::Event) => {
                    crate::commands::sdr::types::SDR_RECORD_TYPE_EVENTONLY_SENSOR
                }
                Some(SdrRecordType::Mcloc) => {
                    crate::commands::sdr::types::SDR_RECORD_TYPE_MC_DEVICE_LOCATOR
                }
                Some(SdrRecordType::Fru) => {
                    crate::commands::sdr::types::SDR_RECORD_TYPE_FRU_DEVICE_LOCATOR
                }
                Some(SdrRecordType::Generic) => {
                    crate::commands::sdr::types::SDR_RECORD_TYPE_GENERIC_DEVICE_LOCATOR
                }
            };
            ipmi_sdr_list(intf, type_filter)
        }
        SdrCommand::Elist { record_type } => {
            // 使用builder模式设置扩展格式
            intf.context().output = OutputContext::default().with_extended(true);
            let type_filter = match record_type {
                None => 0xfe,                     // Default: all except OEM
                Some(SdrRecordType::All) => 0xff, // All records including OEM
                Some(SdrRecordType::Full) => {
                    crate::commands::sdr::types::SDR_RECORD_TYPE_FULL_SENSOR
                }
                Some(SdrRecordType::Compact) => {
                    crate::commands::sdr::types::SDR_RECORD_TYPE_COMPACT_SENSOR
                }
                Some(SdrRecordType::Event) => {
                    crate::commands::sdr::types::SDR_RECORD_TYPE_EVENTONLY_SENSOR
                }
                Some(SdrRecordType::Mcloc) => {
                    crate::commands::sdr::types::SDR_RECORD_TYPE_MC_DEVICE_LOCATOR
                }
                Some(SdrRecordType::Fru) => {
                    crate::commands::sdr::types::SDR_RECORD_TYPE_FRU_DEVICE_LOCATOR
                }
                Some(SdrRecordType::Generic) => {
                    crate::commands::sdr::types::SDR_RECORD_TYPE_GENERIC_DEVICE_LOCATOR
                }
            };
            ipmi_sdr_list(intf, type_filter)
        }
    }
}

/// Display SDR repository information (matching C ipmi_sdr_print_info)
pub fn ipmi_sdr_info(mut intf: Box<dyn crate::ipmi::intf::IpmiIntf>) -> Result<(), Box<dyn Error>> {
    debug3!("Starting SDR repository info query");

    // Get SDR repository info using the same logic as C version
    let repo_info = get_sdr_repository_info(&mut intf)?;

    // Use structured output with format_standard (matching C output)
    print!("{}", repo_info.format_standard());

    Ok(())
}

/// Get SDR Repository Info (matching C ipmi_sdr_get_info function)
fn get_sdr_repository_info(
    intf: &mut Box<dyn crate::ipmi::intf::IpmiIntf>,
) -> IpmiResult<SdrRepositoryInfo> {
    debug2!("Sending Get SDR Repository Info command");

    let mut req = IpmiRq::default();
    req.msg.netfn_mut(crate::ipmi::ipmi::IPMI_NETFN_STORAGE);
    req.msg.cmd = 0x20; // IPMI_GET_SDR_REPOSITORY_INFO

    let rsp = intf
        .sendrecv(&req)
        .ok_or(crate::error::IpmiError::ResponseError)?;

    if rsp.ccode != 0 {
        debug3!(
            "Get SDR Repository Info failed with completion code: 0x{:02x}",
            rsp.ccode
        );
        return Err(crate::error::IpmiError::CompletionCode(rsp.ccode));
    }

    debug3!("Got SDR Repository Info response: {} bytes", rsp.data_len);

    SdrRepositoryInfo::from_response_data(&rsp.data[..rsp.data_len as usize])
}

/// Execute SDR list command (matching C ipmi_sdr_list)
/// 注意：这个函数实际上是显示传感器读数，而不是SDR记录列表
/// 根据intf.context().output.extended决定是否使用扩展格式
pub fn ipmi_sdr_list(
    mut intf: Box<dyn crate::ipmi::intf::IpmiIntf>,
    type_filter: u8,
) -> Result<(), Box<dyn Error>> {
    let extended = intf.context().output.extended;
    debug2!(
        "Starting SDR list command with type filter: 0x{:02x}, extended: {}",
        type_filter,
        extended
    );

    // 使用现有的传感器列表功能，这才是C版本真正做的事情
    // C版本的 "sdr list" 实际上调用 ipmi_sdr_print_sensor_fc() 来显示传感器读数
    use crate::commands::sensor::sensor::ipmi_sensor_list;

    // 调用传感器列表功能，从IpmiIntf内部获取OutputContext
    ipmi_sensor_list(intf)?;

    Ok(())
}

/// Print SDR record (matching C ipmi_sdr_print_rawentry)
#[allow(dead_code)]
fn print_sdr_record(header: &SdrRecordHeader, record_data: &[u8], ctx: &OutputContext) {
    if ctx.csv {
        // CSV format: record_id,record_type,record_type_name,record_length
        println!(
            "{},{},\"{}\",{}",
            header.id,
            header.record_type,
            get_sdr_record_type_name(header.record_type),
            header.length
        );
    } else {
        // Standard format (matching C version)
        println!(
            "Record ID: 0x{:04x}, Type: 0x{:02x} ({}), Length: {} bytes",
            header.id,
            header.record_type,
            get_sdr_record_type_name(header.record_type),
            header.length
        );

        if ctx.verbose > 0 {
            // Print hex dump of record data (matching C version)
            print!("Data: ");
            for (i, byte) in record_data.iter().enumerate() {
                if i > 0 && i % 16 == 0 {
                    print!("\n      ");
                }
                print!("{:02x} ", byte);
            }
            println!();
        }
    }
}

#[derive(Debug, AsBytes, RAWDATA)]
#[repr(C)]
pub struct SdrRecordCommonSensor {
    pub keys: SensorKeys,    //3
    pub entity: EntityId,    //2
    pub sensor: SensorInfo,  //3
    pub event_type: u8,      //1
    pub mask: SdrRecordMask, //6=2*3成员SdrRecordMaskType是联合体
    pub unit: UnitInfo,      //3
} //18字节

impl SdrRecordCommonSensor {
    pub fn are_discrete(&self) -> bool {
        self.unit.analog() == 3
    }
    pub fn is_threshold_sensor(&self) -> bool {
        self.event_type == 1
    }
}

#[derive(Debug, AsBytes, RAWDATA)]
#[repr(C)]
pub struct SensorKeys {
    pub owner_id: u8,
    pub lun_channel: u8, // Combined field for lun(2bits), reserved(2bits), channel(4bits)
    pub sensor_num: u8,
}

impl SensorKeys {
    /// 获取低2位的LUN值
    pub fn lun(&self) -> u8 {
        self.lun_channel & 0b0000_0011
    }

    /// 获取高4位的Channel值
    pub fn channel(&self) -> u8 {
        (self.lun_channel & 0b1111_0000) >> 4
    }
}

#[derive(AsBytes, RAWDATA)]
#[repr(C)]
pub struct EntityId {
    pub id: u8,               // physical entity id
    pub instance_logical: u8, // Combined field: instance(7bits), logical(1bit),physical/logical
}

impl EntityId {
    pub fn new(id: u8, logical: bool, instance: u8) -> Self {
        // 固定 logical 为最高位，instance 为低7位
        let mut instance_logical = 0u8;
        instance_logical |= (logical as u8) << 7; // logical 占第7位（最高位）
        instance_logical |= instance & 0x7F; // instance 占低7位（0x7F = 0b0111_1111）

        Self {
            id,
            instance_logical,
        }
    }

    #[inline]
    pub fn logical(&self) -> bool {
        (self.instance_logical & 0x80) != 0 //清除低7位
    }

    #[inline]
    pub fn instance(&self) -> u8 {
        self.instance_logical & 0x7F //清除高1位
    }

    #[inline]
    pub fn set_logical(&mut self, logical: bool) {
        if logical {
            self.instance_logical |= 0x80;
        } else {
            self.instance_logical &= 0x7F;
        }
    }

    #[inline]
    pub fn set_instance(&mut self, instance: u8) {
        self.instance_logical = (self.instance_logical & 0x80) | (instance & 0x7F);
    }
}
impl std::fmt::Debug for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EntityId {{ id: 0x{:02x}, instance: 0x{:02x}, logical: {} }}",
            self.id,
            self.instance(),
            self.logical()
        )
    }
}

#[derive(Debug, AsBytes, RAWDATA)]
#[repr(C)]
pub struct SensorInfo {
    pub init: InitFlags,               //InitFlags,8 bits 位域
    pub capabilities: CapabilityFlags, //CapabilityFlags 8 bits
    pub sensor_type: u8,
}
//#[derive(Debug,AsBytes)]

bitflags! {
    #[derive(Debug)]
    #[repr(C)]
    pub struct InitFlags: u8 {
        const SENSOR_SCAN  = 1 << 0;
        const EVENT_GEN    = 1 << 1;
        const TYPE         = 1 << 2;
        const HYSTERESIS   = 1 << 3;
        const THRESHOLDS   = 1 << 4;
        const EVENTS       = 1 << 5;
        const SCANNING     = 1 << 6;
        const RESERVED     = 1 << 7;
    }
}

impl InitFlags {
    fn from_le_bytes(data: &[u8]) -> Result<Self, &'static str> {
        if data.is_empty() {
            // 如果切片长度不足，返回默认值或采取其他措施
            return Err("Input bytes too short");
        }
        // 假设是小端字节序，从切片前两个字节读取u16值
        //let bits = LittleEndian::read_u16(&slice[0..2]);
        let bits = u8::from_le_bytes([data[0]]);
        Ok(InitFlags::from_bits_truncate(bits))
    }
}

#[derive(AsBytes, RAWDATA)]
#[repr(C)]
pub struct CapabilityFlags {
    flags: u8, // event_msg:2, threshold:2, hysteresis:2, rearm:1, ignore:1
}

impl CapabilityFlags {
    // event_msg: 低2位 (bits 0-1)
    pub fn event_msg(&self) -> u8 {
        self.flags & 0b0000_0011
    }

    pub fn set_event_msg(&mut self, value: u8) {
        self.flags = (self.flags & !0b0000_0011) | (value & 0b0000_0011);
    }

    // threshold: 2-3位 (bits 2-3)
    pub fn threshold(&self) -> u8 {
        (self.flags >> 2) & 0b0000_0011
    }

    pub fn set_threshold(&mut self, value: u8) {
        self.flags = (self.flags & !0b0000_1100) | ((value & 0b0000_0011) << 2);
    }

    // hysteresis: 4-5位 (bits 4-5)
    pub fn hysteresis(&self) -> u8 {
        (self.flags >> 4) & 0b0000_0011
    }

    pub fn set_hysteresis(&mut self, value: u8) {
        self.flags = (self.flags & !0b0011_0000) | ((value & 0b0000_0011) << 4);
    }

    pub fn rearm(&self) -> bool {
        (self.flags & 0b0100_0000) != 0
    }

    pub fn set_rearm(&mut self, val: bool) {
        if val {
            self.flags |= 0b0100_0000;
        } else {
            self.flags &= !0b0100_0000;
        }
    }

    // ignore: 位7
    pub fn ignore(&self) -> bool {
        (self.flags & 0b1000_0000) != 0
    }

    pub fn set_ignore(&mut self, val: bool) {
        if val {
            self.flags |= 0b1000_0000;
        } else {
            self.flags &= !0b1000_0000;
        }
    }
}

impl std::fmt::Debug for CapabilityFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CapabilityFlags {{\n\t\
            event_msg: 0x{:x}, \
            threshold: 0x{:x}, \
            hysteresis: 0x{:x}, \
            rearm: {}, \
            ignore: {}\n}}",
            self.event_msg(),
            self.threshold(),
            self.hysteresis(),
            self.rearm(),
            self.ignore()
        )
    }
}

#[derive(AsBytes, RAWDATA)]
#[repr(C)]
pub struct UnitInfo {
    pub flags: u8, // pct:1, modifier:2, rate:3, analog:2
    pub unit_type: UnitType,
}

#[derive(Debug, AsBytes, RAWDATA)]
#[repr(C)]
pub struct UnitType {
    pub base: u8,
    pub modifier: u8,
}

impl UnitInfo {
    // pct:1 (LSB)
    pub fn pct(&self) -> bool {
        (self.flags & 0x01) != 0
    }

    pub fn set_pct(&mut self, val: bool) {
        self.flags = (self.flags & !0x01) | (val as u8);
    }

    // modifier:2 (bits 1-2)
    pub fn modifier(&self) -> u8 {
        (self.flags >> 1) & 0x03
    }

    pub fn set_modifier(&mut self, val: u8) {
        self.flags = (self.flags & !0x06) | ((val & 0x03) << 1);
    }

    // rate:3 (bits 3-5)
    pub fn rate(&self) -> u8 {
        (self.flags >> 3) & 0x07
    }

    pub fn set_rate(&mut self, val: u8) {
        self.flags = (self.flags & !0x38) | ((val & 0x07) << 3);
    }

    // analog:2 (MSB bits 6-7)
    pub fn analog(&self) -> u8 {
        (self.flags >> 6) & 0x03
    }

    pub fn set_analog(&mut self, val: u8) {
        self.flags = (self.flags & !0xC0) | ((val & 0x03) << 6);
    }
}
/*
            \tbase: 0x{:02x},\n\
            \tmodifier_type: 0x{:02x}\n\
*/
impl std::fmt::Debug for UnitInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 第一次输出基础字段
        write!(f, "UnitInfo {{\n\tFlags {{\n\t\tpct: {}, modifier: 0x{:x}, rate: 0x{:x}, analog: 0x{:x}\n\t}}",
            self.pct(), self.modifier(), self.rate(), self.analog())?;

        // 第二次输出嵌套结构
        write!(
            f,
            "\n\tUnitType {{\n\t\tbase: 0x{:02x}, modifier: 0x{:02x}\n\t}}\n}}",
            self.unit_type.base, self.unit_type.modifier
        )
    }
}

/* IPMI 2.0, Table 43-1, byte 21[7:6] Analog (numeric) Data Format */

// 数据格式常量
pub const SDR_UNIT_FMT_UNSIGNED: u8 = 0; // unsigned
pub const SDR_UNIT_FMT_1S_COMPL: u8 = 1; // 1's complement (signed)
pub const SDR_UNIT_FMT_2S_COMPL: u8 = 2; // 2's complement (signed)
pub const SDR_UNIT_FMT_NA: u8 = 3; // does not return analog reading

// 速率单位常量
pub const SDR_UNIT_RATE_NONE: u8 = 0; // none
pub const SDR_UNIT_RATE_MICROSEC: u8 = 1; // per us
pub const SDR_UNIT_RATE_MILLISEC: u8 = 2; // per ms
pub const SDR_UNIT_RATE_SEC: u8 = 3; // per s
pub const SDR_UNIT_RATE_MIN: u8 = 4; // per min
pub const SDR_UNIT_RATE_HR: u8 = 5; // per hour
pub const SDR_UNIT_RATE_DAY: u8 = 6; // per day
pub const SDR_UNIT_RATE_RSVD: u8 = 7; // reserved

// 修饰符单位常量
pub const SDR_UNIT_MOD_NONE: u8 = 0; // none
pub const SDR_UNIT_MOD_DIV: u8 = 1; // Basic Unit / Modifier Unit
pub const SDR_UNIT_MOD_MUL: u8 = 2; // Basic Unit * Modifier Unit
pub const SDR_UNIT_MOD_RSVD: u8 = 3; // Reserved

// 百分比标志常量
pub const SDR_UNIT_PCT_NO: u8 = 0;
pub const SDR_UNIT_PCT_YES: u8 = 1;

#[derive(Debug, AsBytes, RAWDATA)]
#[repr(C)]
pub struct SdrRecordMask {
    // discrete:DiscreteEvent,
    threshold: ThresholdMask,
}
//之前有联合体，根据具体的参数，输出不同的成员，是延迟使用类型
//不能用枚举，因为枚举需要初始化的时候就要解析确定类型
//需要占位符，在使用的时候再解析
//ipmi_sdr_print_sensor_mask内使用discrete已经禁用了。
//#[derive(Debug)]
// pub enum SdrRecordMaskType {
//     Discrete(DiscreteEvent),//DISCRETE_SENSOR 1
//     Threshold(ThresholdMask),//ANALOG_SENSOR 0,使用参数解析
// }

#[derive(Debug, AsBytes)]
#[repr(C)]
pub struct DiscreteEvent {
    assert_event: u16,   // assertion event mask
    deassert_event: u16, // de-assertion event mask
    read: u16,           // discrete reading mask
}

use bitflags::bitflags;

bitflags! {
    /// 断言事件标志位（第一部分）
    #[derive(Debug,Clone,Copy)]
    pub struct AssertFlags: u16 {
        const LNC_LOW    = 1 << 0;
        const LNC_HIGH   = 1 << 1;
        const LCR_LOW    = 1 << 2;
        const LCR_HIGH   = 1 << 3;
        const LNR_LOW    = 1 << 4;
        const LNR_HIGH   = 1 << 5;
        const UNC_LOW    = 1 << 6;
        const UNC_HIGH   = 1 << 7;
        const UCR_LOW    = 1 << 8;
        const UCR_HIGH   = 1 << 9;
        const UNR_LOW    = 1 << 10;
        const UNR_HIGH   = 1 << 11;
        const STATUS_LNC = 1 << 12;
        const STATUS_LCR = 1 << 13;
        const STATUS_LNR = 1 << 14;
        const RESERVED   = 1 << 15;
    }
}

bitflags! {
    /// 解除断言事件标志位（第二部分）
    #[derive(Debug)]
    pub struct DeassertFlags: u16 {
        const LNC_LOW    = 1 << 0;
        const LNC_HIGH   = 1 << 1;
        const LCR_LOW    = 1 << 2;
        const LCR_HIGH   = 1 << 3;
        const LNR_LOW    = 1 << 4;
        const LNR_HIGH   = 1 << 5;
        const UNC_LOW    = 1 << 6;
        const UNC_HIGH   = 1 << 7;
        const UCR_LOW    = 1 << 8;
        const UCR_HIGH   = 1 << 9;
        const UNR_LOW    = 1 << 10;
        const UNR_HIGH   = 1 << 11;
        const STATUS_UNC = 1 << 12;
        const STATUS_UCR = 1 << 13;
        const STATUS_UNR = 1 << 14;
        const RESERVED2  = 1 << 15;
    }
}

//TODO bitflags高低位还要学习
bitflags! {
    /// 可设阈值标志位（set分支，使用高8位）
    #[derive(Debug)]
    pub struct SetFlags: u16 {
        // 低8位对应readable字段
        const READABLE = 0x00FF;  // 位0-7
        // 高8位对应阈值标志位
        const LNC = 1 << 8;  // 位8
        const LCR = 1 << 9;  // 位9
        const LNR = 1 << 10; // 位10
        const UNC = 1 << 11; // 位11
        const UCR = 1 << 12; // 位12
        const UNR = 1 << 13; // 位13
        // 保留位（高2位）
        const RSV = 0xC000; // 位14-15 (0b11000000 00000000)
    }
}

bitflags! {
    /// 可读阈值标志位（read分支，低8位）
    #[derive(Debug)]
    pub struct ReadFlags: u16 {
        const LNC = 1 << 0;  // 位0
        const LCR = 1 << 1;  // 位1
        const LNR = 1 << 2;  // 位2
        const UNC = 1 << 3;  // 位3
        const UCR = 1 << 4;  // 位4
        const UNR = 1 << 5;  // 位5
        const RSV = 0x03 << 6; // 保留位6-7
        const SETTABLE = 0xFF << 8; // 高8位（位8-15）
    }
}

#[derive(Debug, RAWDATA)]
#[repr(C)]
pub struct ThresholdMask {
    assert: AssertFlags,     //16
    deassert: DeassertFlags, //16
    set_read: u16,           // 联合体部分16
}

impl ThresholdMask {
    //bytes: &[u8,6]
    pub fn from_le_bytes(data: &[u8]) -> Result<Self, &'static str> {
        // 添加长度检查
        if data.len() < 6 {
            return Err("输入数据长度不足");
        }

        let assert_part = u16::from_le_bytes([data[0], data[1]]);
        let deassert_part = u16::from_le_bytes([data[2], data[3]]);
        let set_read = u16::from_le_bytes([data[4], data[5]]);

        Ok(Self {
            assert: AssertFlags::from_bits_truncate(assert_part),
            deassert: DeassertFlags::from_bits_truncate(deassert_part),
            set_read,
        })
    }

    /// 获取set分支数据
    pub fn set(&self) -> SetFlags {
        SetFlags::from_bits_truncate(self.set_read)
    }

    /// 获取read分支数据  
    pub fn read(&self) -> ReadFlags {
        ReadFlags::from_bits_truncate(self.set_read)
    }
}

// 定义实现RawSize的宏，支持字节序处理
macro_rules! impl_rawsize_for_bitflags {
    ($($ty:ty),+) => {
        $(
            impl ::unpack::RawSize for $ty {
                const RAW_SIZE: usize = std::mem::size_of::<<$ty as ::bitflags::Flags>::Bits>();
                const ENDIAN: ::unpack::Endianness = ::unpack::Endianness::Native;

                fn from_bytes_with_endian(
                    bytes: &[u8],
                    endian: ::unpack::Endianness
                ) -> Result<Self, Box<dyn std::error::Error>> {
                    if bytes.len() < Self::RAW_SIZE {
                        return Err("Insufficient data for deserialization".into());
                    }

                    let bits = match endian {
                        ::unpack::Endianness::Big => {
                            <<$ty as ::bitflags::Flags>::Bits>::from_be_bytes(
                                bytes[..Self::RAW_SIZE].try_into()?
                            )
                        },
                        ::unpack::Endianness::Little => {
                            <<$ty as ::bitflags::Flags>::Bits>::from_le_bytes(
                                bytes[..Self::RAW_SIZE].try_into()?
                            )
                        },
                        ::unpack::Endianness::Native => {
                            <<$ty as ::bitflags::Flags>::Bits>::from_ne_bytes(
                                bytes[..Self::RAW_SIZE].try_into()?
                            )
                        }
                    };
                    Ok(Self::from_bits_truncate(bits))
                }
            }
        )+
    };
}

impl_rawsize_for_bitflags! {
    AssertFlags,
    DeassertFlags,
    SetFlags,
    ReadFlags,
    InitFlags
    // 可以继续添加其他flags类型
}
// use bitflags::bitflags;

// bitflags! {
//     pub struct ThresholdAssertMask: u16 {
//         const ASSERT_LNR_HIGH = 1 << 0;
//         const ASSERT_LNR_LOW = 1 << 1;
//         const ASSERT_LCR_HIGH = 1 << 2;
//         const ASSERT_LCR_LOW = 1 << 3;
//         const ASSERT_LNC_HIGH = 1 << 4;
//         const ASSERT_LNC_LOW = 1 << 5;
//         const ASSERT_UNC_HIGH = 1 << 6;
//         const ASSERT_UNC_LOW = 1 << 7;
//         const ASSERT_UCR_HIGH = 1 << 8;
//         const ASSERT_UCR_LOW = 1 << 9;
//         const ASSERT_UNR_HIGH = 1 << 10;
//         const ASSERT_UNR_LOW = 1 << 11;
//         const STATUS_LNC = 1 << 12;
//         const STATUS_LCR = 1 << 13;
//         const STATUS_LNR = 1 << 14;
//         const RESERVED = 1 << 15;
//     }
// }

// bitflags! {
//     pub struct ThresholdDeassertMask: u16 {
//         const DEASSERT_LNR_HIGH = 1 << 0;
//         const DEASSERT_LNR_LOW = 1 << 1;
//         const DEASSERT_LCR_HIGH = 1 << 2;
//         const DEASSERT_LCR_LOW = 1 << 3;
//         const DEASSERT_LNC_HIGH = 1 << 4;
//         const DEASSERT_LNC_LOW = 1 << 5;
//         const DEASSERT_UNC_HIGH = 1 << 6;
//         const DEASSERT_UNC_LOW = 1 << 7;
//         const DEASSERT_UCR_HIGH = 1 << 8;
//         const DEASSERT_UCR_LOW = 1 << 9;
//         const DEASSERT_UNR_HIGH = 1 << 10;
//         const DEASSERT_UNR_LOW = 1 << 11;
//         const STATUS_UNC = 1 << 12;
//         const STATUS_UCR = 1 << 13;
//         const STATUS_UNR = 1 << 14;
//         const RESERVED_2 = 1 << 15;
//     }
// }

// #[derive(Copy, Clone,Debug,AsBytes)]
// #[repr(C, packed)]
// pub union ThresholdAccessMask {
//     //set: ThresholdSetMask,
//     //read: ThresholdReadMask,
//     set: u16,
//     read: u16,
// }

//#[repr(C, packed)]
// #[derive(Copy, Clone)]
// pub struct ThresholdSetMask {
//     readable: u8,      // 低8位
//     lnc: bool,         // bit 8
//     lcr: bool,         // bit 9
//     lnr: bool,         // bit 10
//     unc: bool,         // bit 11
//     ucr: bool,         // bit 12
//     unr: bool,         // bit 13
//     reserved: u8,      // 剩余2位
// }

// #[repr(C, packed)]
// #[derive(Copy, Clone)]
// pub struct ThresholdReadMask {
//     lnc: bool,         // bit 0
//     lcr: bool,         // bit 1
//     lnr: bool,         // bit 2
//     unc: bool,         // bit 3
//     ucr: bool,         // bit 4
//     unr: bool,         // bit 5
//     reserved: u8,      // 2 bits
//     settable: u8,      // 高8位
// }

#[derive(Debug, AsBytes, RAWDATA)]
#[repr(C)]
pub struct SdrRecordEventonlySensor {
    pub keys: EventonlySensorKeys,
    pub entity: EntityId,
    pub sensor_type: u8, /* sensor type */
    pub event_type: u8,  /* event/reading type code */
    pub share: EventonlyShareInfo,
    pub reserved: u8,
    pub oem: u8,             /* reserved for OEM use */
    pub id_code: u8,         /* sensor ID string type/length code */
    pub id_string: [u8; 16], /* sensor ID string bytes, only if id_code != 0 */
}

#[derive(Debug, AsBytes, RAWDATA)]
#[repr(C)]
pub struct EventonlySensorKeys {
    pub owner_id: u8,
    pub lun_fru_channel: u8, // 组合字段: lun(2bits), fru_owner(2bits), channel(4bits)
    pub sensor_num: u8,      /* unique sensor number */
}
//SensorKeys中间部分(fru_owner)是保留,其它都一样，理论上是可以复用。

impl EventonlySensorKeys {
    pub fn lun(&self) -> u8 {
        self.lun_fru_channel & 0b0000_0011
    }

    pub fn fru_owner(&self) -> u8 {
        (self.lun_fru_channel >> 2) & 0b0000_0011
    }

    pub fn channel(&self) -> u8 {
        (self.lun_fru_channel >> 4) & 0b0000_1111
    }
}

#[derive(Debug, AsBytes, RAWDATA)]
#[repr(C)]
pub struct EventonlyShareInfo {
    pub count_modtype: u8,    // count:4, mod_type:2, __reserved:2
    pub modoffset_entity: u8, // mod_offset:7, entity_inst:1
}

impl EventonlyShareInfo {
    pub fn count(&self) -> u8 {
        self.count_modtype & 0b0000_1111
    }

    pub fn mod_type(&self) -> u8 {
        (self.count_modtype >> 4) & 0b0000_0011
    }

    pub fn mod_offset(&self) -> u8 {
        self.modoffset_entity & 0b0111_1111
    }

    pub fn entity_inst(&self) -> bool {
        (self.modoffset_entity & 0x01) != 0
    }
}

// ============================================================================
// 优化版本的实现（暂时不使用）
// ============================================================================
//
// TODO: 未来优化计划
// 1. 启用下面的优化版本实现
// 2. 将 SdrIterator 的返回类型改为 (SdrRecordHeader, Vec<u8>)
// 3. 实现单次IPMI调用的原子操作
// 4. 添加内置的记录类型过滤
// 5. 简化错误处理逻辑
//
// 优化收益：
// - 减少50%的IPMI调用次数
// - 提高数据一致性（原子操作）
// - 简化API使用
// - 更好的内存效率

/*
/// Get SDR record by ID (优化版本 - 单次IPMI调用获取完整数据)
fn get_sdr_record_optimized(intf: &mut dyn crate::ipmi::intf::IpmiIntf, record_id: u16, reservation_id: u16) -> IpmiResult<(SdrRecordHeader, Vec<u8>)> {
    let mut req = IpmiRq::default();
    req.msg.netfn_mut(crate::ipmi::ipmi::IPMI_NETFN_STORAGE);
    req.msg.cmd = crate::commands::sdr::sdr::GET_SDR;

    let req_data = SdrGetRq {
        reserve_id: reservation_id,
        id: record_id,
        offset: 0,
        length: 0xff, // Read entire record
    };

    let mut data_bytes = req_data.as_bytes().to_vec();
    req.msg.data = data_bytes.as_mut_ptr();
    req.msg.data_len = data_bytes.len() as u16;

    match intf.sendrecv(&req) {
        Some(rsp) => {
            if rsp.ccode != 0 {
                return Err(crate::error::IpmiError::ResponseError);
            }

            // Parse IPMI Get SDR response format:
            // Byte 0-1: Next Record ID
            // Byte 2-3: Record ID
            // Byte 4:   SDR Version
            // Byte 5:   Record Type
            // Byte 6:   Record Length
            // Byte 7+:  Record Data

            if rsp.data.len() < 7 {
                return Err(crate::error::IpmiError::ResponseError);
            }

            let header = SdrRecordHeader {
                id: u16::from_le_bytes([rsp.data[2], rsp.data[3]]),
                version: rsp.data[4],
                record_type: rsp.data[5],
                length: rsp.data[6],
            };

            let record_data = rsp.data[7..].to_vec();

            Ok((header, record_data))
        }
        None => Err(crate::error::IpmiError::ResponseError),
    }
}

/// 优化版本的SDR迭代器 (暂时不使用)
pub struct SdrIteratorOptimized<'a> {
    intf: &'a mut dyn crate::ipmi::intf::IpmiIntf,
    current_id: u16,
    reservation_id: u16,
    record_type_filter: Option<u8>,
}

impl<'a> SdrIteratorOptimized<'a> {
    pub fn new_optimized(intf: &'a mut dyn crate::ipmi::intf::IpmiIntf, record_type_filter: Option<u8>) -> IpmiResult<Self> {
        // Get reservation ID
        let reservation_id = match ipmi_sdr_get_reservation(intf, false) {
            Some(id) => id,
            None => return Err(crate::error::IpmiError::ResponseError),
        };

        Ok(SdrIteratorOptimized {
            intf,
            current_id: 0x0000, // Start from first record
            reservation_id,
            record_type_filter,
        })
    }
}

impl<'a> Iterator for SdrIteratorOptimized<'a> {
    type Item = (SdrRecordHeader, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_id == 0xFFFF {
            return None; // End of records
        }

        loop {
            match get_sdr_record_optimized(self.intf, self.current_id, self.reservation_id) {
                Ok((header, data)) => {
                    // Parse next record ID from IPMI response
                    // We need to get it from the raw response, not from the header
                    let mut req = IpmiRq::default();
                    req.msg.netfn_mut(crate::ipmi::ipmi::IPMI_NETFN_STORAGE);
                    req.msg.cmd = crate::commands::sdr::sdr::GET_SDR;

                    let req_data = SdrGetRq {
                        reserve_id: self.reservation_id,
                        id: self.current_id,
                        offset: 0,
                        length: 0xff,
                    };

                    let mut data_bytes = req_data.as_bytes().to_vec();
                    req.msg.data = data_bytes.as_mut_ptr();
                    req.msg.data_len = data_bytes.len() as u16;

                    if let Some(rsp) = self.intf.sendrecv(&req) {
                        if rsp.ccode == 0 && rsp.data.len() >= 2 {
                            self.current_id = u16::from_le_bytes([rsp.data[0], rsp.data[1]]);
                        } else {
                            self.current_id = 0xFFFF; // End iteration
                        }
                    } else {
                        self.current_id = 0xFFFF; // End iteration
                    }

                    // Apply filter if specified
                    if let Some(filter_type) = self.record_type_filter {
                        if header.record_type != filter_type {
                            continue; // Skip this record
                        }
                    }

                    return Some((header, data));
                }
                Err(_) => {
                    self.current_id = 0xFFFF; // End iteration on error
                    return None;
                }
            }
        }
    }
}

// 优化版本的使用示例（暂时不启用）:
// pub fn ipmi_sdr_list_optimized(
//     mut intf: Box<dyn crate::ipmi::intf::IpmiIntf>,
//     ctx: &OutputContext,
//     type_filter: u8,
// ) -> Result<(), Box<dyn Error>> {
//     let filter = if type_filter == 0xFF { None } else { Some(type_filter) };
//     let mut iter = SdrIteratorOptimized::new_optimized(intf.as_mut(), filter)?;
//
//     while let Some((header, record_data)) = iter.next() {
//         print_sdr_record(&header, &record_data, ctx);
//     }
//
//     Ok(())
// }
*/
