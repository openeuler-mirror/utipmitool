/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
mod cli;
use clap::Parser;
use cli::{Cli, InterfaceType, MainCommand};
use std::sync::atomic::Ordering;
use utipmitool::commands::chassis::ipmi_chassis_main;
use utipmitool::commands::lan::ipmi_lan_main;
use utipmitool::commands::mc::ipmi_mc_main;
use utipmitool::commands::sdr::ipmi_sdr_main;
use utipmitool::commands::sel::ipmi_sel_main;
use utipmitool::commands::sensor::ipmi_sensor_main;
use utipmitool::commands::sensor::SensorCommand;
use utipmitool::commands::user::ipmi_user_main;
use utipmitool::debug_control;
use utipmitool::interface::open::open::OpenIntf; //open::OpenIntf
                                                 //open::OpenIntf
use utipmitool::ipmi::picmg::*;
use utipmitool::ipmi::vita::*;
use utipmitool::VERBOSE_LEVEL;

use utipmitool::debug;
// use utipmitool::debug1;
use utipmitool::debug2;
use utipmitool::debug3;
use utipmitool::logging;
//use ipmi_tool::log;
use utipmitool::interface::open::open::IPMI_BMC_CHANNEL;
use utipmitool::ipmi::context::{IpmiBaseContext, OutputContext, ProtocolContext};
use utipmitool::ipmi::intf::{IpmiContext, IpmiIntf, IpmiIntfExt};
use utipmitool::ipmi::ipmi::IPMI_BMC_SLAVE_ADDR;

