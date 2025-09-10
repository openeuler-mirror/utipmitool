/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]

use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use secrecy::SecretString;
//use std::arch::x86_64;
//use std::net::{Ipv4Addr, Ipv6Addr};
use std::path::PathBuf;

//pub mod commands;
use utipmitool::commands::chassis::ChassisCommand;
use utipmitool::commands::lan::LanCommand;
use utipmitool::commands::mc::McCommand;
use utipmitool::commands::sdr::SdrCommand;
use utipmitool::commands::sel::SelCommand;
use utipmitool::commands::sensor::SensorCommand;
use utipmitool::commands::user::UserCommand;

//use crate::MainCommand;

// 核心接口类型枚举
#[derive(clap::ValueEnum, Clone, Debug)]
pub enum InterfaceType {
    #[clap(name = "open")]
    Open,
    // #[clap(name = "imb")]
    // Imb,
    // #[clap(name = "lipmi")]
    // Lipmi,
    // #[clap(name = "bmc")]
    // Bmc,
    // #[clap(name = "lan")]
    // Lan,
    // #[clap(name = "lanplus")]
    // LanPlus,
    // #[clap(name = "free")]
    // Free,
    // #[clap(name = "serial-terminal")]
    // SerialTerm,
    // #[clap(name = "serial-basic")]
    // SerialBm,
    // #[clap(name = "dummy")]
    // Dummy,
    // #[clap(name = "usb")]
    // Usb,
    // #[clap(name = "dbus")]
    // Dbus,
}

// 权限级别枚举，为啥对应的默认值是小写administrator？
#[derive(Debug, Clone, ValueEnum)]
pub enum PrivilegeLevel {
    Callback,
    User,
    Operator,
    Administrator,
    OEM,
}

// 认证类型枚举
#[derive(Debug, Clone, ValueEnum)]
pub enum AuthType {
    None,
    MD2,
    MD5,
    OEM,
    Password,
}

// 主命令结构
#[derive(Parser, Debug)]
#[command(
    name = "utipmitool",
    version = "0.9.0",
    about = "IPMI management utility",
    max_term_width = 100,
    disable_help_flag = true,
    disable_version_flag = true
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,

    #[command(subcommand)]
    pub command: MainCommand,
}

// 全局参数
#[derive(Args, Debug)]
pub struct GlobalArgs {
    // 基本信息参数
    #[arg(short = 'h', long, action = ArgAction::Help)]
    pub help: Option<bool>,

    #[arg(short = 'V', long, action = ArgAction::Version)]
    pub version: Option<bool>,

    #[arg(short = 'v', action = ArgAction::Count, help = "Verbose (can use multiple times)")]
    pub verbose: u8,

    #[arg(short = 'c', long)]
    pub csv_output: bool,

    // 设备接口参数
    #[arg(short = 'I', long, default_value = "open")]
    pub interface: InterfaceType,
    #[arg(short = 'd', default_value_t = 0)]
    pub devnum: u8, //open ioctl
    #[arg(short = 'D', long)]
    pub devfile: Option<PathBuf>, //串口

    // 网络参数
    #[arg(short = 'H', long)]
    pub hostname: Option<String>,
    #[arg(short = 'p', long, default_value_t = 623)]
    pub port: u16,

    //ipmi_intf_socket_connect,lan,lanplus
    #[arg(short = '4', conflicts_with = "ipv6", help = "Use only IPv4")]
    pub ipv4: bool,

    #[arg(short = '6', conflicts_with = "ipv4", help = "Use only IPv6")]
    pub ipv6: bool,
    //ai_family=AF_INET|AF_INET6

    // 安全认证参数
    #[arg(short = 'U', long)]
    pub username: Option<String>,
    #[arg(short = 'P', long)]
    pub password: Option<SecretString>,
    #[arg(short = 'f', long)]
    pub password_file: Option<PathBuf>,
    #[arg(short = 'a', long)]
    pub password_prompt: bool,

    // 新增: 从环境变量读取密码
    #[arg(
        short = 'E',
        help = "Read password from IPMI_PASSWORD environment variable"
    )]
    pub password_env: bool,

    #[arg(short = 'L', long, default_value = "administrator")]
    pub privilege: PrivilegeLevel,

    // 在现有参数中的适当位置添加
    // 新增: 修改通信通道大小
    #[arg(short = 'z', help = "Change Size of Communication Channel (OEM)")]
    pub comm_size: Option<u32>,

    // 新增: OEM类型设置
    #[arg(
        short = 'o',
        help = "Setup for OEM (use 'list' to see available OEM types)"
    )]
    pub oemtype: Option<String>,

    #[arg(short = 'A', long)]
    pub authtype: Option<AuthType>,

    // 高级功能参数
    #[arg(short = 'S', long)]
    pub sdr_cache: Option<PathBuf>,
    #[arg(short = 'O', long)]
    pub sel_oem: Option<PathBuf>,
    #[arg(short = 'N', long, default_value_t = 2)]
    pub timeout: u32,
    #[arg(short = 'R', long, default_value_t = 4)]
    pub retries: u32,

    //addr = arg_addr;
    //ipmi_main_intf->my_addr = addr;
    //ipmi_main_intf->target_addr = ipmi_main_intf->my_addr;

    // 桥接参数
    #[arg(short = 'b', long, default_value_t = 0)]
    pub target_channel: u8,
    #[arg(short = 't', long, default_value_t = 0)]
    pub target_addr: u8,
    #[arg(short = 'T', long, default_value_t = 0)]
    pub transit_addr: u8,
    #[arg(short = 'B', long, default_value_t = 0)]
    pub transit_channel: u8,
    #[arg(short = 'l', long, default_value_t = 0)]
    pub target_lun: u8,
    #[arg(short = 'm', long, default_value_t = 0)]
    // default_value = "0x20", value_parser = parse_hex)]
    pub arg_addr: u8,

    // 密钥管理
    #[arg(short = 'k', long)]
    pub kg_key: Option<SecretString>,
    #[arg(short = 'K', long)]
    pub kg_env: bool,
    #[arg(short = 'y', long)]
    pub hex_key: Option<String>,
    #[arg(short = 'Y', long)]
    pub key_prompt: bool,

    // LANPlus专用参数
    #[arg(short = 'C', long)]
    pub cipher_suite: Option<u8>,
    // SOL参数
    #[arg(short = 'e', long)]
    pub sol_escape: Option<char>,
}

