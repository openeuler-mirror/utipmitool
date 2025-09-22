/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
/*
 * These are put at the head so they are found first because they
 * may overlap with IANA specified numbers found in the registry.
 */
struct valstr {
    val: u32,
    str: &'static str,
}

const IPMI_OEM_INFO_HEAD: &[valstr] = &[
    valstr { val: IPMI_OEM_UNKNOWN, str: "Unknown" }, /* IPMI Unknown */
    valstr { val: IPMI_OEM_RESERVED, str: "Unspecified" }, /* IPMI Reserved */
    valstr { val: u32::MAX, str: "" },
];

/*
 * These are our own technical values. We don't want them to take precedence
 * over IANA's defined values, so they go at the very end of the array.
 */
const IPMI_OEM_INFO_TAIL: &[valstr] = &[
    valstr { val: IPMI_OEM_DEBUG, str: "A Debug Assisting Company, Ltd." },
    valstr { val: u32::MAX, str: "" },
];

/*
 * This is used when ipmi_oem_info couldn't be allocated.
 * ipmitool would report all OEMs as unknown, but would be functional otherwise.
 */ 
const IPMI_OEM_INFO_DUMMY: &[valstr] = &[
   valstr { val: u32::MAX, str: "" },
];

/* This will point to an array filled from IANA's enterprise numbers registry */
static mut IPMI_OEM_INFO: *mut valstr = std::ptr::null_mut();

/* Single-linked list of OEM valstrs */
struct OemValstrNode {
   valstr: valstr,
   next: Option<Box<OemValstrNode>>
}

const IPMI_OEM_PRODUCT_INFO: &[oemvalstr] = &[
   /* Keep OEM grouped together */

   /* For ipmitool debugging */
   oemvalstr { oem: IPMI_OEM_DEBUG, code: 0x1234, desc: "Great Debuggable BMC" },

   /* Intel stuff, thanks to Tim Bell */
   oemvalstr { oem: IPMI_OEM_INTEL, code: 0x000C, desc: "TSRLT2" },
   oemvalstr { oem: IPMI_OEM_INTEL, code: 0x001B, desc: "TIGPR2U" },
   oemvalstr { oem: IPMI_OEM_INTEL, code: 0x0022, desc: "TIGI2U" },
   oemvalstr { oem: IPMI_OEM_INTEL, code: 0x0026, desc: "Bridgeport" },
   oemvalstr { oem: IPMI_OEM_INTEL, code: 0x0028, desc: "S5000PAL" },
   oemvalstr { oem: IPMI_OEM_INTEL, code: 0x0029, desc: "S5000PSL" },
   oemvalstr { oem: IPMI_OEM_INTEL, code: 0x0100, desc: "Tiger4" },
   oemvalstr { oem: IPMI_OEM_INTEL, code: 0x0103, desc: "McCarran" },
   oemvalstr { oem: IPMI_OEM_INTEL, code: 0x0800, desc: "ZT5504" },
   oemvalstr { oem: IPMI_OEM_INTEL, code: 0x0808, desc: "MPCBL0001" },
   oemvalstr { oem: IPMI_OEM_INTEL, code: 0x0811, desc: "TIGW1U" },
   oemvalstr { oem: IPMI_OEM_INTEL, code: 0x4311, desc: "NSI2U" },

   /* Kontron */
   oemvalstr { oem: IPMI_OEM_KONTRON, code: 4000, desc: "AM4000 AdvancedMC" },
   oemvalstr { oem: IPMI_OEM_KONTRON, code: 4001, desc: "AM4001 AdvancedMC" },
   oemvalstr { oem: IPMI_OEM_KONTRON, code: 4002, desc: "AM4002 AdvancedMC" },
   oemvalstr { oem: IPMI_OEM_KONTRON, code: 4010, desc: "AM4010 AdvancedMC" },
   oemvalstr { oem: IPMI_OEM_KONTRON, code: 5503, desc: "AM4500/4520 AdvancedMC" },
   oemvalstr { oem: IPMI_OEM_KONTRON, code: 5504, desc: "AM4300 AdvancedMC" },
   oemvalstr { oem: IPMI_OEM_KONTRON, code: 5507, desc: "AM4301 AdvancedMC" },
   oemvalstr { oem: IPMI_OEM_KONTRON, code: 5508, desc: "AM4330 AdvancedMC" },
   // ... (continuing with similarly formatted entries)

   // YADRO
   oemvalstr { oem: IPMI_OEM_YADRO, code: 0x0001, desc: "VESNIN BMC" },
   oemvalstr { oem: IPMI_OEM_YADRO, code: 0x000A, desc: "TATLIN.UNIFIED Storage Controller BMC" },
   oemvalstr { oem: IPMI_OEM_YADRO, code: 0x0014, desc: "VEGMAN Series BMC" },
   oemvalstr { oem: IPMI_OEM_YADRO, code: 0x0015, desc: "TATLIN.ARCHIVE/xS BMC" },

   oemvalstr { oem: 0xffffff, code: 0xffff, desc: "" }
];

