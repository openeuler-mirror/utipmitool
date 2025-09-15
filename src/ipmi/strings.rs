/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(dead_code)]
#![allow(unexpected_cfgs)]

use crate::ipmi::constants::*;
pub struct ValStr {
    pub val: u32,
    pub desc: &'static str,
}

pub struct U8Str {
    pub val: u8,
    pub desc: &'static str,
}

pub struct OemValStr {
    pub oem: u32,
    pub code: u8,
    pub desc: &'static str,
}

pub const IPMI_NETFN_CHASSIS: u32 = 0x0;
pub const IPMI_NETFN_BRIDGE: u32 = 0x2;
pub const IPMI_NETFN_SE: u32 = 0x4;
pub const IPMI_NETFN_APP: u32 = 0x6;
pub const IPMI_NETFN_FIRMWARE: u32 = 0x8;
pub const IPMI_NETFN_STORAGE: u32 = 0xa;
pub const IPMI_NETFN_TRANSPORT: u32 = 0xc;
pub const IPMI_NETFN_PICMG: u32 = 0x2C;
pub const IPMI_NETFN_DCGRP: u32 = 0x2C;
pub const IPMI_NETFN_OEM: u32 = 0x2E;
pub const IPMI_NETFN_ISOL: u32 = 0x34;
pub const IPMI_NETFN_TSOL: u32 = 0x30;

pub const IPMI_BMC_SLAVE_ADDR: u32 = 0x20;
pub const IPMI_REMOTE_SWID: u32 = 0x81;

const IPMI_OEM_UNKNOWN: u32 = 0;
const IPMI_OEM_DEBUG: u32 = 0xFFFFFE; /* Hoping IANA won't hit this soon */
const IPMI_OEM_RESERVED: u32 = 0x0FFFFF; /* As per IPMI 2.0 specification */
const IPMI_OEM_KONTRON: u32 = 15000;
const IPMI_OEM_PICMG: u32 = 12634;
const IPMI_OEM_VITA: u32 = 33196;

const IPMI_OEM_INFO_HEAD: &[ValStr] = &[
    ValStr {
        val: IPMI_OEM_UNKNOWN,
        desc: "Unknown",
    }, /* IPMI Unknown */
    ValStr {
        val: IPMI_OEM_RESERVED,
        desc: "Unspecified",
    }, /* IPMI Reserved */
    ValStr {
        val: u32::MAX,
        desc: "",
    },
];

/*
 * These are our own technical values. We don't want them to take precedence
 * over IANA's defined values, so they go at the very end of the array.
 */
const IPMI_OEM_INFO_TAIL: &[ValStr] = &[
    ValStr {
        val: IPMI_OEM_DEBUG,
        desc: "A Debug Assisting Company, Ltd.",
    },
    ValStr {
        val: u32::MAX,
        desc: "",
    },
];

/*
 * This is used when ipmi_oem_info couldn't be allocated.
 * ipmitool would report all OEMs as unknown, but would be functional otherwise.
 */
const IPMI_OEM_INFO_DUMMY: &[ValStr] = &[ValStr {
    val: u32::MAX,
    desc: "",
}];

/* This will point to an array filled from IANA's enterprise numbers registry */
static mut IPMI_OEM_INFO: *mut ValStr = std::ptr::null_mut();

/* Single-linked list of OEM valstrs */
pub struct OemValstrNode {
    valdesc: ValStr,
    next: Option<Box<OemValstrNode>>,
}

pub const IPMI_GENERIC_SENSOR_TYPE_VALS: &[&str] = &[
    "reserved",
    "Temperature",
    "Voltage",
    "Current",
    "Fan",
    "Physical Security",
    "Platform Security",
    "Processor",
    "Power Supply",
    "Power Unit",
    "Cooling Device",
    "Other",
    "Memory",
    "Drive Slot / Bay",
    "POST Memory Resize",
    "System Firmwares",
    "Event Logging Disabled",
    "Watchdog1",
    "System Event",
    "Critical Interrupt",
    "Button",
    "Module / Board",
    "Microcontroller",
    "Add-in Card",
    "Chassis",
    "Chip Set",
    "Other FRU",
    "Cable / Interconnect",
    "Terminator",
    "System Boot Initiated",
    "Boot Error",
    "OS Boot",
    "OS Critical Stop",
    "Slot / Connector",
    "System ACPI Power State",
    "Watchdog2",
    "Platform Alert",
    "Entity Presence",
    "Monitor ASIC",
    "LAN",
    "Management Subsys Health",
    "Battery",
    "Session Audit",
    "Version Change",
    "FRU State",
];

