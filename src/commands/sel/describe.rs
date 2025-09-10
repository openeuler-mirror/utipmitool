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
use crate::commands::mc::BMC_GET_DEVICE_ID;
use crate::commands::sel::define::*;
use crate::commands::sel::sel::*;
use crate::commands::sel::supermicro;

use crate::commands::sel::sel::get_cached_device_info;
use crate::commands::sel::sel::get_cached_oem_id;
use crate::error::IpmiError;
use crate::helper::ipmi24toh;
use crate::ipmi::intf::IpmiIntf;
use crate::ipmi::ipmi::*;
//use super::supermicro;
/* NOTE: unused paramter kept in for consistency. */

pub fn get_kontron_evt_desc(rec: &SelEventRecord) -> Option<String> {
    /*
     * Kontron OEM事件在产品用户手册中有描述，但主要针对特定传感器
     */

    // 仅处理标准记录类型
    if rec.record_type < 0xC0 {
        let std_type = unsafe { rec.sel_type.standard_type };
        for st in OEM_KONTRON_EVENT_TYPES.iter() {
            if st.code == std_type.event_type() {
                return Some(st.desc.to_string());
            }
        }
    }

    None
}

pub fn get_viking_evt_desc(intf: &mut dyn IpmiIntf, rec: &SelEventRecord) -> Option<String> {
    /*
     * Viking OEM事件描述需要通过OEM IPMI命令获取
     */
    let sel_id = rec.record_id.to_le_bytes();
    let mut req = IpmiRq::default();
    let mut msg_data = [
        0x15,      // IANA LSB
        0x24,      // IANA
        0x00,      // IANA MSB
        0x01,      // 子命令
        sel_id[0], // SEL记录ID LSB
        sel_id[1], // SEL记录ID MSB
    ];

    req.msg.netfn_mut(0x2E);
    req.msg.cmd = 0x01;
    req.msg.data = msg_data.as_mut_ptr();
    req.msg.data_len = msg_data.len() as u16;

    let rsp = match intf.sendrecv(&req) {
        Some(rsp) => rsp,
        None => {
            //eprintln!("Error issuing OEM command: {}", e);
            return None;
        }
    };

    if rsp.ccode != 0 {
        //eprintln!("OEM command returned error code: %s" IpmiError::CompletionCode(rsp.ccode));
        return None;
    }
    // 验证响应
    if rsp.data_len < 5 {
        //eprintln!("Viking OEM response too short");
        return None;
    }
    if rsp.data_len != 4 + rsp.data[3] as i32 {
        eprintln!("Viking OEM response has unexpected length");
        return None;
        //IPMI_OEM_VIKING
    }
    let oem = ipmi24toh(&[rsp.data[0], rsp.data[1], rsp.data[2]]);
    if oem != IPMI_OEM::Viking as u32 {
        eprintln!("Viking OEM response has unexpected length");
        return None;
    }

    // 提取描述字符串
    let desc_len = rsp.data[3] as usize;
    let desc_bytes = &rsp.data[4..4 + desc_len];
    Some(String::from_utf8_lossy(desc_bytes).to_string())
}

pub fn get_supermicro_evt_desc(intf: &mut dyn IpmiIntf, rec: &SelEventRecord) -> Option<String> {
    let standard_type = unsafe { rec.sel_type.standard_type };
    let (data1, data2, data3) = standard_type.data();

    // 仅处理标准事件类型0x6F
    if standard_type.event_type() != 0x6F {
        return None;
    }

    match standard_type.sensor_type {
        SENSOR_TYPE_MEMORY => {
            // 性能优化：使用缓存的设备信息
            if let Some(_device_info) = get_cached_device_info(intf) {
                let oem_id = get_cached_oem_id(intf);
                let chipset_type = ChipsetType::from(oem_id as u16);
                let dest = chipset_type.format_dimm(data2, data3);
                if dest.is_empty() {
                    None
                } else {
                    Some(dest)
                }
            } else {
                None
            }
        }
        SENSOR_TYPE_SUPERMICRO_OEM if data1 == 0x80 && data3 == 0xFF => match data2 {
            0x0 => Some("BMC unexpected reset".into()),
            0x1 => Some("BMC cold reset".into()),
            0x2 => Some("BMC warm reset".into()),
            _ => None,
        },
        _ => None,
    }
}

