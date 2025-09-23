/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(unexpected_cfgs)]
use crate::commands::CommandResult;
use crate::error::IpmiError;
use crate::ipmi::constants::{
    IPMI_CHASSIS_BOOTPARAM_BOOT_FLAGS, IPMI_CHASSIS_BOOTPARAM_FLAG_VALID,
};
use crate::ipmi::intf::IpmiIntf;
use crate::ipmi::ipmi::{IpmiRq, IPMI_NETFN_CHASSIS};

// 从bootparam模块导入相关类型和函数
use super::bootparam::{
    chassis_bootparam_clear_ack, chassis_bootparam_set_in_progress, ipmi_chassis_set_bootparam,
    BootinfoAck, Progress,
};

pub fn ipmi_chassis_set_bootdev(
    intf: &mut dyn IpmiIntf,
    arg: Option<&str>,
    iflags: Option<&[u8; 5]>,
) -> CommandResult {
    let mut flags = [0u8; 5];

    // Start set in progress
    chassis_bootparam_set_in_progress(intf, Progress::SetInProgress)?;

    // Clear BIOS POST acknowledgement
    if let Err(e) = chassis_bootparam_clear_ack(intf, BootinfoAck::BiosPostAck) {
        // Make sure to set progress complete on error
        chassis_bootparam_set_in_progress(intf, Progress::SetComplete)?;
        return Err(e);
    }

    // Copy input flags if provided
    if let Some(input_flags) = iflags {
        flags.copy_from_slice(input_flags);
    }

    // Set boot device flags based on argument
    match arg {
        None | Some("none") => flags[1] |= 0x00,
        Some("pxe") | Some("force_pxe") => flags[1] |= 0x04,
        Some("disk") | Some("force_disk") => flags[1] |= 0x08,
        Some("safe") | Some("force_safe") => flags[1] |= 0x0c,
        Some("diag") | Some("force_diag") => flags[1] |= 0x10,
        Some("cdrom") | Some("force_cdrom") => flags[1] |= 0x14,
        Some("floppy") | Some("force_floppy") => flags[1] |= 0x3c,
        Some("bios") | Some("force_bios") => flags[1] |= 0x18,
        Some(invalid) => {
            // Make sure to set progress complete on error
            chassis_bootparam_set_in_progress(intf, Progress::SetComplete)?;
            return Err(IpmiError::InvalidData(format!(
                "Invalid argument: {}",
                invalid
            )));
        }
    }

    // Set flag valid bit
    flags[0] |= 0x80;

    // Set boot parameters
    let result = ipmi_chassis_set_bootparam(intf, IPMI_CHASSIS_BOOTPARAM_BOOT_FLAGS, &flags);

    if result.is_ok() {
        // If successful, commit the changes
        chassis_bootparam_set_in_progress(intf, Progress::CommitWrite)?;
        println!("Set Boot Device to {}", arg.unwrap_or("none"));
    }

    // Always set progress complete at the end
    chassis_bootparam_set_in_progress(intf, Progress::SetComplete)?;

    result
}

pub fn ipmi_chassis_set_bootvalid(
    intf: &mut dyn IpmiIntf,
    set_flag: u8,
    clr_flag: u8,
) -> CommandResult {
    // Set in progress
    chassis_bootparam_set_in_progress(intf, Progress::SetInProgress)?;

    // Clear BIOS POST ACK
    if let Err(e) = chassis_bootparam_clear_ack(intf, BootinfoAck::BiosPostAck) {
        // Make sure to set progress to complete even on error
        chassis_bootparam_set_in_progress(intf, Progress::SetComplete)?;
        return Err(e);
    }

    // Get current boot valid value
    let bootvalid = match ipmi_chassis_get_bootvalid(intf) {
        Ok(val) => val,
        Err(e) => {
            // Make sure to set progress to complete even on error
            chassis_bootparam_set_in_progress(intf, Progress::SetComplete)?;
            return Err(IpmiError::InvalidData(format!(
                "Failed to read boot valid flag: {}",
                e
            )));
        }
    };

    // Set new flags
    let flags = [(bootvalid & !clr_flag) | set_flag];

    let result = ipmi_chassis_set_bootparam(intf, IPMI_CHASSIS_BOOTPARAM_FLAG_VALID, &flags);

    if result.is_ok() {
        // If successful, commit the changes
        chassis_bootparam_set_in_progress(intf, Progress::CommitWrite)?;
    }

    // Always set progress to complete at the end
    chassis_bootparam_set_in_progress(intf, Progress::SetComplete)?;

    result
}

pub fn ipmi_chassis_get_bootvalid(intf: &mut dyn IpmiIntf) -> Result<u8, IpmiError> {
    let param_id = IPMI_CHASSIS_BOOTPARAM_FLAG_VALID;
    let msg_data = [param_id & 0x7f, 0, 0];

    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_CHASSIS);
    req.msg.cmd = 0x9;
    req.msg.data = msg_data.as_ptr() as *mut u8;
    req.msg.data_len = msg_data.len() as u16;

    match intf.sendrecv(&req) {
        Some(rsp) => {
            if rsp.ccode != 0 {
                return Err(IpmiError::CompletionCode(rsp.ccode));
            }

            if cfg!(feature = "verbose") {
                println!("Boot Option: {:?}", &rsp.data[..rsp.data_len as usize]);
            }

            Ok(rsp.data[2])
        }
        None => Err(IpmiError::InvalidData(format!(
            "Error Getting Chassis Boot Parameter {}",
            param_id
        ))),
    }
}