pub const IPMI_OEM_SENSOR_TYPE_VALS: &[OemValStr] = &[
    // Keep OEM grouped together
    OemValStr {
        oem: IPMI_OEM_KONTRON,
        code: 0xC0,
        desc: "Firmware Info",
    },
    OemValStr {
        oem: IPMI_OEM_KONTRON,
        code: 0xC2,
        desc: "Init Agent",
    },
    OemValStr {
        oem: IPMI_OEM_KONTRON,
        code: 0xC2,
        desc: "Board Reset(cPCI)",
    },
    OemValStr {
        oem: IPMI_OEM_KONTRON,
        code: 0xC3,
        desc: "IPMBL Link State",
    },
    OemValStr {
        oem: IPMI_OEM_KONTRON,
        code: 0xC4,
        desc: "Board Reset",
    },
    OemValStr {
        oem: IPMI_OEM_KONTRON,
        code: 0xC5,
        desc: "FRU Information Agent",
    },
    OemValStr {
        oem: IPMI_OEM_KONTRON,
        code: 0xC6,
        desc: "POST Value Sensor",
    },
    OemValStr {
        oem: IPMI_OEM_KONTRON,
        code: 0xC7,
        desc: "FWUM Status",
    },
    OemValStr {
        oem: IPMI_OEM_KONTRON,
        code: 0xC8,
        desc: "Switch Mngt Software Status",
    },
    OemValStr {
        oem: IPMI_OEM_KONTRON,
        code: 0xC9,
        desc: "OEM Diagnostic Status",
    },
    OemValStr {
        oem: IPMI_OEM_KONTRON,
        code: 0xCA,
        desc: "Component Firmware Upgrade",
    },
    OemValStr {
        oem: IPMI_OEM_KONTRON,
        code: 0xCB,
        desc: "FRU Over Current",
    },
    OemValStr {
        oem: IPMI_OEM_KONTRON,
        code: 0xCC,
        desc: "FRU Sensor Error",
    },
    OemValStr {
        oem: IPMI_OEM_KONTRON,
        code: 0xCD,
        desc: "FRU Power Denied",
    },
    OemValStr {
        oem: IPMI_OEM_KONTRON,
        code: 0xCE,
        desc: "Reserved",
    },
    OemValStr {
        oem: IPMI_OEM_KONTRON,
        code: 0xCF,
        desc: "Board Reset",
    },
    OemValStr {
        oem: IPMI_OEM_KONTRON,
        code: 0xD0,
        desc: "Clock Resource Control",
    },
    OemValStr {
        oem: IPMI_OEM_KONTRON,
        code: 0xD1,
        desc: "Power State",
    },
    OemValStr {
        oem: IPMI_OEM_KONTRON,
        code: 0xD2,
        desc: "FRU Mngt Power Failure",
    },
    OemValStr {
        oem: IPMI_OEM_KONTRON,
        code: 0xD3,
        desc: "Jumper Status",
    },
    OemValStr {
        oem: IPMI_OEM_KONTRON,
        code: 0xF2,
        desc: "RTM Module Hotswap",
    },
    // PICMG Sensor Types
    OemValStr {
        oem: IPMI_OEM_PICMG,
        code: 0xF0,
        desc: "FRU Hot Swap",
    },
    OemValStr {
        oem: IPMI_OEM_PICMG,
        code: 0xF1,
        desc: "IPMB Physical Link",
    },
    OemValStr {
        oem: IPMI_OEM_PICMG,
        code: 0xF2,
        desc: "Module Hot Swap",
    },
    OemValStr {
        oem: IPMI_OEM_PICMG,
        code: 0xF3,
        desc: "Power Channel Notification",
    },
    OemValStr {
        oem: IPMI_OEM_PICMG,
        code: 0xF4,
        desc: "Telco Alarm Input",
    },
    // VITA 46.11 Sensor Types
    OemValStr {
        oem: IPMI_OEM_VITA,
        code: 0xF0,
        desc: "FRU State",
    },
    OemValStr {
        oem: IPMI_OEM_VITA,
        code: 0xF1,
        desc: "System IPMB Link",
    },
    OemValStr {
        oem: IPMI_OEM_VITA,
        code: 0xF2,
        desc: "FRU Health",
    },
    OemValStr {
        oem: IPMI_OEM_VITA,
        code: 0xF3,
        desc: "FRU Temperature",
    },
    OemValStr {
        oem: IPMI_OEM_VITA,
        code: 0xF4,
        desc: "Payload Test Results",
    },
    OemValStr {
        oem: IPMI_OEM_VITA,
        code: 0xF5,
        desc: "Payload Test Status",
    },
    // Sentinel value
    //OemValStr { oem: 0xffffff, code: 0x00, desc: "" }
];

const IPMI_NETFN_VALS: &[ValStr] = &[
    ValStr {
        val: IPMI_NETFN_CHASSIS,
        desc: "Chassis",
    },
    ValStr {
        val: IPMI_NETFN_BRIDGE,
        desc: "Bridge",
    },
    ValStr {
        val: IPMI_NETFN_SE,
        desc: "SensorEvent",
    },
    ValStr {
        val: IPMI_NETFN_APP,
        desc: "Application",
    },
    ValStr {
        val: IPMI_NETFN_FIRMWARE,
        desc: "Firmware",
    },
    ValStr {
        val: IPMI_NETFN_STORAGE,
        desc: "Storage",
    },
    ValStr {
        val: IPMI_NETFN_TRANSPORT,
        desc: "Transport",
    },
    ValStr {
        val: 0xff,
        desc: "",
    },
];

// From table 26-4 of the IPMI v2 specification
const IPMI_BIT_RATE_VALS: &[ValStr] = &[
    ValStr {
        val: 0x00,
        desc: "IPMI-Over-Serial-Setting",
    }, // Using the value in the IPMI Over Serial Config
    ValStr {
        val: 0x06,
        desc: "9.6",
    },
    ValStr {
        val: 0x07,
        desc: "19.2",
    },
    ValStr {
        val: 0x08,
        desc: "38.4",
    },
    ValStr {
        val: 0x09,
        desc: "57.6",
    },
    ValStr {
        val: 0x0A,
        desc: "115.2",
    },
    ValStr {
        val: 0x00,
        desc: "",
    },
];

const IPMI_CHANNEL_ACTIVITY_TYPE_VALS: &[ValStr] = &[
    ValStr {
        val: 0,
        desc: "IPMI Messaging session active",
    },
    ValStr {
        val: 1,
        desc: "Callback Messaging session active",
    },
    ValStr {
        val: 2,
        desc: "Dial-out Alert active",
    },
    ValStr {
        val: 3,
        desc: "TAP Page Active",
    },
    ValStr {
        val: 0x00,
        desc: "",
    },
];