#[derive(Debug, PartialEq)]
enum ChipsetType {
    X8,
    Romely,
    X9,
    Brickland,
    X10QRH,
    X10OBi,
    Default,
}

impl ChipsetType {
    fn from(oem_id: u16) -> Self {
        if supermicro::SUPERMICRO_X8.contains(&oem_id)
            || supermicro::SUPERMICRO_OLDER.contains(&oem_id)
        {
            Self::X8
        } else if supermicro::SUPERMICRO_ROMELY.contains(&oem_id) {
            Self::Romely
        } else if supermicro::SUPERMICRO_X9.contains(&oem_id) {
            Self::X9
        } else if supermicro::SUPERMICRO_BRICKLAND.contains(&oem_id) {
            Self::Brickland
        } else if supermicro::SUPERMICRO_X10QRH.contains(&oem_id)
            || supermicro::SUPERMICRO_X10QBL.contains(&oem_id)
        {
            Self::X10QRH
        } else if supermicro::SUPERMICRO_X10OBI.contains(&oem_id) {
            Self::X10OBi
        } else {
            Self::Default // 默认类型
        }
    }

    fn format_dimm(&self, data2: u8, data3: u8) -> String {
        match self {
            ChipsetType::X8 => format!("@DIMM{:02X}(CPU{})", data2, (data3 & 0x03) + 1),
            ChipsetType::Romely => {
                let c1 = (data2 >> 4) + 0x40 + (data3 & 0x3) * 4;
                let c2 = (data2 & 0xf) + 0x27;
                format!(
                    "@DIMM{}{}(CPU{})",
                    c1 as char,
                    c2 as char,
                    (data3 & 0x03) + 1
                )
            }
            ChipsetType::X9 => {
                let c1 = (data2 >> 4) + 0x40 + (data3 & 0x3) * 3;
                let c2 = (data2 & 0xf) + 0x27;
                format!(
                    "@DIMM{}{}(CPU{})",
                    c1 as char,
                    c2 as char,
                    (data3 & 0x03) + 1
                )
            }
            ChipsetType::Brickland => {
                let c1 = if (data2 >> 4) > 4 {
                    b'@' - 4 + (data2 >> 4)
                } else {
                    b'@' + (data2 >> 4)
                };
                format!(
                    "@DIMM{}{}(P{}M{})",
                    c1 as char,
                    (data2 & 0xf) - 0x09,
                    (data3 & 0x0f) + 1,
                    if (data2 >> 4) > 4 { 2 } else { 1 }
                )
            }
            ChipsetType::X10QRH | ChipsetType::X10OBi => {
                let c1 = (data2 >> 4) + 0x40;
                let c2 = (data2 & 0xf) + 0x27;
                //复用一部分代码，另外一个类型
                let cpu_num = match self {
                    ChipsetType::X10OBi => (data3 & 0x07) + 1,
                    _ => (data3 & 0x03) + 1,
                };
                format!("@DIMM{}{}(CPU{})", c1 as char, c2 as char, cpu_num)
            }
            ChipsetType::Default => String::new(),
        }
    }
}

const SIZE_OF_DESC: usize = 256;

fn process_processor_event(rec: &StandardSpecSelRec) -> Option<String> {
    let (data1, data2, data3) = rec.data();
    let mut desc = String::with_capacity(SIZE_OF_DESC);

    if (data1 & DATA_BYTE2_SPECIFIED_MASK) == OEM_CODE_IN_BYTE2 {
        match data1 & MASK_LOWER_NIBBLE {
            0x00 => desc.push_str("CPU Internal Err | "),
            0x06 => desc.push_str("CPU Protocol Err | "),
            _ => (),
        }

        if let Some(bit_pos) = (0..8).find(|&i| (data2 & (1 << i)) != 0) {
            if (data1 & MASK_LOWER_NIBBLE) == 0x06 && rec.sensor_num == 0x0A {
                desc.push_str(&format!("FSB {} ", bit_pos + 1));
            } else {
                desc.push_str(&format!("CPU {} | APIC ID {} ", bit_pos + 1, data3));
            }
        }
    }

    (!desc.is_empty()).then_some(desc)
}

