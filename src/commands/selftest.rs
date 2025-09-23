/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::error::{val2str, COMPLETION_CODE_VALS};
use crate::ipmi::intf::IpmiIntf;
use crate::ipmi::ipmi::{IpmiRq, IPMI_NETFN_APP};

pub fn ipmi_chassis_selftest(intf: &mut dyn IpmiIntf) -> Result<(), String> {
    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_APP);
    req.msg.cmd = 0x4;

    match intf.sendrecv(&req) {
        Some(rsp) => {
            if rsp.ccode != 0 {
                return Err(format!(
                    "Error sending Get Self Test command: {}",
                    val2str(rsp.ccode, &COMPLETION_CODE_VALS)
                ));
            }

            print!("Self Test Results    : ");
            match rsp.data[0] {
                0x55 => println!("passed"),
                0x56 => println!("not implemented"),
                0x57 => {
                    log::error!("device error");
                    let broken_dev_vals = [
                        (0, "firmware corrupted"),
                        (1, "boot block corrupted"),
                        (2, "FRU Internal Use Area corrupted"),
                        (3, "SDR Repository empty"),
                        (4, "IPMB not responding"),
                        (5, "cannot access BMC FRU"),
                        (6, "cannot access SDR Repository"),
                        (7, "cannot access SEL Device"),
                    ];
                    for i in 0..8 {
                        if rsp.data[1] & (1 << i) != 0 {
                            println!(
                                "                       [{}]",
                                broken_dev_vals
                                    .iter()
                                    .find(|&&(val, _)| val == i)
                                    .map(|&(_, desc)| desc)
                                    .unwrap_or("Unknown")
                            );
                        }
                    }
                }
                0x58 => log::error!("Fatal hardware error: {:02x}h", rsp.data[1]),
                _ => println!(
                    "Device-specific failure {:02x}h:{:02x}h",
                    rsp.data[0], rsp.data[1]
                ),
            }

            Ok(())
        }
        None => Err("Error sending Get Self Test command".to_string()),
    }
}
