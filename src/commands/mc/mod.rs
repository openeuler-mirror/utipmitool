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
use crate::ipmi::intf::IpmiIntf;
use crate::ipmi::ipmi::{IpmiRq, IPMI_NETFN_APP};
use clap::Subcommand;
use ipmi_macros::AsBytes;

// MC子命令
#[derive(Debug, Clone, Subcommand)]
pub enum McCommand {
    /// Get device ID and capabilities information
    Info,
    /// Reset the management controller
    Reset {
        /// Reset type: warm (default) or cold
        #[arg(default_value = "")]
        reset_type: String,
    },
}

// IPMI constant definitions matching C reference
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

// Bit masks from C reference code
const IPM_DEV_DEVICE_ID_REV_MASK: u8 = 0x0F; // BCD-encoded
const IPM_DEV_DEVICE_ID_SDR_MASK: u8 = 0x80; // 1 = provides SDRs
const IPM_DEV_FWREV1_AVAIL_MASK: u8 = 0x80; // 0 = normal operation
const IPM_DEV_FWREV1_MAJOR_MASK: u8 = 0x7F; // Major firmware revision

/// IPMI Device ID Response Structure
/// This structure represents the response from the Get Device ID command
/// Consolidated from the former BMC module (now deprecated) into MC module
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

impl IpmDevidRsp {
    fn format_device_info(&self) -> String {
        let mut output = String::new();

        // Device ID
        output.push_str(&format!("Device ID                 : {}\n", self.device_id));

        // Device Revision - FIXED: Exact format matching C code
        output.push_str(&format!(
            "Device Revision           : {}\n",
            self.device_revision & IPM_DEV_DEVICE_ID_REV_MASK
        ));

        // Firmware Revision - FIXED: Exact format matching C code "%u.%02x"
        output.push_str(&format!(
            "Firmware Revision         : {}.{:02x}\n",
            self.fw_rev1 & IPM_DEV_FWREV1_MAJOR_MASK,
            self.fw_rev2
        ));

        // IPMI Version - exact format matching C code
        let ipmi_major = self.ipmi_version & 0x0f;
        let ipmi_minor = (self.ipmi_version & 0xf0) >> 4;
        output.push_str(&format!(
            "IPMI Version              : {}.{}\n",
            ipmi_major, ipmi_minor
        ));

        // Manufacturer ID - convert from little-endian 3-byte array
        let manufacturer_id = u32::from_le_bytes([
            self.manufacturer_id[0],
            self.manufacturer_id[1],
            self.manufacturer_id[2],
            0,
        ]) & 0x00ffffff;
        output.push_str(&format!(
            "Manufacturer ID           : {}\n",
            manufacturer_id
        ));

        // Manufacturer Name (would need lookup table like C code)
        let manufacturer_name = get_manufacturer_name(manufacturer_id);
        output.push_str(&format!(
            "Manufacturer Name         : {}\n",
            manufacturer_name
        ));

        // Product ID - convert from little-endian 2-byte array
        let product_id = u16::from_le_bytes(self.product_id);
        output.push_str(&format!(
            "Product ID                : {} (0x{:02x}{:02x})\n",
            product_id, self.product_id[1], self.product_id[0]
        ));

        let mut product_name = get_product_name(manufacturer_id, product_id);
        if product_name.is_empty() {
            product_name = "Unknown";
        }
        output.push_str(&format!("Product Name              : {}\n", product_name));

        // Device Available
        let available = if self.fw_rev1 & IPM_DEV_FWREV1_AVAIL_MASK != 0 {
            "no"
        } else {
            "yes"
        };
        output.push_str(&format!("Device Available          : {}\n", available));

        // Provides Device SDRs
        let provides_sdrs = if self.device_revision & IPM_DEV_DEVICE_ID_SDR_MASK != 0 {
            "yes"
        } else {
            "no"
        };
        output.push_str(&format!("Provides Device SDRs      : {}\n", provides_sdrs));

        // Additional Device Support
        output.push_str("Additional Device Support :\n");
        for i in 0..8 {
            if self.adtl_device_support & (1 << i) != 0 {
                let support_desc = get_additional_support_description(i);
                output.push_str(&format!("    {}\n", support_desc));
            }
        }

        // Aux Firmware Rev Info (if available)
        output.push_str("Aux Firmware Rev Info     : \n");
        for &aux_rev in &self.aux_fw_rev {
            output.push_str(&format!("    0x{:02x}\n", aux_rev));
        }

        output
    }
}

// Helper function to get manufacturer name (simplified version)
fn get_manufacturer_name(manufacturer_id: u32) -> &'static str {
    match manufacturer_id {
        2 => "IBM",
        7 => "Hitachi",
        11 => "Nokia",
        15 => "Dell Inc",
        19 => "Ericsson",
        42 => "Intel Corporation",
        343 => "Intel Corporation",
        5703 => "SUPERMICRO",
        _ => "Unknown",
    }
}

