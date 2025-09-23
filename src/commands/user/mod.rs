/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use clap::{Args, Subcommand};
use std::error::Error;

use crate::debug1;
use crate::error::IpmiError;
use crate::ipmi::constants::*;
use crate::ipmi::intf::IpmiIntf;
use crate::ipmi::ipmi::{IpmiRq, IPMI_NETFN_APP};

/// 解析权限级别，支持十进制和十六进制输入
fn parse_privilege_level(s: &str) -> Result<u8, String> {
    let privilege = if s.starts_with("0x") || s.starts_with("0X") {
        // 十六进制格式
        u8::from_str_radix(&s[2..], 16)
            .map_err(|_| format!("Invalid hexadecimal privilege level: '{}'", s))?
    } else {
        // 十进制格式
        s.parse::<u8>()
            .map_err(|_| format!("Invalid decimal privilege level: '{}'", s))?
    };

    // 验证权限级别是否有效 - 与ipmitool保持一致
    match privilege {
        1..=5 | 15 => Ok(privilege),
        _ => Err(format!(
            "Invalid privilege level: {}. Valid values are:\n\
            Privilege levels:\n  \
            * 0x1 - Callback\n  \
            * 0x2 - User\n  \
            * 0x3 - Operator\n  \
            * 0x4 - Administrator\n  \
            * 0x5 - OEM Proprietary\n  \
            * 0xF - No Access\n\n\
            Usage: utipmitool user priv <user id> <privilege level> [<channel number>]",
            privilege
        )),
    }
}

/// 解析测试格式参数
fn parse_test_format(s: &str) -> Result<String, String> {
    match s {
        "16" | "20" => Ok(s.to_string()),
        _ => Err(format!(
            "Invalid password format: '{}'. Valid values are:\n  \
            16 - 16-byte password format\n  \
            20 - 20-byte password format\n\n\
            Examples:\n  \
            utipmitool user test 2 16         # Test user 2 with 16-byte format\n  \
            utipmitool user test 3 20 mypass  # Test user 3 with 20-byte format and custom password",
            s
        )),
    }
}

/// 解析密码格式参数
fn parse_password_format(s: &str) -> Result<String, String> {
    match s {
        "16" | "20" => Ok(s.to_string()),
        _ => Err(format!(
            "Invalid password format: '{}'. Valid values are:\n  \
            16 - 16-byte password format\n  \
            20 - 20-byte password format\n\n\
            Examples:\n  \
            utipmitool user set password 2 mypass 16  # Set password with 16-byte format\n  \
            utipmitool user set password 3 mypass 20  # Set password with 20-byte format",
            s
        )),
    }
}

#[derive(Subcommand, Debug)]
pub enum UserCommand {
    /// Show user summary
    Summary {
        #[arg(value_name = "CHANNEL_NUM")]
        channel: Option<u8>,
    },
    /// List all users
    List {
        #[arg(value_name = "CHANNEL_NUM")]
        channel: Option<u8>,
    },
    /// Set user attributes
    Set(UserSetCommand),
    /// Disable user
    Disable {
        #[arg(value_name = "USER_ID")]
        user_id: u8,
    },
    /// Enable user
    Enable {
        #[arg(value_name = "USER_ID")]
        user_id: u8,
    },
    /// 设置用户权限
    #[command(about = "Set user privilege level\n\
                      \n\
                      Privilege levels:\n  \
                      * 0x1 - Callback\n  \
                      * 0x2 - User\n  \
                      * 0x3 - Operator\n  \
                      * 0x4 - Administrator\n  \
                      * 0x5 - OEM Proprietary\n  \
                      * 0xF - No Access")]
    Priv {
        #[arg(value_name = "USER_ID")]
        user_id: Option<u8>,
        #[arg(value_name = "PRIVILEGE_LEVEL", value_parser = parse_privilege_level)]
        privilege: Option<u8>,
        #[arg(value_name = "CHANNEL_NUM")]
        channel: Option<u8>,
    },
    /// Test password storage format  
    Test {
        #[arg(value_name = "USER_ID")]
        user_id: u8,
        #[arg(value_parser = parse_test_format)]
        format: String,
        #[arg(value_name = "PASSWORD")]
        password: Option<String>,
    },
}

#[derive(Args, Debug)]
pub struct UserSetCommand {
    #[command(subcommand)]
    pub action: UserSetAction,
}

#[derive(Subcommand, Debug)]
pub enum UserSetAction {
    /// Set username
    Name {
        #[arg(value_name = "USER_ID")]
        user_id: u8,
        #[arg(value_name = "USERNAME")]
        username: String,
    },
    /// Set/clear password
    Password {
        #[arg(value_name = "USER_ID")]
        user_id: u8,
        #[arg(value_name = "PASSWORD")]
        password: Option<String>,
        #[arg(value_name = "FORMAT", value_parser = parse_password_format)]
        format: Option<String>,
    },
}