fn parse_hex(s: &str) -> Result<u8, String> {
    u8::from_str_radix(s.trim_start_matches("0x"), 16)
        .map_err(|e| format!("无效的十六进制值: {}", e))
}
// 主命令枚举
#[derive(Subcommand, Debug)]
pub enum MainCommand {
    /// 原始IPMI命令
    // Raw {
    //     netfn: u8,
    //     cmd: u8,
    //     data: Vec<String>,
    // },MainCommand::Chassis

    /// 机箱控制
    Chassis {
        #[command(subcommand)]
        subcmd: ChassisCommand,
    },

    /// BMC管理控制器命令
    Mc {
        #[command(subcommand)]
        subcmd: McCommand,
    },

    /// 传感器管理
    Sensor {
        #[command(subcommand)]
        subcmd: Option<SensorCommand>,
    },

    /// SDR repository管理
    Sdr {
        #[command(subcommand)]
        subcmd: SdrCommand,
    },

    /// User management  
    User {
        #[command(subcommand)]
        subcmd: UserCommand,
    },

    // 其他主要命令...
    /// 网络配置
    #[command(name = "lan")]
    Lan {
        #[command(subcommand)]
        subcmd: LanCommand,
    },

    /// 远程串口控制台
    #[command(name = "sol")]
    Sol {
        #[command(subcommand)]
        subcmd: SolCommand,
    },

    /// 硬件事件日志
    #[command(name = "sel")]
    Sel {
        #[command(subcommand)]
        subcmd: SelCommand,
    },
}

// 启动设备
#[derive(ValueEnum, Clone, Debug)]
enum BootDevice {
    None,
    Pxe,
    Disk,
    Safe,
    Diag,
    Cdrom,
    Bios,
    Floppy,
}

// // 传感器子命令
// #[derive(Subcommand, Debug)]
// enum SensorCommand {
//     List,
//     Get {
//         sensor_ids: Vec<String>,
//     },
//     Thresh {
//         sensor_id: String,
//         threshold: ThresholdType,
//         value: f32,
//     },
// }

// // 阈值类型
// #[derive(ValueEnum, Clone, Debug)]
// enum ThresholdType {
//     Unr, Ucr, Unc, Lnc, Lcr, Lnr,
// }

// // 用户管理子命令
// #[derive(Subcommand, Debug)]
// pub enum UserCommand {
//     List,
//     Set {
//         user_id: u8,
//         #[command(flatten)]
//         params: UserParams,
//     },
//     Enable {
//         user_id: u8,
//     },
//     Disable {
//         user_id: u8,
//     },
// }

#[derive(Args, Debug)]
pub struct UserParams {
    name: Option<String>,
    password: Option<SecretString>,
    privilege: Option<PrivilegeLevel>,
}

// SOL管理子命令
#[derive(Subcommand, Debug)]
pub enum SolCommand {
    Info,
    Activate {
        #[arg(long)]
        instance: Option<u8>,
    },
    Deactivate {
        #[arg(long)]
        instance: Option<u8>,
    },
    Set {
        param: SolParam,
        value: String,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum SolParam {
    Enabled,
    PrivilegeLevel,
    BaudRate,
    // 其他参数...
}

impl Cli {
    pub fn validate(&self) -> Result<(), String> {
        // 验证密码选项互斥
        let password_sources = [
            self.global.password.is_some(),
            self.global.password_file.is_some(),
            self.global.password_prompt,
        ];
        if password_sources.iter().filter(|&&x| x).count() > 1 {
            return Err("只能指定一种密码源".into());
        }

        // // 验证接口类型相关参数
        // match self.global.interface {
        //     InterfaceType::LanPlus => {
        //         if self.global.cipher_suite.is_none() {
        //             return Err("LAN+接口需要指定密码套件(-C)".into());
        //         }
        //     },
        //     InterfaceType::Lan => {
        //         if self.global.authtype.is_none() {
        //             return Err("LAN接口需要指定认证类型(-A)".into());
        //         }
        //     },
        //     _ => {}
        // }

        // 更多验证逻辑...
        Ok(())
    }
}
