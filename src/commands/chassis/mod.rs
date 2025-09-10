/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
pub mod common;
pub mod power;
pub mod status;

use clap::Subcommand;
use clap::ValueEnum;

// use super::define_chassis::*;
// use super::status::*;

use crate::commands::bootparam::{
    chassis_bootparam_clear_ack, chassis_bootparam_set_in_progress, ipmi_chassis_set_bootparam,
    BootinfoAck, Progress,
};
use crate::ipmi::constants::*;
use crate::ipmi::intf::*;
use crate::ipmi::ipmi::{IpmiRq, IPMI_NETFN_CHASSIS};
use crate::ipmi::strings::IPMI_CHASSIS_RESTART_CAUSE_VALS;
use power::*;
use status::*;

// 机箱子命令
#[derive(Subcommand, Debug)]
pub enum ChassisCommand {
    Status,
    Power {
        #[command(subcommand)]
        action: PowerAction,
    },
    Identify {
        seconds: Option<u32>,
    },

    // 修改：指定命令名为 restart_cause，保留 restart-cause 作为别名
    #[command(name = "restart_cause", alias = "restart-cause")]
    RestartCause,
    BootDev {
        device: BootDevice,
        #[arg(long)]
        clear_cmos: Option<bool>,
    },
}

// 电源操作
#[derive(Subcommand, Clone, Debug)]
pub enum PowerAction {
    #[command(name = "status")]
    Status,
    #[command(name = "on", alias = "up")]
    On,
    #[command(name = "off", alias = "down")]
    Off,
    #[command(name = "cycle")]
    Cycle,
    #[command(name = "reset")]
    Reset,
    #[command(name = "diag")]
    Diag,
    #[command(name = "soft", alias = "acpi")]
    Soft,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum BootDevice {
    None,
    Pxe,
    Disk,
    Safe,
    Diag,
    Cdrom,
    Bios,
    Floppy,
}

//forced boot parameter
#[derive(ValueEnum, Clone, Debug)]
pub enum BootParam {
    Pxe,
    Disk,
    Safe,
    Diag,
    Cdrom,
    Bios,
}

pub const MBOX_PARSE_USE_TEXT: u32 = PARAM_SPECIFIC;
pub const MBOX_PARSE_ALLBLOCKS: u32 = PARAM_SPECIFIC + 1;
const PARAM_SPECIFIC: u32 = 3;

pub fn ipmi_chassis_main(cmd: ChassisCommand, intf: Box<dyn IpmiIntf>) -> Result<(), String> {
    match cmd {
        ChassisCommand::Status => ipmi_chassis_status(intf),
        ChassisCommand::Power { action } => match action {
            PowerAction::Status => ipmi_chassis_print_power_status(intf),
            PowerAction::On => ipmi_chassis_power_control(intf, IPMI_CHASSIS_CTL_POWER_UP),
            PowerAction::Off => ipmi_chassis_power_control(intf, IPMI_CHASSIS_CTL_POWER_DOWN),
            PowerAction::Cycle => ipmi_chassis_power_control(intf, IPMI_CHASSIS_CTL_POWER_CYCLE),
            PowerAction::Reset => ipmi_chassis_power_control(intf, IPMI_CHASSIS_CTL_HARD_RESET),
            PowerAction::Diag => ipmi_chassis_power_control(intf, IPMI_CHASSIS_CTL_PULSE_DIAG),
            PowerAction::Soft => ipmi_chassis_power_control(intf, IPMI_CHASSIS_CTL_ACPI_SOFT),
            //非法参数clap已经处理了
        },
        ChassisCommand::Identify { seconds } => ipmi_chassis_identify(intf, seconds),
        ChassisCommand::RestartCause => ipmi_chassis_restart_cause(intf),
        ChassisCommand::BootDev { device, clear_cmos } => {
            ipmi_chassis_set_bootdev(intf, device, clear_cmos)
        }
    }
}

pub fn ipmi_chassis_identify(
    mut intf: Box<dyn IpmiIntf>,
    seconds: Option<u32>,
) -> Result<(), String> {
    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_CHASSIS);
    req.msg.cmd = 0x4; // Chassis Identify command

    let mut identify_data = [0u8; 2];
    let mut data_len = 0;

    if let Some(interval) = seconds {
        if interval > 255 {
            return Err("Identify interval must be between 0-255 seconds".to_string());
        }
        identify_data[0] = interval as u8;
        data_len = 1;

        // Note: force_on parameter is not implemented in this version
        // It would be identify_data[1] = 1 for force on
    }

    if data_len > 0 {
        req.msg.data = identify_data.as_mut_ptr();
        req.msg.data_len = data_len;
    }