fn main() {
    //setup_logger().expect("Failed to initialize logger");
    //log::info!("This will go to both syslog and stdout!");
    //log::error!("This error will also go to both!");
    //log::debug!("Basic debug info");     // 需要至少1个-v

    // 添加这段代码到main函数开头
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 && args[1] == "chassis" {
        // 没有子命令或者子命令是help，显示自定义帮助
        if args.len() == 2 || (args.len() == 3 && args[2] == "help") {
            println!("Chassis Commands:  status, power, identify, restart_cause, boot-dev");
            return;
        }
    }

    // 添加在函数开头，在解析命令行参数前
    use crate::debug_control;
    debug_control::hide_loading_interface_message(); // 隐藏"Loading interface: Open"信息

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            // 检查是否是用户命令相关的错误
            let input = std::env::args().collect::<Vec<_>>();
            if input.len() >= 2 && input[1] == "user" {
                match err.kind() {
                    clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
                        if input.len() == 2 =>
                    {
                        // 显示 ipmitool 兼容的用户命令帮助
                        use utipmitool::commands::user;
                        user::show_user_commands_help();
                        return;
                    }
                    clap::error::ErrorKind::InvalidSubcommand if input.len() >= 3 => {
                        // 显示无效用户命令错误
                        use utipmitool::commands::user;
                        println!("Invalid user command: '{}'", input[2]);
                        println!();
                        user::show_user_commands_help_impl(false);
                        return;
                    }
                    _ => {}
                }
            }
            // 检查是否是LAN命令相关的错误
            if input.len() >= 2 && input[1] == "lan" {
                match err.kind() {
                    clap::error::ErrorKind::InvalidSubcommand if input.len() >= 3 => {
                        // 显示无效LAN命令错误，匹配ipmitool格式
                        println!("Invalid LAN command: {}", input[2]);
                        return;
                    }
                    _ => {}
                }
            }

            // 添加对 chassis 命令的特殊处理
            if input.len() >= 2 && input[1] == "chassis" {
                match err.kind() {
                    clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
                        if input.len() == 2 =>
                    {
                        // 显示 ipmitool 兼容的 chassis 命令帮助
                        println!("Chassis Commands:  status, power, identify, policy, restart_cause, poh, bootdev, bootparam, selftest");
                        return;
                    }
                    clap::error::ErrorKind::InvalidSubcommand if input.len() >= 3 => {
                        // 显示无效 chassis 命令错误
                        println!("Invalid chassis command: {}", input[2]);
                        println!("Chassis Commands:  status, power, identify, policy, restart_cause, poh, bootdev, bootparam, selftest");
                        return;
                    }
                    _ => {}
                }
            }

            err.exit();
        }
    };
    logging::setup_logger(cli.global.verbose);
    // debug1!("More detailed info");  // 需要至少1个-v
    // debug2!("Even more info");      // 需要至少2个-v
    // debug3!("Most detailed info");  // 需要至少3个-v
    // log::error!("log::error");
    // log::info!("log::info");
    // log::debug!("log::debug");
    // log::warn!("log::warn");
    // log::trace!("log::trace");

    VERBOSE_LEVEL.store(cli.global.verbose as usize, Ordering::Relaxed);

    // 先检查是否是只需要显示帮助的用户命令
    if let MainCommand::User {
        subcmd:
            utipmitool::commands::user::UserCommand::Priv {
                user_id, privilege, ..
            },
    } = &cli.command
    {
        if user_id.is_none() || privilege.is_none() {
            // 显示帮助信息并退出
            use utipmitool::commands::user;
            user::show_user_commands_help_impl(false);
            return;
        }
    }

    let ctx = IpmiContext {
        base: IpmiBaseContext {
            my_addr: if cli.global.arg_addr != 0 {
                cli.global.arg_addr as u32
            } else {
                IPMI_BMC_SLAVE_ADDR
            },
            target_addr: 0, // 默认目标地址为0
            target_channel: IPMI_BMC_CHANNEL,
            target_lun: 0,
            target_ipmb_addr: IPMI_BMC_SLAVE_ADDR as u8,
        },
        bridging: None,
        protocol: ProtocolContext::default(),
        output: OutputContext::new(cli.global.csv_output, cli.global.verbose),
    };

    // 加载接口
    debug3!("Loading interface: {:?}", cli.global.interface);
    let mut intf = match cli.global.interface {
        InterfaceType::Open => {
            //ipmi_intf_load

            //intf.setup();
            //intf.open
            Box::new(OpenIntf::new(cli.global.devnum, ctx))
        } //Some(InterfaceType::Lan) => load_lan_interface(),
          //Some(InterfaceType::LanPlus) => load_lanplus_interface(),
          //None => get_default_interface(), // 默认接口
          // 其他接口处理...
    };
    //会调用set_my_addr设置一个默认地址
    if let Err(e) = intf.setup() {
        eprintln!("Unable to setup interface: {}", e);
        return;
    }
    if let Err(e) = intf.open() {
        eprintln!("{}", e);
        return;
    }
    debug3!("Interface opened successfully");

    // -vv级别的调试信息：开始获取IPMB地址
    debug2!("Acquire IPMB address");

    let (final_addr, target_ipmb_addr) = if cli.global.arg_addr == 0 {
        let addr = ipmi_acquire_ipmb_address(intf.as_mut());
        (addr, addr)
    } else {
        let ipmb = ipmi_acquire_ipmb_address(intf.as_mut());
        (cli.global.arg_addr, ipmb)
    };

    // 当最终地址有效且与接口当前地址不同时更新

    let my_addr = intf.with_context(|ctx| ctx.my_addr());
    if final_addr != 0 && final_addr != my_addr as u8 {
        if let Err(e) = intf.set_my_addr(final_addr) {
            eprintln!("Unable to set my address to 0x{:x}: {}", final_addr, e);
            intf.with_context(|ctx| {
                ctx.set_my_addr(final_addr as u32);
            });
        }
    }

    intf.with_context(|ctx| {
        ctx.set_target_addr(final_addr as u32);

        if cli.global.transit_addr > 0 || cli.global.target_addr > 0 {
            if (cli.global.transit_addr != 0 || cli.global.transit_channel != 0)
                && cli.global.target_addr ==0 {
                debug!(
                    "Transit address/channel [0x{:02x}/0x{:02x}] ignored. Target address must be specified!",
                    cli.global.transit_addr,  cli.global.transit_channel
                );
                return;
            }
            ctx.set_target_addr(cli.global.target_addr as u32);
            ctx.set_target_channel(cli.global.target_channel);
            ctx.set_transit_addr(cli.global.transit_addr as u32);
            ctx.set_transit_channel(cli.global.transit_channel);
        }
    });

    intf.with_context(|ctx| {
        ctx.set_target_ipmb_addr(target_ipmb_addr);
        debug3!(
            "Specified addressing     Target  0x{:02x}:0x{:02x} Transit 0x{:02x}:0x{:02x}",
            ctx.target_addr(),
            ctx.target_channel(),
            ctx.transit_addr(),
            ctx.transit_channel()
        );
        if ctx.target_ipmb_addr() != 0 {
            log::info!(
                "Discovered Target IPMB-0 address 0x{:02x}",
                ctx.target_ipmb_addr()
            );
        }

        // -vv级别的调试信息：接口地址信息
        debug2!(
            "Interface address: my_addr 0x{:02x} transit {}:{} target 0x{:02x}:{} ipmb_target {}",
            ctx.my_addr(),
            ctx.transit_addr(),
            ctx.transit_channel(),
            ctx.target_addr(),
            ctx.target_channel(),
            ctx.target_ipmb_addr()
        );

        debug3!(
            "Interface address: 0x{:02x}\n                transit {}:{} target {}:{}\n                ipmb_target 0x{:02x}",
            ctx.my_addr(),
            ctx.transit_addr(),
            ctx.transit_channel(),
            ctx.target_addr(),
            ctx.target_channel(),
            ctx.target_ipmb_addr(),
        );
    });

    // let verbose = VERBOSE_LEVEL.load(Ordering::Relaxed) > 0;

    // intf.with_context(|ctx| {
    //     debug1!(
    //         "Bridge Info: {} tgt_addr:{:#x} tgt_channel:{:#x} txn_addr:{:#x} txn_channel:{:#x}",
    //         if ctx.get_bridging_level() > 0 {
    //             "enabled"
    //         } else {
    //             "disabled"
    //         },
    //         ctx.target_addr(), ctx.target_channel(), ctx.transit_addr(), ctx.transit_channel()
    //     );

    //     if ctx.target_ipmb_addr() != 0 {
    //         log::info!(
    //             "Discovered Target IPMB-0 address 0x{:02x}",
    //             ctx.target_ipmb_addr()
    //         );
    //     }

    //     if verbose {
    //         println!(
    //             "Info: local={:#x} transit={:#x}:{:#x} target={:#x}:{:#x} ipmb={:#x}",
    //             ctx.my_addr(),
    //             ctx.transit_addr(),
    //             ctx.transit_channel(),
    //             ctx.target_addr(),
    //             ctx.target_channel(),
    //             ctx.target_ipmb_addr()
    //         );
    //     }
    // });

    // 在这里添加 debug_control::reset_debug_state(); 这一行
    debug_control::reset_debug_state();

    match cli.command {
        MainCommand::Chassis { subcmd } => {
            ipmi_chassis_main(subcmd, intf).unwrap_or_else(|e| log::error!("Error: {}", e))
        }
        MainCommand::Lan { subcmd } => {
            ipmi_lan_main(subcmd, intf).unwrap_or_else(|e| log::error!("Error: {}", e))
        }
        MainCommand::Mc { subcmd } => {
            ipmi_mc_main(subcmd, intf).unwrap_or_else(|e| log::error!("Error: {}", e))
        }
        MainCommand::Sensor { subcmd } => {
            let command = subcmd.unwrap_or(SensorCommand::List);
            ipmi_sensor_main(command, intf).unwrap_or_else(|e| log::error!("Error: {}", e))
        }

        MainCommand::Sel { subcmd } => {
            ipmi_sel_main(subcmd, intf).unwrap_or_else(|e| log::error!("Error: {}", e))
        }

        MainCommand::Sdr { subcmd } => {
            ipmi_sdr_main(subcmd, intf).unwrap_or_else(|e| log::error!("Error: {}", e))
        }
        MainCommand::User { subcmd } => {
            ipmi_user_main(subcmd, intf).unwrap_or_else(|e| {
                // 与ipmitool保持一致的错误输出格式
                eprintln!("{}", e)
            })
        }
        _ => unimplemented!(),
    }
}

