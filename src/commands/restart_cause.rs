/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::error::{val2str, COMPLETION_CODE_VALS};
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
                    val2str(rsp.ccode, &COMPLETION_CODE_VALS)
                ));
            }

            println!(
                "System restart cause: {}",
                val2str(rsp.data[0] & 0xf, &COMPLETION_CODE_VALS)
            );

            Ok(())
        }
        None => Err("Unable to get Chassis Restart Cause".to_string()),
    }
}