const IPMI_PRIVLVL_VALS: &[U8Str] = &[
    U8Str {
        val: IPMI_SESSION_PRIV_CALLBACK,
        desc: "CALLBACK",
    },
    U8Str {
        val: IPMI_SESSION_PRIV_USER,
        desc: "USER",
    },
    U8Str {
        val: IPMI_SESSION_PRIV_OPERATOR,
        desc: "OPERATOR",
    },
    U8Str {
        val: IPMI_SESSION_PRIV_ADMIN,
        desc: "ADMINISTRATOR",
    },
    U8Str {
        val: IPMI_SESSION_PRIV_OEM,
        desc: "OEM",
    },
    U8Str {
        val: IPMI_SESSION_PRIV_NOACCESS,
        desc: "NO ACCESS",
    },
    U8Str {
        val: u8::MAX,
        desc: "",
    },
];

const IPMI_SET_IN_PROGRESS_VALS: &[U8Str] = &[
    U8Str {
        val: IPMI_SET_IN_PROGRESS_SET_COMPLETE,
        desc: "set-complete",
    },
    U8Str {
        val: IPMI_SET_IN_PROGRESS_IN_PROGRESS,
        desc: "set-in-progress",
    },
    U8Str {
        val: IPMI_SET_IN_PROGRESS_COMMIT_WRITE,
        desc: "commit-write",
    },
    U8Str { val: 0, desc: "" },
];

const IPMI_AUTHTYPE_SESSION_VALS: &[U8Str] = &[
    U8Str {
        val: IPMI_SESSION_AUTHTYPE_NONE,
        desc: "NONE",
    },
    U8Str {
        val: IPMI_SESSION_AUTHTYPE_MD2,
        desc: "MD2",
    },
    U8Str {
        val: IPMI_SESSION_AUTHTYPE_MD5,
        desc: "MD5",
    },
    U8Str {
        val: IPMI_SESSION_AUTHTYPE_PASSWORD,
        desc: "PASSWORD",
    },
    U8Str {
        val: IPMI_SESSION_AUTHTYPE_OEM,
        desc: "OEM",
    },
    U8Str {
        val: IPMI_SESSION_AUTHTYPE_RMCP_PLUS,
        desc: "RMCP+",
    },
    U8Str {
        val: 0xFF,
        desc: "",
    },
];

