/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use ipmi_macros::AsBytes;

pub const BMC_GET_DEVICE_ID: u8 = 0x01;
pub const BMC_COLD_RESET: u8 = 0x02;
pub const BMC_WARM_RESET: u8 = 0x03;
pub const BMC_GET_SELF_TEST: u8 = 0x04;
pub const BMC_RESET_WATCHDOG_TIMER: u8 = 0x22;
pub const BMC_SET_WATCHDOG_TIMER: u8 = 0x24;
pub const BMC_GET_WATCHDOG_TIMER: u8 = 0x25;
pub const BMC_SET_GLOBAL_ENABLES: u8 = 0x2e;
pub const BMC_GET_GLOBAL_ENABLES: u8 = 0x2f;
pub const BMC_GET_GUID: u8 = 0x37;

/// ipmi_mc.h
#[repr(C)]
#[derive(Debug, Default, AsBytes)]
pub struct IpmDevidRsp {
    pub device_id: u8,
    pub device_revision: u8,
    pub fw_rev1: u8,
    pub fw_rev2: u8,
    pub ipmi_version: u8,
    pub adtl_device_support: u8,
    pub manufacturer_id: [u8; 3],
    pub product_id: [u8; 2],
    pub aux_fw_rev: [u8; 4],
}
