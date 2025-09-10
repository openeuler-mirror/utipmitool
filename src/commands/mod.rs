/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::error::IpmiError;
use crate::ipmi::intf::IpmiIntf;

// 统一的命令结果类型 - 解决返回类型混乱问题
pub type CommandResult<T = ()> = Result<T, IpmiError>;

// 统一的命令处理接口
pub trait IpmiCommandHandler {
    /// 执行IPMI命令
    fn execute(&mut self, intf: &mut dyn IpmiIntf) -> CommandResult;

    /// 获取命令描述（用于日志和错误报告）
    fn description(&self) -> &'static str;
}

// 命令工厂trait - 用于创建命令处理器
pub trait CommandFactory<T> {
    type Handler: IpmiCommandHandler;

    /// 从命令参数创建处理器
    fn create_handler(params: T) -> Self::Handler;
}

// 便利宏 - 简化错误转换
#[macro_export]
macro_rules! command_error {
    ($msg:expr) => {
        Err(IpmiError::Interface($msg.to_string()))
    };
    ($fmt:expr, $($arg:tt)*) => {
        Err(IpmiError::Interface(format!($fmt, $($arg)*)))
    };
}

// 子模块声明
pub mod bootdev;
pub mod bootparam;
pub mod chassis;
pub mod identify;
pub mod lan;
pub mod mc;
pub mod poh;
pub mod restart_cause;
pub mod sdr;
pub mod sel;
pub mod selftest;
pub mod sensor;
pub mod user;
