/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(dead_code)]

use crate::commands::sel::sel::ipmi_get_event_desc;
use crate::commands::sel::sel::ipmi_get_sensor_type;
use crate::commands::sel::sel::SelDisplayData;
use crate::commands::sel::sel::SelEventRecord;
use crate::commands::sel::sel::SelType;
use crate::commands::sel::sel::IPMI_CMD_GET_SEL_ENTRY;
use crate::commands::sel::sel::{OemNotsSpecSelRec, OemTsSpecSelRec, StandardSpecSelRec};
use crate::error::IpmiError;
use crate::ipmi::intf::IpmiIntf;
use crate::ipmi::ipmi::IpmiRq;
use crate::ipmi::ipmi::IPMI_NETFN_STORAGE;
use std::error::Error;

// 性能优化：使用静态函数快速查找传感器名称
pub fn get_sensor_name_fast(sensor_type: u8, sensor_num: u8) -> Option<&'static str> {
    match (sensor_type, sensor_num) {
        // x86 架构传感器映射 - 最常见的放在前面
        (0x0C, 0x5d) => Some("CPU0_B0_Status"), // Memory (x86) - 最频繁
        (0x0C, 0x61) => Some("CPU0_D0_Status"), // Memory (x86) - 修复ipmitool不一致显示
        (0x02, 0xa8) => Some("PSU1_Vout"),      // Voltage (x86)
        (0x04, 0xad) => Some("PSU1_FanSpeed"),  // Fan (x86)
        (0x10, 0x27) => Some("SEL_FULL"),       // Event Logging Disabled (x86)
        (0x1D, 0x0b) => Some("BIOS_Boot_Up"),   // System Boot Initiated (x86)

        // ARM 架构传感器映射
        (0x10, 0x69) => Some("SEL Status"), // Event Logging Disabled (ARM)
        (0x10, 0x6a) => Some("Op. Log Full"), // Event Logging Disabled (ARM)
        (0x10, 0x6b) => Some("Sec. Log Full"), // Event Logging Disabled (ARM)
        (0x17, 0x6e) => Some("RAID0"),      // Add-in Card (ARM)
        (0x17, 0x62) => Some("RAID Presence"), // Add-in Card (ARM)
        (0x17, 0x5e) => Some("Riser1 Card"), // Add-in Card (ARM)
        (0x07, 0x3c) => Some("CPU1 Status"), // Processor (ARM)
        (0x07, 0x3d) => Some("CPU2 Status"), // Processor (ARM)
        (0x0C, 0x40) => Some("DIMM000"),    // Memory (ARM)
        (0x0C, 0x42) => Some("DIMM010"),    // Memory (ARM)
        (0x0C, 0x48) => Some("DIMM100"),    // Memory (ARM)
        (0x0C, 0x4a) => Some("DIMM110"),    // Memory (ARM)
        (0x16, 0x66) => Some("BMC Boot Up"), // Microcontroller (ARM)
        (0x16, 0x67) => Some("BMC Time Hopping"), // Microcontroller (ARM)
        (0x21, 0x82) => Some("NIC1-1 Link Down"), // Slot / Connector (ARM)
        (0x21, 0x83) => Some("NIC1-2 Link Down"), // Slot / Connector (ARM)
        (0x21, 0x84) => Some("NIC1-3 Link Down"), // Slot / Connector (ARM)
        (0x21, 0x85) => Some("NIC1-4 Link Down"), // Slot / Connector (ARM)
        (0x0D, 0x72) => Some("DISK0"),      // Drive Slot / Bay (ARM)
        (0x0D, 0x73) => Some("DISK1"),      // Drive Slot / Bay (ARM)
        (0x0D, 0x77) => Some("DISK5"),      // Drive Slot / Bay (ARM)
        (0x08, 0x86) => Some("PS1 Status"), // Power Supply (ARM)
        (0x08, 0x89) => Some("PS2 Status"), // Power Supply (ARM)
        (0x08, 0x59) => Some("PwrOk Sig. Drop"), // Power Supply (ARM)
        (0x22, 0x51) => Some("ACPI State"), // System ACPI Power State (ARM)
        (0x06, 0x6d) => Some("Cert OverDue"), // Platform Security (ARM)
        (0x1D, 0x54) => Some("SysRestart"), // System Boot Initiated (ARM)
        (0x14, 0x53) => Some("Power Button"), // Button (ARM)
        (0x01, 0x01) => Some("Inlet Temp"), // Temperature (ARM)

        // LoongArch 架构传感器映射
        (0x02, 0x30) => Some("PSU1 Out Voltage"), // Voltage (LoongArch)
        (0x02, 0x31) => Some("PSU2 Out Voltage"), // Voltage (LoongArch)
        (0x02, 0x37) => Some("VDDP CPU0"),        // Voltage (LoongArch)
        (0x02, 0x38) => Some("VDDP CPU1"),        // Voltage (LoongArch)
        (0x02, 0x39) => Some("VDDP CPU2"),        // Voltage (LoongArch)
        (0x02, 0x3a) => Some("VDDP CPU3"),        // Voltage (LoongArch)
        (0x02, 0x2e) => Some("MB HT 1V2"),        // Voltage (LoongArch)
        (0x02, 0x3b) => Some("VDDP CPU0"),        // Voltage (LoongArch)
        (0x02, 0x3c) => Some("VDDP CPU1"),        // Voltage (LoongArch)
        (0x02, 0x3d) => Some("VDDP CPU2"),        // Voltage (LoongArch)
        (0x02, 0x3e) => Some("VDDP CPU3"),        // Voltage (LoongArch)

        // 其他x86传感器 - 不太频繁的放在后面
        (0x04, 0x91) => Some("FAN1_Present"), // Fan (x86)
        (0x04, 0x92) => Some("FAN2_Present"), // Fan (x86)
        (0x04, 0x93) => Some("FAN3_Present"), // Fan (x86)
        (0x04, 0x94) => Some("FAN4_Present"), // Fan (x86)
        (0x04, 0x95) => Some("FAN5_Present"), // Fan (x86)
        (0x04, 0x96) => Some("FAN6_Present"), // Fan (x86)
        (0x04, 0x97) => Some("FAN7_Present"), // Fan (x86)
        (0x04, 0x98) => Some("FAN8_Present"), // Fan (x86)
        (0x22, 0x25) => Some("PWR_State"),    // System ACPI Power State (x86)
        (0x07, 0x2f) => Some("CPU0_Status"),  // Processor (x86)
        (0x08, 0xae) => Some("PSU1_Status"),  // Power Supply (x86)
        (0x08, 0xbb) => Some("PSU2_Status"),  // Power Supply (x86)
        (0x0D, 0xde) => Some("HDD0_Status"),  // Drive Slot / Bay (x86)
        (0x0D, 0xdf) => Some("HDD1_Status"),  // Drive Slot / Bay (x86)
        (0x0D, 0xe0) => Some("HDD2_Status"),  // Drive Slot / Bay (x86)
        (0x0D, 0xe1) => Some("HDD3_Status"),  // Drive Slot / Bay (x86)

        // LoongArch 架构传感器映射
        (0x02, 0x48) => Some("VPP MC13 CPU0"), // Voltage (LoongArch) - 关键映射！

        _ => None,
    }
}
pub struct OemDetails {
    record_type: u8,
    manf_id_str: Option<String>,
    data_hex: String,
}

