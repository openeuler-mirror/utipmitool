/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

pub const CHASSIS_BOOT_MBOX_IANA_SZ: usize = 3;
pub const CHASSIS_BOOT_MBOX_BLOCK_SZ: usize = 16;
pub const CHASSIS_BOOT_MBOX_BLOCK0_SZ: usize = CHASSIS_BOOT_MBOX_BLOCK_SZ - CHASSIS_BOOT_MBOX_IANA_SZ;
pub const CHASSIS_BOOT_MBOX_MAX_BLOCK: u8 = 0xFF;
pub const CHASSIS_BOOT_MBOX_MAX_BLOCKS: usize = (CHASSIS_BOOT_MBOX_MAX_BLOCK as usize) + 1;

/* Get/Set system boot option boot flags bit definitions */
/* Boot flags byte 1 bits */
pub const BF1_VALID_SHIFT: u8 = 7;
pub const BF1_INVALID: u8 = 0;
pub const BF1_VALID: u8 = 1 << BF1_VALID_SHIFT;
pub const BF1_VALID_MASK: u8 = BF1_VALID;

pub const BF1_PERSIST_SHIFT: u8 = 6;
pub const BF1_ONCE: u8 = 0;
pub const BF1_PERSIST: u8 = 1 << BF1_PERSIST_SHIFT;
pub const BF1_PERSIST_MASK: u8 = BF1_PERSIST;

pub const BF1_BOOT_TYPE_SHIFT: u8 = 5;
pub const BF1_BOOT_TYPE_LEGACY: u8 = 0;
pub const BF1_BOOT_TYPE_EFI: u8 = 1 << BF1_BOOT_TYPE_SHIFT;
pub const BF1_BOOT_TYPE_MASK: u8 = BF1_BOOT_TYPE_EFI;

/* Boot flags byte 2 bits */
pub const BF2_CMOS_CLEAR_SHIFT: u8 = 7;
pub const BF2_CMOS_CLEAR: u8 = 1 << BF2_CMOS_CLEAR_SHIFT;
pub const BF2_CMOS_CLEAR_MASK: u8 = BF2_CMOS_CLEAR;

pub const BF2_KEYLOCK_SHIFT: u8 = 6;
pub const BF2_KEYLOCK: u8 = 1 << BF2_KEYLOCK_SHIFT;
pub const BF2_KEYLOCK_MASK: u8 = BF2_KEYLOCK;

pub const BF2_BOOTDEV_SHIFT: u8 = 2;
pub const BF2_BOOTDEV_DEFAULT: u8 = 0 << BF2_BOOTDEV_SHIFT;
pub const BF2_BOOTDEV_PXE: u8 = 1 << BF2_BOOTDEV_SHIFT;
pub const BF2_BOOTDEV_HDD: u8 = 2 << BF2_BOOTDEV_SHIFT;
pub const BF2_BOOTDEV_HDD_SAFE: u8 = 3 << BF2_BOOTDEV_SHIFT;
pub const BF2_BOOTDEV_DIAG_PART: u8 = 4 << BF2_BOOTDEV_SHIFT;
pub const BF2_BOOTDEV_CDROM: u8 = 5 << BF2_BOOTDEV_SHIFT;
pub const BF2_BOOTDEV_SETUP: u8 = 6 << BF2_BOOTDEV_SHIFT;
pub const BF2_BOOTDEV_REMOTE_FDD: u8 = 7 << BF2_BOOTDEV_SHIFT;
pub const BF2_BOOTDEV_REMOTE_CDROM: u8 = 8 << BF2_BOOTDEV_SHIFT;
pub const BF2_BOOTDEV_REMOTE_PRIMARY_MEDIA: u8 = 9 << BF2_BOOTDEV_SHIFT;
pub const BF2_BOOTDEV_REMOTE_HDD: u8 = 11 << BF2_BOOTDEV_SHIFT;
pub const BF2_BOOTDEV_FDD: u8 = 15 << BF2_BOOTDEV_SHIFT;
pub const BF2_BOOTDEV_MASK: u8 = 0xF << BF2_BOOTDEV_SHIFT;

pub const BF2_BLANK_SCREEN_SHIFT: u8 = 1;
pub const BF2_BLANK_SCREEN: u8 = 1 << BF2_BLANK_SCREEN_SHIFT;
pub const BF2_BLANK_SCREEN_MASK: u8 = BF2_BLANK_SCREEN;

pub const BF2_RESET_LOCKOUT_SHIFT: u8 = 0;
pub const BF2_RESET_LOCKOUT: u8 = 1 << BF2_RESET_LOCKOUT_SHIFT;
pub const BF2_RESET_LOCKOUT_MASK: u8 = BF2_RESET_LOCKOUT;

