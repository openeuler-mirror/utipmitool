/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::debug3;

/// IPMI Context Module - 重构后的上下文管理
///
/// 按照职责分离原则，将原来的IpmiContext拆分为多个专门的上下文结构

/// 输出格式上下文 - 控制命令输出的格式和详细程度
#[derive(Clone, Debug, Default)]
pub struct OutputContext {
    /// 是否使用CSV格式输出
    pub csv: bool,
    /// 详细输出级别 (0=正常, 1=详细, 2=非常详细, 3=调试级别)
    pub verbose: u8,
    /// 是否使用扩展格式输出 (显示传感器号、实体ID等额外信息)
    pub extended: bool,
}

impl OutputContext {
    /// 创建新的输出上下文
    pub fn new(csv: bool, verbose: u8) -> Self {
        Self {
            csv,
            verbose,
            extended: false,
        }
    }

    /// 创建带有扩展格式的输出上下文
    pub fn new_with_extended(csv: bool, verbose: u8, extended: bool) -> Self {
        Self {
            csv,
            verbose,
            extended,
        }
    }

    /// 创建默认输出上下文的builder方法
    pub fn builder() -> Self {
        Self::default()
    }

    /// 设置CSV格式（链式调用）
    pub fn with_csv(mut self, csv: bool) -> Self {
        self.csv = csv;
        self
    }

    /// 设置详细程度（链式调用）
    pub fn with_verbose(mut self, verbose: u8) -> Self {
        self.verbose = verbose;
        self
    }

    /// 设置扩展格式（链式调用）
    pub fn with_extended(mut self, extended: bool) -> Self {
        self.extended = extended;
        self
    }

    /// 是否启用详细输出
    pub fn is_verbose(&self) -> bool {
        self.verbose > 0
    }

    /// 是否启用非常详细输出
    pub fn is_very_verbose(&self) -> bool {
        self.verbose > 1
    }

    /// 是否启用调试级别输出
    pub fn is_debug(&self) -> bool {
        self.verbose > 2
    }

    /// 是否启用扩展格式输出
    pub fn is_extended(&self) -> bool {
        self.extended
    }

    /// 设置扩展格式输出（可变引用方式）
    pub fn set_extended(&mut self, extended: bool) {
        self.extended = extended;
    }

    /// 设置CSV格式（可变引用方式）
    pub fn set_csv(&mut self, csv: bool) {
        self.csv = csv;
    }

    /// 设置详细程度（可变引用方式）
    pub fn set_verbose(&mut self, verbose: u8) {
        self.verbose = verbose;
    }
}

/// 基础IPMI上下文 - 包含所有接口类型都需要的通用字段
#[derive(Clone, Default, Debug)]
pub struct IpmiBaseContext {
    /// 本地地址
    pub my_addr: u32,
    /// 目标地址
    pub target_addr: u32,
    /// 目标逻辑单元号
    pub target_lun: u8,
    /// 目标通道
    pub target_channel: u8,
    /// 目标IPMB地址
    pub target_ipmb_addr: u8,
}

/// 桥接上下文 - 用于多跳IPMI通信
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BridgingContext {
    /// 中转地址
    pub transit_addr: u32,
    /// 中转通道
    pub transit_channel: u8,
}

/// 协议上下文 - 包含协议相关的配置
#[derive(Clone, Debug)]
pub struct ProtocolContext {
    /// 最大请求数据大小
    pub max_request_data_size: u16,
    /// 最大响应数据大小
    pub max_response_data_size: u16,
}

impl Default for ProtocolContext {
    fn default() -> Self {
        Self {
            max_request_data_size: 25, // IPMI_DEFAULT_PAYLOAD_SIZE
            max_response_data_size: 25,
        }
    }
}

/// 完整的IPMI上下文 - 组合所有子上下文
#[derive(Clone, Default, Debug)]
pub struct IpmiContext {
    /// 基础上下文
    pub base: IpmiBaseContext,
    /// 桥接上下文 (可选，只在需要桥接时使用)
    pub bridging: Option<BridgingContext>,
    /// 协议上下文
    pub protocol: ProtocolContext,
    /// 输出上下文
    pub output: OutputContext,
}

impl IpmiContext {
    /// 创建新的上下文
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建带有指定基础配置的上下文
    pub fn with_base(base: IpmiBaseContext) -> Self {
        Self {
            base,
            bridging: None,
            protocol: ProtocolContext::default(),
            output: OutputContext::default(),
        }
    }

    /// 创建带有输出配置的上下文
    pub fn with_output(output: OutputContext) -> Self {
        Self {
            base: IpmiBaseContext::default(),
            bridging: None,
            protocol: ProtocolContext::default(),
            output,
        }
    }

