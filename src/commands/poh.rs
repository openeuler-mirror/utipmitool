/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::error::{val2str, COMPLETION_CODE_VALS};
use crate::ipmi::intf::IpmiIntf;
use crate::ipmi::ipmi::{IpmiRq, IPMI_NETFN_CHASSIS};

pub fn ipmi_chassis_poh(intf: &mut dyn IpmiIntf) -> Result<(), String> {
    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_CHASSIS);
    req.msg.cmd = 0xf;

    match intf.sendrecv(&req) {
        Some(rsp) => {
            if rsp.ccode != 0 {
                return Err(format!(
                    "Get Chassis Power-On-Hours failed: {}",
                    val2str(rsp.ccode, &COMPLETION_CODE_VALS)
                ));
            }

            let mins_per_count = rsp.data[0];
            let count = u32::from_le_bytes([rsp.data[1], rsp.data[2], rsp.data[3], rsp.data[4]]);

            let mut minutes = (count as f32) * (mins_per_count as f32);
            let days = (minutes / 1440.0).floor() as u32;
            minutes -= (days as f32) * 1440.0;
            let hours = (minutes / 60.0).floor() as u32;
            minutes -= (hours as f32) * 60.0;

            if mins_per_count < 60 {
                println!(
                    "POH Counter  : {} days, {} hours, {} minutes",
                    days, hours, minutes as u32
                );
            } else {
                println!("POH Counter  : {} days, {} hours", days, hours);
            }

            Ok(())
        }
        None => Err("Unable to get Chassis Power-On-Hours".to_string()),
    }
}