// 内存事件
fn process_memory_event(intf: &mut dyn IpmiIntf, rec: &StandardSpecSelRec) -> Option<String> {
    let (data1, data2, data3) = rec.data();
    let mut desc = String::with_capacity(SIZE_OF_DESC);

    // 获取BMC版本号（第5字节为版本号）

    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_APP);
    req.msg.lun_mut(0);
    req.msg.cmd = BMC_GET_DEVICE_ID;
    req.msg.data = std::ptr::null_mut();
    req.msg.data_len = 0;

    let rsp = match intf.sendrecv(&req) {
        Some(rsp) => rsp,
        None => {
            log::error!(" Error getting system info");
            return None;
        }
    };

    if rsp.ccode != 0 {
        log::error!(
            " Error getting system info: {}",
            IpmiError::CompletionCode(rsp.ccode)
        );
        return None;
    }

    let version = rsp.data[4];
    if (data1 & OEM_CODE_IN_BYTE2) != 0 || (data1 & OEM_CODE_IN_BYTE3) != 0 {
        if rec.event_type() == 0x0B {
            match data1 & MASK_LOWER_NIBBLE {
                0x00 => desc.push_str("Redundancy Regained | "),
                0x01 => desc.push_str("Redundancy Lost | "),
                _ => (),
            }
        } else {
            match data1 & MASK_LOWER_NIBBLE {
                0x00 => {
                    if rec.sensor_num == 0x1C {
                        desc.push_str("CRC Error on:");
                        let mut count = 0;

                        for i in 0..4 {
                            if (data2 & (1 << i)) != 0 {
                                if count > 0 {
                                    desc.push(',');
                                }

                                let part = match i {
                                    0 => "South Bound Memory",
                                    1 => "South Bound Config",
                                    2 => "North Bound memory",
                                    3 => "North Bound memory-corr",
                                    _ => continue,
                                };

                                desc.push_str(part);
                                count += 1;
                            }
                        }

                        desc.push_str(&format!("|Failing_Channel:{}", data3));
                    } else {
                        desc.push_str("Correctable ECC | ");
                    }
                }
                0x01 => desc.push_str("UnCorrectable ECC | "),
                _ => (),
            }
        }
    }

    // 解码内存位置信息
    if (data1 & OEM_CODE_IN_BYTE2) != 0 {
        let card_type = (data2 >> 4) & 0x0F;
        if card_type != 0x0F && card_type < 0x08 {
            let tmp_data = (b'A' + card_type) as char;

            if rec.event_type() == 0x0B {
                desc.push_str(&format!("Bad Card {}", tmp_data));
            } else {
                desc.push_str(&format!("Card {}", tmp_data));
            }
        }

        let bank_num = data2 & MASK_LOWER_NIBBLE;
        if bank_num != 0x0F && version == 0x51 {
            desc.push_str(&format!("Bank {}", bank_num + 1));
        }
    }

    if (data1 & OEM_CODE_IN_BYTE3) != 0 {
        if version == 0x51 {
            desc.push_str(&format!("DIMM {}", (b'A' + data3) as char));
        } else {
            let card_type = (data2 >> 4) & 0x0F;
            if card_type > 0x07 && card_type != 0x0F {
                let dimms_per_node = match card_type {
                    0x09 => 6,
                    0x0A => 8,
                    0x0B => 9,
                    0x0C => 12,
                    0x0D => 24,
                    0x0E => 3,
                    _ => 4,
                };

                let mut dimm_list = Vec::new();
                for i in 0..8 {
                    if (data3 & (1 << i)) != 0 {
                        let node = (i / dimms_per_node) as u8;
                        let dimm_num = (i % dimms_per_node) + 1;
                        dimm_list.push(format!("DIMM{}{}", (b'A' + node) as char, dimm_num));
                    }
                }

                if !dimm_list.is_empty() {
                    desc.push_str(&dimm_list.join(","));
                }
            } else {
                let mut dimm_list = Vec::new();
                for i in 0..8 {
                    if (data3 & (1 << i)) != 0 {
                        dimm_list.push(format!("DIMM{}", i + 1));
                    }
                }

                if !dimm_list.is_empty() {
                    desc.push_str(&dimm_list.join(","));
                }
            }
        }
    }

    (!desc.is_empty()).then_some(desc)
}