    /// 创建带有完整配置的上下文
    pub fn with_config(base: IpmiBaseContext, output: OutputContext) -> Self {
        Self {
            base,
            bridging: None,
            protocol: ProtocolContext::default(),
            output,
        }
    }

    /// 启用桥接功能
    pub fn enable_bridging(&mut self, transit_addr: u32, transit_channel: u8) {
        self.bridging = Some(BridgingContext {
            transit_addr,
            transit_channel,
        });
    }

    /// 禁用桥接功能
    pub fn disable_bridging(&mut self) {
        self.bridging = None;
    }

    /// 检查是否启用了桥接
    pub fn has_bridging(&self) -> bool {
        self.bridging.is_some()
    }

    /// 获取桥接级别
    pub fn get_bridging_level(&self) -> u8 {
        if self.base.target_addr > 0 && self.base.target_addr != self.base.my_addr {
            if let Some(bridging) = &self.bridging {
                if bridging.transit_addr > 0
                    && (bridging.transit_addr != self.base.target_addr
                        || bridging.transit_channel != self.base.target_channel)
                {
                    2 // 双重桥接
                } else {
                    1 // 单重桥接
                }
            } else {
                1 // 简单桥接
            }
        } else {
            0 // 无桥接
        }
    }

    // ===========================
    // 便利方法 - 兼容旧接口
    // ===========================

    /// 设置本地地址
    pub fn set_my_addr(&mut self, addr: u32) {
        self.base.my_addr = addr;
    }

    /// 获取本地地址
    pub fn my_addr(&self) -> u32 {
        self.base.my_addr
    }

    /// 设置目标地址
    pub fn set_target_addr(&mut self, addr: u32) {
        self.base.target_addr = addr;
    }

    /// 获取目标地址
    pub fn target_addr(&self) -> u32 {
        self.base.target_addr
    }

    /// 设置目标通道
    pub fn set_target_channel(&mut self, channel: u8) {
        self.base.target_channel = channel;
    }

    /// 获取目标通道
    pub fn target_channel(&self) -> u8 {
        self.base.target_channel
    }

    /// 设置目标LUN
    pub fn set_target_lun(&mut self, lun: u8) {
        self.base.target_lun = lun;
    }

    /// 获取目标LUN
    pub fn target_lun(&self) -> u8 {
        self.base.target_lun
    }

    /// 设置目标IPMB地址
    pub fn set_target_ipmb_addr(&mut self, addr: u8) {
        self.base.target_ipmb_addr = addr;
    }

    /// 获取目标IPMB地址
    pub fn target_ipmb_addr(&self) -> u8 {
        self.base.target_ipmb_addr
    }

    /// 设置中转地址
    pub fn set_transit_addr(&mut self, addr: u32) {
        if addr > 0 {
            if let Some(ref mut bridging) = self.bridging {
                bridging.transit_addr = addr;
            } else {
                self.bridging = Some(BridgingContext {
                    transit_addr: addr,
                    transit_channel: 0,
                });
            }
        } else if let Some(ref mut bridging) = self.bridging {
            bridging.transit_addr = 0;
            // 如果中转地址和通道都为0，则禁用桥接
            if bridging.transit_channel == 0 {
                self.bridging = None;
            }
        }
    }

    /// 获取中转地址
    pub fn transit_addr(&self) -> u32 {
        self.bridging.as_ref().map_or(0, |b| b.transit_addr)
    }

    /// 设置中转通道
    pub fn set_transit_channel(&mut self, channel: u8) {
        if let Some(ref mut bridging) = self.bridging {
            bridging.transit_channel = channel;
        } else if channel > 0 {
            self.bridging = Some(BridgingContext {
                transit_addr: 0,
                transit_channel: channel,
            });
        }
    }

    /// 获取中转通道
    pub fn transit_channel(&self) -> u8 {
        self.bridging.as_ref().map_or(0, |b| b.transit_channel)
    }

    // ===========================
    // 协议相关方法
    // ===========================

    /// 设置最大请求数据大小
    pub fn set_max_request_data_size(&mut self, size: u16) {
        const IPMI_DEFAULT_PAYLOAD_SIZE: u16 = 25;
        if size < IPMI_DEFAULT_PAYLOAD_SIZE {
            log::warn!(
                "Request size {} is too small, minimum is {}",
                size,
                IPMI_DEFAULT_PAYLOAD_SIZE
            );
            return;
        }
        self.protocol.max_request_data_size = size;
    }

    /// 设置最大响应数据大小
    pub fn set_max_response_data_size(&mut self, size: u16) {
        const IPMI_DEFAULT_PAYLOAD_SIZE: u16 = 25;
        if size < IPMI_DEFAULT_PAYLOAD_SIZE - 1 {
            log::warn!(
                "Response size {} is too small, minimum is {}",
                size,
                IPMI_DEFAULT_PAYLOAD_SIZE - 1
            );
            return;
        }
        self.protocol.max_response_data_size = size;
    }

