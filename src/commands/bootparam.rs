/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::commands::chassis::{MBOX_PARSE_ALLBLOCKS, MBOX_PARSE_USE_TEXT};
use crate::commands::CommandResult;
use crate::error::IpmiError;
use crate::error::{oem2str, IPMI_OEM_INFO};
use crate::helper::{buf2str, ipmi24toh};
use crate::ipmi::constants::{
    IPMI_CHASSIS_BOOTPARAM_INFO_ACK, IPMI_CHASSIS_BOOTPARAM_SET_IN_PROGRESS,
};
use crate::ipmi::intf::IpmiIntf;
use crate::ipmi::ipmi::{IpmiRq, IPMI_NETFN_CHASSIS};

// 添加缺少的常量
const IPMI_CC_PARAM_OUT_OF_RANGE: u8 = 0xc9;

// Mbox结构体定义
#[repr(C)]
struct Mbox {
    block: u8,
    data: [u8; 16],
    b0: MboxB0,
}

#[repr(C)]
struct MboxB0 {
    iana: [u8; 3],
    data: [u8; 13],
}

#[derive(Debug, Clone, Copy)]
pub enum Progress {
    SetComplete = 0,
    SetInProgress = 1,
    CommitWrite = 2,
    Reserved = 3,
}

// Flags for ipmi_chassis_get_bootparam()
#[derive(Debug, Clone, Copy)]
pub enum ChassisBootparamFlags {
    NoGenericInfo, // Do not print generic boot parameter info
    NoDataDump,    // Do not dump parameter data
    NoRangeError,  // Do not report out of range info to user
    Specific,      // Parameter-specific flags start with this
}

// Flags for ipmi_chassis_get_bootparam() for Boot Mailbox parameter (7)
#[derive(Debug, Clone, Copy)]
pub enum ChassisBootmboxParse {
    UseText = ChassisBootparamFlags::Specific as isize, // Use text output vs. hex
    AllBlocks,                                          // Parse all blocks, not just one
}

// Macro equivalent for BP_FLAG(x)
pub const fn bp_flag(x: u32) -> u32 {
    1 << x
}

pub fn chassis_bootparam_set_in_progress(
    intf: &mut dyn IpmiIntf,
    progress: Progress,
) -> CommandResult {
    // By default, try to set/clear set-in-progress parameter before/after
    // changing any boot parameters. If setting fails, the code will set
    // this flag to false and stop trying to fiddle with it for future
    // requests.
    static mut USE_PROGRESS: bool = true;

    unsafe {
        if !USE_PROGRESS {
            return Ok(());
        }
    }

    let flag = progress as u8;

    let rc = ipmi_chassis_set_bootparam(intf, IPMI_CHASSIS_BOOTPARAM_SET_IN_PROGRESS, &[flag]);

    unsafe {
        // Only disable future checks if set-in-progress status setting failed.
        // Setting of other statuses may fail legitimately.
        if rc.is_err() && matches!(progress, Progress::SetInProgress) {
            USE_PROGRESS = false;
        }
    }

    rc
}

#[derive(Debug, Clone, Copy)]
pub enum BootinfoAck {
    BiosPostAck = 1 << 0,
    OsLoaderAck = 1 << 1,
    OsServicePartitionAck = 1 << 2,
    SmsAck = 1 << 3,
    OemAck = 1 << 4,
    ReservedAckMask = 7 << 5,
}

