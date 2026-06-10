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