const IPMI_GENERIC_SENSOR_TYPE_VALS: &[&str] = &[
   "reserved",
   "Temperature", "Voltage", "Current", "Fan",
   "Physical Security", "Platform Security", "Processor", 
   "Power Supply", "Power Unit", "Cooling Device", "Other",
   "Memory", "Drive Slot / Bay", "POST Memory Resize",
   "System Firmwares", "Event Logging Disabled", "Watchdog1",
   "System Event", "Critical Interrupt", "Button",
   "Module / Board", "Microcontroller", "Add-in Card",
   "Chassis", "Chip Set", "Other FRU", "Cable / Interconnect",
   "Terminator", "System Boot Initiated", "Boot Error",
   "OS Boot", "OS Critical Stop", "Slot / Connector",
   "System ACPI Power State", "Watchdog2", "Platform Alert",
   "Entity Presence", "Monitor ASIC", "LAN",
   "Management Subsys Health", "Battery", "Session Audit",
   "Version Change", "FRU State",
];

const IPMI_OEM_SENSOR_TYPE_VALS: &[oemvalstr] = &[
	// Keep OEM grouped together 
	oemvalstr { oem: IPMI_OEM_KONTRON, code: 0xC0, desc: "Firmware Info" },
	oemvalstr { oem: IPMI_OEM_KONTRON, code: 0xC2, desc: "Init Agent" },
	oemvalstr { oem: IPMI_OEM_KONTRON, code: 0xC2, desc: "Board Reset(cPCI)" },
	oemvalstr { oem: IPMI_OEM_KONTRON, code: 0xC3, desc: "IPMBL Link State" },
	oemvalstr { oem: IPMI_OEM_KONTRON, code: 0xC4, desc: "Board Reset" },
	oemvalstr { oem: IPMI_OEM_KONTRON, code: 0xC5, desc: "FRU Information Agent" },
	oemvalstr { oem: IPMI_OEM_KONTRON, code: 0xC6, desc: "POST Value Sensor" },
	oemvalstr { oem: IPMI_OEM_KONTRON, code: 0xC7, desc: "FWUM Status" },
	oemvalstr { oem: IPMI_OEM_KONTRON, code: 0xC8, desc: "Switch Mngt Software Status" },
	oemvalstr { oem: IPMI_OEM_KONTRON, code: 0xC9, desc: "OEM Diagnostic Status" },
	oemvalstr { oem: IPMI_OEM_KONTRON, code: 0xCA, desc: "Component Firmware Upgrade" },
	oemvalstr { oem: IPMI_OEM_KONTRON, code: 0xCB, desc: "FRU Over Current" },
	oemvalstr { oem: IPMI_OEM_KONTRON, code: 0xCC, desc: "FRU Sensor Error" },
	oemvalstr { oem: IPMI_OEM_KONTRON, code: 0xCD, desc: "FRU Power Denied" },
	oemvalstr { oem: IPMI_OEM_KONTRON, code: 0xCE, desc: "Reserved" },
	oemvalstr { oem: IPMI_OEM_KONTRON, code: 0xCF, desc: "Board Reset" },
	oemvalstr { oem: IPMI_OEM_KONTRON, code: 0xD0, desc: "Clock Resource Control" },
	oemvalstr { oem: IPMI_OEM_KONTRON, code: 0xD1, desc: "Power State" },
	oemvalstr { oem: IPMI_OEM_KONTRON, code: 0xD2, desc: "FRU Mngt Power Failure" },
	oemvalstr { oem: IPMI_OEM_KONTRON, code: 0xD3, desc: "Jumper Status" },
	oemvalstr { oem: IPMI_OEM_KONTRON, code: 0xF2, desc: "RTM Module Hotswap" },
	// PICMG Sensor Types
	oemvalstr { oem: IPMI_OEM_PICMG, code: 0xF0, desc: "FRU Hot Swap" },
	oemvalstr { oem: IPMI_OEM_PICMG, code: 0xF1, desc: "IPMB Physical Link" }, 
	oemvalstr { oem: IPMI_OEM_PICMG, code: 0xF2, desc: "Module Hot Swap" },
	oemvalstr { oem: IPMI_OEM_PICMG, code: 0xF3, desc: "Power Channel Notification" },
	oemvalstr { oem: IPMI_OEM_PICMG, code: 0xF4, desc: "Telco Alarm Input" },
	// VITA 46.11 Sensor Types
	oemvalstr { oem: IPMI_OEM_VITA, code: 0xF0, desc: "FRU State" },
	oemvalstr { oem: IPMI_OEM_VITA, code: 0xF1, desc: "System IPMB Link" },
	oemvalstr { oem: IPMI_OEM_VITA, code: 0xF2, desc: "FRU Health" },
	oemvalstr { oem: IPMI_OEM_VITA, code: 0xF3, desc: "FRU Temperature" },
	oemvalstr { oem: IPMI_OEM_VITA, code: 0xF4, desc: "Payload Test Results" },
	oemvalstr { oem: IPMI_OEM_VITA, code: 0xF5, desc: "Payload Test Status" },
	// Sentinel value
	oemvalstr { oem: 0xffffff, code: 0x00, desc: "" }
];