const IPMI_AUTHTYPE_VALS: &[U8Str] = &[
    U8Str {
        val: IPMI_1_5_AUTH_TYPE_BIT_NONE,
        desc: "NONE",
    },
    U8Str {
        val: IPMI_1_5_AUTH_TYPE_BIT_MD2,
        desc: "MD2",
    },
    U8Str {
        val: IPMI_1_5_AUTH_TYPE_BIT_MD5,
        desc: "MD5",
    },
    U8Str {
        val: IPMI_1_5_AUTH_TYPE_BIT_PASSWORD,
        desc: "PASSWORD",
    },
    U8Str {
        val: IPMI_1_5_AUTH_TYPE_BIT_OEM,
        desc: "OEM",
    },
    U8Str { val: 0, desc: "" },
];
const ENTITY_ID_VALS: &[ValStr] = &[
    ValStr {
        val: 0x00,
        desc: "Unspecified",
    },
    ValStr {
        val: 0x01,
        desc: "Other",
    },
    ValStr {
        val: 0x02,
        desc: "Unknown",
    },
    ValStr {
        val: 0x03,
        desc: "Processor",
    },
    ValStr {
        val: 0x04,
        desc: "Disk or Disk Bay",
    },
    ValStr {
        val: 0x05,
        desc: "Peripheral Bay",
    },
    ValStr {
        val: 0x06,
        desc: "System Management Module",
    },
    ValStr {
        val: 0x07,
        desc: "System Board",
    },
    ValStr {
        val: 0x08,
        desc: "Memory Module",
    },
    ValStr {
        val: 0x09,
        desc: "Processor Module",
    },
    ValStr {
        val: 0x0a,
        desc: "Power Supply",
    },
    ValStr {
        val: 0x0b,
        desc: "Add-in Card",
    },
    ValStr {
        val: 0x0c,
        desc: "Front Panel Board",
    },
    ValStr {
        val: 0x0d,
        desc: "Back Panel Board",
    },
    ValStr {
        val: 0x0e,
        desc: "Power System Board",
    },
    ValStr {
        val: 0x0f,
        desc: "Drive Backplane",
    },
    ValStr {
        val: 0x10,
        desc: "System Internal Expansion Board",
    },
    ValStr {
        val: 0x11,
        desc: "Other System Board",
    },
    ValStr {
        val: 0x12,
        desc: "Processor Board",
    },
    ValStr {
        val: 0x13,
        desc: "Power Unit",
    },
    ValStr {
        val: 0x14,
        desc: "Power Module",
    },
    ValStr {
        val: 0x15,
        desc: "Power Management",
    },
    ValStr {
        val: 0x16,
        desc: "Chassis Back Panel Board",
    },
    ValStr {
        val: 0x17,
        desc: "System Chassis",
    },
    ValStr {
        val: 0x18,
        desc: "Sub-Chassis",
    },
    ValStr {
        val: 0x19,
        desc: "Other Chassis Board",
    },
    ValStr {
        val: 0x1a,
        desc: "Disk Drive Bay",
    },
    ValStr {
        val: 0x1b,
        desc: "Peripheral Bay",
    },
    ValStr {
        val: 0x1c,
        desc: "Device Bay",
    },
    ValStr {
        val: 0x1d,
        desc: "Fan Device",
    },
    ValStr {
        val: 0x1e,
        desc: "Cooling Unit",
    },
    ValStr {
        val: 0x1f,
        desc: "Cable/Interconnect",
    },
    ValStr {
        val: 0x20,
        desc: "Memory Device",
    },
    ValStr {
        val: 0x21,
        desc: "System Management Software",
    },
    ValStr {
        val: 0x22,
        desc: "BIOS",
    },
    ValStr {
        val: 0x23,
        desc: "Operating System",
    },
    ValStr {
        val: 0x24,
        desc: "System Bus",
    },
    ValStr {
        val: 0x25,
        desc: "Group",
    },
    ValStr {
        val: 0x26,
        desc: "Remote Management Device",
    },
    ValStr {
        val: 0x27,
        desc: "External Environment",
    },
    ValStr {
        val: 0x28,
        desc: "Battery",
    },
    ValStr {
        val: 0x29,
        desc: "Processing Blade",
    },
    ValStr {
        val: 0x2A,
        desc: "Connectivity Switch",
    },
    ValStr {
        val: 0x2B,
        desc: "Processor/Memory Module",
    },
    ValStr {
        val: 0x2C,
        desc: "I/O Module",
    },
    ValStr {
        val: 0x2D,
        desc: "Processor/IO Module",
    },
    ValStr {
        val: 0x2E,
        desc: "Management Controller Firmware",
    },
    ValStr {
        val: 0x2F,
        desc: "IPMI Channel",
    },
    ValStr {
        val: 0x30,
        desc: "PCI Bus",
    },
    ValStr {
        val: 0x31,
        desc: "PCI Express Bus",
    },
    ValStr {
        val: 0x32,
        desc: "SCSI Bus (parallel)",
    },
    ValStr {
        val: 0x33,
        desc: "SATA/SAS Bus",
    },
    ValStr {
        val: 0x34,
        desc: "Processor/Front-Side Bus",
    },
    ValStr {
        val: 0x35,
        desc: "Real Time Clock(RTC)",
    },
    ValStr {
        val: 0x36,
        desc: "Reserved",
    },
    ValStr {
        val: 0x37,
        desc: "Air Inlet",
    },
    ValStr {
        val: 0x38,
        desc: "Reserved",
    },
    ValStr {
        val: 0x39,
        desc: "Reserved",
    },
    ValStr {
        val: 0x3A,
        desc: "Reserved",
    },
    ValStr {
        val: 0x3B,
        desc: "Reserved",
    },
    ValStr {
        val: 0x3C,
        desc: "Reserved",
    },
    ValStr {
        val: 0x3D,
        desc: "Reserved",
    },
    ValStr {
        val: 0x3E,
        desc: "Reserved",
    },
    ValStr {
        val: 0x3F,
        desc: "Reserved",
    },
    ValStr {
        val: 0x40,
        desc: "Air Inlet",
    },
    ValStr {
        val: 0x41,
        desc: "Processor",
    },
    ValStr {
        val: 0x42,
        desc: "Baseboard/Main System Board",
    },
    // PICMG
    ValStr {
        val: 0xA0,
        desc: "PICMG Front Board",
    },
    ValStr {
        val: 0xC0,
        desc: "PICMG Rear Transition Module",
    },
    ValStr {
        val: 0xC1,
        desc: "PICMG AdvancedMC Module",
    },
    ValStr {
        val: 0xF0,
        desc: "PICMG Shelf Management Controller",
    },
    ValStr {
        val: 0xF1,
        desc: "PICMG Filtration Unit",
    },
    ValStr {
        val: 0xF2,
        desc: "PICMG Shelf FRU Information",
    },
    ValStr {
        val: 0xF3,
        desc: "PICMG Alarm Panel",
    },
    ValStr {
        val: 0x00,
        desc: "",
    },
];