    match intf.sendrecv(&req) {
        Some(rsp) => {
            if rsp.ccode != 0 {
                Err(format!(
                    "Set Chassis Identify failed: completion code 0x{:02x}",
                    rsp.ccode
                ))
            } else {
                print!("Chassis identify interval: ");
                match seconds {
                    None => println!("default (15 seconds)"),
                    Some(0) => println!("off"),
                    Some(interval) => println!("{} seconds", interval),
                }
                Ok(())
            }
        }
        None => Err("Unable to set Chassis Identify".to_string()),
    }
}

pub fn ipmi_chassis_restart_cause(mut intf: Box<dyn IpmiIntf>) -> Result<(), String> {
    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_CHASSIS);
    req.msg.cmd = 0x7; // Get System Restart Cause command

    match intf.sendrecv(&req) {
        Some(rsp) => {
            if rsp.ccode != 0 {
                Err(format!(
                    "Get Chassis Restart Cause failed: completion code 0x{:02x}",
                    rsp.ccode
                ))
            } else {
                if rsp.data_len > 0 {
                    let cause = rsp.data[0] & 0xf;
                    let cause_str = IPMI_CHASSIS_RESTART_CAUSE_VALS
                        .iter()
                        .find(|v| v.val == cause as u32)
                        .map(|v| v.desc)
                        .unwrap_or("Unknown");
                    println!("System restart cause: {}", cause_str);
                } else {
                    return Err("Invalid response data length".to_string());
                }
                Ok(())
            }
        }
        None => Err("Unable to get Chassis Restart Cause".to_string()),
    }
}

pub fn ipmi_chassis_set_bootdev(
    mut intf: Box<dyn IpmiIntf>,
    device: BootDevice,
    clear_cmos: Option<bool>,
) -> Result<(), String> {
    // Set boot parameter in progress
    chassis_bootparam_set_in_progress(intf.as_mut(), Progress::SetInProgress)
        .map_err(|e| format!("Failed to set progress: {}", e))?;

    // Clear BIOS POST ACK
    chassis_bootparam_clear_ack(intf.as_mut(), BootinfoAck::BiosPostAck)
        .map_err(|e| format!("Failed to clear ACK: {}", e))?;

    let mut flags = [0u8; 5];

    // Set boot device flags based on device type
    match device {
        BootDevice::None => flags[1] |= 0x00,
        BootDevice::Pxe => flags[1] |= 0x04,
        BootDevice::Disk => flags[1] |= 0x08,
        BootDevice::Safe => flags[1] |= 0x0c,
        BootDevice::Diag => flags[1] |= 0x10,
        BootDevice::Cdrom => flags[1] |= 0x14,
        BootDevice::Bios => flags[1] |= 0x18,
        BootDevice::Floppy => flags[1] |= 0x3c,
    }

    // Set flag valid bit
    flags[0] |= 0x80;

    // Handle clear_cmos option if provided
    if let Some(true) = clear_cmos {
        flags[2] |= 0x80; // Set CMOS clear bit
    }

    // Set boot flags parameter
    let boot_flags_result =
        ipmi_chassis_set_bootparam(intf.as_mut(), IPMI_CHASSIS_BOOTPARAM_BOOT_FLAGS, &flags);

    // 按照ipmitool的逻辑：只有在设置boot flags成功时才进行后续操作和显示成功消息
    if boot_flags_result.is_ok() {
        // Only commit if setting boot flags succeeded
        let _commit_result =
            chassis_bootparam_set_in_progress(intf.as_mut(), Progress::CommitWrite);
        // Note: we don't fail if commit fails, just like ipmitool
        if _commit_result.is_err() {
            log::debug!("Commit write failed, but continuing anyway");
        }

        let device_name = match device {
            BootDevice::None => "none",
            BootDevice::Pxe => "pxe",
            BootDevice::Disk => "disk",
            BootDevice::Safe => "safe",
            BootDevice::Diag => "diag",
            BootDevice::Cdrom => "cdrom",
            BootDevice::Bios => "bios",
            BootDevice::Floppy => "floppy",
        };

        println!("Set Boot Device to {}", device_name);
    } else {
        // 直接打印错误消息，避免额外包装
        if let Err(e) = boot_flags_result {
            eprintln!("{}", e);
        }
    }

    // Always set to complete at the end, regardless of success/failure
    let _complete_result = chassis_bootparam_set_in_progress(intf.as_mut(), Progress::SetComplete);
    if _complete_result.is_err() {
        log::debug!("Failed to set progress to complete, but continuing anyway");
    }

    // Return success regardless, since we've already handled error display
    Ok(())
}