const IPMI_NETFN_VALS: &[valstr] = &[
	valstr { val: IPMI_NETFN_CHASSIS, str: "Chassis" },
	valstr { val: IPMI_NETFN_BRIDGE, str: "Bridge" },
	valstr { val: IPMI_NETFN_SE, str: "SensorEvent" },
	valstr { val: IPMI_NETFN_APP, str: "Application" },
	valstr { val: IPMI_NETFN_FIRMWARE, str: "Firmware" }, 
	valstr { val: IPMI_NETFN_STORAGE, str: "Storage" },
	valstr { val: IPMI_NETFN_TRANSPORT, str: "Transport" },
	valstr { val: 0xff, str: "" },
];

// From table 26-4 of the IPMI v2 specification
const IPMI_BIT_RATE_VALS: &[valstr] = &[
	valstr { val: 0x00, str: "IPMI-Over-Serial-Setting" }, // Using the value in the IPMI Over Serial Config
	valstr { val: 0x06, str: "9.6" },
	valstr { val: 0x07, str: "19.2" },
	valstr { val: 0x08, str: "38.4" },
	valstr { val: 0x09, str: "57.6" },
	valstr { val: 0x0A, str: "115.2" },
	valstr { val: 0x00, str: "" },
];

const IPMI_CHANNEL_ACTIVITY_TYPE_VALS: &[valstr] = &[
	valstr { val: 0, str: "IPMI Messaging session active" },
	valstr { val: 1, str: "Callback Messaging session active" },
	valstr { val: 2, str: "Dial-out Alert active" },
	valstr { val: 3, str: "TAP Page Active" },
	valstr { val: 0x00, str: "" },
];

const IPMI_PRIVLVL_VALS: &[valstr] = &[
	valstr { val: IPMI_SESSION_PRIV_CALLBACK, str: "CALLBACK" },
	valstr { val: IPMI_SESSION_PRIV_USER, str: "USER" },
	valstr { val: IPMI_SESSION_PRIV_OPERATOR, str: "OPERATOR" },
	valstr { val: IPMI_SESSION_PRIV_ADMIN, str: "ADMINISTRATOR" },
	valstr { val: IPMI_SESSION_PRIV_OEM, str: "OEM" },
	valstr { val: IPMI_SESSION_PRIV_NOACCESS, str: "NO ACCESS" },
	valstr { val: u32::MAX, str: "" },
];

const IPMI_SET_IN_PROGRESS_VALS: &[valstr] = &[
	valstr { val: IPMI_SET_IN_PROGRESS_SET_COMPLETE, str: "set-complete" },
	valstr { val: IPMI_SET_IN_PROGRESS_IN_PROGRESS, str: "set-in-progress" },
	valstr { val: IPMI_SET_IN_PROGRESS_COMMIT_WRITE, str: "commit-write" },
	valstr { val: 0, str: "" },
];

const IPMI_AUTHTYPE_SESSION_VALS: &[valstr] = &[
	valstr { val: IPMI_SESSION_AUTHTYPE_NONE, str: "NONE" },
	valstr { val: IPMI_SESSION_AUTHTYPE_MD2, str: "MD2" },
	valstr { val: IPMI_SESSION_AUTHTYPE_MD5, str: "MD5" },
	valstr { val: IPMI_SESSION_AUTHTYPE_PASSWORD, str: "PASSWORD" },
	valstr { val: IPMI_SESSION_AUTHTYPE_OEM, str: "OEM" },
	valstr { val: IPMI_SESSION_AUTHTYPE_RMCP_PLUS, str: "RMCP+" },
	valstr { val: 0xFF, str: "" },
];