    /// 获取有效的最大请求数据大小（考虑桥接开销）
    pub fn get_max_request_data_size(&self) -> u16 {
        const IPMI_DEFAULT_PAYLOAD_SIZE: u16 = 25;
        let mut size = self.protocol.max_request_data_size as i16;
        let bridging_level = self.get_bridging_level();

        // 如果请求大小未指定，使用默认值
        if size == 0 {
            size = IPMI_DEFAULT_PAYLOAD_SIZE as i16;

            // 如果有桥接，增加Send Message请求大小
            if bridging_level > 0 {
                size += 8;
            }
        }

        // 如果有桥接，减去Send Message请求大小
        if bridging_level > 0 {
            size -= 8;

            // 确保转发的请求大小不超过默认负载大小
            if size > IPMI_DEFAULT_PAYLOAD_SIZE as i16 {
                size = IPMI_DEFAULT_PAYLOAD_SIZE as i16;
            }

            // 检查双重桥接
            if bridging_level == 2 {
                size -= 8; // 减去内部Send Message请求大小
            }
        }

        // 检查下溢
        if size < 0 {
            return 0;
        }

        size as u16
    }

    /// 获取有效的最大响应数据大小（考虑桥接开销）
    pub fn get_max_response_data_size(&self) -> u16 {
        const IPMI_DEFAULT_PAYLOAD_SIZE: u16 = 25;
        let mut size = self.protocol.max_response_data_size;
        let bridging_level = self.get_bridging_level();

        // 如果响应大小未指定，使用默认值
        if size == 0 {
            size = IPMI_DEFAULT_PAYLOAD_SIZE;

            // 如果有桥接，增加Send Message头大小
            if bridging_level > 0 {
                size = size.saturating_add(8);
            }
        }

        // 如果有桥接，减去内部消息头大小
        if bridging_level > 0 {
            size = size.saturating_sub(8);

            // 确保转发的响应不超过默认负载大小
            if size > IPMI_DEFAULT_PAYLOAD_SIZE {
                size = IPMI_DEFAULT_PAYLOAD_SIZE;
            }

            // 检查双重桥接
            if bridging_level == 2 {
                size = size.saturating_sub(8); // 减去内部Send Message头大小
            }
        }

        size
    }

    /// 检查是否需要通过传感器桥接到目标
    pub fn bridge_to_sensor(&self, addr: u8, chan: u8) -> bool {
        debug3!(
            "intf target_ipmb_addr {:#x} intf target_addr{:#x} intf target_channel {:#x}",
            self.base.target_ipmb_addr,
            self.base.target_addr,
            self.base.target_channel
        );
        !((chan == 0 && self.base.target_ipmb_addr != 0 && self.base.target_ipmb_addr == addr)
            || (addr == self.base.target_addr as u8 && chan == self.base.target_channel))
    }

    // ===========================
    // 输出上下文相关方法
    // ===========================

    /// 设置输出配置
    pub fn set_output_config(&mut self, csv: bool, verbose: u8) {
        self.output = OutputContext::new(csv, verbose);
    }

    /// 设置输出配置（包含扩展格式）
    pub fn set_output_config_extended(&mut self, csv: bool, verbose: u8, extended: bool) {
        self.output = OutputContext::new_with_extended(csv, verbose, extended);
    }

    /// 获取输出配置的引用
    pub fn output_config(&self) -> &OutputContext {
        &self.output
    }

    /// 获取输出配置的可变引用
    pub fn output_config_mut(&mut self) -> &mut OutputContext {
        &mut self.output
    }

    /// 是否使用CSV格式
    pub fn is_csv_output(&self) -> bool {
        self.output.csv
    }

    /// 设置CSV输出格式
    pub fn set_csv_output(&mut self, csv: bool) {
        self.output.csv = csv;
    }

    /// 获取详细输出级别
    pub fn verbose_level(&self) -> u8 {
        self.output.verbose
    }

    /// 设置详细输出级别
    pub fn set_verbose_level(&mut self, level: u8) {
        self.output.verbose = level;
    }

    /// 是否启用详细输出
    pub fn is_verbose(&self) -> bool {
        self.output.is_verbose()
    }

    /// 是否启用非常详细输出
    pub fn is_very_verbose(&self) -> bool {
        self.output.is_very_verbose()
    }

    /// 是否启用调试级别输出
    pub fn is_debug(&self) -> bool {
        self.output.is_debug()
    }

    /// 是否启用扩展格式输出
    pub fn is_extended(&self) -> bool {
        self.output.is_extended()
    }

    /// 设置扩展格式输出
    pub fn set_extended(&mut self, extended: bool) {
        self.output.set_extended(extended);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = IpmiContext::new();
        assert_eq!(ctx.base.my_addr, 0);
        assert_eq!(ctx.bridging, None);
        assert_eq!(ctx.protocol.max_request_data_size, 25);
    }

