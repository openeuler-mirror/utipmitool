/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

use crate::error::completion_code_to_string;
use crate::ipmi::intf::IpmiIntf;
use crate::ipmi::ipmi::{IpmiRq, IPMI_NETFN_CHASSIS};

pub fn ipmi_chassis_restart_cause(intf: &mut dyn IpmiIntf) -> Result<(), String> {
    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_CHASSIS);
    req.msg.cmd = 0x7;

    match intf.sendrecv(&req) {
        Some(rsp) => {
            if rsp.ccode != 0 {
                return Err(format!(
                    "Get Chassis Restart Cause failed: {}",
                    completion_code_to_string(rsp.ccode)
                ));
            }

            println!(
                "System restart cause: {}",
                completion_code_to_string(rsp.data[0] & 0xf)
            );

            Ok(())
        }
        None => Err("Unable to get Chassis Restart Cause".to_string()),
    }
}