const IPMI_AUTHTYPE_VALS: &[valstr] = &[
	valstr { val: IPMI_1_5_AUTH_TYPE_BIT_NONE, str: "NONE" },
	valstr { val: IPMI_1_5_AUTH_TYPE_BIT_MD2, str: "MD2" },
	valstr { val: IPMI_1_5_AUTH_TYPE_BIT_MD5, str: "MD5" },
	valstr { val: IPMI_1_5_AUTH_TYPE_BIT_PASSWORD, str: "PASSWORD" },
	valstr { val: IPMI_1_5_AUTH_TYPE_BIT_OEM, str: "OEM" },
	valstr { val: 0, str: "" },
];
const ENTITY_ID_VALS: &[valstr] = &[
	valstr { val: 0x00, str: "Unspecified" },
	valstr { val: 0x01, str: "Other" },
	valstr { val: 0x02, str: "Unknown" },
	valstr { val: 0x03, str: "Processor" },
	valstr { val: 0x04, str: "Disk or Disk Bay" },
	valstr { val: 0x05, str: "Peripheral Bay" },
	valstr { val: 0x06, str: "System Management Module" },
	valstr { val: 0x07, str: "System Board" }, 
	valstr { val: 0x08, str: "Memory Module" },
	valstr { val: 0x09, str: "Processor Module" },
	valstr { val: 0x0a, str: "Power Supply" },
	valstr { val: 0x0b, str: "Add-in Card" },
	valstr { val: 0x0c, str: "Front Panel Board" },
	valstr { val: 0x0d, str: "Back Panel Board" },
	valstr { val: 0x0e, str: "Power System Board" },
	valstr { val: 0x0f, str: "Drive Backplane" },
	valstr { val: 0x10, str: "System Internal Expansion Board" },
	valstr { val: 0x11, str: "Other System Board" },
	valstr { val: 0x12, str: "Processor Board" },
	valstr { val: 0x13, str: "Power Unit" },
	valstr { val: 0x14, str: "Power Module" },
	valstr { val: 0x15, str: "Power Management" },
	valstr { val: 0x16, str: "Chassis Back Panel Board" },
	valstr { val: 0x17, str: "System Chassis" },
	valstr { val: 0x18, str: "Sub-Chassis" },
	valstr { val: 0x19, str: "Other Chassis Board" },
	valstr { val: 0x1a, str: "Disk Drive Bay" },
	valstr { val: 0x1b, str: "Peripheral Bay" },
	valstr { val: 0x1c, str: "Device Bay" },
	valstr { val: 0x1d, str: "Fan Device" },
	valstr { val: 0x1e, str: "Cooling Unit" },
	valstr { val: 0x1f, str: "Cable/Interconnect" },
	valstr { val: 0x20, str: "Memory Device" },
	valstr { val: 0x21, str: "System Management Software" },
	valstr { val: 0x22, str: "BIOS" },
	valstr { val: 0x23, str: "Operating System" },
	valstr { val: 0x24, str: "System Bus" },
	valstr { val: 0x25, str: "Group" },
	valstr { val: 0x26, str: "Remote Management Device" },
	valstr { val: 0x27, str: "External Environment" },
	valstr { val: 0x28, str: "Battery" },
	valstr { val: 0x29, str: "Processing Blade" },
	valstr { val: 0x2A, str: "Connectivity Switch" },
	valstr { val: 0x2B, str: "Processor/Memory Module" },
	valstr { val: 0x2C, str: "I/O Module" },
	valstr { val: 0x2D, str: "Processor/IO Module" },
	valstr { val: 0x2E, str: "Management Controller Firmware" },
	valstr { val: 0x2F, str: "IPMI Channel" },
	valstr { val: 0x30, str: "PCI Bus" },
	valstr { val: 0x31, str: "PCI Express Bus" },
	valstr { val: 0x32, str: "SCSI Bus (parallel)" },
	valstr { val: 0x33, str: "SATA/SAS Bus" },
	valstr { val: 0x34, str: "Processor/Front-Side Bus" },
	valstr { val: 0x35, str: "Real Time Clock(RTC)" },
	valstr { val: 0x36, str: "Reserved" },
	valstr { val: 0x37, str: "Air Inlet" },
	valstr { val: 0x38, str: "Reserved" },
	valstr { val: 0x39, str: "Reserved" },
	valstr { val: 0x3A, str: "Reserved" },
	valstr { val: 0x3B, str: "Reserved" },
	valstr { val: 0x3C, str: "Reserved" },
	valstr { val: 0x3D, str: "Reserved" },
	valstr { val: 0x3E, str: "Reserved" },
	valstr { val: 0x3F, str: "Reserved" },
	valstr { val: 0x40, str: "Air Inlet" },
	valstr { val: 0x41, str: "Processor" },
	valstr { val: 0x42, str: "Baseboard/Main System Board" },
	// PICMG
	valstr { val: 0xA0, str: "PICMG Front Board" },
	valstr { val: 0xC0, str: "PICMG Rear Transition Module" },
	valstr { val: 0xC1, str: "PICMG AdvancedMC Module" },
	valstr { val: 0xF0, str: "PICMG Shelf Management Controller" },
	valstr { val: 0xF1, str: "PICMG Filtration Unit" },
	valstr { val: 0xF2, str: "PICMG Shelf FRU Information" },
	valstr { val: 0xF3, str: "PICMG Alarm Panel" },
	valstr { val: 0x00, str: "" },
];