const ENTITY_DEVICE_TYPE_VALS: &[ValStr] = &[
    ValStr {
        val: 0x00,
        desc: "Reserved",
    },
    ValStr {
        val: 0x01,
        desc: "Reserved",
    },
    ValStr {
        val: 0x02,
        desc: "DS1624 temperature sensor",
    },
    ValStr {
        val: 0x03,
        desc: "DS1621 temperature sensor",
    },
    ValStr {
        val: 0x04,
        desc: "LM75 Temperature Sensor",
    },
    ValStr {
        val: 0x05,
        desc: "Heceta ASIC",
    },
    ValStr {
        val: 0x06,
        desc: "Reserved",
    },
    ValStr {
        val: 0x07,
        desc: "Reserved",
    },
    ValStr {
        val: 0x08,
        desc: "EEPROM, 24C01",
    },
    ValStr {
        val: 0x09,
        desc: "EEPROM, 24C02",
    },
    ValStr {
        val: 0x0a,
        desc: "EEPROM, 24C04",
    },
    ValStr {
        val: 0x0b,
        desc: "EEPROM, 24C08",
    },
    ValStr {
        val: 0x0c,
        desc: "EEPROM, 24C16",
    },
    ValStr {
        val: 0x0d,
        desc: "EEPROM, 24C17",
    },
    ValStr {
        val: 0x0e,
        desc: "EEPROM, 24C32",
    },
    ValStr {
        val: 0x0f,
        desc: "EEPROM, 24C64",
    },
    ValStr {
        val: 0x1000,
        desc: "IPMI FRU Inventory",
    },
    ValStr {
        val: 0x1001,
        desc: "DIMM Memory ID",
    },
    ValStr {
        val: 0x1002,
        desc: "IPMI FRU Inventory",
    },
    ValStr {
        val: 0x1003,
        desc: "System Processor Cartridge FRU",
    },
    ValStr {
        val: 0x11,
        desc: "Reserved",
    },
    ValStr {
        val: 0x12,
        desc: "Reserved",
    },
    ValStr {
        val: 0x13,
        desc: "Reserved",
    },
    ValStr {
        val: 0x14,
        desc: "PCF 8570 256 byte RAM",
    },
    ValStr {
        val: 0x15,
        desc: "PCF 8573 clock/calendar",
    },
    ValStr {
        val: 0x16,
        desc: "PCF 8574A I/O Port",
    },
    ValStr {
        val: 0x17,
        desc: "PCF 8583 clock/calendar",
    },
    ValStr {
        val: 0x18,
        desc: "PCF 8593 clock/calendar",
    },
    ValStr {
        val: 0x19,
        desc: "Clock calendar",
    },
    ValStr {
        val: 0x1a,
        desc: "PCF 8591 A/D, D/A Converter",
    },
    ValStr {
        val: 0x1b,
        desc: "I/O Port",
    },
    ValStr {
        val: 0x1c,
        desc: "A/D Converter",
    },
    ValStr {
        val: 0x1d,
        desc: "D/A Converter",
    },
    ValStr {
        val: 0x1e,
        desc: "A/D, D/A Converter",
    },
    ValStr {
        val: 0x1f,
        desc: "LCD Controller/Driver",
    },
    ValStr {
        val: 0x20,
        desc: "Core Logic (Chip set) Device",
    },
    ValStr {
        val: 0x21,
        desc: "LMC6874 Intelligent Battery controller",
    },
    ValStr {
        val: 0x22,
        desc: "Intelligent Batter controller",
    },
    ValStr {
        val: 0x23,
        desc: "Combo Management ASIC",
    },
    ValStr {
        val: 0x24,
        desc: "Maxim 1617 Temperature Sensor",
    },
    ValStr {
        val: 0xbf,
        desc: "Other/Unspecified",
    },
    ValStr {
        val: 0x00,
        desc: "",
    },
];
const IPMI_CHANNEL_PROTOCOL_VALS: &[ValStr] = &[
    ValStr {
        val: 0x00,
        desc: "reserved",
    },
    ValStr {
        val: 0x01,
        desc: "IPMB-1.0",
    },
    ValStr {
        val: 0x02,
        desc: "ICMB-1.0",
    },
    ValStr {
        val: 0x03,
        desc: "reserved",
    },
    ValStr {
        val: 0x04,
        desc: "IPMI-SMBus",
    },
    ValStr {
        val: 0x05,
        desc: "KCS",
    },
    ValStr {
        val: 0x06,
        desc: "SMIC",
    },
    ValStr {
        val: 0x07,
        desc: "BT-10",
    },
    ValStr {
        val: 0x08,
        desc: "BT-15",
    },
    ValStr {
        val: 0x09,
        desc: "TMode",
    },
    ValStr {
        val: 0x1c,
        desc: "OEM 1",
    },
    ValStr {
        val: 0x1d,
        desc: "OEM 2",
    },
    ValStr {
        val: 0x1e,
        desc: "OEM 3",
    },
    ValStr {
        val: 0x1f,
        desc: "OEM 4",
    },
    ValStr {
        val: 0x00,
        desc: "",
    },
];

const IPMI_CHANNEL_MEDIUM_VALS: &[U8Str] = &[
    U8Str {
        val: IPMI_CHANNEL_MEDIUM_RESERVED,
        desc: "reserved",
    },
    U8Str {
        val: IPMI_CHANNEL_MEDIUM_IPMB_I2C,
        desc: "IPMB (I2C)",
    },
    U8Str {
        val: IPMI_CHANNEL_MEDIUM_ICMB_1,
        desc: "ICMB v1.0",
    },
    U8Str {
        val: IPMI_CHANNEL_MEDIUM_ICMB_09,
        desc: "ICMB v0.9",
    },
    U8Str {
        val: IPMI_CHANNEL_MEDIUM_LAN,
        desc: "802.3 LAN",
    },
    U8Str {
        val: IPMI_CHANNEL_MEDIUM_SERIAL,
        desc: "Serial/Modem",
    },
    U8Str {
        val: IPMI_CHANNEL_MEDIUM_LAN_OTHER,
        desc: "Other LAN",
    },
    U8Str {
        val: IPMI_CHANNEL_MEDIUM_SMBUS_PCI,
        desc: "PCI SMBus",
    },
    U8Str {
        val: IPMI_CHANNEL_MEDIUM_SMBUS_1,
        desc: "SMBus v1.0/v1.1",
    },
    U8Str {
        val: IPMI_CHANNEL_MEDIUM_SMBUS_2,
        desc: "SMBus v2.0",
    },
    U8Str {
        val: IPMI_CHANNEL_MEDIUM_USB_1,
        desc: "USB 1.x",
    },
    U8Str {
        val: IPMI_CHANNEL_MEDIUM_USB_2,
        desc: "USB 2.x",
    },
    U8Str {
        val: IPMI_CHANNEL_MEDIUM_SYSTEM,
        desc: "System Interface",
    },
    U8Str {
        val: 0x00,
        desc: "",
    },
];