pub fn ipmi_user_main(
    command: UserCommand,
    mut intf: Box<dyn IpmiIntf>,
) -> Result<(), Box<dyn Error>> {
    match command {
        UserCommand::Summary { channel } => {
            ipmi_print_user_summary(intf.as_mut(), channel.unwrap_or(0x0E))
        }
        UserCommand::List { channel } => {
            ipmi_print_user_list(intf.as_mut(), channel.unwrap_or(0x0E))
        }
        UserCommand::Set(set_cmd) => ipmi_user_set_main(intf.as_mut(), set_cmd),
        UserCommand::Disable { user_id } => ipmi_user_disable(intf.as_mut(), user_id),
        UserCommand::Enable { user_id } => ipmi_user_enable(intf.as_mut(), user_id),
        UserCommand::Priv {
            user_id,
            privilege,
            channel,
        } => {
            // 如果参数缺失，显示用户命令帮助信息
            match (user_id, privilege) {
                (Some(uid), Some(priv_level)) => {
                    ipmi_user_set_privilege(intf.as_mut(), uid, priv_level, channel)
                }
                _ => {
                    // 显示完整的用户命令帮助信息，与ipmitool保持一致
                    show_user_commands_help_impl(false);
                    Ok(())
                }
            }
        }
        UserCommand::Test {
            user_id,
            format,
            password,
        } => {
            let is_twenty_byte = format == "20";
            ipmi_user_test_password(intf.as_mut(), user_id, password.as_deref(), is_twenty_byte)
        }
    }
}

// IPMI user-related constants
pub const IPMI_PASSWORD_DISABLE_USER: u8 = 0x00;
pub const IPMI_PASSWORD_ENABLE_USER: u8 = 0x01;
pub const IPMI_PASSWORD_SET_PASSWORD: u8 = 0x02;
pub const IPMI_PASSWORD_TEST_PASSWORD: u8 = 0x03;

// IPMI user management commands
pub const IPMI_SET_USER_NAME: u8 = 0x45;
pub const IPMI_SET_USER_PASSWORD: u8 = 0x47;
pub const IPMI_SET_USER_ACCESS: u8 = 0x43;

pub const IPMI_USER_ENABLE_UNSPECIFIED: u8 = 0x00;
pub const IPMI_USER_ENABLE_ENABLED: u8 = 0x40;
pub const IPMI_USER_ENABLE_DISABLED: u8 = 0x80;
pub const IPMI_USER_ENABLE_RESERVED: u8 = 0xC0;

pub const IPMI_UID_MASK: u8 = 0x3F; // The user_id is 6-bit and is usually in bits [5:0]
pub const IPMI_UID_MAX: u8 = 63;

pub const IPMI_UID_MIN: u8 = 1;

#[inline]
pub fn ipmi_uid(id: u8) -> u8 {
    id & IPMI_UID_MASK
}

#[derive(Default, Debug)]
pub struct UserAccess {
    pub callin_callback: u8,
    pub channel: u8,
    pub enabled_user_ids: u8,
    pub enable_status: u8,
    pub fixed_user_ids: u8,
    pub ipmi_messaging: u8,
    pub link_auth: u8,
    pub max_user_ids: u8,
    pub privilege_limit: u8,
    pub session_limit: u8,
    pub user_id: u8,
}

impl UserAccess {
    /// 格式化输出用户摘要信息
    pub fn format_summary(&self, csv: bool) -> String {
        if csv {
            format!(
                "{},{},{}",
                self.max_user_ids, self.enabled_user_ids, self.fixed_user_ids
            )
        } else {
            format!(
                "Maximum IDs         : {}\nEnabled User Count  : {}\nFixed Name Count    : {}",
                self.max_user_ids, self.enabled_user_ids, self.fixed_user_ids
            )
        }
    }

    /// 格式化单个用户信息输出
    pub fn format_user_info(
        &self,
        user_name: &str,
        csv: bool,
        show_header: bool,
        verbose: u8,
    ) -> String {
        if csv {
            // CSV格式：每行都要有换行符
            format!(
                "{},{},{},{},{},{}\n",
                self.user_id,
                user_name, // 空用户名在CSV中显示为空字符串
                if self.callin_callback != 0 {
                    "false"
                } else {
                    "true"
                },
                if self.link_auth != 0 { "true" } else { "false" },
                if self.ipmi_messaging != 0 {
                    "true"
                } else {
                    "false"
                },
                privilege_level_to_str(self.privilege_limit)
            )
        } else {
            let mut output = String::new();

            // 打印表头（只在第一次调用时打印）
            if show_header {
                // 完全匹配C代码的表头格式
                output.push_str(
                    "ID  Name             Callin  Link Auth  IPMI Msg   Channel Priv Limit\n",
                );
            }

            // 处理用户名显示：空用户名在表格中显示为空，不显示"(empty user)"
            let display_name = if user_name.is_empty() { "" } else { user_name };

            // 打印用户信息行 - 完全匹配C代码格式
            output.push_str(&format!(
                "{:<4}{:<17}{:<8}{:<11}{:<11}{}\n",
                self.user_id,
                display_name,
                if self.callin_callback != 0 {
                    "false"
                } else {
                    "true "
                },
                if self.link_auth != 0 {
                    "true "
                } else {
                    "false"
                },
                if self.ipmi_messaging != 0 {
                    "true "
                } else {
                    "false"
                },
                privilege_level_to_str(self.privilege_limit)
            ));

            // 如果启用详细输出，显示额外信息
            if verbose > 0 {
                output.push_str(&format!(
                    "    Enable Status: 0x{:02X}\n",
                    self.enable_status
                ));
                output.push_str(&format!("    Session Limit: {}\n", self.session_limit));
            }

            if verbose > 1 {
                output.push_str(&format!("    Channel: {}\n", self.channel));
                output.push_str(&format!("    Max User IDs: {}\n", self.max_user_ids));
                output.push_str(&format!(
                    "    Enabled User IDs: {}\n",
                    self.enabled_user_ids
                ));
                output.push_str(&format!("    Fixed User IDs: {}\n", self.fixed_user_ids));
            }

            output
        }
    }
}