const ENTITY_DEVICE_TYPE_VALS: &[valstr] = &[
	valstr { val: 0x00, str: "Reserved" },
	valstr { val: 0x01, str: "Reserved" },
	valstr { val: 0x02, str: "DS1624 temperature sensor" },
	valstr { val: 0x03, str: "DS1621 temperature sensor" },
	valstr { val: 0x04, str: "LM75 Temperature Sensor" },
	valstr { val: 0x05, str: "Heceta ASIC" },
	valstr { val: 0x06, str: "Reserved" },
	valstr { val: 0x07, str: "Reserved" },
	valstr { val: 0x08, str: "EEPROM, 24C01" },
	valstr { val: 0x09, str: "EEPROM, 24C02" },
	valstr { val: 0x0a, str: "EEPROM, 24C04" },
	valstr { val: 0x0b, str: "EEPROM, 24C08" },
	valstr { val: 0x0c, str: "EEPROM, 24C16" },
	valstr { val: 0x0d, str: "EEPROM, 24C17" },
	valstr { val: 0x0e, str: "EEPROM, 24C32" },
	valstr { val: 0x0f, str: "EEPROM, 24C64" },
	valstr { val: 0x1000, str: "IPMI FRU Inventory" },
	valstr { val: 0x1001, str: "DIMM Memory ID" },
	valstr { val: 0x1002, str: "IPMI FRU Inventory" },
	valstr { val: 0x1003, str: "System Processor Cartridge FRU" },
	valstr { val: 0x11, str: "Reserved" },
	valstr { val: 0x12, str: "Reserved" },
	valstr { val: 0x13, str: "Reserved" },
	valstr { val: 0x14, str: "PCF 8570 256 byte RAM" },
	valstr { val: 0x15, str: "PCF 8573 clock/calendar" },
	valstr { val: 0x16, str: "PCF 8574A I/O Port" },
	valstr { val: 0x17, str: "PCF 8583 clock/calendar" },
	valstr { val: 0x18, str: "PCF 8593 clock/calendar" },
	valstr { val: 0x19, str: "Clock calendar" },
	valstr { val: 0x1a, str: "PCF 8591 A/D, D/A Converter" },
	valstr { val: 0x1b, str: "I/O Port" },
	valstr { val: 0x1c, str: "A/D Converter" },
	valstr { val: 0x1d, str: "D/A Converter" },
	valstr { val: 0x1e, str: "A/D, D/A Converter" },
	valstr { val: 0x1f, str: "LCD Controller/Driver" },
	valstr { val: 0x20, str: "Core Logic (Chip set) Device" },
	valstr { val: 0x21, str: "LMC6874 Intelligent Battery controller" },
	valstr { val: 0x22, str: "Intelligent Batter controller" },
	valstr { val: 0x23, str: "Combo Management ASIC" },
	valstr { val: 0x24, str: "Maxim 1617 Temperature Sensor" },
	valstr { val: 0xbf, str: "Other/Unspecified" },
	valstr { val: 0x00, str: "" },
];
const IPMI_CHANNEL_PROTOCOL_VALS: &[valstr] = &[
	valstr { val: 0x00, str: "reserved" },
	valstr { val: 0x01, str: "IPMB-1.0" },
	valstr { val: 0x02, str: "ICMB-1.0" }, 
	valstr { val: 0x03, str: "reserved" },
	valstr { val: 0x04, str: "IPMI-SMBus" },
	valstr { val: 0x05, str: "KCS" },
	valstr { val: 0x06, str: "SMIC" },
	valstr { val: 0x07, str: "BT-10" },
	valstr { val: 0x08, str: "BT-15" },
	valstr { val: 0x09, str: "TMode" },
	valstr { val: 0x1c, str: "OEM 1" },
	valstr { val: 0x1d, str: "OEM 2" },
	valstr { val: 0x1e, str: "OEM 3" },
	valstr { val: 0x1f, str: "OEM 4" },
	valstr { val: 0x00, str: "" },
];

const IPMI_CHANNEL_MEDIUM_VALS: &[valstr] = &[
	valstr { val: IPMI_CHANNEL_MEDIUM_RESERVED, str: "reserved" },
	valstr { val: IPMI_CHANNEL_MEDIUM_IPMB_I2C, str: "IPMB (I2C)" },
	valstr { val: IPMI_CHANNEL_MEDIUM_ICMB_1, str: "ICMB v1.0" },
	valstr { val: IPMI_CHANNEL_MEDIUM_ICMB_09, str: "ICMB v0.9" },
	valstr { val: IPMI_CHANNEL_MEDIUM_LAN, str: "802.3 LAN" },
	valstr { val: IPMI_CHANNEL_MEDIUM_SERIAL, str: "Serial/Modem" },
	valstr { val: IPMI_CHANNEL_MEDIUM_LAN_OTHER, str: "Other LAN" },
	valstr { val: IPMI_CHANNEL_MEDIUM_SMBUS_PCI, str: "PCI SMBus" },
	valstr { val: IPMI_CHANNEL_MEDIUM_SMBUS_1, str: "SMBus v1.0/v1.1" },
	valstr { val: IPMI_CHANNEL_MEDIUM_SMBUS_2, str: "SMBus v2.0" },
	valstr { val: IPMI_CHANNEL_MEDIUM_USB_1, str: "USB 1.x" },
	valstr { val: IPMI_CHANNEL_MEDIUM_USB_2, str: "USB 2.x" },
	valstr { val: IPMI_CHANNEL_MEDIUM_SYSTEM, str: "System Interface" },
	valstr { val: 0x00, str: "" },
];

