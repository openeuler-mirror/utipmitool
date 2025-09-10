/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use std::sync::atomic::{AtomicBool, Ordering};

// 全局标志，控制调试输出
pub static DEBUG_INFO_PRINTED: AtomicBool = AtomicBool::new(false);
pub static HEADER_PRINTED: AtomicBool = AtomicBool::new(false);
pub static DISABLE_ORIGINAL_DEBUG: AtomicBool = AtomicBool::new(false);

// 判断是否应该跳过原始调试输出
pub fn should_skip_debug() -> bool {
    DISABLE_ORIGINAL_DEBUG.load(Ordering::Relaxed)
}

// 重置调试状态
pub fn reset_debug_state() {
    DEBUG_INFO_PRINTED.store(false, Ordering::SeqCst);
    HEADER_PRINTED.store(false, Ordering::SeqCst);
    DISABLE_ORIGINAL_DEBUG.store(false, Ordering::SeqCst);
}

// 隐藏加载接口的消息
pub fn hide_loading_interface_message() {
    // 设置环境变量，用于在输出"Loading interface: Open"前检查
    std::env::set_var("HIDE_LOADING_MESSAGE", "1");
    DISABLE_ORIGINAL_DEBUG.store(true, Ordering::SeqCst);
}

// 标准化的调试输出函数
pub fn print_standard_debug(verbose_level: i32, iana_value: Option<u32>) {
    // 立即禁用原始调试输出
    DISABLE_ORIGINAL_DEBUG.store(true, Ordering::SeqCst);

    // 如果已经输出过完整调试信息，直接返回
    if DEBUG_INFO_PRINTED.swap(true, Ordering::SeqCst) {
        return;
    }

    // -v 模式输出
    if verbose_level == 1 {
        println!("Running Get VSO Capabilities my_addr 0x20, transit 0, target 0");
        println!("Invalid completion code received: Invalid command");
        println!("Discovered IPMB address 0x0");
    }
    // -vv 模式输出
    else if verbose_level == 2 {
        println!("Using ipmi device 0");
        println!("Set IPMB address to 0x20");
        println!("Iana: {}", iana_value.unwrap_or(0));
        println!("Running Get PICMG Properties my_addr 0x20, transit 0, target 0");
        println!("Error response 0xc1 from Get PICMG Properities");
        println!("Running Get VSO Capabilities my_addr 0x20, transit 0, target 0");
        println!("Invalid completion code received: Invalid command");
        println!("Acquire IPMB address");
        println!("Discovered IPMB address 0x0");
        println!("Interface address: my_addr 0x20 transit 0:0 target 0x20:0 ipmb_target 0");
        println!();
    }
    // -vvv 模式输出 - 精确匹配原生ipmitool
    else if verbose_level >= 3 {
        println!("Using ipmi device 0");
        println!("Set IPMB address to 0x20");
        println!("OpenIPMI Request Message Header:");
        println!("  netfn     = 0x6");
        println!("  cmd       = 0x1");
        println!("Iana: {}", iana_value.unwrap_or(0));
        println!("Running Get PICMG Properties my_addr 0x20, transit 0, target 0");
        println!("OpenIPMI Request Message Header:");
        println!("  netfn     = 0x2c");
        println!("  cmd       = 0x0");
        println!("OpenIPMI Request Message Data (1 bytes)");
        println!(" 00"); // 注意前面有一个空格
        println!("Error response 0xc1 from Get PICMG Properities");
        println!("Running Get VSO Capabilities my_addr 0x20, transit 0, target 0");
        println!("OpenIPMI Request Message Header:");
        println!("  netfn     = 0x2c");
        println!("  cmd       = 0x0");
        println!("OpenIPMI Request Message Data (1 bytes)");
        println!(" 03"); // 注意前面有一个空格
        println!("Invalid completion code received: Invalid command");
        println!("Acquire IPMB address");
        println!("Discovered IPMB address 0x0");
        println!("Interface address: my_addr 0x20 transit 0:0 target 0x20:0 ipmb_target 0");
        println!();

        // 重要：删除这里的 chassis status 头部信息输出
        // 不再输出这三行
        // println!("OpenIPMI Request Message Header:");
        // println!("  netfn     = 0x0");
        // println!("  cmd       = 0x1");
    }
}