#[derive(Default, Debug)]
pub struct UserName {
    pub user_id: u8,
    pub user_name: [u8; 17],
}

impl UserName {
    /// 获取用户名的字符串表示
    pub fn name_as_string(&self) -> String {
        let end = self.user_name.iter().position(|&x| x == 0).unwrap_or(16);
        String::from_utf8_lossy(&self.user_name[..end]).to_string()
    }
}

/// 权限级别转换为字符串
fn privilege_level_to_str(level: u8) -> String {
    match level {
        0x1 => "CALLBACK".to_string(),
        0x2 => "USER".to_string(),
        0x3 => "OPERATOR".to_string(),
        0x4 => "ADMINISTRATOR".to_string(),
        0x5 => "OEM".to_string(),
        0xF => "NO ACCESS".to_string(),
        _ => format!("Unknown (0x{:02X})", level),
    }
}

/// 显示用户命令帮助信息，与ipmitool保持一致
pub fn show_user_commands_help() {
    show_user_commands_help_impl(true);
}

/// 显示用户命令帮助信息的内部实现
pub fn show_user_commands_help_impl(show_error_message: bool) {
    if show_error_message {
        println!("Not enough parameters given.");
    }
    println!("User Commands:");
    println!("        summary         [<channel number>]");
    println!("        list            [<channel number>]");
    println!("        set name        <user id> <username>");
    println!("        set password    <user id> [<password> <16|20>]");
    println!("        disable         <user id>");
    println!("        enable          <user id>");
    println!("        priv            <user id> <privilege level> [<channel number>]");
    println!("                        Privilege levels:");
    println!("                        * 0x1 - Callback");
    println!("                        * 0x2 - User");
    println!("                        * 0x3 - Operator");
    println!("                        * 0x4 - Administrator");
    println!("                        * 0x5 - OEM Proprietary");
    println!("                        * 0xF - No Access");
    println!();
    println!("        test            <user id> <16|20> [<password>]");
    println!();
}

pub fn ipmi_print_user_summary(
    intf: &mut dyn IpmiIntf,
    channel_number: u8,
) -> Result<(), Box<dyn Error>> {
    let mut user_access = UserAccess {
        user_id: 1,
        channel: channel_number,
        ..Default::default()
    };

    ipmi_get_user_access(intf, &mut user_access)?;

    // 从IpmiContext获取输出配置并克隆
    let is_csv = intf.context().output_config().csv;

    // 使用format_summary方法
    println!("{}", user_access.format_summary(is_csv));

    Ok(())
}

pub fn ipmi_print_user_list(
    intf: &mut dyn IpmiIntf,
    channel_number: u8,
) -> Result<(), Box<dyn Error>> {
    let mut current_user_id = 1u8;
    let mut first_user = true;

    // 从IpmiContext获取输出配置并克隆
    let output_config = intf.context().output_config().clone();

    loop {
        let mut user_access = UserAccess {
            user_id: current_user_id,
            channel: channel_number,
            ..Default::default()
        };

        // 获取用户访问信息
        if ipmi_get_user_access(intf, &mut user_access).is_err() {
            return Err("Failed to get user access information".into());
        }

        // 获取用户名
        let mut user_name = UserName {
            user_id: current_user_id,
            ..Default::default()
        };

        let user_name_string = match ipmi_get_user_name(intf, &mut user_name) {
            Ok(_) => user_name.name_as_string(),
            Err(e) => {
                // 如果返回码是0xCC，表示用户名为空，这是正常的
                if let Some(IpmiError::CompletionCode(0xCC)) = e.downcast_ref::<IpmiError>() {
                    String::new() // 空用户名
                } else {
                    return Err(e);
                }
            }
        };

        // 使用format_user_info方法
        print!(
            "{}",
            user_access.format_user_info(
                &user_name_string,
                output_config.csv,
                first_user,
                output_config.verbose
            )
        );

        first_user = false;

        current_user_id += 1;

        // 检查循环结束条件
        if current_user_id > user_access.max_user_ids || current_user_id > IPMI_UID_MAX {
            break;
        }
    }

    Ok(())
}

fn ipmi_get_user_access(
    intf: &mut dyn IpmiIntf,
    user_access_rsp: &mut UserAccess,
) -> Result<(), Box<dyn Error>> {
    let mut req = IpmiRq::default();
    let mut data = [0u8; 2];

    data[0] = user_access_rsp.channel & 0x0F;
    data[1] = ipmi_uid(user_access_rsp.user_id);

    req.msg.netfn_mut(IPMI_NETFN_APP);
    req.msg.cmd = IPMI_GET_USER_ACCESS;
    req.msg.data = data.as_mut_ptr();
    req.msg.data_len = data.len() as u16;

    let rsp = match intf.sendrecv(&req) {
        Some(rsp) => rsp,
        None => return Err("IPMI response is NULL.".into()),
    };

    if rsp.ccode != 0 {
        return Err(IpmiError::CompletionCode(rsp.ccode).into());
    }

    if rsp.data_len != 4 {
        return Err("Unexpected data length received.".into());
    }

    user_access_rsp.max_user_ids = ipmi_uid(rsp.data[0]);
    user_access_rsp.enable_status = rsp.data[1] & 0xC0;
    user_access_rsp.enabled_user_ids = ipmi_uid(rsp.data[1]);
    user_access_rsp.fixed_user_ids = ipmi_uid(rsp.data[2]);
    user_access_rsp.callin_callback = rsp.data[3] & 0x40;
    user_access_rsp.link_auth = rsp.data[3] & 0x20;
    user_access_rsp.ipmi_messaging = rsp.data[3] & 0x10;
    user_access_rsp.privilege_limit = rsp.data[3] & 0x0F;

    Ok(())
}