const COMPLETION_CODE_VALS: &[valstr] = &[
	valstr { val: 0x00, str: "Command completed normally" },
	valstr { val: 0xc0, str: "Node busy" },
	valstr { val: 0xc1, str: "Invalid command" },
	valstr { val: 0xc2, str: "Invalid command on LUN" },
	valstr { val: 0xc3, str: "Timeout" },
	valstr { val: 0xc4, str: "Out of space" },
	valstr { val: 0xc5, str: "Reservation cancelled or invalid" },
	valstr { val: 0xc6, str: "Request data truncated" },
	valstr { val: 0xc7, str: "Request data length invalid" },
	valstr { val: 0xc8, str: "Request data field length limit exceeded" },
	valstr { val: 0xc9, str: "Parameter out of range" },
	valstr { val: 0xca, str: "Cannot return number of requested data bytes" },
	valstr { val: 0xcb, str: "Requested sensor, data, or record not found" },
	valstr { val: 0xcc, str: "Invalid data field in request" },
	valstr { val: 0xcd, str: "Command illegal for specified sensor or record type" },
	valstr { val: 0xce, str: "Command response could not be provided" },
	valstr { val: 0xcf, str: "Cannot execute duplicated request" },
	valstr { val: 0xd0, str: "SDR Repository in update mode" },
	valstr { val: 0xd1, str: "Device firmeware in update mode" },
	valstr { val: 0xd2, str: "BMC initialization in progress" },
	valstr { val: 0xd3, str: "Destination unavailable" },
	valstr { val: 0xd4, str: "Insufficient privilege level" },
	valstr { val: 0xd5, str: "Command not supported in present state" },
	valstr { val: 0xd6, str: "Cannot execute command, command disabled" },
	valstr { val: 0xff, str: "Unspecified error" },
	valstr { val: 0x00, str: "" },
];

const IPMI_CHASSIS_POWER_CONTROL_VALS: &[valstr] = &[
	valstr { val: IPMI_CHASSIS_CTL_POWER_DOWN, str: "Down/Off" },
	valstr { val: IPMI_CHASSIS_CTL_POWER_UP, str: "Up/On" },
	valstr { val: IPMI_CHASSIS_CTL_POWER_CYCLE, str: "Cycle" },
	valstr { val: IPMI_CHASSIS_CTL_HARD_RESET, str: "Reset" },
	valstr { val: IPMI_CHASSIS_CTL_PULSE_DIAG, str: "Diag" },
	valstr { val: IPMI_CHASSIS_CTL_ACPI_SOFT, str: "Soft" },
	valstr { val: 0x00, str: "" },
];
/*
 * See Table 28-11, Get System Restart Cause Command 
 */
const IPMI_CHASSIS_RESTART_CAUSE_VALS: &[valstr] = &[
	valstr { val: 0x0, str: "unknown" },
	valstr { val: 0x1, str: "chassis power control command" },
	valstr { val: 0x2, str: "reset via pushbutton" }, 
	valstr { val: 0x3, str: "power-up via pushbutton" },
	valstr { val: 0x4, str: "watchdog expired" },
	valstr { val: 0x5, str: "OEM" },
	valstr { val: 0x6, str: "power-up due to always-restore power policy" },
	valstr { val: 0x7, str: "power-up due to restore-previous power policy" },
	valstr { val: 0x8, str: "reset via PEF" },
	valstr { val: 0x9, str: "power-cycle via PEF" },
	valstr { val: 0xa, str: "soft reset" },
	valstr { val: 0xb, str: "power-up via RTC wakeup" },
	valstr { val: 0xFF, str: "" },
];

const IPMI_AUTH_ALGORITHMS: &[valstr] = &[
	valstr { val: IPMI_AUTH_RAKP_NONE, str: "none" },
	valstr { val: IPMI_AUTH_RAKP_HMAC_SHA1, str: "hmac_sha1" },
	valstr { val: IPMI_AUTH_RAKP_HMAC_MD5, str: "hmac_md5" },
	#[cfg(feature = "crypto-sha256")]
	valstr { val: IPMI_AUTH_RAKP_HMAC_SHA256, str: "hmac_sha256" },
	valstr { val: 0x00, str: "" },
];

const IPMI_INTEGRITY_ALGORITHMS: &[valstr] = &[
	valstr { val: IPMI_INTEGRITY_NONE, str: "none" },
	valstr { val: IPMI_INTEGRITY_HMAC_SHA1_96, str: "hmac_sha1_96" },
	valstr { val: IPMI_INTEGRITY_HMAC_MD5_128, str: "hmac_md5_128" },
	valstr { val: IPMI_INTEGRITY_MD5_128, str: "md5_128" },
	#[cfg(feature = "crypto-sha256")]
	valstr { val: IPMI_INTEGRITY_HMAC_SHA256_128, str: "sha256_128" },
	valstr { val: 0x00, str: "" },
];