fn process_text_command_error(rec: &StandardSpecSelRec) -> Option<String> {
    let (data1, _, data3) = rec.data();

    if (data1 & MASK_LOWER_NIBBLE) == 0x00
        && (data1 & OEM_CODE_IN_BYTE2) != 0
        && (data1 & OEM_CODE_IN_BYTE3) != 0
    {
        Some(match data3 {
            0x01 => "BIOS TXT Error".to_string(),
            0x02 => "Processor/FIT TXT".to_string(),
            0x03 => "BIOS ACM TXT Error".to_string(),
            0x04 => "SINIT ACM TXT Error".to_string(),
            0xFF => "Unrecognized TT Error12".to_string(),
            _ => return None,
        })
    } else {
        None
    }
}

// 看门狗事件
fn process_watchdog_event(rec: &StandardSpecSelRec) -> Option<String> {
    let (data1, data2, _) = rec.data();

    if data1 == 0x25 && data2 == 0x04 {
        Some("Hard Reset|Interrupt type None,SMS/OS Timer used at expiration".to_string())
    } else {
        None
    }
}

// 版本变更事件
fn process_version_change_event(rec: &StandardSpecSelRec) -> Option<String> {
    let (data1, data2, data3) = rec.data();

    if (data1 & MASK_LOWER_NIBBLE) == 0x02
        && (data1 & OEM_CODE_IN_BYTE2) != 0
        && (data1 & OEM_CODE_IN_BYTE3) != 0
    {
        if data2 == 0x02 {
            Some(match data3 {
                0x00 => "between BMC/iDRAC Firmware and other hardware".to_string(),
                0x01 => "between BMC/iDRAC Firmware and CPU".to_string(),
                _ => return None,
            })
        } else {
            None
        }
    } else {
        None
    }
}

// OEM安全事件
fn process_oem_sec_event(rec: &StandardSpecSelRec) -> Option<String> {
    let (data1, data2, data3) = rec.data();

    if rec.sensor_num == 0x25 {
        let mut desc = String::new();

        match data1 & MASK_LOWER_NIBBLE {
            0x01 => {
                desc.push_str("Failed to program Virtual Mac Address");
                if (data1 & OEM_CODE_IN_BYTE2) != 0 && (data1 & OEM_CODE_IN_BYTE3) != 0 {
                    desc.push_str(&format!(
                        " at bus:{:02x} device:{:02x} function:{}",
                        data3 & 0x7F,
                        (data2 >> 3) & 0x1F,
                        data2 & 0x07
                    ));
                }
            }
            0x02 => {
                desc.push_str("Device option ROM failed to support link tuning or flex address")
            }
            0x03 => desc.push_str("Failed to get link tuning or flex address data from BMC/iDRAC"),
            _ => return None,
        }

        Some(desc)
    } else {
        None
    }
}