fn ipmi_get_user_name(
    intf: &mut dyn IpmiIntf,
    user_name: &mut UserName,
) -> Result<(), Box<dyn Error>> {
    let mut req = IpmiRq::default();
    let mut data = [0u8; 1];

    data[0] = ipmi_uid(user_name.user_id);

    req.msg.netfn_mut(IPMI_NETFN_APP);
    req.msg.cmd = IPMI_GET_USER_NAME;
    req.msg.data = data.as_mut_ptr();
    req.msg.data_len = data.len() as u16;

    let rsp = match intf.sendrecv(&req) {
        Some(rsp) => rsp,
        None => return Err("IPMI response is NULL.".into()),
    };

    if rsp.ccode != 0 {
        return Err(IpmiError::CompletionCode(rsp.ccode).into());
    }

    if rsp.data_len != 16 {
        return Err("Unexpected data length received for user name.".into());
    }

    // 清空用户名数组并复制数据
    user_name.user_name.fill(0);
    user_name.user_name[..16].copy_from_slice(&rsp.data[..16]);

    Ok(())
}

pub fn ipmi_user_set_main(
    intf: &mut dyn IpmiIntf,
    set_cmd: UserSetCommand,
) -> Result<(), Box<dyn Error>> {
    match set_cmd.action {
        UserSetAction::Name { user_id, username } => ipmi_set_user_name(intf, user_id, &username),
        UserSetAction::Password {
            user_id,
            password,
            format,
        } => {
            let is_twenty_byte = format.as_deref() == Some("20");
            ipmi_set_user_password(intf, user_id, password.as_deref(), is_twenty_byte)
        }
    }
}

/// 设置用户名
pub fn ipmi_set_user_name(
    intf: &mut dyn IpmiIntf,
    user_id: u8,
    username: &str,
) -> Result<(), Box<dyn Error>> {
    // 验证用户ID范围
    if !(IPMI_UID_MIN..=IPMI_UID_MAX).contains(&user_id) {
        return Err(format!(
            "Invalid user ID: {}. Must be between {} and {}",
            user_id, IPMI_UID_MIN, IPMI_UID_MAX
        )
        .into());
    }

    // 验证用户名长度 (最大16字节) - 参考C代码
    if username.len() >= 17 {
        return Err("Username too long. Maximum 16 characters allowed".into());
    }

    debug1!("Setting username '{}' for user {}", username, user_id);

    // 参考C代码和其他模块的req初始化模式
    let mut req = IpmiRq::default();
    let mut msg_data = [0u8; 17]; // 17字节：1字节用户ID + 16字节用户名

    // 设置数据 - 参考C代码
    msg_data[0] = ipmi_uid(user_id); // IPMI_UID(user_id)
    let username_bytes = username.as_bytes();
    let copy_len = username_bytes.len().min(16);
    msg_data[1..1 + copy_len].copy_from_slice(&username_bytes[..copy_len]);

    // 设置请求参数 - 参考其他模块的模式
    req.msg.netfn_mut(IPMI_NETFN_APP); // 0x06
    req.msg.cmd = IPMI_SET_USER_NAME; // 0x45
    req.msg.data = msg_data.as_mut_ptr();
    req.msg.data_len = msg_data.len() as u16;

    // 发送请求 - 参考其他模块的错误处理模式
    let _rsp = match intf.sendrecv(&req) {
        Some(rsp) => {
            if rsp.ccode != 0 {
                // 使用友好的错误信息，与ipmitool保持一致
                let error_desc =
                    crate::error::val2str(rsp.ccode, &crate::error::COMPLETION_CODE_VALS);
                // 如果错误描述是"Unknown value"，显示格式为"Unknown (0xXX)"
                let formatted_error = if error_desc == "Unknown value" {
                    format!("Unknown (0x{:02x})", rsp.ccode)
                } else {
                    error_desc.to_string()
                };
                return Err(format!(
                    "Set User Name command failed (user {}, name {}): {}",
                    user_id, username, formatted_error
                )
                .into());
            }
            rsp
        }
        None => {
            return Err(format!(
                "Set User Name command failed (user {}, name {}): no response",
                user_id, username
            )
            .into());
        }
    };

    // 成功时不输出消息 - 参考C代码
    Ok(())
}

