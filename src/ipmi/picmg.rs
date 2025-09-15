/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::debug2;
use crate::debug3;
use crate::error::IpmiError;
use crate::ipmi::intf::IpmiIntf;
use crate::ipmi::ipmi::*;

// PICMG 版本常量
pub const PICMG_CPCI_MAJOR_VERSION: u8 = 1;
pub const PICMG_ATCA_MAJOR_VERSION: u8 = 2;
pub const PICMG_AMC_MAJOR_VERSION: u8 = 4;
pub const PICMG_UTCA_MAJOR_VERSION: u8 = 5;

// PICMG 命令常量
pub const PICMG_GET_PICMG_PROPERTIES_CMD: u8 = 0x00;
pub const PICMG_GET_ADDRESS_INFO_CMD: u8 = 0x01;
pub const PICMG_GET_SHELF_ADDRESS_INFO_CMD: u8 = 0x02;
pub const PICMG_SET_SHELF_ADDRESS_INFO_CMD: u8 = 0x03;
pub const PICMG_FRU_CONTROL_CMD: u8 = 0x04;
pub const PICMG_GET_FRU_LED_PROPERTIES_CMD: u8 = 0x05;
pub const PICMG_GET_LED_COLOR_CAPABILITIES_CMD: u8 = 0x06;
pub const PICMG_SET_FRU_LED_STATE_CMD: u8 = 0x07;
pub const PICMG_GET_FRU_LED_STATE_CMD: u8 = 0x08;
pub const PICMG_SET_IPMB_CMD: u8 = 0x09;
pub const PICMG_SET_FRU_POLICY_CMD: u8 = 0x0A;
pub const PICMG_GET_FRU_POLICY_CMD: u8 = 0x0B;
pub const PICMG_FRU_ACTIVATION_CMD: u8 = 0x0C;
pub const PICMG_GET_DEVICE_LOCATOR_RECORD_CMD: u8 = 0x0D;
pub const PICMG_SET_PORT_STATE_CMD: u8 = 0x0E;
pub const PICMG_GET_PORT_STATE_CMD: u8 = 0x0F;
pub const PICMG_COMPUTE_POWER_PROPERTIES_CMD: u8 = 0x10;
pub const PICMG_SET_POWER_LEVEL_CMD: u8 = 0x11;
pub const PICMG_GET_POWER_LEVEL_CMD: u8 = 0x12;
pub const PICMG_RENEGOTIATE_POWER_CMD: u8 = 0x13;
pub const PICMG_GET_FAN_SPEED_PROPERTIES_CMD: u8 = 0x14;
pub const PICMG_SET_FAN_LEVEL_CMD: u8 = 0x15;
pub const PICMG_GET_FAN_LEVEL_CMD: u8 = 0x16;
pub const PICMG_BUSED_RESOURCE_CMD: u8 = 0x17;

// AMC 命令常量
pub const PICMG_AMC_SET_PORT_STATE_CMD: u8 = 0x19;
pub const PICMG_AMC_GET_PORT_STATE_CMD: u8 = 0x1A;
// AMC.0 R2.0 命令
pub const PICMG_AMC_SET_CLK_STATE_CMD: u8 = 0x2C;
pub const PICMG_AMC_GET_CLK_STATE_CMD: u8 = 0x2D;

// 站点类型常量
pub const PICMG_ATCA_BOARD: u8 = 0x00;
pub const PICMG_POWER_ENTRY: u8 = 0x01;
pub const PICMG_SHELF_FRU: u8 = 0x02;
pub const PICMG_DEDICATED_SHMC: u8 = 0x03;
pub const PICMG_FAN_TRAY: u8 = 0x04;
pub const PICMG_FAN_FILTER_TRAY: u8 = 0x05;
pub const PICMG_ALARM: u8 = 0x06;
pub const PICMG_AMC: u8 = 0x07;
pub const PICMG_PMC: u8 = 0x08;
pub const PICMG_RTM: u8 = 0x09;

pub fn ipmi_picmg_ipmb_address(intf: &mut dyn IpmiIntf) -> u8 {
    let mut data = vec![0x00]; // 请求数据
    let mut req = IpmiRq::default();

    req.msg.netfn_mut(IPMI_NETFN_PICMG);
    req.msg.cmd = PICMG_GET_ADDRESS_INFO_CMD;
    req.msg.data = data.as_mut_ptr();
    req.msg.data_len = data.len() as u16;

    let rsp = match intf.sendrecv(&req) {
        Some(r) => r,
        None => {
            debug3!("Get Address Info failed: No Response");
            return 0;
        }
    };

    if rsp.ccode == 0 && rsp.data.len() >= 3 {
        return rsp.data[2];
    }

    if rsp.ccode != 0 {
        debug3!(
            "Get Address Info failed: {}",
            IpmiError::CompletionCode(rsp.ccode)
        );
    } else {
        debug3!("Invalid response length {}", rsp.data.len());
    }

    0
}
pub fn picmg_discover(intf: &mut dyn IpmiIntf) -> u8 {
    debug2!(
        "Running Get PICMG Properties my_addr 0x{:02x}, transit 0, target 0",
        intf.context().my_addr()
    );

    // 构造 PICMG 属性请求
    //let data:u8=0;
    let mut data = vec![0x00];
    let mut req = IpmiRq::default();

    req.msg.netfn_mut(IPMI_NETFN_PICMG);
    req.msg.cmd = PICMG_GET_PICMG_PROPERTIES_CMD;
    req.msg.data = data.as_mut_ptr();
    req.msg.data_len = data.len() as u16;

    // 发送请求并获取响应
    let rsp = match intf.sendrecv(&req) {
        Some(r) => r,
        None => {
            debug2!("No response from Get PICMG Properties");
            return 0;
        }
    };

    // 检查响应状态码
    if rsp.ccode != 0 {
        debug2!(
            "Error response 0x{:02x} from Get PICMG Properities",
            rsp.ccode
        );
        return 0;
    }

    // 验证响应数据长度
    if rsp.data.len() < 4 {
        log::info!(
            "Invalid Get PICMG Properties response length {}",
            rsp.data.len()
        );
        return 0;
    }

    // 检查组扩展标识
    if rsp.data[0] != 0 {
        log::info!(
            "Invalid Get PICMG Properties group extension {:#x}",
            rsp.data[0]
        );
        return 0;
    }

    // 提取主版本号
    let major_version = rsp.data[1] & 0x0F;
    if ![
        PICMG_ATCA_MAJOR_VERSION,
        PICMG_AMC_MAJOR_VERSION,
        PICMG_UTCA_MAJOR_VERSION,
    ]
    .contains(&major_version)
    {
        log::info!(
            "Unknown PICMG Extension Version {}.{}",
            major_version,
            rsp.data[1] >> 4
        );
        return 0;
    }

    //记录调试信息
    debug3!(
        "Discovered PICMG Extension Version {}.{}",
        major_version,
        rsp.data[1] >> 4
    );
    1
}