// Product information table based on ipmitool's ipmi_oem_product_info
struct ProductInfo {
    manufacturer_id: u32,
    product_id: u16,
    name: &'static str,
}

const PRODUCT_INFO_TABLE: &[ProductInfo] = &[
    // Intel products
    ProductInfo {
        manufacturer_id: 343,
        product_id: 0x000C,
        name: "TSRLT2",
    },
    ProductInfo {
        manufacturer_id: 343,
        product_id: 0x001B,
        name: "TIGPR2U",
    },
    ProductInfo {
        manufacturer_id: 343,
        product_id: 0x0022,
        name: "TIGI2U",
    },
    ProductInfo {
        manufacturer_id: 343,
        product_id: 0x0026,
        name: "Bridgeport",
    },
    ProductInfo {
        manufacturer_id: 343,
        product_id: 0x0028,
        name: "S5000PAL",
    },
    ProductInfo {
        manufacturer_id: 343,
        product_id: 0x0029,
        name: "S5000PSL",
    },
    ProductInfo {
        manufacturer_id: 343,
        product_id: 0x0100,
        name: "Tiger4",
    },
    ProductInfo {
        manufacturer_id: 343,
        product_id: 0x0103,
        name: "McCarran",
    },
    ProductInfo {
        manufacturer_id: 343,
        product_id: 0x0800,
        name: "ZT5504",
    },
    ProductInfo {
        manufacturer_id: 343,
        product_id: 0x0808,
        name: "MPCBL0001",
    },
    ProductInfo {
        manufacturer_id: 343,
        product_id: 0x0811,
        name: "TIGW1U",
    },
    ProductInfo {
        manufacturer_id: 343,
        product_id: 0x4311,
        name: "NSI2U",
    },
    // Kontron products
    ProductInfo {
        manufacturer_id: 15000,
        product_id: 4000,
        name: "AM4000 AdvancedMC",
    },
    ProductInfo {
        manufacturer_id: 15000,
        product_id: 4001,
        name: "AM4001 AdvancedMC",
    },
    ProductInfo {
        manufacturer_id: 15000,
        product_id: 4002,
        name: "AM4002 AdvancedMC",
    },
    ProductInfo {
        manufacturer_id: 15000,
        product_id: 4010,
        name: "AM4010 AdvancedMC",
    },
    ProductInfo {
        manufacturer_id: 15000,
        product_id: 5503,
        name: "AM4500/4520 AdvancedMC",
    },
    ProductInfo {
        manufacturer_id: 15000,
        product_id: 5504,
        name: "AM4300 AdvancedMC",
    },
    ProductInfo {
        manufacturer_id: 15000,
        product_id: 5507,
        name: "AM4301 AdvancedMC",
    },
    ProductInfo {
        manufacturer_id: 15000,
        product_id: 5508,
        name: "AM4330 AdvancedMC",
    },
    ProductInfo {
        manufacturer_id: 15000,
        product_id: 5520,
        name: "KTC5520/EATX",
    },
    ProductInfo {
        manufacturer_id: 15000,
        product_id: 6000,
        name: "CP6000",
    },
    ProductInfo {
        manufacturer_id: 15000,
        product_id: 6006,
        name: "DT-64",
    },
    ProductInfo {
        manufacturer_id: 15000,
        product_id: 6010,
        name: "CP6010",
    },
    ProductInfo {
        manufacturer_id: 15000,
        product_id: 6011,
        name: "CP6011",
    },
    ProductInfo {
        manufacturer_id: 15000,
        product_id: 6012,
        name: "CP6012",
    },
    ProductInfo {
        manufacturer_id: 15000,
        product_id: 6014,
        name: "CP6014",
    },
    // Supermicro products
    ProductInfo {
        manufacturer_id: 5703,
        product_id: 0x1000,
        name: "X9DRi-LN4+/X9DR3-LN4+",
    },
    ProductInfo {
        manufacturer_id: 5703,
        product_id: 0x1001,
        name: "X9SRi-F",
    },
    ProductInfo {
        manufacturer_id: 5703,
        product_id: 0x1002,
        name: "X9SCL/X9SCM",
    },
    ProductInfo {
        manufacturer_id: 5703,
        product_id: 0x1003,
        name: "X9DRW",
    },
];

// Helper function to get product name based on manufacturer_id and product_id
fn get_product_name(manufacturer_id: u32, product_id: u16) -> &'static str {
    for product in PRODUCT_INFO_TABLE {
        if product.manufacturer_id == manufacturer_id && product.product_id == product_id {
            return product.name;
        }
    }
    "" // Return empty string if not found, matching ipmitool behavior
}