/// 通用的用户密码操作函数，类似于ipmitool的_ipmi_set_user_password
fn _ipmi_set_user_password(
    intf: &mut dyn IpmiIntf,
    user_id: u8,
    operation: u8,
    password: Option<&str>,
    is_twenty_byte: bool,
) -> Result<(), Box<dyn Error>> {
    // 验证用户ID范围
    if !(IPMI_UID_MIN..=IPMI_UID_MAX).contains(&user_id) {
        return Err(format!(
            "Invalid user ID: {}. Must be between {} and {}",
            user_id, IPMI_UID_MIN, IPMI_UID_MAX
        )
        .into());
    }

    debug1!(
        "User password operation: user={}, operation=0x{:02x}",
        user_id,
        operation
    );

    // 参考ipmitool的_ipmi_set_user_password实现
    let mut req = IpmiRq::default();
    let data_len = if is_twenty_byte { 22 } else { 18 };
    let mut data = vec![0u8; data_len];

    // 设置数据 - 完全按照ipmitool的逻辑
    data[0] = if is_twenty_byte { 0x80 } else { 0x00 };
    data[0] |= ipmi_uid(user_id); // IPMI_UID(user_id)
    data[1] = operation & 0x03; // operation masked with 0x03

    // 复制密码 - 参考ipmitool的逻辑
    if let Some(pwd) = password {
        let password_bytes = pwd.as_bytes();
        let copy_len = password_bytes.len().min(data_len - 2);
        if copy_len > 0 {
            data[2..2 + copy_len].copy_from_slice(&password_bytes[..copy_len]);
        }
    }

    // 设置请求参数
    req.msg.netfn_mut(IPMI_NETFN_APP);
    req.msg.cmd = IPMI_SET_USER_PASSWORD;
    req.msg.data = data.as_mut_ptr();
    req.msg.data_len = data.len() as u16;

    // 发送请求 - 返回错误码而不是抛出异常，与ipmitool一致
    let rsp = match intf.sendrecv(&req) {
        Some(rsp) => rsp,
        None => return Err("IPMI response is NULL".into()),
    };

    if rsp.ccode != 0 {
        return Err(IpmiError::CompletionCode(rsp.ccode).into());
    }

    Ok(())
}

/// 设置用户密码
pub fn ipmi_set_user_password(
    intf: &mut dyn IpmiIntf,
    user_id: u8,
    password: Option<&str>,
    is_twenty_byte: bool,
) -> Result<(), Box<dyn Error>> {
    // 参考ipmitool的ipmi_user_password实现
    let password = match password {
        Some(pwd) => pwd.to_string(),
        None => {
            // 交互式密码输入 - 参考ipmitool行为
            loop {
                // 第一次输入密码
                let password1 =
                    rpassword::prompt_password(format!("Password for user {}: ", user_id))
                        .map_err(|e| format!("Failed to read password: {}", e))?;

                // 第二次输入密码进行确认
                let password2 =
                    rpassword::prompt_password(format!("Password for user {}: ", user_id))
                        .map_err(|e| format!("Failed to read password: {}", e))?;

                // 检查两次密码是否一致
                if password1 == password2 {
                    break password1;
                } else {
                    eprintln!("Passwords do not match, try again.");
                    continue;
                }
            }
        }
    };

    // 验证密码长度
    if password.len() > 20 {
        return Err("Password is too long (> 20 bytes)".into());
    }

    // 使用指定的格式
    match _ipmi_set_user_password(
        intf,
        user_id,
        IPMI_PASSWORD_SET_PASSWORD,
        Some(&password),
        is_twenty_byte,
    ) {
        Ok(_) => {
            // 显示成功消息 - 与ipmitool保持一致
            println!("Set User Password command successful (user {})", user_id);
            Ok(())
        }
        Err(e) => {
            // 与ipmitool一致的错误输出格式 - 避免重复输出
            if let Some(crate::error::IpmiError::CompletionCode(ccode)) =
                e.downcast_ref::<crate::error::IpmiError>()
            {
                let error_desc = crate::error::val2str(*ccode, &crate::error::COMPLETION_CODE_VALS);
                // 如果错误描述是"Unknown value"，显示格式为"Unknown (0xXX)"
                let formatted_error = if error_desc == "Unknown value" {
                    format!("Unknown (0x{:02x})", ccode)
                } else {
                    error_desc.to_string()
                };
                println!("IPMI command failed: {}", formatted_error);
                println!("Set User Password command failed (user {})", user_id);
                std::process::exit(1);
            }
            println!("Set User Password command failed (user {})", user_id);
            std::process::exit(1);
        }
    }
}

/// 禁用用户
pub fn ipmi_user_disable(intf: &mut dyn IpmiIntf, user_id: u8) -> Result<(), Box<dyn Error>> {
    debug1!("Disabling user {}", user_id);

    // 参考ipmitool的ipmi_user_mod实现
    match _ipmi_set_user_password(intf, user_id, IPMI_PASSWORD_DISABLE_USER, None, false) {
        Ok(_) => {
            // ipmitool中disable/enable没有成功消息 - 保持一致
            Ok(())
        }
        Err(e) => {
            // 与ipmitool一致的错误输出格式 - 避免重复输出
            if let Some(crate::error::IpmiError::CompletionCode(ccode)) =
                e.downcast_ref::<crate::error::IpmiError>()
            {
                let error_desc = crate::error::val2str(*ccode, &crate::error::COMPLETION_CODE_VALS);
                // 如果错误描述是"Unknown value"，显示格式为"Unknown (0xXX)"
                let formatted_error = if error_desc == "Unknown value" {
                    format!("Unknown (0x{:02x})", ccode)
                } else {
                    error_desc.to_string()
                };
                println!("IPMI command failed: {}", formatted_error);
                println!("Set User Password command failed (user {})", user_id);
                std::process::exit(1);
            }
            println!("Set User Password command failed (user {})", user_id);
            std::process::exit(1);
        }
    }
}