const IPMI_ENCRYPTION_ALGORITHMS: &[valstr] = &[
	valstr { val: IPMI_CRYPT_NONE, str: "none" },
	valstr { val: IPMI_CRYPT_AES_CBC_128, str: "aes_cbc_128" },
	valstr { val: IPMI_CRYPT_XRC4_128, str: "xrc4_128" },
	valstr { val: IPMI_CRYPT_XRC4_40, str: "xrc4_40" },
	valstr { val: 0x00, str: "" },
];

const IPMI_USER_ENABLE_STATUS_VALS: &[valstr] = &[
	valstr { val: 0x00, str: "unknown" },
	valstr { val: 0x40, str: "enabled" },
	valstr { val: 0x80, str: "disabled" },
	valstr { val: 0xC0, str: "reserved" },
	valstr { val: 0xFF, str: "" },
];

const PICMG_FRUCONTROL_VALS: &[valstr] = &[
	valstr { val: 0, str: "Cold Reset" },
	valstr { val: 1, str: "Warm Reset" },
	valstr { val: 2, str: "Graceful Reboot" },
	valstr { val: 3, str: "Issue Diagnostic Interrupt" },
	valstr { val: 4, str: "Quiesce" },
	valstr { val: 5, str: "" },
];

const PICMG_CLK_FAMILY_VALS: &[valstr] = &[
	valstr { val: 0x00, str: "Unspecified" },
	valstr { val: 0x01, str: "SONET/SDH/PDH" },
	valstr { val: 0x02, str: "Reserved for PCI Express" },
	valstr { val: 0x03, str: "Reserved" }, // from 03h to C8h
	valstr { val: 0xC9, str: "Vendor defined clock family" }, // from C9h to FFh
	valstr { val: 0x00, str: "" },
];

const PICMG_CLK_ACCURACY_VALS: &[oemvalstr] = &[
	oemvalstr { oem: 0x01, code: 10, desc: "PRS" },
	oemvalstr { oem: 0x01, code: 20, desc: "STU" },
	oemvalstr { oem: 0x01, code: 30, desc: "ST2" },
	oemvalstr { oem: 0x01, code: 40, desc: "TNC" },
	oemvalstr { oem: 0x01, code: 50, desc: "ST3E" },
	oemvalstr { oem: 0x01, code: 60, desc: "ST3" },
	oemvalstr { oem: 0x01, code: 70, desc: "SMC" },
	oemvalstr { oem: 0x01, code: 80, desc: "ST4" },
	oemvalstr { oem: 0x01, code: 90, desc: "DUS" },
	oemvalstr { oem: 0x02, code: 0xE0, desc: "PCI Express Generation 2" },
	oemvalstr { oem: 0x02, code: 0xF0, desc: "PCI Express Generation 1" },
	oemvalstr { oem: 0xffffff, code: 0x00, desc: "" },
];

const PICMG_CLK_RESOURCE_VALS: &[oemvalstr] = &[
	oemvalstr { oem: 0x0, code: 0, desc: "On-Carrier Device 0" },
	oemvalstr { oem: 0x0, code: 1, desc: "On-Carrier Device 1" },
	oemvalstr { oem: 0x1, code: 1, desc: "AMC Site 1 - A1" },
	oemvalstr { oem: 0x1, code: 2, desc: "AMC Site 1 - A2" },
	oemvalstr { oem: 0x1, code: 3, desc: "AMC Site 1 - A3" },
	oemvalstr { oem: 0x1, code: 4, desc: "AMC Site 1 - A4" },
	oemvalstr { oem: 0x1, code: 5, desc: "AMC Site 1 - B1" },
	oemvalstr { oem: 0x1, code: 6, desc: "AMC Site 1 - B2" },
	oemvalstr { oem: 0x1, code: 7, desc: "AMC Site 1 - B3" },
	oemvalstr { oem: 0x1, code: 8, desc: "AMC Site 1 - B4" },
	oemvalstr { oem: 0x2, code: 0, desc: "ATCA Backplane" },
	oemvalstr { oem: 0xffffff, code: 0x00, desc: "" },
];

