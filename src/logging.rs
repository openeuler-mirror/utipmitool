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
//use log::{debug, info, trace, warn, error};
//pub use log::{debug, info, trace, warn, error};

use env_logger;
use env_logger::Env;
use log;
use std::env;
use std::io::Write;

/// 日志颜色配置
struct LogColors {
    error: &'static str,
    warn: &'static str,
    info: &'static str,
    debug: &'static str,
    trace: &'static str,
    reset: &'static str,
}

impl LogColors {
    fn new(enable_color: bool) -> Self {
        if enable_color {
            Self {
                error: "\x1b[31m", // 红色
                warn: "\x1b[33m",  // 黄色
                info: "\x1b[32m",  // 绿色
                debug: "\x1b[36m", // 青色
                trace: "\x1b[35m", // 紫色
                reset: "\x1b[0m",  // 重置
            }
        } else {
            Self {
                error: "",
                warn: "",
                info: "",
                debug: "",
                trace: "",
                reset: "",
            }
        }
    }
}

/// 设置日志系统
///
/// # 参数
/// - `verbose`: 详细级别 (0-5)
///   - 0: 只显示 ERROR, WARN, INFO
///   - 1: + debug1 (-v)
///   - 2: + debug2 (-vv)
///   - 3: + debug3 (-vvv)
///   - 4: + debug4 (-vvvv)
///   - 5: + debug5 (-vvvvv)
pub fn setup_logger(verbose: u8) {
    // 检查是否支持颜色输出 (简单检测方法)
    let enable_color =
        env::var("NO_COLOR").is_err() && env::var("TERM").map_or(false, |term| term != "dumb");

    // 构建日志配置 - 默认只显示ERROR和WARN，INFO只在verbose模式下显示
    let mut log_config = vec!["error".to_string(), "warn".to_string()];

    // 只有在verbose模式下才添加info级别
    if verbose > 0 {
        log_config.push("info".to_string());
    }

    // 根据verbose级别添加debug targets
    for level in 1..=verbose.min(5) {
        let target = format!("debug{}", level);
        let log_level = if level <= 4 { "debug" } else { "trace" };
        log_config.push(format!("{}={}", target, log_level));
    }

    // 只在未设置RUST_LOG时设置环境变量
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", log_config.join(","));
    }

    let colors = LogColors::new(enable_color);

    // 初始化日志系统
    env_logger::Builder::from_env(Env::default().filter("RUST_LOG"))
        .format(move |buf, record| {
            let level_color = match record.level() {
                log::Level::Error => colors.error,
                log::Level::Warn => colors.warn,
                log::Level::Info => colors.info,
                log::Level::Debug => colors.debug,
                log::Level::Trace => colors.trace,
            };

            // 根据target判断输出格式 - 匹配ipmitool风格
            match record.target() {
                "debug1" | "debug2" | "debug3" | "debug4" | "debug5" => {
                    // ipmitool风格：直接输出消息，无前缀
                    writeln!(buf, "{}", record.args())
                }
                _ => {
                    // 普通日志保持原有格式
                    let level_text = match record.level() {
                        log::Level::Error => "ERROR",
                        log::Level::Warn => "WARN ",
                        log::Level::Info => "INFO ",
                        log::Level::Debug => "DEBUG",
                        log::Level::Trace => "TRACE",
                    };

                    writeln!(
                        buf,
                        "{}[{}]{} {}",
                        level_color,
                        level_text,
                        colors.reset,
                        record.args()
                    )
                }
            }
        })
        .init();
}

/// 辅助函数：检查指定调试级别是否启用
pub fn is_debug_enabled(level: u8) -> bool {
    match level {
        1 => log::log_enabled!(target: "debug1", log::Level::Debug),
        2 => log::log_enabled!(target: "debug2", log::Level::Debug),
        3 => log::log_enabled!(target: "debug3", log::Level::Debug),
        4 => log::log_enabled!(target: "debug4", log::Level::Debug),
        5 => log::log_enabled!(target: "debug5", log::Level::Trace),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{debug1, debug2, debug3};

    #[test]
    fn test_setup_logger() {
        setup_logger(2);
        debug1!("This is debug1 message");
        debug2!("This is debug2 message");
        debug3!("This should not appear");
    }
}