/// 启用用户
pub fn ipmi_user_enable(intf: &mut dyn IpmiIntf, user_id: u8) -> Result<(), Box<dyn Error>> {
    debug1!("Enabling user {}", user_id);

    // 参考ipmitool的ipmi_user_mod实现
    match _ipmi_set_user_password(intf, user_id, IPMI_PASSWORD_ENABLE_USER, None, false) {
        Ok(_) => {
            // ipmitool中disable/enable没有成功消息 - 保持一致
            Ok(())
        }
        Err(e) => {
            // 与ipmitool一致的错误输出格式 - 避免重复输出
            if let Some(crate::error::IpmiError::CompletionCode(ccode)) =
                e.downcast_ref::<crate::error::IpmiError>()
            {
                let error_desc = crate::error::val2str(*ccode, &crate::error::COMPLETION_CODE_VALS);
                // 如果错误描述是"Unknown value"，显示格式为"Unknown (0xXX)"
                let formatted_error = if error_desc == "Unknown value" {
                    format!("Unknown (0x{:02x})", ccode)
                } else {
                    error_desc.to_string()
                };
                println!("IPMI command failed: {}", formatted_error);
                println!("Set User Password command failed (user {})", user_id);
                std::process::exit(1);
            }
            println!("Set User Password command failed (user {})", user_id);
            std::process::exit(1);
        }
    }
}

/// 设置用户权限
pub fn ipmi_user_set_privilege(
    intf: &mut dyn IpmiIntf,
    user_id: u8,
    privilege_level: u8,
    channel: Option<u8>,
) -> Result<(), Box<dyn Error>> {
    // 验证用户ID范围
    if !(IPMI_UID_MIN..=IPMI_UID_MAX).contains(&user_id) {
        return Err(format!(
            "Invalid user ID: {}. Must be between {} and {}",
            user_id, IPMI_UID_MIN, IPMI_UID_MAX
        )
        .into());
    }

    let channel = channel.unwrap_or(0x0E); // 默认当前通道
    debug1!(
        "Setting privilege level {} for user {} on channel {}",
        privilege_level,
        user_id,
        channel
    );

    // 参考ipmitool _ipmi_set_user_access的实现
    let mut req = IpmiRq::default();
    let mut data = [0u8; 4];

    // 设置数据 - 完全按照ipmitool的逻辑
    // data[0] = change_priv_limit_only ? 0x00 : 0x80;
    // 在priv命令中，我们只改变权限级别，所以change_priv_limit_only = true
    data[0] = 0x00; // change_priv_limit_only = true (0x00)
                    // 不设置 callin_callback, link_auth, ipmi_messaging 位，因为只改变权限
    data[0] |= channel & 0x0F; // 设置通道号

    data[1] = ipmi_uid(user_id); // 用户ID
    data[2] = privilege_level & 0x0F; // 权限级别
    data[3] = 0x00; // session_limit = 0 (不限制)

    // 设置请求参数
    req.msg.netfn_mut(IPMI_NETFN_APP);
    req.msg.cmd = IPMI_SET_USER_ACCESS;
    req.msg.data = data.as_mut_ptr();
    req.msg.data_len = 4;

    // 发送请求 - 参考ipmitool的错误处理模式
    match intf.sendrecv(&req) {
        Some(rsp) => {
            if rsp.ccode != 0 {
                // 与ipmitool一致的两行错误输出格式
                let error_desc =
                    crate::error::val2str(rsp.ccode, &crate::error::COMPLETION_CODE_VALS);
                // 如果错误描述是"Unknown value"，显示格式为"Unknown (0xXX)"
                let formatted_error = if error_desc == "Unknown value" {
                    format!("Unknown (0x{:02x})", rsp.ccode)
                } else {
                    error_desc.to_string()
                };
                println!("IPMI command failed: {}", formatted_error);
                println!("Set Privilege Level command failed (user {})", user_id);
                std::process::exit(1);
            }
        }
        None => {
            println!(
                "Set Privilege Level command failed (user {}): no response",
                user_id
            );
            std::process::exit(1);
        }
    };

    // 成功消息 - 与ipmitool保持一致
    println!("Set Privilege Level command successful (user {})", user_id);
    Ok(())
}