/* Boot flags byte 3 bits */
pub const BF3_POWER_LOCKOUT_SHIFT: u8 = 7;
pub const BF3_POWER_LOCKOUT: u8 = 1 << BF3_POWER_LOCKOUT_SHIFT;
pub const BF3_POWER_LOCKOUT_MASK: u8 = BF3_POWER_LOCKOUT;

pub const BF3_VERBOSITY_SHIFT: u8 = 5;
pub const BF3_VERBOSITY_DEFAULT: u8 = 0 << BF3_VERBOSITY_SHIFT;
pub const BF3_VERBOSITY_QUIET: u8 = 1 << BF3_VERBOSITY_SHIFT;
pub const BF3_VERBOSITY_VERBOSE: u8 = 2 << BF3_VERBOSITY_SHIFT;
pub const BF3_VERBOSITY_MASK: u8 = 3 << BF3_VERBOSITY_SHIFT;

pub const BF3_EVENT_TRAPS_SHIFT: u8 = 4;
pub const BF3_EVENT_TRAPS: u8 = 1 << BF3_EVENT_TRAPS_SHIFT;
pub const BF3_EVENT_TRAPS_MASK: u8 = BF3_EVENT_TRAPS;

pub const BF3_PASSWD_BYPASS_SHIFT: u8 = 3;
pub const BF3_PASSWD_BYPASS: u8 = 1 << BF3_PASSWD_BYPASS_SHIFT;
pub const BF3_PASSWD_BYPASS_MASK: u8 = BF3_PASSWD_BYPASS;

pub const BF3_SLEEP_LOCKOUT_SHIFT: u8 = 2;
pub const BF3_SLEEP_LOCKOUT: u8 = 1 << BF3_SLEEP_LOCKOUT_SHIFT;
pub const BF3_SLEEP_LOCKOUT_MASK: u8 = BF3_SLEEP_LOCKOUT;

pub const BF3_CONSOLE_REDIR_SHIFT: u8 = 0;
pub const BF3_CONSOLE_REDIR_DEFAULT: u8 = 0 << BF3_CONSOLE_REDIR_SHIFT;
pub const BF3_CONSOLE_REDIR_SUPPRESS: u8 = 1 << BF3_CONSOLE_REDIR_SHIFT;
pub const BF3_CONSOLE_REDIR_ENABLE: u8 = 2 << BF3_CONSOLE_REDIR_SHIFT;
pub const BF3_CONSOLE_REDIR_MASK: u8 = 3 << BF3_CONSOLE_REDIR_SHIFT;

/* Boot flags byte 4 bits */
pub const BF4_SHARED_MODE_SHIFT: u8 = 3;
pub const BF4_SHARED_MODE: u8 = 1 << BF4_SHARED_MODE_SHIFT;
pub const BF4_SHARED_MODE_MASK: u8 = BF4_SHARED_MODE;

pub const BF4_BIOS_MUX_SHIFT: u8 = 0;
pub const BF4_BIOS_MUX_DEFAULT: u8 = 0 << BF4_BIOS_MUX_SHIFT;
pub const BF4_BIOS_MUX_BMC: u8 = 1 << BF4_BIOS_MUX_SHIFT;
pub const BF4_BIOS_MUX_SYSTEM: u8 = 2 << BF4_BIOS_MUX_SHIFT;
pub const BF4_BIOS_MUX_MASK: u8 = 7 << BF4_BIOS_MUX_SHIFT;

#[repr(C)]
pub struct MboxB0Data {
    pub iana: [u8; CHASSIS_BOOT_MBOX_IANA_SZ],
    pub data: [u8; CHASSIS_BOOT_MBOX_BLOCK0_SZ],
}
// 使用枚举替代联合体
#[warn(non_camel_case_types)]
pub enum MboxData {
    data([u8; CHASSIS_BOOT_MBOX_BLOCK_SZ]),
    b0(MboxB0Data),
}

#[repr(C)]
pub struct Mbox {
    pub block: u8,
    pub data: MboxData,
}

//pub static mut VERBOSE: i32 = 0;

pub struct ValStr {
    pub value: u8,
    pub description: Option<&'static str>,
}

pub const GET_BOOTPARAM_CC_VALS: &[ValStr] = &[
    ValStr { value: 0x80, description: Some("Unsupported parameter") },
    ValStr { value: 0x00, description: None },
];

pub const SET_BOOTPARAM_CC_VALS: &[ValStr] = &[
    ValStr { value: 0x80, description: Some("Unsupported parameter") },
    ValStr { value: 0x81, description: Some("Attempt to set 'in progress' while not in 'complete' state") },
    ValStr { value: 0x82, description: Some("Parameter is read-only") },
    ValStr { value: 0x00, description: None },
];