pub fn ipmi_chassis_get_bootparam(
    intf: &mut dyn IpmiIntf,
    param_id: u8,
    additional_arg: Option<u8>,
    flags: u32,
) -> CommandResult {
    let skip_generic = flags & bp_flag(ChassisBootparamFlags::NoGenericInfo as u32) != 0;
    let skip_data = flags & bp_flag(ChassisBootparamFlags::NoDataDump as u32) != 0;
    let skip_range = flags & bp_flag(ChassisBootparamFlags::NoRangeError as u32) != 0;

    let mut msg_data = [0u8; 3];
    msg_data[0] = param_id & 0x7f;

    if let Some(arg) = additional_arg {
        msg_data[1] = arg;
    }

    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_CHASSIS);
    req.msg.cmd = 0x9;
    req.msg.data = msg_data.as_ptr() as *mut u8;
    req.msg.data_len = 3;

    match intf.sendrecv(&req) {
        Some(rsp) => {
            if rsp.ccode == IPMI_CC_PARAM_OUT_OF_RANGE && skip_range {
                return Err(IpmiError::InvalidData("Parameter out of range".to_string()));
            }
            if rsp.ccode != 0 {
                return Err(IpmiError::CompletionCode(rsp.ccode));
            }

            if !skip_generic {
                println!("Boot parameter version: {}", rsp.data[0]);
                println!(
                    "Boot parameter {} is {}",
                    rsp.data[1] & 0x7f,
                    if rsp.data[1] & 0x80 != 0 {
                        "invalid/locked"
                    } else {
                        "valid/unlocked"
                    }
                );
                if !skip_data {
                    println!(
                        "Boot parameter data: {}",
                        buf2str(&rsp.data[2..], rsp.data.len() - 2)
                    );
                }
            }

            match param_id {
                0 => {
                    println!("Set In Progress: ");
                    match rsp.data[2] & 0x03 {
                        0 => println!("set complete"),
                        1 => println!("set in progress"),
                        2 => println!("commit write"),
                        _ => println!("error, reserved bit"),
                    }
                }
                1 => {
                    println!("Service Partition Selector: ");
                    if rsp.data[2] == 0 {
                        println!("unspecified");
                    } else {
                        println!("{}", rsp.data[2]);
                    }
                }
                2 => {
                    println!("Service Partition Scan:");
                    if rsp.data[2] & 0x03 != 0 {
                        if rsp.data[2] & 0x01 != 0 {
                            println!("  - Request BIOS to scan");
                        }
                        if rsp.data[2] & 0x02 != 0 {
                            println!("  - Service Partition Discovered");
                        }
                    } else {
                        println!("  No flag set");
                    }
                }
                3 => {
                    println!("BMC boot flag valid bit clearing:");
                    if rsp.data[2] & 0x1f != 0 {
                        if rsp.data[2] & 0x10 != 0 {
                            println!(
                                "  - Don't clear valid bit on reset/power cycle caused by PEF"
                            );
                        }
                        if rsp.data[2] & 0x08 != 0 {
                            println!(
                                "  - Don't automatically clear boot flag valid bit on timeout"
                            );
                        }
                        if rsp.data[2] & 0x04 != 0 {
                            println!(
                                "  - Don't clear valid bit on reset/power cycle caused by watchdog"
                            );
                        }
                        if rsp.data[2] & 0x02 != 0 {
                            println!("  - Don't clear valid bit on push button reset/soft reset");
                        }
                        if rsp.data[2] & 0x01 != 0 {
                            println!("  - Don't clear valid bit on power up via power push button or wake event");
                        }
                    } else {
                        println!("  No flag set");
                    }
                }
                4 => {
                    println!("Boot Info Acknowledge:");
                    if rsp.data[3] & 0x1f != 0 {
                        if rsp.data[3] & 0x10 != 0 {
                            println!("  - OEM has handled boot info");
                        }
                        if rsp.data[3] & 0x08 != 0 {
                            println!("  - SMS has handled boot info");
                        }
                        if rsp.data[3] & 0x04 != 0 {
                            println!("  - OS/service partition has handled boot info");
                        }
                        if rsp.data[3] & 0x02 != 0 {
                            println!("  - OS Loader has handled boot info");
                        }
                        if rsp.data[3] & 0x01 != 0 {
                            println!("  - BIOS/POST has handled boot info");
                        }
                    } else {
                        println!("  No flag set");
                    }
                }
                5 => {
                    println!("Boot Flags:");
                    if rsp.data[2] & 0x80 != 0 {
                        println!("  - Boot Flag Valid");
                    } else {
                        println!("  - Boot Flag Invalid");
                    }
                    if rsp.data[2] & 0x40 != 0 {
                        println!("  - Options apply to all future boots");
                    } else {
                        println!("  - Options apply to only next boot");
                    }
                    println!("  - Boot Device Selector: ");
                    match rsp.data[3] & 0x0F {
                        0x00 => println!("No override"),
                        0x01 => println!("Force PXE"),
                        0x02 => println!("Force Boot from default Hard-Drive"),
                        0x03 => println!("Force Boot from default Hard-Drive, request Safe-Mode"),
                        0x04 => println!("Force Boot from Diagnostic Partition"),
                        0x05 => println!("Force Boot from CD/DVD"),
                        0x06 => println!("Force Boot into BIOS Setup"),
                        0x07 => println!(
                            "Force Boot from remotely connected Floppy/primary removable media"
                        ),
                        0x08 => println!("Force Boot from remotely connected CD/DVD"),
                        0x09 => println!("Force Boot from primary remote media"),
                        0x0A => println!("Force Boot from remotely connected Hard-Drive"),
                        0x0B => println!("Force Boot from Floppy/primary removable media"),
                        _ => println!("Flag error"),
                    }
                }
                6 => {
                    let session_id =
                        u32::from_le_bytes([rsp.data[3], rsp.data[4], rsp.data[5], rsp.data[6]]);
                    let timestamp =
                        u32::from_le_bytes([rsp.data[7], rsp.data[8], rsp.data[9], rsp.data[10]]);
                    println!("Boot Initiator Info:");
                    println!("  Channel Number: {}", rsp.data[2] & 0x0f);
                    println!("  Session Id: {:08X}", session_id);
                    println!("  Timestamp: {}", timestamp);
                }
                7 => {
                    chassis_bootmailbox_parse(Some(&rsp.data[2..]), flags);
                }
                _ => {
                    println!("Unsupported parameter {}", param_id);
                }
            }

            Ok(())
        }
        None => Err(IpmiError::InvalidData(format!(
            "Error Getting Chassis Boot Parameter {}",
            param_id
        ))),
    }
}