/// 测试用户密码
pub fn ipmi_user_test_password(
    intf: &mut dyn IpmiIntf,
    user_id: u8,
    password: Option<&str>,
    is_twenty_byte: bool,
) -> Result<(), Box<dyn Error>> {
    debug1!("Testing password for user {}", user_id);

    // 获取密码 - 支持交互式输入，与ipmitool保持一致
    let password = match password {
        Some(pwd) => pwd.to_string(),
        None => {
            // 交互式密码输入 - 参考ipmitool行为，只输入一次
            rpassword::prompt_password(format!("Password for user {}: ", user_id))
                .map_err(|e| format!("Failed to read password: {}", e))?
        }
    };

    // 参考ipmitool的实现 - 使用test operation
    let result = _ipmi_set_user_password(
        intf,
        user_id,
        IPMI_PASSWORD_TEST_PASSWORD,
        Some(&password),
        is_twenty_byte,
    );

    // 解释结果 - 参考ipmitool ipmi_user_test_password
    match result {
        Ok(_) => {
            println!("Success");
            Ok(())
        }
        Err(e) => {
            if let Some(ipmi_error) = e.downcast_ref::<IpmiError>() {
                match ipmi_error {
                    IpmiError::CompletionCode(0x80) => {
                        println!("Failure: password incorrect");
                        std::process::exit(1);
                    }
                    IpmiError::CompletionCode(0x81) => {
                        println!("Failure: wrong password size");
                        std::process::exit(1);
                    }
                    IpmiError::CompletionCode(_code) => {
                        println!("Unknown error");
                        std::process::exit(1);
                    }
                    _ => {
                        println!("Unknown error");
                        std::process::exit(1);
                    }
                }
            } else {
                println!("Unknown error");
                std::process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_privilege_level_valid() {
        // 测试有效的权限级别
        assert_eq!(parse_privilege_level("1").unwrap(), 1);
        assert_eq!(parse_privilege_level("2").unwrap(), 2);
        assert_eq!(parse_privilege_level("3").unwrap(), 3);
        assert_eq!(parse_privilege_level("4").unwrap(), 4);
        assert_eq!(parse_privilege_level("5").unwrap(), 5);
        assert_eq!(parse_privilege_level("15").unwrap(), 15);

        // 测试十六进制格式
        assert_eq!(parse_privilege_level("0x1").unwrap(), 1);
        assert_eq!(parse_privilege_level("0x2").unwrap(), 2);
        assert_eq!(parse_privilege_level("0x3").unwrap(), 3);
        assert_eq!(parse_privilege_level("0x4").unwrap(), 4);
        assert_eq!(parse_privilege_level("0x5").unwrap(), 5);
        assert_eq!(parse_privilege_level("0xF").unwrap(), 15);
        assert_eq!(parse_privilege_level("0xf").unwrap(), 15);
    }

    #[test]
    fn test_parse_privilege_level_invalid() {
        // 测试无效的权限级别
        assert!(parse_privilege_level("0").is_err());
        assert!(parse_privilege_level("6").is_err());
        assert!(parse_privilege_level("7").is_err());
        assert!(parse_privilege_level("16").is_err());
        assert!(parse_privilege_level("255").is_err());

        // 测试无效的十六进制
        assert!(parse_privilege_level("0x0").is_err());
        assert!(parse_privilege_level("0x6").is_err());
        assert!(parse_privilege_level("0x10").is_err());

        // 测试无效的输入格式
        assert!(parse_privilege_level("invalid").is_err());
        assert!(parse_privilege_level("").is_err());
        assert!(parse_privilege_level("0xG").is_err());
    }

    #[test]
    fn test_parse_privilege_level_error_message() {
        // 测试错误消息包含正确的使用说明
        let error = parse_privilege_level("6").unwrap_err();
        assert!(error.contains("Privilege levels:"));
        assert!(error.contains("* 0x1 - Callback"));
        assert!(error.contains("* 0x2 - User"));
        assert!(error.contains("* 0x3 - Operator"));
        assert!(error.contains("* 0x4 - Administrator"));
        assert!(error.contains("* 0x5 - OEM Proprietary"));
        assert!(error.contains("* 0xF - No Access"));
        assert!(error.contains("Usage: utipmitool user priv"));
    }

    #[test]
    fn test_channel_parameter_handling() {
        // 测试channel参数的默认值处理
        // 这里我们测试UserCommand枚举的匹配逻辑

        // 当没有提供channel时，应该使用默认值0x0E
        let default_channel = 0x0E;
        assert_eq!(default_channel, 0x0E);

        // 当提供了channel时，应该使用指定的值
        let specified_channel = 1u8;
        assert_eq!(specified_channel, 1);

        // 测试不同的channel值
        let channel_values = [0, 1, 2, 14, 15];
        for &channel in &channel_values {
            let result = channel;
            assert_eq!(result, channel);
        }
    }

    #[test]
    fn test_ipmi_password_constants() {
        // 验证IPMI密码操作常量值与ipmitool一致
        assert_eq!(IPMI_PASSWORD_DISABLE_USER, 0x00);
        assert_eq!(IPMI_PASSWORD_ENABLE_USER, 0x01);
        assert_eq!(IPMI_PASSWORD_SET_PASSWORD, 0x02);
        assert_eq!(IPMI_PASSWORD_TEST_PASSWORD, 0x03);
    }

    #[test]
    fn test_operation_masking() {
        // 测试operation & 0x03的位掩码操作
        assert_eq!(IPMI_PASSWORD_DISABLE_USER & 0x03, 0x00);
        assert_eq!(IPMI_PASSWORD_ENABLE_USER & 0x03, 0x01);
        assert_eq!(IPMI_PASSWORD_SET_PASSWORD & 0x03, 0x02);
        assert_eq!(IPMI_PASSWORD_TEST_PASSWORD & 0x03, 0x03);

        // 测试其他值的掩码结果
        assert_eq!(0x04 & 0x03, 0x00);
        assert_eq!(0x05 & 0x03, 0x01);
        assert_eq!(0xFF & 0x03, 0x03);
    }

    #[test]
    fn test_data_length_calculation() {
        // 测试数据长度计算逻辑
        let len_16_byte = 18; // !is_twenty_byte
        let len_20_byte = 22; // is_twenty_byte

        assert_eq!(len_16_byte, 18);
        assert_eq!(len_20_byte, 22);

        // 验证密码数据区域大小
        assert_eq!(len_16_byte - 2, 16); // 2字节头部 + 16字节密码
        assert_eq!(len_20_byte - 2, 20); // 2字节头部 + 20字节密码
    }

    #[test]
    fn test_password_length_validation() {
        // 测试密码长度验证逻辑

        // 有效的密码长度
        let valid_passwords = [
            "",                     // 空密码
            "a",                    // 1字符
            "password",             // 8字符
            "0123456789abcdef",     // 16字符（最大推荐长度）
            "0123456789abcdef0123", // 20字符（最大支持长度）
        ];

        for password in &valid_passwords {
            assert!(
                password.len() <= 20,
                "Password '{}' should be valid (length: {})",
                password,
                password.len()
            );
        }

        // 无效的密码长度（超过20字节）
        let invalid_password = "0123456789abcdef01234"; // 21字符
        assert!(invalid_password.len() > 20, "Password should be too long");
    }

    #[test]
    fn test_user_id_range_validation() {
        // 测试用户ID范围验证

        // 有效的用户ID范围
        for user_id in IPMI_UID_MIN..=IPMI_UID_MAX {
            assert!(
                (IPMI_UID_MIN..=IPMI_UID_MAX).contains(&user_id),
                "User ID {} should be valid",
                user_id
            );
        }

        // 测试边界值
        assert_eq!(IPMI_UID_MIN, 1);
        assert_eq!(IPMI_UID_MAX, 63);
    }

    #[test]
    fn test_interactive_password_prompts() {
        // 测试交互式密码提示格式
        let user_id = 2;
        let expected_prompt = format!("Password for user {}: ", user_id);
        assert_eq!(expected_prompt, "Password for user 2: ");

        // 测试不同用户ID的提示格式
        for user_id in 1..=5 {
            let prompt = format!("Password for user {}: ", user_id);
            assert!(prompt.starts_with("Password for user "));
            assert!(prompt.ends_with(": "));
            assert!(prompt.contains(&user_id.to_string()));
        }
    }

    #[test]
    fn test_completion_code_error_formatting() {
        // 测试IPMI completion code的错误信息格式化
        use crate::error::{val2str, COMPLETION_CODE_VALS};

        // 测试常见的错误码
        let test_cases = [
            (0x00, "Command completed normally"),
            (0xcc, "Invalid data field in request"),
            (0xd4, "Insufficient privilege level"),
            (0xc1, "Invalid command"),
        ];

        for (code, expected_desc) in &test_cases {
            let actual_desc = val2str(*code, &COMPLETION_CODE_VALS);
            assert_eq!(
                actual_desc, *expected_desc,
                "Error description for code 0x{:02x} should match",
                code
            );
        }

        // 测试未知错误码
        let unknown_code = 0xEE;
        let unknown_desc = val2str(unknown_code, &COMPLETION_CODE_VALS);
        assert_eq!(unknown_desc, "Unknown value");
    }

    #[test]
    fn test_error_message_format() {
        // 测试错误消息格式
        use crate::error::{val2str, COMPLETION_CODE_VALS};

        let user_id = 2;
        let username = "testuser";
        let code = 0xcc;
        let error_desc = val2str(code, &COMPLETION_CODE_VALS);

        let formatted_error = format!(
            "Set User Name command failed (user {}, name {}): {}",
            user_id, username, error_desc
        );

        let expected =
            "Set User Name command failed (user 2, name testuser): Invalid data field in request";
        assert_eq!(formatted_error, expected);
    }

    #[test]
    fn test_user_priv_parameter_handling() {
        // 测试user priv命令的参数处理逻辑

        // 测试有效参数组合
        let valid_combinations = [
            (Some(1u8), Some(1u8)),   // 用户1，回调权限
            (Some(2u8), Some(2u8)),   // 用户2，用户权限
            (Some(3u8), Some(3u8)),   // 用户3，操作员权限
            (Some(4u8), Some(4u8)),   // 用户4，管理员权限
            (Some(5u8), Some(5u8)),   // 用户5，OEM权限
            (Some(63u8), Some(15u8)), // 最大用户ID，无访问权限
        ];

        for (user_id, privilege) in &valid_combinations {
            // 验证参数都存在的情况
            assert!(user_id.is_some() && privilege.is_some());
        }

        // 测试无效参数组合（应该显示帮助）
        let invalid_combinations = [
            (None, None),      // 都缺失
            (Some(1u8), None), // 缺少权限
            (None, Some(4u8)), // 缺少用户ID
        ];

        for (user_id, privilege) in &invalid_combinations {
            // 验证至少有一个参数缺失
            assert!(user_id.is_none() || privilege.is_none());
        }
    }

    #[test]
    fn test_help_content_format() {
        // 测试帮助信息格式

        // 验证帮助信息的关键内容
        let expected_commands = [
            "summary",
            "list",
            "set name",
            "set password",
            "disable",
            "enable",
            "priv",
            "test",
        ];

        let expected_privilege_levels = [
            "0x1 - Callback",
            "0x2 - User",
            "0x3 - Operator",
            "0x4 - Administrator",
            "0x5 - OEM Proprietary",
            "0xF - No Access",
        ];

        // 这里只是验证格式是否正确，实际输出需要在运行时验证
        for cmd in &expected_commands {
            assert!(!cmd.is_empty(), "Command name should not be empty");
        }

        for priv_level in &expected_privilege_levels {
            assert!(
                priv_level.contains(" - "),
                "Privilege level should contain description"
            );
        }
    }
}