const COMPLETION_CODE_VALS: &[ValStr] = &[
    ValStr {
        val: 0x00,
        desc: "Command completed normally",
    },
    ValStr {
        val: 0xc0,
        desc: "Node busy",
    },
    ValStr {
        val: 0xc1,
        desc: "Invalid command",
    },
    ValStr {
        val: 0xc2,
        desc: "Invalid command on LUN",
    },
    ValStr {
        val: 0xc3,
        desc: "Timeout",
    },
    ValStr {
        val: 0xc4,
        desc: "Out of space",
    },
    ValStr {
        val: 0xc5,
        desc: "Reservation cancelled or invalid",
    },
    ValStr {
        val: 0xc6,
        desc: "Request data truncated",
    },
    ValStr {
        val: 0xc7,
        desc: "Request data length invalid",
    },
    ValStr {
        val: 0xc8,
        desc: "Request data field length limit exceeded",
    },
    ValStr {
        val: 0xc9,
        desc: "Parameter out of range",
    },
    ValStr {
        val: 0xca,
        desc: "Cannot return number of requested data bytes",
    },
    ValStr {
        val: 0xcb,
        desc: "Requested sensor, data, or record not found",
    },
    ValStr {
        val: 0xcc,
        desc: "Invalid data field in request",
    },
    ValStr {
        val: 0xcd,
        desc: "Command illegal for specified sensor or record type",
    },
    ValStr {
        val: 0xce,
        desc: "Command response could not be provided",
    },
    ValStr {
        val: 0xcf,
        desc: "Cannot execute duplicated request",
    },
    ValStr {
        val: 0xd0,
        desc: "SDR Repository in update mode",
    },
    ValStr {
        val: 0xd1,
        desc: "Device firmeware in update mode",
    },
    ValStr {
        val: 0xd2,
        desc: "BMC initialization in progress",
    },
    ValStr {
        val: 0xd3,
        desc: "Destination unavailable",
    },
    ValStr {
        val: 0xd4,
        desc: "Insufficient privilege level",
    },
    ValStr {
        val: 0xd5,
        desc: "Command not supported in present state",
    },
    ValStr {
        val: 0xd6,
        desc: "Cannot execute command, command disabled",
    },
    ValStr {
        val: 0xff,
        desc: "Unspecified error",
    },
    ValStr {
        val: 0x00,
        desc: "",
    },
];

pub const IPMI_CHASSIS_POWER_CONTROL_VALS: &[U8Str] = &[
    U8Str {
        val: IPMI_CHASSIS_CTL_POWER_DOWN,
        desc: "Down/Off",
    },
    U8Str {
        val: IPMI_CHASSIS_CTL_POWER_UP,
        desc: "Up/On",
    },
    U8Str {
        val: IPMI_CHASSIS_CTL_POWER_CYCLE,
        desc: "Cycle",
    },
    U8Str {
        val: IPMI_CHASSIS_CTL_HARD_RESET,
        desc: "Reset",
    },
    U8Str {
        val: IPMI_CHASSIS_CTL_PULSE_DIAG,
        desc: "Diag",
    },
    U8Str {
        val: IPMI_CHASSIS_CTL_ACPI_SOFT,
        desc: "Soft",
    },
    U8Str {
        val: 0x00,
        desc: "",
    },
];
/*
 * See Table 28-11, Get System Restart Cause Command
 */
pub const IPMI_CHASSIS_RESTART_CAUSE_VALS: &[ValStr] = &[
    ValStr {
        val: 0x0,
        desc: "unknown",
    },
    ValStr {
        val: 0x1,
        desc: "chassis power control command",
    },
    ValStr {
        val: 0x2,
        desc: "reset via pushbutton",
    },
    ValStr {
        val: 0x3,
        desc: "power-up via pushbutton",
    },
    ValStr {
        val: 0x4,
        desc: "watchdog expired",
    },
    ValStr {
        val: 0x5,
        desc: "OEM",
    },
    ValStr {
        val: 0x6,
        desc: "power-up due to always-restore power policy",
    },
    ValStr {
        val: 0x7,
        desc: "power-up due to restore-previous power policy",
    },
    ValStr {
        val: 0x8,
        desc: "reset via PEF",
    },
    ValStr {
        val: 0x9,
        desc: "power-cycle via PEF",
    },
    ValStr {
        val: 0xa,
        desc: "soft reset",
    },
    ValStr {
        val: 0xb,
        desc: "power-up via RTC wakeup",
    },
    ValStr {
        val: 0xFF,
        desc: "",
    },
];

const IPMI_AUTH_ALGORITHMS: &[U8Str] = &[
    U8Str {
        val: IPMI_AUTH_RAKP_NONE,
        desc: "none",
    },
    U8Str {
        val: IPMI_AUTH_RAKP_HMAC_SHA1,
        desc: "hmac_sha1",
    },
    U8Str {
        val: IPMI_AUTH_RAKP_HMAC_MD5,
        desc: "hmac_md5",
    },
    #[cfg(feature = "crypto-sha256")]
    U8Str {
        val: IPMI_AUTH_RAKP_HMAC_SHA256,
        desc: "hmac_sha256",
    },
    U8Str {
        val: 0x00,
        desc: "",
    },
];

const IPMI_INTEGRITY_ALGORITHMS: &[U8Str] = &[
    U8Str {
        val: IPMI_INTEGRITY_NONE,
        desc: "none",
    },
    U8Str {
        val: IPMI_INTEGRITY_HMAC_SHA1_96,
        desc: "hmac_sha1_96",
    },
    U8Str {
        val: IPMI_INTEGRITY_HMAC_MD5_128,
        desc: "hmac_md5_128",
    },
    U8Str {
        val: IPMI_INTEGRITY_MD5_128,
        desc: "md5_128",
    },
    #[cfg(feature = "crypto-sha256")]
    U8Str {
        val: IPMI_INTEGRITY_HMAC_SHA256_128,
        desc: "sha256_128",
    },
    U8Str {
        val: 0x00,
        desc: "",
    },
];

const IPMI_ENCRYPTION_ALGORITHMS: &[U8Str] = &[
    U8Str {
        val: IPMI_CRYPT_NONE,
        desc: "none",
    },
    U8Str {
        val: IPMI_CRYPT_AES_CBC_128,
        desc: "aes_cbc_128",
    },
    U8Str {
        val: IPMI_CRYPT_XRC4_128,
        desc: "xrc4_128",
    },
    U8Str {
        val: IPMI_CRYPT_XRC4_40,
        desc: "xrc4_40",
    },
    U8Str {
        val: 0x00,
        desc: "",
    },
];

