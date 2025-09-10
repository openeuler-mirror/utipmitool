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
use crate::error::{val2str, COMPLETION_CODE_VALS};
use crate::ipmi::intf::*;
use crate::ipmi::ipmi::*;
use crate::ipmi::strings::IPMI_CHASSIS_POWER_CONTROL_VALS;
use crate::VERBOSE_LEVEL;
use std::sync::atomic::Ordering;

pub fn ipmi_chassis_power_status(mut intf: Box<dyn IpmiIntf>) -> Result<bool, String> {
    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_CHASSIS);
    req.msg.cmd = 0x1;
    req.msg.data_len = 0;

    match intf.sendrecv(&req) {
        Some(rsp) => {
            if rsp.ccode != 0 {
                Err(format!(
                    "Get Chassis Power Status failed: {}",
                    val2str(rsp.ccode, &COMPLETION_CODE_VALS)
                ))
            } else {
                Ok(rsp.data[0] & 1 != 0)
            }
        }
        None => Err("Unable to get Chassis Power Status".to_string()),
    }
}

pub fn ipmi_chassis_print_power_status(intf: Box<dyn IpmiIntf>) -> Result<(), String> {
    match ipmi_chassis_power_status(intf) {
        Ok(ps) => {
            println!("Chassis Power is {}", if ps { "on" } else { "off" });
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub fn ipmi_chassis_power_control(mut intf: Box<dyn IpmiIntf>, ctl: u8) -> Result<(), String> {
    // 只在verbose模式下显示调试信息
    if VERBOSE_LEVEL.load(Ordering::Relaxed) > 0 {
        println!(
            "DEBUG: Sending chassis power control command with value: 0x{:02x}",
            ctl
        );

        let control_str = IPMI_CHASSIS_POWER_CONTROL_VALS
            .iter()
            .find(|v| v.val == ctl)
            .map(|v| v.desc)
            .unwrap_or("Unknown");

        println!("DEBUG: Operation: {}", control_str);
    }

    let control_str = IPMI_CHASSIS_POWER_CONTROL_VALS
        .iter()
        .find(|v| v.val == ctl)
        .map(|v| v.desc)
        .unwrap_or("Unknown");

    // 只检查完全无效的值
    if ctl > 0x05 {
        return Err(format!(
            "Invalid chassis power control value: 0x{:02x}",
            ctl
        ));
    }

    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_CHASSIS);
    req.msg.cmd = 0x2;

    // 使用Box确保数据在堆上
    let data = Box::new([ctl]);
    req.msg.data = data.as_ptr() as *mut u8;
    req.msg.data_len = 1;

    let result = intf.sendrecv(&req);

    // 确保data在这里才被释放
    drop(data);

    match result {
        Some(rsp) => {
            if rsp.ccode != 0 {
                Err(format!(
                    "Set Chassis Power Control to {} failed: {}",
                    control_str,
                    val2str(rsp.ccode, &COMPLETION_CODE_VALS),
                ))
            } else {
                println!("Chassis Power Control: {}", control_str);
                Ok(())
            }
        }
        None => Err(format!(
            "Unable to set Chassis Power Control to {}",
            control_str
        )),
    }
}
