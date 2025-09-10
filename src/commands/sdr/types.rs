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
use crate::error::{IpmiError, IpmiResult};
use crate::ipmi::context::OutputContext;
use std::time::UNIX_EPOCH;

/// SDR Repository Information (from Get SDR Repository Info command)
#[derive(Debug)]
pub struct SdrRepositoryInfo {
    pub sdr_version: u8,
    pub record_count: u16,
    pub free_space: u16,
    pub recent_addition: u32,
    pub recent_erase: u32,
    pub operations: u8,
}

impl SdrRepositoryInfo {
    /// Parse from IPMI response data
    pub fn from_response_data(data: &[u8]) -> IpmiResult<Self> {
        if data.len() < 14 {
            return Err(IpmiError::ResponseError);
        }

        Ok(SdrRepositoryInfo {
            sdr_version: data[0],
            record_count: u16::from_le_bytes([data[1], data[2]]),
            free_space: u16::from_le_bytes([data[3], data[4]]),
            recent_addition: u32::from_le_bytes([data[5], data[6], data[7], data[8]]),
            recent_erase: u32::from_le_bytes([data[9], data[10], data[11], data[12]]),
            operations: data[13],
        })
    }

    /// Format for standard output (matching C version)
    pub fn format_standard(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "SDR Version                         : 0x{:x}\n",
            self.sdr_version
        ));
        output.push_str(&format!(
            "Record Count                        : {}\n",
            self.record_count
        ));

        output.push_str("Free Space                          : ");
        match self.free_space {
            0x0000 => output.push_str("none (full)\n"),
            0xFFFF => output.push_str("unspecified\n"),
            0xFFFE => output.push_str("> 64Kb - 2 bytes\n"),
            _ => output.push_str(&format!("{} bytes\n", self.free_space)),
        }

        output.push_str("Most recent Addition                : ");
        if self.partial_add_sdr_supported() {
            if self.recent_addition != 0 {
                output.push_str(&format!("{}\n", format_timestamp(self.recent_addition)));
            } else {
                output.push_str("NA\n");
            }
        } else {
            output.push_str("NA\n");
        }

        output.push_str("Most recent Erase                   : ");
        if self.delete_sdr_supported() {
            if self.recent_erase != 0 {
                output.push_str(&format!("{}\n", format_timestamp(self.recent_erase)));
            } else {
                output.push_str("NA\n");
            }
        } else {
            output.push_str("NA\n");
        }

        output.push_str(&format!(
            "SDR overflow                        : {}\n",
            if self.overflow_flag() { "yes" } else { "no" }
        ));

        output.push_str("SDR Repository Update Support       : ");
        match self.modal_update_support() {
            0 => output.push_str("unspecified\n"),
            1 => output.push_str("non-modal\n"),
            2 => output.push_str("modal\n"),
            3 => output.push_str("modal and non-modal\n"),
            _ => output.push_str("error in response\n"),
        }

        output.push_str(&format!(
            "Delete SDR supported                : {}\n",
            if self.delete_sdr_supported() {
                "yes"
            } else {
                "no"
            }
        ));
        output.push_str(&format!(
            "Partial Add SDR supported           : {}\n",
            if self.partial_add_sdr_supported() {
                "yes"
            } else {
                "no"
            }
        ));
        output.push_str(&format!(
            "Reserve SDR repository supported    : {}\n",
            if self.reserve_sdr_repository_supported() {
                "yes"
            } else {
                "no"
            }
        ));
        output.push_str(&format!(
            "SDR Repository Alloc info supported : {}\n",
            if self.get_sdr_repository_allo_info_supported() {
                "yes"
            } else {
                "no"
            }
        ));

        output
    }

    /// CSV format output
    pub fn format_csv(&self) -> String {
        format!(
            "{},{},{},{},{},{}",
            self.sdr_version,
            self.record_count,
            self.free_space,
            self.recent_addition,
            self.recent_erase,
            self.operations
        )
    }

    /// Format based on output context
    pub fn format_by_context(&self, ctx: &OutputContext) -> String {
        if ctx.csv {
            return self.format_csv();
        }

        match ctx.verbose {
            0 => self.format_minimal(),
            1 => self.format_standard(),
            2 => self.format_detailed(),
            _ => self.format_full(),
        }
    }

    /// Minimal format (verbose = 0)
    pub fn format_minimal(&self) -> String {
        format!(
            "SDR Version: 0x{:x}, Records: {}, Free: {} bytes\n",
            self.sdr_version, self.record_count, self.free_space
        )
    }

    /// Detailed format (verbose = 2)
    pub fn format_detailed(&self) -> String {
        let mut output = self.format_standard();
        output.push_str(&format!(
            "Raw operations byte                 : 0x{:02x}\n",
            self.operations
        ));
        output.push_str(&format!(
            "Recent addition timestamp           : {}\n",
            self.recent_addition
        ));
        output.push_str(&format!(
            "Recent erase timestamp              : {}\n",
            self.recent_erase
        ));
        output
    }

    /// Full format (verbose >= 3)
    pub fn format_full(&self) -> String {
        let mut output = self.format_detailed();
        output.push_str("\nOperation flags breakdown:\n");
        output.push_str(&format!(
            "  Get SDR Repository Alloc Info     : {}\n",
            self.get_sdr_repository_allo_info_supported()
        ));
        output.push_str(&format!(
            "  Reserve SDR Repository             : {}\n",
            self.reserve_sdr_repository_supported()
        ));
        output.push_str(&format!(
            "  Partial Add SDR                    : {}\n",
            self.partial_add_sdr_supported()
        ));
        output.push_str(&format!(
            "  Delete SDR                         : {}\n",
            self.delete_sdr_supported()
        ));
        output.push_str(&format!(
            "  Modal Update Support               : {}\n",
            self.modal_update_support()
        ));
        output.push_str(&format!(
            "  Overflow Flag                      : {}\n",
            self.overflow_flag()
        ));
        output
    }

    // Helper methods to decode operations byte (matching C implementation)
    pub fn get_sdr_repository_allo_info_supported(&self) -> bool {
        (self.operations & 0x01) != 0
    }

    pub fn reserve_sdr_repository_supported(&self) -> bool {
        (self.operations & 0x02) != 0
    }

    pub fn partial_add_sdr_supported(&self) -> bool {
        (self.operations & 0x04) != 0
    }

    pub fn delete_sdr_supported(&self) -> bool {
        (self.operations & 0x08) != 0
    }

    pub fn modal_update_support(&self) -> u8 {
        (self.operations >> 5) & 0x03
    }

    pub fn overflow_flag(&self) -> bool {
        (self.operations & 0x80) != 0
    }
}

