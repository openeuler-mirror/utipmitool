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
use crate::ipmi::intf::*;
use crate::ipmi::ipmi::*;
use crate::VERBOSE_LEVEL;
use std::sync::atomic::Ordering;

// 在文件顶部添加
use crate::debug_control;

use serde::Serialize;

#[derive(Debug, Serialize)]
struct ChassisStatus {
    system_power: bool,
    power_overload: bool,
    power_interlock: bool,
    main_power_fault: bool,
    power_control_fault: bool,
    power_restore_policy: String,
    last_power_events: Vec<String>,
    chassis_intrusion: bool,
    front_panel_lockout: bool,
    drive_fault: bool,
    cooling_fan_fault: bool,
    front_panel_control: Option<FrontPanelControl>,
}

#[derive(Debug, Serialize)]
struct FrontPanelControl {
    sleep_button_disable: bool,
    diag_button_disable: bool,
    reset_button_disable: bool,
    power_button_disable: bool,
    sleep_button_disabled: bool,
    diag_button_disabled: bool,
    reset_button_disabled: bool,
    power_button_disabled: bool,
}

fn decode_chassis_status(data: &[u8]) -> Result<ChassisStatus, String> {
    if data.len() < 3 {
        return Err("Invalid data length".into());
    }

    // 完全移除调试输出 - 原生ipmitool不在这里显示调试信息
    // 所有调试信息都应该在接口层处理

    // 解码字节0
    let status = ChassisStatus {
        system_power: data[0] & 0x01 != 0,
        power_overload: data[0] & 0x02 != 0,
        power_interlock: data[0] & 0x04 != 0,
        main_power_fault: data[0] & 0x08 != 0,
        power_control_fault: data[0] & 0x10 != 0,
        power_restore_policy: match (data[0] & 0x60) >> 5 {
            0x0 => "always-off",
            0x1 => "previous",
            0x2 => "always-on",
            _ => "unknown",
        }
        .to_string(),

        /*
                last_power_events: {
                    let event_byte = data[1];

                    // 移除所有调试输出

            let event_code = if (event_byte & 0x0F) == 0x02 {
                "command".to_string()
            } else if event_byte & 0x80 != 0 {
                // 如果高位设置，根据低4位解析
                match event_byte & 0x0F {
                    0x00 => "ac-on".to_string(),
                    0x01 => "ac-failed".to_string(),
                    // 0x02 已在前面处理
                    0x03 => "power-limit-exceeded".to_string(),
                    0x04 => "interlock".to_string(),
                    0x05 => "fault".to_string(),
                    0x06 => "overheated".to_string(),
                    0x10 => "overrrrrrrr".to_string(),
                    _ => String::new(),
                }
            } else {
                // 如果高位未设置且不是特殊情况，返回空
                String::new()
            };

            println!("event_code is {}",event_code);


                    // 创建事件向量
                    if event_code.is_empty() {
                        vec![]  // 如果事件代码为空，返回空向量
                    } else {
                        vec![event_code]
                    }

                },
        */
        last_power_events: {
            let event_byte = data[1];
            let mut events = Vec::new();

            // 使用位检测方式，与原生 ipmitool 保持一致
            if event_byte & 0x01 != 0 {
                events.push("ac-failed".to_string());
            }
            if event_byte & 0x02 != 0 {
                events.push("overload".to_string());
            }
            if event_byte & 0x04 != 0 {
                events.push("interlock".to_string());
            }
            if event_byte & 0x08 != 0 {
                events.push("fault".to_string());
            }
            if event_byte & 0x10 != 0 {
                events.push("command".to_string());
            }

            events
        },

        chassis_intrusion: data[2] & 0x01 != 0,
        front_panel_lockout: data[2] & 0x02 != 0,
        drive_fault: data[2] & 0x04 != 0,
        cooling_fan_fault: data[2] & 0x08 != 0,

        front_panel_control: if data.len() > 3 {
            Some(FrontPanelControl {
                sleep_button_disable: data[3] & 0x80 != 0,
                diag_button_disable: data[3] & 0x40 != 0,
                reset_button_disable: data[3] & 0x20 != 0,
                power_button_disable: data[3] & 0x10 != 0,
                sleep_button_disabled: data[3] & 0x08 != 0,
                diag_button_disabled: data[3] & 0x04 != 0,
                reset_button_disabled: data[3] & 0x02 != 0,
                power_button_disabled: data[3] & 0x01 != 0,
            })
        } else {
            None
        },
    };

    Ok(status)
}

