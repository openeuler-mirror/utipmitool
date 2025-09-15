/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::debug1;
use crate::debug2;
use crate::error::IpmiError;
use crate::ipmi::intf::IpmiIntf;
use crate::ipmi::ipmi::*;

// VITA 命令常量
pub const VITA_GET_VSO_CAPABILITIES_CMD: u8 = 0x00;
pub const VITA_FRU_CONTROL_CMD: u8 = 0x04;
pub const VITA_GET_FRU_LED_PROPERTIES_CMD: u8 = 0x05;
pub const VITA_GET_LED_COLOR_CAPABILITIES_CMD: u8 = 0x06;
pub const VITA_SET_FRU_LED_STATE_CMD: u8 = 0x07;
pub const VITA_GET_FRU_LED_STATE_CMD: u8 = 0x08;
pub const VITA_SET_FRU_STATE_POLICY_BITS_CMD: u8 = 0x0A;
pub const VITA_GET_FRU_STATE_POLICY_BITS_CMD: u8 = 0x0B;
pub const VITA_SET_FRU_ACTIVATION_CMD: u8 = 0x0C;
pub const VITA_GET_FRU_ADDRESS_INFO_CMD: u8 = 0x40;

// VITA 46.11 站点类型
pub const VITA_FRONT_VPX_MODULE: u8 = 0x00;
pub const VITA_POWER_ENTRY: u8 = 0x01;
pub const VITA_CHASSIS_FRU: u8 = 0x02;
pub const VITA_DEDICATED_CHMC: u8 = 0x03;
pub const VITA_FAN_TRAY: u8 = 0x04;
pub const VITA_FAN_TRAY_FILTER: u8 = 0x05;
pub const VITA_ALARM_PANEL: u8 = 0x06;
pub const VITA_XMC: u8 = 0x07;
pub const VITA_VPX_RTM: u8 = 0x09;
pub const VITA_FRONT_VME_MODULE: u8 = 0x0A;
pub const VITA_FRONT_VXS_MODULE: u8 = 0x0B;
pub const VITA_POWER_SUPPLY: u8 = 0x0C;
pub const VITA_FRONT_VITA62_MODULE: u8 = 0x0D;
pub const VITA_71_MODULE: u8 = 0x0E;
pub const VITA_FMC: u8 = 0x0F;

pub const GROUP_EXT_VITA: u8 = 0x03; // 根据实际值调整

pub fn ipmi_vita_ipmb_address(intf: &mut dyn IpmiIntf) -> u8 {
    let mut data = vec![GROUP_EXT_VITA];
    let mut req = IpmiRq::default();

    req.msg.netfn_mut(IPMI_NETFN_PICMG);
    req.msg.cmd = VITA_GET_FRU_ADDRESS_INFO_CMD;
    req.msg.data = data.as_mut_ptr();
    req.msg.data_len = data.len() as u16;

    let rsp = match intf.sendrecv(&req) {
        Some(r) => r,
        None => {
            debug1!("No valid response received");
            return 0;
        }
    };

    if rsp.ccode != 0 {
        debug1!(
            "Invalid completion code received: {}",
            IpmiError::CompletionCode(rsp.ccode)
        );
        return 0;
    }

    if rsp.data.len() < 7 {
        debug1!("Invalid response length {}", rsp.data.len());
        return 0;
    }

    if rsp.data[0] != GROUP_EXT_VITA {
        debug1!("Invalid group extension {:#x}", rsp.data[0]);
        return 0;
    }

    rsp.data[2]
}

pub fn vita_discover(intf: &mut dyn IpmiIntf) -> u8 {
    let mut data = vec![GROUP_EXT_VITA];
    let mut req = IpmiRq::default();

    req.msg.netfn_mut(IPMI_NETFN_PICMG);
    req.msg.cmd = VITA_GET_VSO_CAPABILITIES_CMD;
    req.msg.data = data.as_mut_ptr();
    req.msg.data_len = data.len() as u16;

    let ctx = intf.context();
    debug2!(
        "Running Get VSO Capabilities my_addr 0x{:02x}, transit {}, target {}",
        ctx.my_addr(),
        ctx.transit_addr(),
        ctx.target_addr()
    );

    match intf.sendrecv(&req) {
        Some(rsp) => {
            if rsp.ccode == 0xC1 {
                debug2!("Invalid completion code received: Invalid command");
                0
            } else if rsp.ccode == 0xCC {
                debug1!(
                    "Invalid data field received: {}",
                    IpmiError::CompletionCode(rsp.ccode)
                );
                0
            } else if rsp.ccode != 0 {
                debug1!(
                    "Invalid completion code received: {}",
                    IpmiError::CompletionCode(rsp.ccode)
                );
                0
            } else if rsp.data.len() < 5 {
                debug1!("Invalid response length {}", rsp.data.len());
                0
            } else if rsp.data[0] != GROUP_EXT_VITA {
                debug1!("Invalid group extension {:#x}", rsp.data[0]);
                0
            } else if (rsp.data[3] & 0x03) != 0 {
                debug1!("Unknown VSO Standard {}", rsp.data[3] & 0x03);
                0
            } else if (rsp.data[4] & 0x0F) != 1 {
                debug1!(
                    "Unknown VSO Specification Revision {}.{}",
                    rsp.data[4] & 0x0F,
                    rsp.data[4] >> 4
                );
                0
            } else {
                debug1!(
                    "Discovered VITA 46.11 Revision {}.{}",
                    rsp.data[4] & 0x0F,
                    rsp.data[4] >> 4
                );
                1
            }
        }
        None => {
            debug1!("No valid response received");
            0
        }
    }
}

// use once_cell::unsync::OnceCell;
// static vita_avail: OnceCell<i32> = OnceCell::new();

// pub fn is_vita_avail(intf: &mut dyn IpmiIntf) -> bool {
//     vita_avail.get_or_init( || {
//         vita_discover(intf) != 0
//     })
// }