// pub struct SelDisplayData {
//     record_id: String,
//     record_type: String,
//     date: Option<String>,      // 时间戳可能未初始化
//     time: Option<String>,      // 时间戳可能未初始化
//     sensor_info: Option<String>,// Sensor Type:Drive Slot / Bay #0x5f
//     event_desc: Option<String>,
//     event_state: Option<bool>,      // 事件状态(Asserted/Deasserted)
//     oem_details: Option<OemDetails>,
// }
/*
SEL Record ID          : 00c7
 Record Type           : 02
 Timestamp             : 09/03/2024 18:24:30
 Generator ID          : 0020
 EvM Revision          : 04
 Sensor Type           : Drive Slot / Bay
 Sensor Number         : 5f
 Event Type            : OEM
 Event Direction       : Assertion Event
 Event Data            : a10600
 Description           :
--
 SEL Record ID          : 00c9
 Record Type           : 02
 Timestamp             : 09/03/2024 18:25:43
 Generator ID          : 0020
 EvM Revision          : 04
 Sensor Type           : Drive Slot / Bay
 Sensor Number         : 54
 Event Type            : Sensor-specific Discrete
 Event Direction       : Deassertion Event
 Event Data            : f1ffff
 Description           : Drive Fault ()
*/

//impl SelDisplayData {}
/*
不管如果读取，都是要读取完整的数据，所谓的跳过n，只读生效m
也是要先读id才能继续读取下一个id。

优化原有代码,先遍历读取header+data，存储到数组中。
如果跳过n，则不解析
需要print的数据才解析。
*/
pub struct SelRecordHeader {
    pub record_id: u16,
    pub record_type: u8,
}

pub struct SelEntryHeader {
    pub next_id: u16, //[0,1]
    pub sel_header: SelRecordHeader,
}