    #[test]
    fn test_bridging_management() {
        let mut ctx = IpmiContext::new();
        assert!(!ctx.has_bridging());
        assert_eq!(ctx.get_bridging_level(), 0);

        ctx.enable_bridging(0x20, 1);
        assert!(ctx.has_bridging());
        assert_eq!(ctx.transit_addr(), 0x20);
        assert_eq!(ctx.transit_channel(), 1);

        ctx.disable_bridging();
        assert!(!ctx.has_bridging());
    }

    #[test]
    fn test_bridging_levels() {
        let mut ctx = IpmiContext::new();
        ctx.set_my_addr(0x81);
        ctx.set_target_addr(0x20);

        // 简单桥接
        assert_eq!(ctx.get_bridging_level(), 1);

        // 双重桥接
        ctx.enable_bridging(0x22, 2);
        assert_eq!(ctx.get_bridging_level(), 2);

        // 无桥接
        ctx.set_target_addr(0x81);
        assert_eq!(ctx.get_bridging_level(), 0);
    }

    #[test]
    fn test_data_size_calculation() {
        let mut ctx = IpmiContext::new();
        ctx.set_max_request_data_size(50);
        ctx.set_max_response_data_size(50);

        // 无桥接
        assert_eq!(ctx.get_max_request_data_size(), 50);
        assert_eq!(ctx.get_max_response_data_size(), 50);

        // 单重桥接
        ctx.set_my_addr(0x81);
        ctx.set_target_addr(0x20);
        assert_eq!(ctx.get_max_request_data_size(), 42); // 50 - 8
        assert_eq!(ctx.get_max_response_data_size(), 42); // 50 - 8

        // 双重桥接
        ctx.enable_bridging(0x22, 2);
        assert_eq!(ctx.get_max_request_data_size(), 25); // 限制为默认大小
        assert_eq!(ctx.get_max_response_data_size(), 25); // 限制为默认大小
    }

    #[test]
    fn test_output_context() {
        let mut ctx = IpmiContext::new();

        // 默认值测试
        assert!(!ctx.is_csv_output());
        assert_eq!(ctx.verbose_level(), 0);
        assert!(!ctx.is_verbose());
        assert!(!ctx.is_very_verbose());
        assert!(!ctx.is_debug());

        // 设置CSV输出
        ctx.set_csv_output(true);
        assert!(ctx.is_csv_output());

        // 设置详细输出级别
        ctx.set_verbose_level(1);
        assert!(ctx.is_verbose());
        assert!(!ctx.is_very_verbose());
        assert!(!ctx.is_debug());

        ctx.set_verbose_level(2);
        assert!(ctx.is_verbose());
        assert!(ctx.is_very_verbose());
        assert!(!ctx.is_debug());

        ctx.set_verbose_level(3);
        assert!(ctx.is_verbose());
        assert!(ctx.is_very_verbose());
        assert!(ctx.is_debug());

        // 批量设置
        ctx.set_output_config(false, 1);
        assert!(!ctx.is_csv_output());
        assert_eq!(ctx.verbose_level(), 1);

        // 使用构造函数
        let output = OutputContext::new(true, 2);
        let ctx_with_output = IpmiContext::with_output(output);
        assert!(ctx_with_output.is_csv_output());
        assert_eq!(ctx_with_output.verbose_level(), 2);
    }

    #[test]
    fn test_output_context_builder() {
        // 测试builder模式
        let ctx = OutputContext::default()
            .with_csv(true)
            .with_verbose(2)
            .with_extended(true);

        assert!(ctx.csv);
        assert_eq!(ctx.verbose, 2);
        assert!(ctx.extended);
    }

    #[test]
    fn test_output_context_single_setting() {
        // 测试单独设置
        let ctx = OutputContext::default().with_extended(true);

        assert!(!ctx.csv);
        assert_eq!(ctx.verbose, 0);
        assert!(ctx.extended);
    }

    #[test]
    fn test_output_context_mutable_setting() {
        // 测试可变引用方式
        let mut ctx = OutputContext::default();
        ctx.set_extended(true);
        ctx.set_verbose(2);
        ctx.set_csv(true);

        assert!(ctx.csv);
        assert_eq!(ctx.verbose, 2);
        assert!(ctx.extended);
    }

    #[test]
    fn test_output_context_convenience_methods() {
        let ctx = OutputContext::default().with_verbose(3);

        assert!(ctx.is_verbose());
        assert!(ctx.is_very_verbose());
        assert!(ctx.is_debug());

        let ctx2 = OutputContext::default().with_verbose(1);
        assert!(ctx2.is_verbose());
        assert!(!ctx2.is_very_verbose());
        assert!(!ctx2.is_debug());
    }
}