const PICMG_CLK_ID_VALS: &[oemvalstr] = &[
	oemvalstr { oem: 0x0, code: 0, desc: "Clock 0" },
	oemvalstr { oem: 0x0, code: 1, desc: "Clock 1" },
	oemvalstr { oem: 0x0, code: 2, desc: "Clock 2" },
	oemvalstr { oem: 0x0, code: 3, desc: "Clock 3" },
	oemvalstr { oem: 0x0, code: 4, desc: "Clock 4" },
	oemvalstr { oem: 0x0, code: 5, desc: "Clock 5" },
	oemvalstr { oem: 0x0, code: 6, desc: "Clock 6" },
	oemvalstr { oem: 0x0, code: 7, desc: "Clock 7" },
	oemvalstr { oem: 0x0, code: 8, desc: "Clock 8" },
	oemvalstr { oem: 0x0, code: 9, desc: "Clock 9" },
	oemvalstr { oem: 0x0, code: 10, desc: "Clock 10" },
	oemvalstr { oem: 0x0, code: 11, desc: "Clock 11" },
	oemvalstr { oem: 0x0, code: 12, desc: "Clock 12" },
	oemvalstr { oem: 0x0, code: 13, desc: "Clock 13" },
	oemvalstr { oem: 0x0, code: 14, desc: "Clock 14" },
	oemvalstr { oem: 0x0, code: 15, desc: "Clock 15" },
	oemvalstr { oem: 0x1, code: 1, desc: "TCLKA" },
	oemvalstr { oem: 0x1, code: 2, desc: "TCLKB" },
	oemvalstr { oem: 0x1, code: 3, desc: "TCLKC" },
	oemvalstr { oem: 0x1, code: 4, desc: "TCLKD" },
	oemvalstr { oem: 0x1, code: 5, desc: "FLCKA" },
	oemvalstr { oem: 0x2, code: 1, desc: "CLK1A" },
	oemvalstr { oem: 0x2, code: 2, desc: "CLK1A" },
	oemvalstr { oem: 0x2, code: 3, desc: "CLK1A" },
	oemvalstr { oem: 0x2, code: 4, desc: "CLK1A" },
	oemvalstr { oem: 0x2, code: 5, desc: "CLK1A" },
	oemvalstr { oem: 0x2, code: 6, desc: "CLK1A" },
	oemvalstr { oem: 0x2, code: 7, desc: "CLK1A" },
	oemvalstr { oem: 0x2, code: 8, desc: "CLK1A" },
	oemvalstr { oem: 0x2, code: 9, desc: "CLK1A" },
	oemvalstr { oem: 0xffffff, code: 0x00, desc: "" },
];
const PICMG_BUSRES_ID_VALS: &[valstr] = &[
	valstr { val: 0x0, str: "Metallic Test Bus pair #1" },
	valstr { val: 0x1, str: "Metallic Test Bus pair #2" }, 
	valstr { val: 0x2, str: "Synch clock group 1 (CLK1)" },
	valstr { val: 0x3, str: "Synch clock group 2 (CLK2)" },
	valstr { val: 0x4, str: "Synch clock group 3 (CLK3)" },
	valstr { val: 0x5, str: "" },
];

const PICMG_BUSRES_BOARD_CMD_VALS: &[valstr] = &[
	valstr { val: 0x0, str: "Query" },
	valstr { val: 0x1, str: "Release" },
	valstr { val: 0x2, str: "Force" }, 
	valstr { val: 0x3, str: "Bus Free" },
	valstr { val: 0x4, str: "" },
];

const PICMG_BUSRES_SHMC_CMD_VALS: &[valstr] = &[
	valstr { val: 0x0, str: "Request" },
	valstr { val: 0x1, str: "Relinquish" },
	valstr { val: 0x2, str: "Notify" },
	valstr { val: 0x3, str: "" },
];

const PICMG_BUSRES_BOARD_STATUS_VALS: &[oemvalstr] = &[
	oemvalstr { oem: 0x0, code: 0x0, desc: "In control" },
	oemvalstr { oem: 0x0, code: 0x1, desc: "No control" },
	oemvalstr { oem: 0x1, code: 0x0, desc: "Ack" },
	oemvalstr { oem: 0x1, code: 0x1, desc: "Refused" },
	oemvalstr { oem: 0x1, code: 0x2, desc: "No control" },
	oemvalstr { oem: 0x2, code: 0x0, desc: "Ack" },
	oemvalstr { oem: 0x2, code: 0x1, desc: "No control" },
	oemvalstr { oem: 0x3, code: 0x0, desc: "Accept" },
	oemvalstr { oem: 0x3, code: 0x1, desc: "Not Needed" },
	oemvalstr { oem: 0xffffff, code: 0x00, desc: "" },
];

const PICMG_BUSRES_SHMC_STATUS_VALS: &[oemvalstr] = &[
	oemvalstr { oem: 0x0, code: 0x0, desc: "Grant" },
	oemvalstr { oem: 0x0, code: 0x1, desc: "Busy" },
	oemvalstr { oem: 0x0, code: 0x2, desc: "Defer" },
	oemvalstr { oem: 0x0, code: 0x3, desc: "Deny" },
	oemvalstr { oem: 0x1, code: 0x0, desc: "Ack" },
	oemvalstr { oem: 0x1, code: 0x1, desc: "Error" },
	oemvalstr { oem: 0x2, code: 0x0, desc: "Ack" },
	oemvalstr { oem: 0x2, code: 0x1, desc: "Error" },
	oemvalstr { oem: 0x2, code: 0x2, desc: "Deny" },
	oemvalstr { oem: 0xffffff, code: 0x00, desc: "" },
];