// 关键中断事件
fn process_critical_interrupt_event(rec: &StandardSpecSelRec) -> Option<String> {
    let (data1, data2, data3) = rec.data();
    let mut desc = String::new();

    if rec.sensor_num == 0x29 {
        if (data1 & MASK_LOWER_NIBBLE) == 0x02
            && (data1 & OEM_CODE_IN_BYTE2) != 0
            && (data1 & OEM_CODE_IN_BYTE3) != 0
        {
            desc.push_str(&format!(
                "Partner-(LinkId:{},AgentId:{})|",
                (data2 & 0xC0) >> 6,
                (data2 & 0x30) >> 4
            ));

            desc.push_str(&format!(
                "ReportingAgent(LinkId:{},AgentId:{})|",
                (data2 & 0x0C) >> 2,
                data2 & 0x03
            ));

            if (data3 & 0xFC) == 0x00 {
                desc.push_str("LinkWidthDegraded|");
            }

            if (data3 & 0x02) != 0 {
                desc.push_str("PA_Type:IOH|");
            } else {
                desc.push_str("PA-Type:CPU|");
            }

            if (data3 & 0x01) != 0 {
                desc.push_str("RA-Type:IOH");
            } else {
                desc.push_str("RA-Type:CPU");
            }
        }
    } else if (data1 & MASK_LOWER_NIBBLE) == 0x02 {
        desc.push_str("IO channel Check NMI");
    } else {
        match data1 & MASK_LOWER_NIBBLE {
            0x00 => desc.push_str("PCIe Error |"),
            0x01 => desc.push_str("I/O Error |"),
            0x04 => desc.push_str("PCI PERR |"),
            0x05 => desc.push_str("PCI SERR |"),
            _ => desc.push(' '),
        }

        if (data3 & 0x80) != 0 {
            desc.push_str(&format!("Slot {}", data3 & 0x7F));
        } else {
            desc.push_str(&format!(
                "PCI bus:{:02x} device:{:02x} function:{}",
                data3 & 0x7F,
                (data2 >> 3) & 0x1F,
                data2 & 0x07
            ));
        }
    }

    (!desc.is_empty()).then_some(desc)
}

// 固件进度事件
fn process_firmware_progress_event(rec: &StandardSpecSelRec) -> Option<String> {
    let (data1, data2, _) = rec.data();

    if (data1 & MASK_LOWER_NIBBLE) == 0x0F && (data1 & OEM_CODE_IN_BYTE2) != 0 {
        Some(match data2 {
            0x80 => "No memory is detected.".to_string(),
            0x81 => "Memory is detected but is not configurable.".to_string(),
            0x82 => "Memory is configured but not usable.".to_string(),
            0x83 => "System BIOS shadow failed.".to_string(),
            0x84 => "CMOS failed.".to_string(),
            0x85 => "DMA controller failed.".to_string(),
            0x86 => "Interrupt controller failed.".to_string(),
            0x87 => "Timer refresh failed.".to_string(),
            0x88 => "Programmable interval timer error.".to_string(),
            0x89 => "Parity error.".to_string(),
            0x8A => "SIO failed.".to_string(),
            0x8B => "Keyboard controller failed.".to_string(),
            0x8C => "System management interrupt initialization failed.".to_string(),
            0x8D => "TXT-SX Error.".to_string(),
            0xC0 => "Shutdown test failed.".to_string(),
            0xC1 => "BIOS POST memory test failed.".to_string(),
            0xC2 => "RAC configuration failed.".to_string(),
            0xC3 => "CPU configuration failed.".to_string(),
            0xC4 => "Incorrect memory configuration.".to_string(),
            0xFE => "General failure after video.".to_string(),
            _ => return None,
        })
    } else {
        None
    }
}

// 优化后的主函数部分
pub fn get_dell_evt_desc(intf: &mut dyn IpmiIntf, rec: &SelEventRecord) -> Option<String> {
    //let data = unsafe {rec.sel_type.standard_type.event_data};
    //let sensor_type = unsafe {rec.sel_type.standard_type.sensor_type};
    let standard_type = unsafe { &rec.sel_type.standard_type };
    // 仅处理标准事件类型0x6F
    if standard_type.event_type() != 0x6F {
        return None;
    }

    match standard_type.sensor_type {
        SENSOR_TYPE_PROCESSOR => process_processor_event(standard_type),
        SENSOR_TYPE_MEMORY | SENSOR_TYPE_EVT_LOG => process_memory_event(intf, standard_type),
        SENSOR_TYPE_TXT_CMD_ERROR => process_text_command_error(standard_type),
        SENSOR_TYPE_WTDOG => process_watchdog_event(standard_type),
        SENSOR_TYPE_VER_CHANGE => process_version_change_event(standard_type),
        SENSOR_TYPE_OEM_SEC_EVENT => process_oem_sec_event(standard_type),
        SENSOR_TYPE_CRIT_INTR | SENSOR_TYPE_OEM_NFATAL_ERROR | SENSOR_TYPE_OEM_FATAL_ERROR => {
            process_critical_interrupt_event(standard_type)
        }
        SENSOR_TYPE_FRM_PROG => process_firmware_progress_event(standard_type),
        _ => None,
    }
}

