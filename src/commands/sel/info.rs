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
use crate::commands::sel::sel::{IPMI_CMD_GET_SEL_ALLOC_INFO, IPMI_CMD_GET_SEL_INFO};
use crate::error::IpmiError;
use crate::ipmi::intf::IpmiIntf;
use crate::ipmi::ipmi::{IpmiRq, IpmiRs, IPMI_NETFN_STORAGE};
use crate::ipmi::time::ipmi_timestamp_numeric;

use std::error::Error;
//用于格式化输出
#[derive(Debug, Default)]
pub struct SelBasicInfo {
    pub version: u8,
    pub entries: u16,
    pub free_space: u32,
    pub last_add_time: u32,
    pub last_del_time: u32,
    pub overflow: bool,
    pub supported_cmds: u8,
}

#[derive(Debug, Default)]
pub struct SelAllocInfo {
    pub alloc_units: u16,
    pub alloc_unit_size: u16,
    pub free_units: u16,
    pub largest_free_blk: u16,
    pub max_record_size: u8,
}

impl SelBasicInfo {
    pub fn from_response(rsp: &IpmiRs) -> Result<Self, IpmiError> {
        let version = rsp.data[0];
        let entries = u16::from_le_bytes([rsp.data[1], rsp.data[2]]);
        let free_space = u16::from_le_bytes([rsp.data[3], rsp.data[4]]) as u32;
        let last_add_time =
            u32::from_le_bytes([rsp.data[5], rsp.data[6], rsp.data[7], rsp.data[8]]);
        let last_del_time =
            u32::from_le_bytes([rsp.data[9], rsp.data[10], rsp.data[11], rsp.data[12]]);
        let overflow = rsp.data[13] & 0x80 != 0;
        let supported_cmds = rsp.data[13] & 0x0f;
        Ok(Self {
            version,
            entries,
            free_space,
            last_add_time,
            last_del_time,
            overflow,
            supported_cmds,
        })
    }

    pub fn format(&self) -> String {
        let mut output = String::new();

        // 格式化版本信息
        let version_str = format!(
            "{}.{} ({})",
            self.version & 0xf,
            (self.version >> 4) & 0xf,
            if self.version == 0x51 || self.version == 0x02 {
                "v1.5, v2 compliant"
            } else {
                "Unknown"
            }
        );

        // 计算使用百分比 - 严格匹配ipmitool逻辑
        let pctfull = if self.entries != 0 {
            let e_bytes = self.entries as u32 * 16; // 每个条目16字节
            let total = self.free_space + e_bytes; // 总大小 = 空闲空间 + 已用空间
                                                   // 使用与ipmitool相同的计算方式：直接截断，不四舍五入
            100 * e_bytes / total
        } else {
            0
        };

        // 格式化时间信息
        let format_time = |time: u32| {
            if time == 0xffffffff || time == 0 {
                "Not Available".to_string()
            } else {
                ipmi_timestamp_numeric(time)
            }
        };

        // 格式化支持的命令
        let mut cmds = Vec::new();
        if self.supported_cmds & 0x08 != 0 {
            cmds.push("'Delete'");
        }
        if self.supported_cmds & 0x04 != 0 {
            cmds.push("'Partial Add'");
        }
        if self.supported_cmds & 0x02 != 0 {
            cmds.push("'Reserve'");
        }
        if self.supported_cmds & 0x01 != 0 {
            cmds.push("'Get Alloc Info'");
        }
        let cmds_str = if cmds.is_empty() {
            "None".to_string()
        } else {
            cmds.join(" ")
        };

        // 构建输出字符串
        output.push_str("SEL Information\n");
        output.push_str(&format!("Version          : {}\n", version_str));
        output.push_str(&format!("Entries          : {}\n", self.entries));
        output.push_str(&format!(
            "Free Space       : {} bytes {}\n",
            self.free_space,
            if self.free_space == 65535 {
                "or more"
            } else {
                ""
            }
        ));
        output.push_str(&format!(
            "Percent Used     : {}\n",
            if self.free_space >= 65535 {
                "unknown".to_string()
            } else {
                format!("{}%", pctfull)
            }
        ));
        output.push_str(&format!(
            "Last Add Time    : {}\n",
            format_time(self.last_add_time)
        ));
        output.push_str(&format!(
            "Last Del Time    : {}\n",
            format_time(self.last_del_time)
        ));
        output.push_str(&format!(
            "Overflow         : {}\n",
            if self.overflow { "true" } else { "false" }
        ));
        output.push_str(&format!("Supported Cmds   : {} ", cmds_str));
        output
    }
}

impl SelAllocInfo {
    pub fn from_response(rsp: &IpmiRs) -> Self {
        Self {
            alloc_units: u16::from_le_bytes([rsp.data[0], rsp.data[1]]),
            alloc_unit_size: u16::from_le_bytes([rsp.data[2], rsp.data[3]]),
            free_units: u16::from_le_bytes([rsp.data[4], rsp.data[5]]),
            largest_free_blk: u16::from_le_bytes([rsp.data[6], rsp.data[7]]),
            max_record_size: rsp.data[8],
        }
    }

    pub fn format(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("# of Alloc Units : {}\n", self.alloc_units));
        output.push_str(&format!("Alloc Unit Size  : {}\n", self.alloc_unit_size));
        output.push_str(&format!("# Free Units     : {}\n", self.free_units));
        output.push_str(&format!("Largest Free Blk : {}\n", self.largest_free_blk));
        output.push_str(&format!("Max Record Size  : {}", self.max_record_size));
        output
    }
}

pub fn ipmi_sel_get_info(intf: &mut Box<dyn IpmiIntf>) -> Result<(), Box<dyn Error>> {
    let mut req = IpmiRq::default();

    req.msg.netfn_mut(IPMI_NETFN_STORAGE);
    req.msg.cmd = IPMI_CMD_GET_SEL_INFO;

    let rsp = match intf.sendrecv(&req) {
        Some(rsp) => {
            if rsp.ccode != 0 {
                let err = format!(
                    "Get SEL Info failed: {}",
                    IpmiError::CompletionCode(rsp.ccode)
                );
                return Err(err.into());
            } else if rsp.data_len != 14 {
                return Err(format!("Invalid data length: {}", rsp.data_len).into());
            }
            rsp
        }
        None => return Err("Command failed: no response".into()),
    };

    // if crate::VERBOSE.load(std::sync::atomic::Ordering::Relaxed) > 2 {
    //     printbuf(&rsp.data, "sel_info");
    // }

    let basic_info = match SelBasicInfo::from_response(&rsp) {
        Ok(info) => info,
        Err(e) => {
            let err = format!("Failed to parse SEL info: {}", e);
            return Err(err.into());
        }
    };
    println!("{}", basic_info.format());

    // get sel allocation info if supported
    if rsp.data[13] & 1 != 0 {
        let mut req = IpmiRq::default();
        req.msg.netfn_mut(IPMI_NETFN_STORAGE);
        req.msg.cmd = IPMI_CMD_GET_SEL_ALLOC_INFO;
        match intf.sendrecv(&req) {
            None => {
                return Err("Get SEL Allocation Info command failed".into());
            }
            Some(rsp) => {
                if rsp.ccode != 0 {
                    let err = format!(
                        "Get SEL Allocation Info command failed:{}",
                        IpmiError::CompletionCode(rsp.ccode)
                    );
                    return Err(err.into());
                }
                let alloc_info = SelAllocInfo::from_response(&rsp);
                println!("{}", alloc_info.format());
            }
        }
    }
    Ok(())
}
