/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
pub mod define;
pub mod describe;
pub mod entry;
pub mod info;
#[allow(clippy::module_inception)]
pub mod sel;
pub mod supermicro;

use crate::ipmi::intf::IpmiIntf;
use clap::Subcommand;
use std::error::Error;

use crate::commands::sel::info::ipmi_sel_get_info;
use crate::commands::sel::sel::ipmi_sel_list;

#[derive(Subcommand, Debug)]
pub enum SelTimeCommand {
    Get,
    Set {
        time: String, // MM/DD/YYYY HH:MM:SS
    },
}
// SEL管理子命令
#[derive(Subcommand, Debug)]
pub enum SelCommand {
    Info,
    #[command(name = "list")]
    List {
        // 支持多种参数模式：
        // list
        // list <count>
        // list first <count>
        // list last <count>
        args: Vec<String>,
    },
    #[command(name = "elist")]
    EList {
        // 支持多种参数模式：
        // elist
        // elist <count>
        // elist first <count>
        // elist last <count>
        args: Vec<String>,
    },
    // Clear,
    // Time {
    //     #[command(subcommand)]
    //     action: SelTimeCommand,
    // },
}

pub fn ipmi_sel_main(
    command: SelCommand,
    mut intf: Box<dyn IpmiIntf>,
) -> Result<(), Box<dyn Error>> {
    match command {
        SelCommand::Info => ipmi_sel_get_info(&mut intf),
        SelCommand::List { args } => {
            let (order, count) = parse_sel_list_args(&args)?;
            ipmi_sel_list(intf.as_mut(), order, count, false)
        }
        SelCommand::EList { args } => {
            let (order, count) = parse_sel_list_args(&args)?;
            ipmi_sel_list(intf.as_mut(), order, count, true)
        }
    }
}

/// 解析 SEL list 命令的参数，模拟 ipmitool 的行为
/// 支持以下格式：
/// - list                 (显示所有)
/// - list <n>             (显示 n 个)  
/// - list first <n>       (显示前 n 个)
/// - list last <n>        (显示后 n 个)
fn parse_sel_list_args(
    args: &[String],
) -> Result<(Option<String>, Option<usize>), Box<dyn std::error::Error>> {
    match args.len() {
        0 => {
            // list 或 elist
            Ok((None, None))
        }
        1 => {
            // list <count>
            let count_str = &args[0];
            let count = parse_count_arg(count_str)?;
            Ok((Some("first".to_string()), Some(count)))
        }
        2 => {
            // list first <count> 或 list last <count>
            let order = &args[0];
            let count_str = &args[1];

            if order != "first" && order != "last" {
                return Err(format!("Unknown sel list option: {}", order).into());
            }

            let count = parse_count_arg(count_str)?;
            Ok((Some(order.clone()), Some(count)))
        }
        _ => Err("Too many arguments for sel list command".into()),
    }
}

/// 解析数值参数，支持负数（用于表示从后往前）
fn parse_count_arg(count_str: &str) -> Result<usize, Box<dyn std::error::Error>> {
    // 首先尝试解析为 i32 来处理负数
    match count_str.parse::<i32>() {
        Ok(n) if n < 0 => {
            // 负数转换为正数，调用方会处理 last 逻辑
            Ok((-n) as usize)
        }
        Ok(n) => Ok(n as usize),
        Err(_) => {
            // 提供与 ipmitool 一致的错误信息
            Err(format!("Numeric argument required; got '{}'", count_str).into())
        }
    }
}