fn ipmi_acquire_ipmb_address(intf: &mut dyn IpmiIntf) -> u8 {
    // 获取和显示IANA厂商ID
    let actual_id = get_manufacturer_id_from_device(intf);
    if actual_id != 0 {
        debug2!("Iana: {}", actual_id);
    }

    // 先尝试 PICMG 扩展
    if picmg_discover(intf) != 0 {
        let addr = ipmi_picmg_ipmb_address(intf);
        debug2!("Discovered IPMB address 0x{:02x}", addr);
        return addr;
    }

    // 尝试 VITA 扩展
    if vita_discover(intf) != 0 {
        let addr = ipmi_vita_ipmb_address(intf);
        debug2!("Discovered IPMB address 0x{:02x}", addr);
        return addr;
    }

    // 默认返回0
    debug2!("Discovered IPMB address 0x0");
    0
}

/// 从设备ID响应中获取厂商ID的数值
fn get_manufacturer_id_from_device(intf: &mut dyn IpmiIntf) -> u32 {
    use utipmitool::commands::mc::{IpmDevidRsp, BMC_GET_DEVICE_ID};
    use utipmitool::helper::ipmi24toh;
    use utipmitool::ipmi::ipmi::{IpmiRq, IPMI_NETFN_APP};

    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_APP);
    req.msg.cmd = BMC_GET_DEVICE_ID;
    req.msg.data_len = 0;

    if let Some(rsp) = intf.sendrecv(&req) {
        if rsp.ccode == 0 {
            if let Ok(devid) = IpmDevidRsp::from_le_bytes(rsp.data.as_slice()) {
                return ipmi24toh(&devid.manufacturer_id);
            }
        }
    }
    0
}
