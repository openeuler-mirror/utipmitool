/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::error::{val2str, COMPLETION_CODE_VALS};
use crate::ipmi::intf::IpmiIntf;
use crate::ipmi::ipmi::{IpmiRq, IPMI_NETFN_CHASSIS};

#[repr(C)]
struct IdentifyParams {
    interval: u8, // 标识间隔时间（0=关闭）
    flags: u8,    // 标识标志位（bit0=强制标识）
}
pub fn ipmi_chassis_identify(intf: &mut dyn IpmiIntf, arg: Option<&str>) -> Result<(), String> {
    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_CHASSIS);
    req.msg.cmd = 0x4;

    //let mut identify_data = [0u8; 2];// 两个uint8
    let mut params = IdentifyParams {
        interval: 0,
        flags: 0,
    };

    let mut data_len: u16 = 1;

    if let Some(arg) = arg {
        if arg == "force" {
            params.flags = 1;
            data_len = 2;
        } else {
            match arg.parse::<u8>() {
                Ok(interval) => params.interval = interval,
                Err(_) => return Err("Invalid interval given.".to_string()),
            }
        }
    }

    // 转换结构体为字节切片
    req.msg.data = &params as *const IdentifyParams as *mut u8;
    req.msg.data_len = data_len;

    match intf.sendrecv(&req) {
        Some(rsp) => {
            if rsp.ccode != 0 {
                if params.flags != 0 {
                    println!("Chassis may not support Force Identify On");
                }
                Err(format!(
                    "Set Chassis Identify failed: {}",
                    val2str(rsp.ccode, &COMPLETION_CODE_VALS)
                ))
            } else {
                print!("Chassis identify interval: ");
                if arg.is_none() {
                    println!("default (15 seconds)");
                } else if params.flags != 0 {
                    println!("indefinite");
                } else if params.interval == 0 {
                    println!("off");
                } else {
                    println!("{} seconds", params.interval);
                }
                Ok(())
            }
        }
        None => Err("Unable to set Chassis Identify".to_string()),
    }
}