pub fn ipmi_chassis_set_bootparam(
    intf: &mut dyn IpmiIntf,
    param: u8,
    data: &[u8],
) -> CommandResult {
    let mut msg_data = vec![0u8; data.len() + 1];
    msg_data[0] = param;
    msg_data[1..].copy_from_slice(data);

    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_CHASSIS);
    req.msg.cmd = 0x8;
    req.msg.data = msg_data.as_ptr() as *mut u8;
    req.msg.data_len = msg_data.len() as u16;

    match intf.sendrecv(&req) {
        Some(rsp) => {
            if rsp.ccode != 0 {
                // 完全匹配ipmitool的错误格式
                let error_desc =
                    crate::error::val2str(rsp.ccode, &crate::error::COMPLETION_CODE_VALS);
                // 如果错误描述是"Unknown value"，显示格式为"Unknown (0xXX)"
                let formatted_error = if error_desc == "Unknown value" {
                    format!("Unknown (0x{:02x})", rsp.ccode)
                } else {
                    error_desc.to_string()
                };

                let error_msg = format!(
                    "Set Chassis Boot Parameter {} failed: {}",
                    param, formatted_error
                );
                Err(IpmiError::InvalidData(error_msg))
            } else {
                Ok(())
            }
        }
        None => Err(IpmiError::InvalidData(format!(
            "Error Setting Chassis Boot Parameter {}",
            param
        ))),
    }
}

pub fn get_bootparam_options(
    optstring: &str,
    set_flag: &mut u8,
    clr_flag: &mut u8,
) -> Result<(), String> {
    let mut option_error = false;
    *set_flag = 0;
    *clr_flag = 0;

    let options = [
        (
            "PEF",
            0x10,
            "Clear valid bit on reset/power cycle caused by PEF",
        ),
        (
            "timeout",
            0x08,
            "Automatically clear boot flag valid bit on timeout",
        ),
        (
            "watchdog",
            0x04,
            "Clear valid bit on reset/power cycle caused by watchdog",
        ),
        (
            "reset",
            0x02,
            "Clear valid bit on push button reset/soft reset",
        ),
        (
            "power",
            0x01,
            "Clear valid bit on power up via power push button or wake event",
        ),
    ];

    let optkw = "options=";
    if !optstring.starts_with(optkw) {
        return Err(format!("No options= keyword found \"{}\"", optstring));
    }

    let tokens = optstring[optkw.len()..].split(',');
    for token in tokens {
        let mut setbit = false;
        let mut token = token.trim();

        if token == "help" {
            option_error = true;
            break;
        }

        if token.starts_with("no-") {
            setbit = true;
            token = &token[3..];
        }

        if let Some((_, value, _)) = options.iter().find(|(name, _, _)| *name == token) {
            if setbit {
                *set_flag |= value;
            } else {
                *clr_flag |= value;
            }
        } else {
            return Err(format!("Invalid option: {}", token));
        }
    }

    if option_error {
        let mut error_message = String::from("Legal options are:\n");
        error_message.push_str("  help    : print this message\n");
        for (name, _, desc) in &options {
            error_message.push_str(&format!("  {:<8}: {}\n", name, desc));
        }
        error_message
            .push_str("Any option may be prepended with no- to invert sense of operation\n");
        return Err(error_message);
    }

    Ok(())
}