// Helper function to get additional device support descriptions
fn get_additional_support_description(bit_position: u8) -> &'static str {
    match bit_position {
        0 => "Sensor Device",
        1 => "SDR Repository Device",
        2 => "SEL Device",
        3 => "FRU Inventory Device",
        4 => "IPMB Event Receiver",
        5 => "IPMB Event Generator",
        6 => "Bridge",
        7 => "Chassis Device",
        _ => "Reserved",
    }
}

fn ipmi_mc_get_device_id(intf: &mut dyn IpmiIntf) -> Result<String, String> {
    let rsp = ipmi_mc_get_device_id_raw(intf)?;

    if rsp.data_len < std::mem::size_of::<IpmDevidRsp>() as i32 {
        return Err("Invalid device ID response length".to_string());
    }

    // Convert response data to device ID structure
    let device_id: IpmDevidRsp = unsafe { std::ptr::read(rsp.data.as_ptr() as *const IpmDevidRsp) };

    Ok(device_id.format_device_info())
}

pub fn ipmi_mc_get_device_id_raw(
    intf: &mut dyn IpmiIntf,
) -> Result<crate::ipmi::ipmi::IpmiRs, String> {
    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_APP);
    req.msg.cmd = BMC_GET_DEVICE_ID;
    req.msg.data_len = 0;

    match intf.sendrecv(&req) {
        Some(rsp) => {
            if rsp.ccode != 0 {
                return Err(format!(
                    "Get Device ID command failed: completion code 0x{:02x}",
                    rsp.ccode
                ));
            }
            Ok(rsp)
        }
        None => Err("Get Device ID command failed: no response".to_string()),
    }
}

fn ipmi_mc_reset(intf: &mut dyn IpmiIntf, reset_type: &str) -> Result<String, String> {
    let (cmd, reset_name) = match reset_type {
        "warm" => (BMC_WARM_RESET, "warm"),
        "cold" => (BMC_COLD_RESET, "cold"),
        _ => return Err("Reset type must be 'warm' or 'cold'".to_string()),
    };

    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_APP);
    req.msg.cmd = cmd;
    req.msg.data_len = 0;

    // For cold reset, set no-answer flag
    if cmd == BMC_COLD_RESET {
        // Note: This would need to be implemented in the interface
        // intf.set_noanswer(true);
    }

    match intf.sendrecv(&req) {
        Some(rsp) => {
            if cmd == BMC_COLD_RESET {
                // For cold reset, BMC might not respond, which is expected
                Ok(format!("Sent {} reset command to MC", reset_name))
            } else if rsp.ccode != 0 {
                Err(format!(
                    "MC reset command failed: completion code 0x{:02x}",
                    rsp.ccode
                ))
            } else {
                Ok(format!("Sent {} reset command to MC", reset_name))
            }
        }
        None => {
            if cmd == BMC_COLD_RESET {
                // For cold reset, no response is expected and normal
                Ok(format!("Sent {} reset command to MC", reset_name))
            } else {
                Err("MC reset command failed: no response".to_string())
            }
        }
    }
}

pub fn ipmi_mc_main(subcmd: McCommand, mut intf: Box<dyn IpmiIntf>) -> Result<(), String> {
    match subcmd {
        McCommand::Info => {
            let result = ipmi_mc_get_device_id(intf.as_mut())?;
            print!("{}", result);
            Ok(())
        }
        McCommand::Reset { reset_type } => {
            let result = ipmi_mc_reset(intf.as_mut(), &reset_type)?;
            println!("{}", result);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_product_name_known_products() {
        // Test Intel products
        assert_eq!(get_product_name(343, 0x000C), "TSRLT2");
        assert_eq!(get_product_name(343, 0x001B), "TIGPR2U");
        assert_eq!(get_product_name(343, 0x0100), "Tiger4");

        // Test Kontron products
        assert_eq!(get_product_name(15000, 4000), "AM4000 AdvancedMC");
        assert_eq!(get_product_name(15000, 6000), "CP6000");

        // Test Supermicro products
        assert_eq!(get_product_name(5703, 0x1000), "X9DRi-LN4+/X9DR3-LN4+");
    }

    #[test]
    fn test_get_product_name_unknown_products() {
        // Test unknown products - should return empty string
        assert_eq!(get_product_name(999, 0x1234), "");
        assert_eq!(get_product_name(343, 0x9999), "");
        assert_eq!(get_product_name(0, 0), "");
    }

    #[test]
    fn test_get_manufacturer_name() {
        assert_eq!(get_manufacturer_name(2), "IBM");
        assert_eq!(get_manufacturer_name(15), "Dell Inc");
        assert_eq!(get_manufacturer_name(42), "Intel Corporation");
        assert_eq!(get_manufacturer_name(343), "Intel Corporation");
        assert_eq!(get_manufacturer_name(5703), "SUPERMICRO");
        assert_eq!(get_manufacturer_name(999), "Unknown");
    }
}