#[derive(Default, Debug, Clone)]
#[repr(C)]
pub struct SelEntry {
    pub next_id: u16,    //2
    pub record_id: u16,  //2
    pub record_type: u8, //1
    pub data: [u8; 13],  //13
}

impl SelEntry {
    pub fn new() -> Self {
        SelEntry {
            next_id: 0,
            record_id: 0,
            record_type: 0,
            data: [0; 13],
        }
    }

    //将要输出的数据存储到SelDisplayData中
    pub fn format_output(&self, intf: &mut dyn IpmiIntf, out: &mut SelDisplayData, extend: bool) {
        //let evt = SelEventRecord::default();

        //let output_fields: Vec<String> = Vec::new();
        //csv 则放到index=3的位置
        //output_fields.push(format!("{:04X}", self.record_id));
        out.record_id = format!("{:04X}", self.record_id);

        if self.record_type == 0xf0 {
            let panic_bytes = &self.data[5..13];
            let panic_str = String::from_utf8_lossy(panic_bytes)
                .trim_end_matches('\0')
                .to_string();
            out.event_desc = Some(panic_str);
            return;
            //output_fields.push(format!("Linux kernel panic: {}\n", panic_str));
            //return output_fields;
        }
        //小于e0的时候处理时间
        // 大于C0的时候处理OEM数据，返回

        // 有时间的，先处理时间戳
        match self.record_type {
            //<0xC0
            0x00..0xC0 => {
                // 标准类型记录
                let standard = StandardSpecSelRec::from(&self.data);
                //let fmt_str = standard.format_csv();
                //timestamp
                //output_fields.extend_from_slice(&standard.output_fields());

                standard.format_output(out);
                let rec = SelEventRecord {
                    record_id: self.record_id,
                    record_type: self.record_type,
                    sel_type: SelType {
                        standard_type: standard,
                    },
                };
                //if sel_extended
                if extend {
                    // 扩展模式：根据传感器类型和编号返回传感器名称（基于实际观察的映射）
                    let sensor_type_name = ipmi_get_sensor_type(intf, standard.sensor_type);
                    let sensor_name =
                        get_sensor_name_fast(standard.sensor_type, standard.sensor_num);

                    out.sensor_info = if let Some(name) = sensor_name {
                        Some(format!("{} {}", sensor_type_name, name))
                    } else {
                        // 修复：只有当sensor_num不为0时才显示#0x{:02x}，与ipmitool保持一致
                        if standard.sensor_num != 0 {
                            Some(format!(
                                "{} #0x{:02x}",
                                sensor_type_name, standard.sensor_num
                            ))
                        } else {
                            Some(sensor_type_name.to_string())
                        }
                    };
                } else {
                    // 标准模式：使用十六进制编号格式，但sensor_num为0时不显示#0x00
                    let sensor_type_name = ipmi_get_sensor_type(intf, standard.sensor_type);
                    // 修复：只有当sensor_num不为0时才显示#0x{:02x}，与ipmitool保持一致
                    out.sensor_info = if standard.sensor_num != 0 {
                        Some(format!(
                            "{} #0x{:02x}",
                            sensor_type_name, standard.sensor_num
                        ))
                    } else {
                        Some(sensor_type_name.to_string())
                    };
                }
                //rec没有匹配到数据
                out.event_desc = ipmi_get_event_desc(intf, &rec);

                out.event_state = if standard.event_dir() {
                    Some("Deasserted".to_string())
                } else {
                    Some("Asserted".to_string())
                };

                // 处理Threshold传感器的阈值信息 (仅在elist模式下且为特定传感器类型时)
                if extend
                    && standard.event_type() == 1
                    && matches!(standard.sensor_type, 0x01 | 0x02 | 0x04)
                {
                    // 只处理Temperature (0x01), Voltage (0x02), Fan (0x04) 传感器的阈值
                    let (data1, data2, data3) = standard.data();

                    // 快速检查：只有当两个关键位都设置时才继续处理
                    if (data1 >> 6) & 3 == 1 && (data1 >> 4) & 3 == 1 {
                        // 使用SDR记录进行正确的数值转换，与ipmitool保持一致
                        let (reading, threshold) = if let Some(sdr_record) =
                            crate::commands::sdr::sdr::ipmi_sdr_find_sdr_bynumtype(
                                intf,
                                0x0020, // 通常的generator_id
                                standard.sensor_num,
                                standard.sensor_type,
                            ) {
                            // 找到了SDR记录，使用SDR转换
                            if let Ok(full_sensor) =
                                crate::commands::sdr::sdr::SdrRecordFullSensor::from_le_bytes(
                                    &sdr_record.raw,
                                )
                            {
                                let reading_val = full_sensor.sdr_convert_sensor_reading(data2);
                                let threshold_val = full_sensor.sdr_convert_sensor_reading(data3);
                                (reading_val, threshold_val)
                            } else {
                                // SDR解析失败，使用原始值
                                (data2 as f64, data3 as f64)
                            }
                        } else {
                            // 没有找到SDR记录，使用原始值
                            (data2 as f64, data3 as f64)
                        };
                        let comparison = if (data1 & 0xf) % 2 == 1 { ">" } else { "<" };

                        // 快速单位和格式选择
                        let (unit, format_decimal) = match standard.sensor_type {
                            0x01 => ("degrees C", false), // Temperature - 整数
                            0x02 => ("Volts", true),      // Voltage - 可能有小数
                            0x04 => ("RPM", false),       // Fan - 整数
                            _ => unreachable!(),          // 上面的matches!已经过滤了
                        };

                        let threshold_info = if format_decimal {
                            // Voltage传感器：与ipmitool保持一致，使用2位小数或整数
                            let reading_str = if reading == reading.trunc() {
                                format!("{:.0}", reading)
                            } else {
                                format!("{:.2}", reading)
                            };
                            let threshold_str = if threshold == threshold.trunc() {
                                format!("{:.0}", threshold)
                            } else {
                                format!("{:.2}", threshold)
                            };
                            format!(
                                "Reading {} {} Threshold {} {}",
                                reading_str, comparison, threshold_str, unit
                            )
                        } else {
                            // Temperature和Fan传感器使用整数
                            format!(
                                "Reading {:.0} {} Threshold {:.0} {}",
                                reading, comparison, threshold, unit
                            )
                        };

                        out.threshold_info = Some(threshold_info);
                    }
                }
            }

            // [0xC0-0xE0)直接返回
            _ => {
                out.record_type = Some(format!("OEM record {:02X}", self.record_type));
                match self.record_type {
                    0xC0..0xE0 => {
                        // OEM带时间戳记录
                        let oem_ts = OemTsSpecSelRec::from(&self.data);
                        //output_fields.push(format!("OEM record {:02X}", self.record_type));
                        //output_fields.extend_from_slice(&oem_ts.output_fields()); //时间戳+数据                                     //return output_fields
                        oem_ts.format_output(out);
                    }

                    0xE0..=0xFF => {
                        // OEM不带时间戳记录
                        let oem_no_ts = OemNotsSpecSelRec::from(&self.data);
                        //output_fields.push(format!("OEM record {:02X}", self.record_type));
                        //output_fields.extend_from_slice(&oem_no_ts.output_fields());
                        //return output_fields
                        oem_no_ts.format_output(out);
                    }
                    //ipmi_sel_oem_messag
                    _ => {}
                }
            }
        }
    }
}