const IPMI_USER_ENABLE_STATUS_VALS: &[ValStr] = &[
    ValStr {
        val: 0x00,
        desc: "unknown",
    },
    ValStr {
        val: 0x40,
        desc: "enabled",
    },
    ValStr {
        val: 0x80,
        desc: "disabled",
    },
    ValStr {
        val: 0xC0,
        desc: "reserved",
    },
    ValStr {
        val: 0xFF,
        desc: "",
    },
];

const PICMG_FRUCONTROL_VALS: &[ValStr] = &[
    ValStr {
        val: 0,
        desc: "Cold Reset",
    },
    ValStr {
        val: 1,
        desc: "Warm Reset",
    },
    ValStr {
        val: 2,
        desc: "Graceful Reboot",
    },
    ValStr {
        val: 3,
        desc: "Issue Diagnostic Interrupt",
    },
    ValStr {
        val: 4,
        desc: "Quiesce",
    },
    ValStr { val: 5, desc: "" },
];

const PICMG_CLK_FAMILY_VALS: &[ValStr] = &[
    ValStr {
        val: 0x00,
        desc: "Unspecified",
    },
    ValStr {
        val: 0x01,
        desc: "SONET/SDH/PDH",
    },
    ValStr {
        val: 0x02,
        desc: "Reserved for PCI Express",
    },
    ValStr {
        val: 0x03,
        desc: "Reserved",
    }, // from 03h to C8h
    ValStr {
        val: 0xC9,
        desc: "Vendor defined clock family",
    }, // from C9h to FFh
    ValStr {
        val: 0x00,
        desc: "",
    },
];

const PICMG_CLK_ACCURACY_VALS: &[OemValStr] = &[
    OemValStr {
        oem: 0x01,
        code: 10,
        desc: "PRS",
    },
    OemValStr {
        oem: 0x01,
        code: 20,
        desc: "STU",
    },
    OemValStr {
        oem: 0x01,
        code: 30,
        desc: "ST2",
    },
    OemValStr {
        oem: 0x01,
        code: 40,
        desc: "TNC",
    },
    OemValStr {
        oem: 0x01,
        code: 50,
        desc: "ST3E",
    },
    OemValStr {
        oem: 0x01,
        code: 60,
        desc: "ST3",
    },
    OemValStr {
        oem: 0x01,
        code: 70,
        desc: "SMC",
    },
    OemValStr {
        oem: 0x01,
        code: 80,
        desc: "ST4",
    },
    OemValStr {
        oem: 0x01,
        code: 90,
        desc: "DUS",
    },
    OemValStr {
        oem: 0x02,
        code: 0xE0,
        desc: "PCI Express Generation 2",
    },
    OemValStr {
        oem: 0x02,
        code: 0xF0,
        desc: "PCI Express Generation 1",
    },
    OemValStr {
        oem: 0xffffff,
        code: 0x00,
        desc: "",
    },
];

const PICMG_CLK_RESOURCE_VALS: &[OemValStr] = &[
    OemValStr {
        oem: 0x0,
        code: 0,
        desc: "On-Carrier Device 0",
    },
    OemValStr {
        oem: 0x0,
        code: 1,
        desc: "On-Carrier Device 1",
    },
    OemValStr {
        oem: 0x1,
        code: 1,
        desc: "AMC Site 1 - A1",
    },
    OemValStr {
        oem: 0x1,
        code: 2,
        desc: "AMC Site 1 - A2",
    },
    OemValStr {
        oem: 0x1,
        code: 3,
        desc: "AMC Site 1 - A3",
    },
    OemValStr {
        oem: 0x1,
        code: 4,
        desc: "AMC Site 1 - A4",
    },
    OemValStr {
        oem: 0x1,
        code: 5,
        desc: "AMC Site 1 - B1",
    },
    OemValStr {
        oem: 0x1,
        code: 6,
        desc: "AMC Site 1 - B2",
    },
    OemValStr {
        oem: 0x1,
        code: 7,
        desc: "AMC Site 1 - B3",
    },
    OemValStr {
        oem: 0x1,
        code: 8,
        desc: "AMC Site 1 - B4",
    },
    OemValStr {
        oem: 0x2,
        code: 0,
        desc: "ATCA Backplane",
    },
    OemValStr {
        oem: 0xffffff,
        code: 0x00,
        desc: "",
    },
];