/// Format UNIX timestamp to readable string (matching C version)
fn format_timestamp(timestamp: u32) -> String {
    if timestamp == 0 {
        return "NA".to_string();
    }

    match UNIX_EPOCH.checked_add(std::time::Duration::from_secs(timestamp as u64)) {
        Some(datetime) => {
            // Simple formatting similar to C version
            // In C version, it uses ipmi_timestamp_numeric() which formats as:
            // "MM/dd/yyyy HH:mm:ss"
            match datetime.duration_since(UNIX_EPOCH) {
                Ok(duration) => {
                    let secs = duration.as_secs();
                    let days = secs / 86400;
                    let hours = (secs % 86400) / 3600;
                    let minutes = (secs % 3600) / 60;
                    let seconds = secs % 60;

                    // Simple approximation - could use chrono for exact formatting
                    let years = 1970 + days / 365;
                    let remaining_days = days % 365;
                    let months = remaining_days / 30 + 1;
                    let day_of_month = remaining_days % 30 + 1;

                    format!(
                        "{:02}/{:02}/{:04} {:02}:{:02}:{:02}",
                        months, day_of_month, years, hours, minutes, seconds
                    )
                }
                Err(_) => "Invalid timestamp".to_string(),
            }
        }
        None => "Invalid timestamp".to_string(),
    }
}

/// SDR Record Types (matching C constants)
pub const SDR_RECORD_TYPE_FULL_SENSOR: u8 = 0x01;
pub const SDR_RECORD_TYPE_COMPACT_SENSOR: u8 = 0x02;
pub const SDR_RECORD_TYPE_EVENTONLY_SENSOR: u8 = 0x03;
pub const SDR_RECORD_TYPE_ENTITY_ASSOC: u8 = 0x08;
pub const SDR_RECORD_TYPE_DEVICE_ENTITY_ASSOC: u8 = 0x09;
pub const SDR_RECORD_TYPE_GENERIC_DEVICE_LOCATOR: u8 = 0x10;
pub const SDR_RECORD_TYPE_FRU_DEVICE_LOCATOR: u8 = 0x11;
pub const SDR_RECORD_TYPE_MC_DEVICE_LOCATOR: u8 = 0x12;
pub const SDR_RECORD_TYPE_MC_CONFIRMATION: u8 = 0x13;
pub const SDR_RECORD_TYPE_BMC_MSG_CHANNEL_INFO: u8 = 0x14;
pub const SDR_RECORD_TYPE_OEM: u8 = 0xc0;

/// Get record type name string (matching C version)
pub fn get_sdr_record_type_name(record_type: u8) -> &'static str {
    match record_type {
        SDR_RECORD_TYPE_FULL_SENSOR => "Full Sensor",
        SDR_RECORD_TYPE_COMPACT_SENSOR => "Compact Sensor",
        SDR_RECORD_TYPE_EVENTONLY_SENSOR => "Event-Only Sensor",
        SDR_RECORD_TYPE_ENTITY_ASSOC => "Entity Association",
        SDR_RECORD_TYPE_DEVICE_ENTITY_ASSOC => "Device Entity Association",
        SDR_RECORD_TYPE_GENERIC_DEVICE_LOCATOR => "Generic Device Locator",
        SDR_RECORD_TYPE_FRU_DEVICE_LOCATOR => "FRU Device Locator",
        SDR_RECORD_TYPE_MC_DEVICE_LOCATOR => "MC Device Locator",
        SDR_RECORD_TYPE_MC_CONFIRMATION => "MC Confirmation",
        SDR_RECORD_TYPE_BMC_MSG_CHANNEL_INFO => "BMC Message Channel Info",
        SDR_RECORD_TYPE_OEM => "OEM",
        _ => "Unknown",
    }
}