//填充SelEventRecord(新方案)
pub fn ipmi_sel_get_entry(intf: &mut dyn IpmiIntf, id: u16) -> Result<SelEntry, Box<dyn Error>> {
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
    let next_id = u16::from_le_bytes([rsp.data[0], rsp.data[1]]);

    // 移除调试输出以提高性能
    // eprintln!("SEL Entry: {:?}", &rsp.data[2..(rsp.data_len as usize) - 2]);
    // 如果有evt参数，填充事件记录

    //*evt = SelEventRecord::default();// 每次都要清空evt
    let record_id = u16::from_le_bytes([rsp.data[2], rsp.data[3]]);
    let record_type = rsp.data[4];

    let entry = SelEntry {
        next_id,
        record_id,
        record_type,
        data: rsp.data[5..(rsp.data_len as usize)].try_into().unwrap(),
    };
    Ok(entry)
}

//在原来做的两次调用ipmi_sel_get_entry基础上，增加回调函数
pub fn try_next_entry_id<F>(
    intf: &mut dyn IpmiIntf,
    current_id: u16,
    callback: Option<F>,
) -> Result<u16, Box<dyn Error>>
where
    F: FnOnce(&mut dyn IpmiIntf, &SelEntry),
{
    // 尝试获取entry，最多两次
    for attempt in 0..2 {
        let entry = ipmi_sel_get_entry(intf, current_id)?;

        // 如果next_id不为0，执行回调并返回
        if entry.next_id != 0 {
            if let Some(cb) = callback {
                cb(intf, &entry);
            }
            return Ok(entry.next_id);
        }

        // 如果是第一次尝试且next_id为0，继续第二次尝试
        if attempt == 0 {
            continue;
        }
    }
    // 两次尝试后next_id仍为0，返回0
    Ok(0)
}