const PICMG_CLK_ID_VALS: &[OemValStr] = &[
    OemValStr {
        oem: 0x0,
        code: 0,
        desc: "Clock 0",
    },
    OemValStr {
        oem: 0x0,
        code: 1,
        desc: "Clock 1",
    },
    OemValStr {
        oem: 0x0,
        code: 2,
        desc: "Clock 2",
    },
    OemValStr {
        oem: 0x0,
        code: 3,
        desc: "Clock 3",
    },
    OemValStr {
        oem: 0x0,
        code: 4,
        desc: "Clock 4",
    },
    OemValStr {
        oem: 0x0,
        code: 5,
        desc: "Clock 5",
    },
    OemValStr {
        oem: 0x0,
        code: 6,
        desc: "Clock 6",
    },
    OemValStr {
        oem: 0x0,
        code: 7,
        desc: "Clock 7",
    },
    OemValStr {
        oem: 0x0,
        code: 8,
        desc: "Clock 8",
    },
    OemValStr {
        oem: 0x0,
        code: 9,
        desc: "Clock 9",
    },
    OemValStr {
        oem: 0x0,
        code: 10,
        desc: "Clock 10",
    },
    OemValStr {
        oem: 0x0,
        code: 11,
        desc: "Clock 11",
    },
    OemValStr {
        oem: 0x0,
        code: 12,
        desc: "Clock 12",
    },
    OemValStr {
        oem: 0x0,
        code: 13,
        desc: "Clock 13",
    },
    OemValStr {
        oem: 0x0,
        code: 14,
        desc: "Clock 14",
    },
    OemValStr {
        oem: 0x0,
        code: 15,
        desc: "Clock 15",
    },
    OemValStr {
        oem: 0x1,
        code: 1,
        desc: "TCLKA",
    },
    OemValStr {
        oem: 0x1,
        code: 2,
        desc: "TCLKB",
    },
    OemValStr {
        oem: 0x1,
        code: 3,
        desc: "TCLKC",
    },
    OemValStr {
        oem: 0x1,
        code: 4,
        desc: "TCLKD",
    },
    OemValStr {
        oem: 0x1,
        code: 5,
        desc: "FLCKA",
    },
    OemValStr {
        oem: 0x2,
        code: 1,
        desc: "CLK1A",
    },
    OemValStr {
        oem: 0x2,
        code: 2,
        desc: "CLK1A",
    },
    OemValStr {
        oem: 0x2,
        code: 3,
        desc: "CLK1A",
    },
    OemValStr {
        oem: 0x2,
        code: 4,
        desc: "CLK1A",
    },
    OemValStr {
        oem: 0x2,
        code: 5,
        desc: "CLK1A",
    },
    OemValStr {
        oem: 0x2,
        code: 6,
        desc: "CLK1A",
    },
    OemValStr {
        oem: 0x2,
        code: 7,
        desc: "CLK1A",
    },
    OemValStr {
        oem: 0x2,
        code: 8,
        desc: "CLK1A",
    },
    OemValStr {
        oem: 0x2,
        code: 9,
        desc: "CLK1A",
    },
    OemValStr {
        oem: 0xffffff,
        code: 0x00,
        desc: "",
    },
];
const PICMG_BUSRES_ID_VALS: &[ValStr] = &[
    ValStr {
        val: 0x0,
        desc: "Metallic Test Bus pair #1",
    },
    ValStr {
        val: 0x1,
        desc: "Metallic Test Bus pair #2",
    },
    ValStr {
        val: 0x2,
        desc: "Synch clock group 1 (CLK1)",
    },
    ValStr {
        val: 0x3,
        desc: "Synch clock group 2 (CLK2)",
    },
    ValStr {
        val: 0x4,
        desc: "Synch clock group 3 (CLK3)",
    },
    ValStr { val: 0x5, desc: "" },
];

const PICMG_BUSRES_BOARD_CMD_VALS: &[ValStr] = &[
    ValStr {
        val: 0x0,
        desc: "Query",
    },
    ValStr {
        val: 0x1,
        desc: "Release",
    },
    ValStr {
        val: 0x2,
        desc: "Force",
    },
    ValStr {
        val: 0x3,
        desc: "Bus Free",
    },
    ValStr { val: 0x4, desc: "" },
];

const PICMG_BUSRES_SHMC_CMD_VALS: &[ValStr] = &[
    ValStr {
        val: 0x0,
        desc: "Request",
    },
    ValStr {
        val: 0x1,
        desc: "Relinquish",
    },
    ValStr {
        val: 0x2,
        desc: "Notify",
    },
    ValStr { val: 0x3, desc: "" },
];

const PICMG_BUSRES_BOARD_STATUS_VALS: &[OemValStr] = &[
    OemValStr {
        oem: 0x0,
        code: 0x0,
        desc: "In control",
    },
    OemValStr {
        oem: 0x0,
        code: 0x1,
        desc: "No control",
    },
    OemValStr {
        oem: 0x1,
        code: 0x0,
        desc: "Ack",
    },
    OemValStr {
        oem: 0x1,
        code: 0x1,
        desc: "Refused",
    },
    OemValStr {
        oem: 0x1,
        code: 0x2,
        desc: "No control",
    },
    OemValStr {
        oem: 0x2,
        code: 0x0,
        desc: "Ack",
    },
    OemValStr {
        oem: 0x2,
        code: 0x1,
        desc: "No control",
    },
    OemValStr {
        oem: 0x3,
        code: 0x0,
        desc: "Accept",
    },
    OemValStr {
        oem: 0x3,
        code: 0x1,
        desc: "Not Needed",
    },
    OemValStr {
        oem: 0xffffff,
        code: 0x00,
        desc: "",
    },
];

const PICMG_BUSRES_SHMC_STATUS_VALS: &[OemValStr] = &[
    OemValStr {
        oem: 0x0,
        code: 0x0,
        desc: "Grant",
    },
    OemValStr {
        oem: 0x0,
        code: 0x1,
        desc: "Busy",
    },
    OemValStr {
        oem: 0x0,
        code: 0x2,
        desc: "Defer",
    },
    OemValStr {
        oem: 0x0,
        code: 0x3,
        desc: "Deny",
    },
    OemValStr {
        oem: 0x1,
        code: 0x0,
        desc: "Ack",
    },
    OemValStr {
        oem: 0x1,
        code: 0x1,
        desc: "Error",
    },
    OemValStr {
        oem: 0x2,
        code: 0x0,
        desc: "Ack",
    },
    OemValStr {
        oem: 0x2,
        code: 0x1,
        desc: "Error",
    },
    OemValStr {
        oem: 0x2,
        code: 0x2,
        desc: "Deny",
    },
    OemValStr {
        oem: 0xffffff,
        code: 0x00,
        desc: "",
    },
];