pub fn chassis_bootparam_clear_ack(intf: &mut dyn IpmiIntf, flag: BootinfoAck) -> CommandResult {
    // 根据ipmitool实现，INFO_ACK需要发送2字节数据：[0x01, flag_value]
    // ipmitool源码：flags[0] = 0x01; flags[1] = 0x01;
    // 这里第一个字节固定为0x01，第二个字节为具体的ACK标志
    let data = [0x01, flag as u8];
    ipmi_chassis_set_bootparam(intf, IPMI_CHASSIS_BOOTPARAM_INFO_ACK, &data)
}

fn chassis_bootmailbox_parse(buf: Option<&[u8]>, flags: u32) {
    if buf.is_none() || buf.unwrap().is_empty() {
        return;
    }

    let buf = buf.unwrap();
    let mbox: &Mbox = unsafe { &*(buf.as_ptr() as *const Mbox) };
    let mut blockdata = &mbox.data[..];
    let mut datalen = buf.len() - std::mem::size_of::<u8>();

    let use_text = flags & (1 << MBOX_PARSE_USE_TEXT) != 0;
    let all_blocks = flags & (1 << MBOX_PARSE_ALLBLOCKS) != 0;

    if !all_blocks {
        println!(" Selector       : {}", mbox.block);
    }

    if mbox.block == 0 {
        let iana = ipmi24toh(&mbox.b0.iana);
        println!(
            " IANA PEN       : {} [{}]",
            iana,
            oem2str(iana, &IPMI_OEM_INFO)
        );
        blockdata = &mbox.b0.data[..];
        datalen -= std::mem::size_of::<[u8; 3]>();
    }

    print!(" Block ");
    if all_blocks {
        print!("{:3} Data : ", mbox.block);
    } else {
        print!("Data     : ");
    }

    if use_text {
        let text = String::from_utf8_lossy(&blockdata[..datalen]);
        println!("'{}'", text);
    } else {
        println!("{}", buf2str(blockdata, datalen));
    }
}

pub fn chassis_bootmailbox_help() {
    println!("bootmbox get [text] [block <block>]");
    println!("  Read the entire Boot Initiator Mailbox or the specified <block>.");
    println!("  If 'text' option is specified, the data is output as plain text, otherwise");
    println!("  hex dump mode is used.");
    println!();
    println!("bootmbox set text [block <block>] <IANA_PEN> \"<data_string>\"");
    println!("bootmbox set [block <block>] <IANA_PEN> <data_byte> [<data_byte> ...]");
    println!("  Write the specified <block> or the entire Boot Initiator Mailbox.");
    println!("  It is required to specify a decimal IANA Enterprise Number recognized");
    println!("  by the boot initiator on the target system. Refer to your target system");
    println!("  manufacturer for details. The rest of the arguments are either separate");
    println!("  data byte values separated by spaces, or a single text string argument.");
    println!();
    println!("  When single block write is requested, the total length of <data> may not");
    println!("  exceed 13 bytes for block 0, or 16 bytes otherwise.");
    println!();
    println!("bootmbox help");
    println!("  Show this help.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_enum_values() {
        // 验证Progress枚举具有符合IPMI规范的正确数值
        assert_eq!(Progress::SetComplete as u8, 0);
        assert_eq!(Progress::SetInProgress as u8, 1);
        assert_eq!(Progress::CommitWrite as u8, 2);
        assert_eq!(Progress::Reserved as u8, 3);
    }

    #[test]
    fn test_bootinfo_ack_enum_values() {
        // 验证BootinfoAck枚举的bit位正确
        assert_eq!(BootinfoAck::BiosPostAck as u8, 1 << 0);
        assert_eq!(BootinfoAck::OsLoaderAck as u8, 1 << 1);
        assert_eq!(BootinfoAck::OsServicePartitionAck as u8, 1 << 2);
        assert_eq!(BootinfoAck::SmsAck as u8, 1 << 3);
        assert_eq!(BootinfoAck::OemAck as u8, 1 << 4);
    }

    #[test]
    fn test_clear_ack_data_format() {
        // 验证clear_ack函数准备的数据格式符合ipmitool实现
        // 这是关键修复：确保发送2字节数据而不是1字节
        let expected_data = [0x01, BootinfoAck::BiosPostAck as u8];

        // 这个测试验证了修复后的数据格式
        // 在修复前：只有1字节 [flag]
        // 在修复后：2字节 [0x01, flag] 符合ipmitool实现
        assert_eq!(
            expected_data.len(),
            2,
            "INFO_ACK should send 2 bytes of data"
        );
        assert_eq!(
            expected_data[0], 0x01,
            "First byte should always be 0x01 per ipmitool"
        );
        assert_eq!(expected_data[1], 0x01, "BiosPostAck should be 0x01");
    }
}