fn console_print(status: &ChassisStatus) {
    println!(
        "System Power         : {}",
        if status.system_power { "on" } else { "off" }
    );
    println!("Power Overload       : {}", status.power_overload);
    println!(
        "Power Interlock      : {}",
        if status.power_interlock {
            "active"
        } else {
            "inactive"
        }
    );
    println!("Main Power Fault     : {}", status.main_power_fault);
    println!("Power Control Fault  : {}", status.power_control_fault);
    println!("Power Restore Policy : {}", status.power_restore_policy);

    print!("Last Power Event     : ");
    if !status.last_power_events.is_empty() {
        println!("{}", status.last_power_events[0]);
    } else {
        //println!("unknown");
        println!();
    }

    println!(
        "Chassis Intrusion    : {}",
        if status.chassis_intrusion {
            "active"
        } else {
            "inactive"
        }
    );
    println!(
        "Front-Panel Lockout  : {}",
        if status.front_panel_lockout {
            "active"
        } else {
            "inactive"
        }
    );
    println!("Drive Fault          : {}", status.drive_fault);
    println!("Cooling/Fan Fault    : {}", status.cooling_fan_fault);

    // 根据前面板控制信息选择正确的输出格式
    if let Some(fpc) = &status.front_panel_control {
        // 检查是否所有按钮状态都为默认值（全为false）
        let all_disabled_false = !fpc.sleep_button_disabled
            && !fpc.diag_button_disabled
            && !fpc.reset_button_disabled
            && !fpc.power_button_disabled;

        let all_disable_false = !fpc.sleep_button_disable
            && !fpc.diag_button_disable
            && !fpc.reset_button_disable
            && !fpc.power_button_disable;

        if all_disabled_false && all_disable_false {
            // 如果所有值都是默认的，显示简洁格式
            println!("Front Panel Control  : none");
        } else {
            // 如果有非默认值，显示详细格式（与原生 ipmitool 一致）
            println!(
                "Sleep Button Disable : {}",
                if fpc.sleep_button_disable {
                    "allowed"
                } else {
                    "not allowed"
                }
            );
            println!(
                "Diag Button Disable  : {}",
                if fpc.diag_button_disable {
                    "allowed"
                } else {
                    "not allowed"
                }
            );
            println!(
                "Reset Button Disable : {}",
                if fpc.reset_button_disable {
                    "allowed"
                } else {
                    "not allowed"
                }
            );
            println!(
                "Power Button Disable : {}",
                if fpc.power_button_disable {
                    "allowed"
                } else {
                    "not allowed"
                }
            );
            println!("Sleep Button Disabled: {}", fpc.sleep_button_disabled);
            println!("Diag Button Disabled : {}", fpc.diag_button_disabled);
            println!("Reset Button Disabled: {}", fpc.reset_button_disabled);
            println!("Power Button Disabled: {}", fpc.power_button_disabled);
        }
    } else {
        println!("Front Panel Control  : none");
    }

    /*
    if let Some(ref fp_control) = status.front_panel_control {
        if fp_control.power_button_disable || fp_control.reset_button_disable ||
           fp_control.diag_button_disable || fp_control.sleep_button_disable ||
           fp_control.power_button_disabled || fp_control.reset_button_disabled ||
           fp_control.diag_button_disabled || fp_control.sleep_button_disabled {
            println!("Front Panel Control  : disabled");
        } else {
            println!("Front Panel Control  : none");
        }
    } else {
        println!("Front Panel Control  : none");
    }
    */
}

// 在 console_print 函数之后添加此函数
pub fn ipmi_chassis_status(mut intf: Box<dyn IpmiIntf>) -> Result<(), String> {
    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_CHASSIS);
    req.msg.cmd = 0x1;

    // 如果是 vvv 模式，控制头信息输出
    let verbose_level = VERBOSE_LEVEL.load(Ordering::Relaxed);
    if verbose_level >= 3 && debug_control::should_skip_debug() {
        // 使用静态变量确保头信息只输出一次
        static HEADER_PRINTED: std::sync::atomic::AtomicBool =
            std::sync::atomic::AtomicBool::new(false);

        if !HEADER_PRINTED.swap(true, Ordering::SeqCst) {
            // 只输出一次 chassis status 命令的头部信息
            println!("OpenIPMI Request Message Header:");
            println!("  netfn     = 0x0");
            println!("  cmd       = 0x1");
        }
    }

    match intf.sendrecv(&req) {
        Some(rsp) => {
            if rsp.ccode != 0 {
                return Err(format!("Error: {}", rsp.ccode));
            }

            let status = decode_chassis_status(&rsp.data)?;
            console_print(&status);
            Ok(())
        }
        None => Err("Command failed".into()),
    }
}