const OEM_QCT_NETFN: u8 = 0x36;
const OEM_QCT_GET_INFO: u8 = 0x65;

#[derive(Debug, PartialEq)]
enum QctPlatform {
    Unknown,
    Grantley,
    Purley,
}

// fn oem_qct_get_platform_id(intf: &mut dyn IpmiIntf) -> QctPlatform {
//     // You need to implement this function based on your platform detection logic.
//     // For now, just return Unknown.
//     QctPlatform::Unknown
// }

fn cpu_num(x: u8) -> u8 {
    (x >> 6) & 0x03
}

fn channel_offset(x: u8) -> u8 {
    (x >> 3) & 0x07
}

fn channel_num(x: u8) -> char {
    (0x41 + channel_offset(x)) as char
}

fn dimm_num(x: u8) -> u8 {
    x & 0x07
}

pub fn oem_qct_get_evt_desc(intf: &mut dyn IpmiIntf, rec: &SelEventRecord) -> Option<String> {
    let standard_type = unsafe { rec.sel_type.standard_type };

    let data = standard_type.event_data[2];
    let event_type = standard_type.event_type();
    if event_type != 0x6F {
        return None;
    }
    let sensor_type = standard_type.sensor_type;
    if sensor_type == SENSOR_TYPE_MEMORY {
        // Get BMC version (for compatibility, not used here)
        let mut req = IpmiRq::default();
        req.msg.netfn_mut(IPMI_NETFN_APP);
        req.msg.lun_mut(0);
        req.msg.cmd = BMC_GET_DEVICE_ID;
        req.msg.data = std::ptr::null_mut();
        req.msg.data_len = 0;

        let rsp = match intf.sendrecv(&req) {
            Some(rsp) => rsp,
            None => {
                log::error!("Error getting system info");
                return None;
            }
        };
        if rsp.ccode != 0 {
            log::error!(
                "Error getting system info: {}",
                IpmiError::CompletionCode(rsp.ccode)
            );
            return None;
        }
        let platform_id = oem_qct_get_platform_id(intf);
        if platform_id == QctPlatform::Purley {
            let desc = format!(
                "CPU{}_{}{}",
                cpu_num(data),
                channel_num(data),
                dimm_num(data)
            );
            return Some(desc);
        }
    }
    None
}

const GET_PLATFORM_ID_DATA_SIZE: usize = 4;

// Magic code to check if it's valid command
const QCT_MAGIC_1: u8 = 0x4C;
const QCT_MAGIC_2: u8 = 0x1C;
const QCT_MAGIC_3: u8 = 0x00;
const QCT_MAGIC_4: u8 = 0x02;

fn oem_qct_get_platform_id(intf: &mut dyn IpmiIntf) -> QctPlatform {
    // Execute a Get platform ID command to determine the board
    let mut msg_data = [0u8; GET_PLATFORM_ID_DATA_SIZE];
    msg_data[0] = QCT_MAGIC_1;
    msg_data[1] = QCT_MAGIC_2;
    msg_data[2] = QCT_MAGIC_3;
    msg_data[3] = QCT_MAGIC_4;

    let mut req = IpmiRq::default();
    req.msg.netfn_mut(OEM_QCT_NETFN);
    req.msg.cmd = OEM_QCT_GET_INFO;
    req.msg.data = msg_data.as_mut_ptr();
    req.msg.data_len = msg_data.len() as u16;

    let rsp = match intf.sendrecv(&req) {
        Some(rsp) => rsp,
        None => {
            log::error!("Get Platform ID command failed");
            return QctPlatform::Unknown;
        }
    };
    if rsp.ccode != 0 {
        log::error!(
            "Get Platform ID command failed: {}",
            IpmiError::CompletionCode(rsp.ccode)
        );
        return QctPlatform::Unknown;
    }
    let platform_id = rsp.data.first().copied().unwrap_or(0);
    eprintln!("Platform ID: {:02x}", platform_id);
    match platform_id {
        0x01 => QctPlatform::Grantley,
        0x02 => QctPlatform::Purley,
        _ => QctPlatform::Unknown,
    }
}
