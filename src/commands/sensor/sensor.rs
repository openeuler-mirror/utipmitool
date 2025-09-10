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
use crate::commands::sdr::iter::SdrIterator;
use crate::commands::sdr::sdr::*;
use crate::commands::sdr::types::{SDR_RECORD_TYPE_COMPACT_SENSOR, SDR_RECORD_TYPE_FULL_SENSOR};
use crate::commands::sdr::{SdrRecordCommonSensor, SdrRecordEventonlySensor};
use crate::error::IpmiError;
use crate::ipmi::context::OutputContext;
use crate::ipmi::intf::IpmiIntf;
use crate::ipmi::ipmi::*;
use crate::{debug2, debug3, debug5};
use unpack::RAWDATA;

use ipmi_macros::AsBytes;
use std::error::Error;

// Threshold specification bits
pub const UPPER_NON_RECOV_SPECIFIED: u8 = 0x20;
pub const UPPER_CRIT_SPECIFIED: u8 = 0x10;
pub const UPPER_NON_CRIT_SPECIFIED: u8 = 0x08;
pub const LOWER_NON_RECOV_SPECIFIED: u8 = 0x04;
pub const LOWER_CRIT_SPECIFIED: u8 = 0x02;
pub const LOWER_NON_CRIT_SPECIFIED: u8 = 0x01;

// State assertion bits for discrete sensors
pub const STATE_0_ASSERTED: u8 = 0x01;
pub const STATE_1_ASSERTED: u8 = 0x02;
pub const STATE_2_ASSERTED: u8 = 0x04;
pub const STATE_3_ASSERTED: u8 = 0x08;
pub const STATE_4_ASSERTED: u8 = 0x10;
pub const STATE_5_ASSERTED: u8 = 0x20;
pub const STATE_6_ASSERTED: u8 = 0x40;
pub const STATE_7_ASSERTED: u8 = 0x80;
pub const STATE_8_ASSERTED: u8 = 0x01;
pub const STATE_9_ASSERTED: u8 = 0x02;
pub const STATE_10_ASSERTED: u8 = 0x04;
pub const STATE_11_ASSERTED: u8 = 0x08;
pub const STATE_12_ASSERTED: u8 = 0x10;
pub const STATE_13_ASSERTED: u8 = 0x20;
pub const STATE_14_ASSERTED: u8 = 0x40;

// Sensor linearization constants
pub const SDR_SENSOR_L_LINEAR: u8 = 0x00;
pub const SDR_SENSOR_L_LN: u8 = 0x01;
pub const SDR_SENSOR_L_LOG10: u8 = 0x02;
pub const SDR_SENSOR_L_LOG2: u8 = 0x03;
pub const SDR_SENSOR_L_E: u8 = 0x04;
pub const SDR_SENSOR_L_EXP10: u8 = 0x05;
pub const SDR_SENSOR_L_EXP2: u8 = 0x06;
pub const SDR_SENSOR_L_1_X: u8 = 0x07;
pub const SDR_SENSOR_L_SQR: u8 = 0x08;
pub const SDR_SENSOR_L_CUBE: u8 = 0x09;
pub const SDR_SENSOR_L_SQRT: u8 = 0x0a;
pub const SDR_SENSOR_L_CUBERT: u8 = 0x0b;
pub const SDR_SENSOR_L_NONLINEAR: u8 = 0x70;

#[derive(Debug, AsBytes, RAWDATA)]
#[repr(C)]
pub struct SdrThresholds {
    pub upper: SdrThresholdUpper,
    pub lower: SdrThresholdLower,
    pub hysteresis: SdrThresholdHysteresis,
}
#[derive(Debug, AsBytes, RAWDATA)]
#[repr(C)]
pub struct SdrThresholdUpper {
    pub non_recover: u8,
    pub critical: u8,
    pub non_critical: u8,
}
#[derive(Debug, AsBytes, RAWDATA)]
#[repr(C)]
pub struct SdrThresholdLower {
    pub non_recover: u8,
    pub critical: u8,
    pub non_critical: u8,
}
#[derive(Debug, AsBytes, RAWDATA)]
#[repr(C)]
pub struct SdrThresholdHysteresis {
    pub positive: u8,
    pub negative: u8,
}

//关键入口
pub fn ipmi_sensor_list(mut intf: Box<dyn IpmiIntf>) -> Result<(), Box<dyn Error>> {
    // 从IpmiIntf获取OutputContext（符合规范）
    let extended = intf.context().output.extended;
    debug3!("Querying SDR for sensor list, extended: {}", extended);

    // -vv级别的调试信息：显示SDR查询开始
    debug2!("Querying SDR for sensor list");

    // 在创建迭代器之前获取所有需要的SDR信息，避免借用冲突
    let sdr_info = get_sdr_repository_info(intf.as_mut());
    let reservation_id = crate::commands::sdr::sdr::ipmi_sdr_get_reservation(intf.as_mut(), false);

    // 显示SDR信息
    match sdr_info {
        Ok(info) => {
            debug2!("SDR free space: {}", info.free_space);
            debug2!("SDR records   : {}", info.record_count);
        }
        Err(_) => {
            debug2!("SDR free space: unknown");
            debug2!("SDR records   : unknown");
        }
    }

    // 显示SDR预留ID
    if let Some(res_id) = reservation_id {
        debug2!("SDR reservation ID {:04x}", res_id);
    }

    let iter_opt = SdrIterator::new(intf.as_mut(), false);

    if let Some(mut iter) = iter_opt {
        // 跟踪当前记录ID
        let mut current_record_id = 0u16;

        while let Some(header) = iter.next() {
            //next多次0x23获取header，用于读取record

            // -vv级别的调试信息：显示每个SDR记录的处理过程
            debug2!("SDR record ID   : 0x{:04x}", current_record_id);
            debug2!("SDR record type : 0x{:02x}", header.record_type);
            // 获取下一个记录ID（从迭代器的next_id字段）
            debug2!("SDR record next : 0x{:04x}", iter.next_id);
            debug2!("SDR record bytes: {}", header.length);

            let rec: Vec<u8> = match iter.ipmi_sdr_get_record(&header) {
                Some(r) => {
                    // -vv级别的调试信息：显示数据获取过程
                    debug2!("Getting 33 bytes from SDR at offset 5");
                    // 对于大部分传感器记录，都会有这个额外的读取操作
                    if header.record_type == SDR_RECORD_TYPE_FULL_SENSOR
                        || header.record_type == SDR_RECORD_TYPE_COMPACT_SENSOR
                    {
                        debug2!("Getting 2 bytes from SDR at offset 38");
                    }
                    r
                }
                None => return Err(Box::new(IpmiError::ResponseError)),
            };
            debug5!(
                "read SdrRecord from header.id:{} ,record_type:{}",
                header.id,
                header.record_type
            );
            // 处理传感器记录类型
            match header.record_type {
                SDR_RECORD_TYPE_FULL_SENSOR | SDR_RECORD_TYPE_COMPACT_SENSOR => {
                    match SdrRecordCommonSensor::from_le_bytes(&rec) {
                        Ok(sensor) => {
                            if sensor.is_threshold_sensor() {
                                ipmi_sensor_print_fc_threshold(iter.intf, &rec, header.record_type);
                            } else {
                                ipmi_sensor_print_fc_discrete(iter.intf, &rec, header.record_type);
                            }
                        }
                        Err(e) => {
                            debug5!("Error: Failed to parse sensor record:{}", e);
                            continue;
                        }
                    }
                }
                0x03 => {
                    // SDR_RECORD_TYPE_EVENTONLY_SENSOR
                    // Event-Only传感器也需要显示（匹配 ipmitool 行为）
                    ipmi_sensor_print_eventonly(iter.intf, &rec);
                }
                _ => {
                    // 跳过其他类型的记录
                }
            }

            // 更新当前记录ID
            current_record_id = header.id;
        }
    }

    Ok(())
}

/*
//关键入口
pub fn ipmi_sensor_print_fc_threshold(
    intf: &mut dyn IpmiIntf,
    sensor_raw: &[u8],
    sdr_record_type: u8,
) -> bool {
    let _ctx = intf.context().output_config().clone();
    let mut thresh_available = true;

    // Read sensor value
    let sr: SensorReading = match ipmi_sdr_read_sensor_value(intf, sensor_raw, sdr_record_type, 3) {
        Some(val) => {
            // Check if sensor name is valid
            let binding = String::from_utf8_lossy(&val.s_id);
            let sensor_name = binding.trim_matches('\0').trim();
            if sensor_name.is_empty() {
                debug5!("Threshold sensor has empty name but continuing with basic display");
                // Don't return false, continue to display this sensor with fallback info
            }
            val
        }
        None => {
            // If sensor parsing fails, try to show basic info from SDR record
            debug5!("Failed to read sensor value, attempting to show basic info from SDR");

            // Try to parse sensor info for display based on record type
            let sensor_name = match sdr_record_type {
                SDR_RECORD_TYPE_FULL_SENSOR => {
                    if let Ok(full_sensor) = SdrRecordFullSensor::from_le_bytes(sensor_raw) {
                        let id_len = (full_sensor.id_code & 0x1f) as usize;
                        if id_len > 0 && id_len <= 16 {
                            String::from_utf8_lossy(&full_sensor.id_string[..id_len])
                                .trim_matches('\0')
                                .trim()
                                .to_string()
                        } else {
                            "Unknown".to_string()
                        }
                    } else {
                        "Unknown".to_string()
                    }
                }
                SDR_RECORD_TYPE_COMPACT_SENSOR => {
                    if let Ok(compact_sensor) = SdrRecordCompactSensor::from_le_bytes(sensor_raw) {
                        let id_len = (compact_sensor.id_code & 0x1f) as usize;
                        if id_len > 0 && id_len <= 16 {
                            String::from_utf8_lossy(&compact_sensor.id_string[..id_len])
                                .trim_matches('\0')
                                .trim()
                                .to_string()
                        } else {
                            "Unknown".to_string()
                        }
                    } else {
                        "Unknown".to_string()
                    }
                }
                _ => "Unknown".to_string(),
            };

            // 放宽过滤条件：对于"Unknown"名称，尝试增强的名称提取
            let final_sensor_name = if sensor_name == "Unknown" || sensor_name.is_empty() {
                debug5!("Threshold sensor has Unknown/empty name, trying enhanced extraction");

                // 尝试从原始数据中提取名称
                let extracted_name =
                    extract_sensor_name_from_full_compact_data(sensor_raw, sdr_record_type);

                if !extracted_name.is_empty() && extracted_name != "Unknown" {
                    extracted_name
                } else {
                    format!(
                        "Sensor_{:02X}",
                        if let Ok(common_sensor) = SdrRecordCommonSensor::from_le_bytes(sensor_raw)
                        {
                            common_sensor.keys.sensor_num
                        } else {
                            0
                        }
                    )
                }
            } else {
                sensor_name
            };

            // Show sensor with 'no reading' like C version (简单格式)
            println!(
                "{:<16} | {:<17} | {:<6}",
                final_sensor_name, "no reading", "na"
            );
            return true;
        }
    };

    // Get threshold status
    //let thresh_status = sr.ipmi_sdr_get_thresh_status( "ns");

    // Get sensor thresholds
    //多次解析sensor对象
    let sensor = match SdrRecordCommonSensor::from_le_bytes(sensor_raw) {
        Ok(s) => s,
        Err(e) => {
            debug5!("Error: Failed to parse sensor record:{}", e);
            return false;
        }
    };

    // Get sensor thresholds and check response
    //3f, 0a, 05, 05, 5d, 62, 62
    let rsp = match ipmi_sdr_get_sensor_thresholds(
        intf,
        sensor.keys.sensor_num,
        sensor.keys.owner_id,
        sensor.keys.lun(),
        sensor.keys.channel(),
    ) {
        Some(r) if r.ccode == 0 && r.data_len > 0 => Some(r),
        _ => {
            thresh_available = false;
            None
        }
    };
    //print!("SensorReading: {:#?} ", sr);
    if let Some(ref response) = rsp {
        // 根据参数和OutputContext决定输出格式
        if _ctx.verbose >= 1 {
            // -v和-vv都显示完整详细格式（调试信息差异主要在SDR读取过程）
            ipmi_sensor_print_fc_threshold_verbose(
                intf,
                &sr,
                thresh_available,
                response,
                sensor_raw,
                sdr_record_type,
            );
        } else if _ctx.csv {
            // CSV格式输出
            let stdout = sr.dump_sensor_fc_threshold_csv(thresh_available, response, &_ctx);
            println!("{}", stdout);
        } else if _ctx.extended {
            // 扩展格式：显示传感器号、实体ID等额外信息（匹配C版本sdr_extended=1）
            let stdout = sr.dump_sensor_fc_threshold_extended(
                thresh_available,
                response,
                &_ctx,
                sensor_raw,
                sdr_record_type,
            );
            println!("{}", stdout);
        } else {
            // 默认格式：显示完整阈值信息（匹配C版本的默认行为）
            let stdout = sr.dump_sensor_fc_thredshold(thresh_available, response, &_ctx);
            println!("{}", stdout);
        }
    } else {
        // 没有阈值数据时的处理
        if _ctx.verbose >= 1 {
            // -v和-vv都显示详细格式，显示详细信息即使没有阈值数据
            let dummy_rsp = IpmiRs {
                ccode: 0,
                data: [0; IPMI_BUF_SIZE],
                data_len: 0,
                msg: IpmiRsMsg::default(),
                session: IpmiSession::default(),
                payload: IpmiRsPayload::IpmiResponse {
                    rq_addr: 0,
                    netfn: 0,
                    rq_lun: 0,
                    rs_addr: 0,
                    rq_seq: 0,
                    rs_lun: 0,
                    cmd: 0,
                },
            };
            ipmi_sensor_print_fc_threshold_verbose(
                intf,
                &sr,
                false,
                &dummy_rsp,
                sensor_raw,
                sdr_record_type,
            );
        } else {
            // 使用简单格式
            let dummy_rsp = IpmiRs {
                ccode: 0,
                data: [0; IPMI_BUF_SIZE],
                data_len: 0,
                msg: IpmiRsMsg::default(),
                session: IpmiSession::default(),
                payload: IpmiRsPayload::IpmiResponse {
                    rq_addr: 0,
                    netfn: 0,
                    rq_lun: 0,
                    rs_addr: 0,
                    rq_seq: 0,
                    rs_lun: 0,
                    cmd: 0,
                },
            };
            let stdout = sr.dump_sensor_fc_threshold_simple(false, &dummy_rsp, &_ctx);
            println!("{}", stdout);
        }
    }

    sr.s_reading_valid //返回是否成功
}
*/

//bgz
pub fn ipmi_sensor_print_fc_threshold(
    intf: &mut dyn IpmiIntf,
    sensor_raw: &[u8],
    sdr_record_type: u8,
) -> bool {
    let _ctx = intf.context().output_config().clone();
    let mut thresh_available = true;

    // Read sensor value
    let sr: SensorReading = match ipmi_sdr_read_sensor_value(intf, sensor_raw, sdr_record_type, 3) {
        Some(val) => {
            // Check if sensor name is valid
            let binding = String::from_utf8_lossy(&val.s_id);
            let sensor_name = binding.trim_matches('\0').trim();
            if sensor_name.is_empty() {
                debug5!("Threshold sensor has empty name but continuing with basic display");
                // Don't return false, continue to display this sensor with fallback info
            }
            val
        }
        None => {
            // If sensor parsing fails, try to show basic info from SDR record
            debug5!("Failed to read sensor value, attempting to show basic info from SDR");

            // Try to parse sensor info for display based on record type
            let sensor_name = match sdr_record_type {
                SDR_RECORD_TYPE_FULL_SENSOR => {
                    if let Ok(full_sensor) = SdrRecordFullSensor::from_le_bytes(sensor_raw) {
                        let id_len = (full_sensor.id_code & 0x1f) as usize;
                        if id_len > 0 && id_len <= 16 {
                            String::from_utf8_lossy(&full_sensor.id_string[..id_len])
                                .trim_matches('\0')
                                .trim()
                                .to_string()
                        } else {
                            "Unknown".to_string()
                        }
                    } else {
                        "Unknown".to_string()
                    }
                }
                SDR_RECORD_TYPE_COMPACT_SENSOR => {
                    if let Ok(compact_sensor) = SdrRecordCompactSensor::from_le_bytes(sensor_raw) {
                        let id_len = (compact_sensor.id_code & 0x1f) as usize;
                        if id_len > 0 && id_len <= 16 {
                            String::from_utf8_lossy(&compact_sensor.id_string[..id_len])
                                .trim_matches('\0')
                                .trim()
                                .to_string()
                        } else {
                            "Unknown".to_string()
                        }
                    } else {
                        "Unknown".to_string()
                    }
                }
                _ => "Unknown".to_string(),
            };

            // 放宽过滤条件：对于"Unknown"名称，尝试增强的名称提取
            let final_sensor_name = if sensor_name == "Unknown" || sensor_name.is_empty() {
                debug5!("Threshold sensor has Unknown/empty name, trying enhanced extraction");

                // 尝试从原始数据中提取名称
                let extracted_name =
                    extract_sensor_name_from_full_compact_data(sensor_raw, sdr_record_type);

                if !extracted_name.is_empty() && extracted_name != "Unknown" {
                    extracted_name
                } else {
                    format!(
                        "Sensor_{:02X}",
                        if let Ok(common_sensor) = SdrRecordCommonSensor::from_le_bytes(sensor_raw)
                        {
                            common_sensor.keys.sensor_num
                        } else {
                            0
                        }
                    )
                }
            } else {
                sensor_name
            };

            // Show sensor with 'no reading' like C version (简单格式)
            println!(
                "{:<16} | {:<17} | {:<6}",
                final_sensor_name, "no reading", "na"
            );
            return true;
        }
    };

    // Get threshold status
    //let thresh_status = sr.ipmi_sdr_get_thresh_status( "ns");

    // Get sensor thresholds
    //多次解析sensor对象
    let sensor = match SdrRecordCommonSensor::from_le_bytes(sensor_raw) {
        Ok(s) => s,
        Err(e) => {
            debug5!("Error: Failed to parse sensor record:{}", e);
            return false;
        }
    };

    // Get sensor thresholds and check response
    //3f, 0a, 05, 05, 5d, 62, 62
    let rsp = match ipmi_sdr_get_sensor_thresholds(
        intf,
        sensor.keys.sensor_num,
        sensor.keys.owner_id,
        sensor.keys.lun(),
        sensor.keys.channel(),
    ) {
        Some(r) if r.ccode == 0 && r.data_len > 0 => Some(r),
        _ => {
            thresh_available = false;
            None
        }
    };
    //print!("SensorReading: {:#?} ", sr);
    if let Some(ref response) = rsp {
        // 根据参数和OutputContext决定输出格式
        if _ctx.verbose >= 1 {
            // -v和-vv都显示完整详细格式（调试信息差异主要在SDR读取过程）
            ipmi_sensor_print_fc_threshold_verbose(
                intf,
                &sr,
                thresh_available,
                response,
                sensor_raw,
                sdr_record_type,
            );
        } else if _ctx.csv {
            // CSV格式输出
            let stdout = sr.dump_sensor_fc_threshold_csv(thresh_available, response, &_ctx);
            println!("{}", stdout);
        } else if _ctx.extended {
            // 扩展格式：显示传感器号、实体ID等额外信息（匹配C版本sdr_extended=1）
            let stdout = sr.dump_sensor_fc_threshold_extended(
                thresh_available,
                response,
                &_ctx,
                sensor_raw,
                sdr_record_type,
            );
            println!("{}", stdout);
        } else {
            // 默认格式：显示完整阈值信息（匹配C版本的默认行为）
            let stdout = sr.dump_sensor_fc_thredshold(thresh_available, response, &_ctx);
            println!("{}", stdout);
        }
    } else {
        // 没有阈值数据时的处理
        if _ctx.verbose >= 1 {
            // -v和-vv都显示详细格式，显示详细信息即使没有阈值数据
            let dummy_rsp = IpmiRs {
                ccode: 0,
                data: [0; IPMI_BUF_SIZE],
                data_len: 0,
                msg: IpmiRsMsg::default(),
                session: IpmiSession::default(),
                payload: IpmiRsPayload::IpmiResponse {
                    rq_addr: 0,
                    netfn: 0,
                    rq_lun: 0,
                    rs_addr: 0,
                    rq_seq: 0,
                    rs_lun: 0,
                    cmd: 0,
                },
            };
            ipmi_sensor_print_fc_threshold_verbose(
                intf,
                &sr,
                false,
                &dummy_rsp,
                sensor_raw,
                sdr_record_type,
            );
        } else {
            // 使用完整格式（匹配C版本默认行为）
            let dummy_rsp = IpmiRs {
                ccode: 0,
                data: [0; IPMI_BUF_SIZE],
                data_len: 0,
                msg: IpmiRsMsg::default(),
                session: IpmiSession::default(),
                payload: IpmiRsPayload::IpmiResponse {
                    rq_addr: 0,
                    netfn: 0,
                    rq_lun: 0,
                    rs_addr: 0,
                    rq_seq: 0,
                    rs_lun: 0,
                    cmd: 0,
                },
            };
            let stdout = sr.dump_sensor_fc_thredshold(false, &dummy_rsp, &_ctx);
            println!("{}", stdout);
        }
    }

    sr.s_reading_valid //返回是否成功
}

/*
pub fn ipmi_sensor_print_fc_discrete(
    intf: &mut dyn IpmiIntf,
    sensor_raw: &[u8],
    sdr_record_type: u8,
) -> bool {
    let _ctx = intf.context().output_config().clone();
    let sr: SensorReading = match ipmi_sdr_read_sensor_value(intf, sensor_raw, sdr_record_type, 3) {
        Some(val) => {
            // Check if sensor name is valid
            let binding = String::from_utf8_lossy(&val.s_id);
            let sensor_name = binding.trim_matches('\0').trim();
            if sensor_name.is_empty() {
                debug5!("Discrete sensor has empty name but continuing with basic display");
                // Don't return false, continue to display this sensor with fallback info
            }
            val
        }
        None => {
            // For discrete sensors, create a basic SensorReading even if data reading fails
            debug5!("Failed to read discrete sensor value, creating basic discrete sensor info");

            let mut sr = SensorReading::new();

            // Try to parse sensor info for display based on record type
            let sensor_name = match sdr_record_type {
                SDR_RECORD_TYPE_FULL_SENSOR => {
                    if let Ok(full_sensor) = SdrRecordFullSensor::from_le_bytes(sensor_raw) {
                        let mut id_len = (full_sensor.id_code & 0x1f) as usize;

                        // Enhanced name extraction logic
                        if id_len == 0 {
                            for i in 0..16 {
                                if full_sensor.id_string[i] == 0
                                    || full_sensor.id_string[i] < 0x20
                                    || full_sensor.id_string[i] > 0x7e
                                {
                                    id_len = i;
                                    break;
                                }
                                if i == 15 {
                                    if full_sensor
                                        .id_string
                                        .iter()
                                        .all(|&b| (0x20..=0x7e).contains(&b))
                                    {
                                        id_len = 16;
                                    } else {
                                        id_len = 0;
                                    }
                                }
                            }
                        }

                        if id_len > 0 && id_len <= 16 {
                            String::from_utf8_lossy(&full_sensor.id_string[..id_len])
                                .trim_matches('\0')
                                .trim()
                                .to_string()
                        } else {
                            "Unknown".to_string()
                        }
                    } else {
                        "Unknown".to_string()
                    }
                }
                SDR_RECORD_TYPE_COMPACT_SENSOR => {
                    if let Ok(compact_sensor) = SdrRecordCompactSensor::from_le_bytes(sensor_raw) {
                        let mut id_len = (compact_sensor.id_code & 0x1f) as usize;

                        // Enhanced name extraction logic
                        if id_len == 0 {
                            for i in 0..16 {
                                if compact_sensor.id_string[i] == 0
                                    || compact_sensor.id_string[i] < 0x20
                                    || compact_sensor.id_string[i] > 0x7e
                                {
                                    id_len = i;
                                    break;
                                }
                                if i == 15 {
                                    if compact_sensor
                                        .id_string
                                        .iter()
                                        .all(|&b| (0x20..=0x7e).contains(&b))
                                    {
                                        id_len = 16;
                                    } else {
                                        id_len = 0;
                                    }
                                }
                            }
                        }

                        if id_len > 0 && id_len <= 16 {
                            String::from_utf8_lossy(&compact_sensor.id_string[..id_len])
                                .trim_matches('\0')
                                .trim()
                                .to_string()
                        } else {
                            "Unknown".to_string()
                        }
                    } else {
                        "Unknown".to_string()
                    }
                }
                _ => "Unknown".to_string(),
            };

            // 放宽过滤条件：对于"Unknown"名称，生成基于传感器号的替代名称
            let final_sensor_name = if sensor_name == "Unknown" || sensor_name.is_empty() {
                debug5!("Discrete sensor has Unknown/empty name, trying enhanced extraction");

                // 尝试从原始数据中提取名称
                let extracted_name =
                    extract_sensor_name_from_full_compact_data(sensor_raw, sdr_record_type);

                if !extracted_name.is_empty() && extracted_name != "Unknown" {
                    extracted_name
                } else {
                    format!(
                        "Discrete_{:02X}",
                        if let Ok(common_sensor) = SdrRecordCommonSensor::from_le_bytes(sensor_raw)
                        {
                            common_sensor.keys.sensor_num
                        } else {
                            0
                        }
                    )
                }
            } else {
                sensor_name
            };

            // Set sensor name in SensorReading
            let name_bytes = final_sensor_name.as_bytes();
            let copy_len = name_bytes.len().min(sr.s_id.len() - 1);
            sr.s_id[..copy_len].copy_from_slice(&name_bytes[..copy_len]);

            // Mark as discrete sensor with basic info
            sr.s_reading_valid = false;
            sr.s_reading_unavailable = true;
            sr.s_has_analog_value = false;

            // Try to get sensor reading for discrete sensors
            if let Ok(common_sensor) = SdrRecordCommonSensor::from_le_bytes(sensor_raw) {
                let rsp = ipmi_sdr_get_sensor_reading_ipmb(
                    intf,
                    common_sensor.keys.sensor_num,
                    common_sensor.keys.owner_id,
                    common_sensor.keys.lun(),
                    common_sensor.keys.channel(),
                );

                if let Some(reading_rsp) = rsp {
                    if reading_rsp.ccode == 0 && reading_rsp.data_len >= 1 {
                        sr.s_reading = reading_rsp.data[0];
                        sr.s_reading_valid = true;
                        if reading_rsp.data_len >= 2 {
                            sr.s_data2 = reading_rsp.data[2];
                        }
                        if reading_rsp.data_len >= 3 {
                            sr.s_data3 = reading_rsp.data[3];
                        }
                    }
                }
            }

            sr
        }
    };

    // 构建输出字符串 - 根据verbose模式选择格式
    let binding = String::from_utf8_lossy(&sr.s_id);
    let sensor_name = binding.trim_matches('\0').trim();

    if _ctx.verbose >= 1 {
        // -v和-vv都显示详细格式（调试信息差异主要在SDR读取过程）
        ipmi_sensor_print_fc_discrete_verbose(intf, &sr, sensor_raw, sdr_record_type);
    } else {
        // 默认表格格式: name | reading | discrete | status | na | na | na | na | na | na
        if sr.s_reading_valid {
            // 显示十六进制读数值
            let status_value = ((sr.s_data3 as u16) << 8) | (sr.s_data2 as u16);
            println!(
                "{:<16} | 0x{:<8x} | {:<10} | 0x{:04x}| {:<9} | {:<9} | {:<9} | {:<9} | {:<9} | {:<9}",
                sensor_name, sr.s_reading, "discrete", status_value, "na", "na", "na", "na", "na", "na"
            );
        } else {
            // 无读数的离散传感器
            println!(
                "{:<16} | 0x{:<8x} | {:<10} | 0x{:04x}| {:<9} | {:<9} | {:<9} | {:<9} | {:<9} | {:<9}",
                sensor_name,
                0,
                "discrete",
                0x0080, // Default status
                "na",
                "na",
                "na",
                "na",
                "na",
                "na"
            );
        }
    }

    true
}
*/

//bgz
pub fn ipmi_sensor_print_fc_discrete(
    intf: &mut dyn IpmiIntf,
    sensor_raw: &[u8],
    sdr_record_type: u8,
) -> bool {
    let _ctx = intf.context().output_config().clone();
    let sr: SensorReading = match ipmi_sdr_read_sensor_value(intf, sensor_raw, sdr_record_type, 3) {
        Some(val) => {
            // Check if sensor name is valid
            let binding = String::from_utf8_lossy(&val.s_id);
            let sensor_name = binding.trim_matches('\0').trim();
            if sensor_name.is_empty() {
                debug5!("Discrete sensor has empty name but continuing with basic display");
                // Don't return false, continue to display this sensor with fallback info
            }
            val
        }
        None => {
            // For discrete sensors, create a basic SensorReading even if data reading fails
            debug5!("Failed to read discrete sensor value, creating basic discrete sensor info");

            let mut sr = SensorReading::new();

            // Try to parse sensor info for display based on record type
            let sensor_name = match sdr_record_type {
                SDR_RECORD_TYPE_FULL_SENSOR => {
                    if let Ok(full_sensor) = SdrRecordFullSensor::from_le_bytes(sensor_raw) {
                        let mut id_len = (full_sensor.id_code & 0x1f) as usize;

                        // Enhanced name extraction logic
                        if id_len == 0 {
                            for i in 0..16 {
                                if full_sensor.id_string[i] == 0
                                    || full_sensor.id_string[i] < 0x20
                                    || full_sensor.id_string[i] > 0x7e
                                {
                                    id_len = i;
                                    break;
                                }
                                if i == 15 {
                                    if full_sensor
                                        .id_string
                                        .iter()
                                        .all(|&b| (0x20..=0x7e).contains(&b))
                                    {
                                        id_len = 16;
                                    } else {
                                        id_len = 0;
                                    }
                                }
                            }
                        }

                        if id_len > 0 && id_len <= 16 {
                            String::from_utf8_lossy(&full_sensor.id_string[..id_len])
                                .trim_matches('\0')
                                .trim()
                                .to_string()
                        } else {
                            "Unknown".to_string()
                        }
                    } else {
                        "Unknown".to_string()
                    }
                }
                SDR_RECORD_TYPE_COMPACT_SENSOR => {
                    if let Ok(compact_sensor) = SdrRecordCompactSensor::from_le_bytes(sensor_raw) {
                        let mut id_len = (compact_sensor.id_code & 0x1f) as usize;

                        // Enhanced name extraction logic
                        if id_len == 0 {
                            for i in 0..16 {
                                if compact_sensor.id_string[i] == 0
                                    || compact_sensor.id_string[i] < 0x20
                                    || compact_sensor.id_string[i] > 0x7e
                                {
                                    id_len = i;
                                    break;
                                }
                                if i == 15 {
                                    if compact_sensor
                                        .id_string
                                        .iter()
                                        .all(|&b| (0x20..=0x7e).contains(&b))
                                    {
                                        id_len = 16;
                                    } else {
                                        id_len = 0;
                                    }
                                }
                            }
                        }

                        if id_len > 0 && id_len <= 16 {
                            String::from_utf8_lossy(&compact_sensor.id_string[..id_len])
                                .trim_matches('\0')
                                .trim()
                                .to_string()
                        } else {
                            "Unknown".to_string()
                        }
                    } else {
                        "Unknown".to_string()
                    }
                }
                _ => "Unknown".to_string(),
            };

            // 放宽过滤条件：对于"Unknown"名称，生成基于传感器号的替代名称
            let final_sensor_name = if sensor_name == "Unknown" || sensor_name.is_empty() {
                debug5!("Discrete sensor has Unknown/empty name, trying enhanced extraction");

                // 尝试从原始数据中提取名称
                let extracted_name =
                    extract_sensor_name_from_full_compact_data(sensor_raw, sdr_record_type);

                if !extracted_name.is_empty() && extracted_name != "Unknown" {
                    extracted_name
                } else {
                    format!(
                        "Discrete_{:02X}",
                        if let Ok(common_sensor) = SdrRecordCommonSensor::from_le_bytes(sensor_raw)
                        {
                            common_sensor.keys.sensor_num
                        } else {
                            0
                        }
                    )
                }
            } else {
                sensor_name
            };

            // Set sensor name in SensorReading
            let name_bytes = final_sensor_name.as_bytes();
            let copy_len = name_bytes.len().min(sr.s_id.len() - 1);
            sr.s_id[..copy_len].copy_from_slice(&name_bytes[..copy_len]);

            // Mark as discrete sensor with basic info
            sr.s_reading_valid = false;
            sr.s_reading_unavailable = true;
            sr.s_has_analog_value = false;

            // Try to get sensor reading for discrete sensors
            if let Ok(common_sensor) = SdrRecordCommonSensor::from_le_bytes(sensor_raw) {
                let rsp = ipmi_sdr_get_sensor_reading_ipmb(
                    intf,
                    common_sensor.keys.sensor_num,
                    common_sensor.keys.owner_id,
                    common_sensor.keys.lun(),
                    common_sensor.keys.channel(),
                );

                if let Some(reading_rsp) = rsp {
                    if reading_rsp.ccode == 0 && reading_rsp.data_len >= 1 {
                        sr.s_reading = reading_rsp.data[0];
                        sr.s_reading_valid = true;
                        if reading_rsp.data_len >= 2 {
                            sr.s_data2 = reading_rsp.data[2];
                        }
                        if reading_rsp.data_len >= 3 {
                            sr.s_data3 = reading_rsp.data[3];
                        }
                    }
                }
            }

            sr
        }
    };

    // 构建输出字符串 - 根据verbose模式选择格式
    let binding = String::from_utf8_lossy(&sr.s_id);
    let sensor_name = binding.trim_matches('\0').trim();

    if _ctx.verbose >= 1 {
        // -v和-vv都显示详细格式（调试信息差异主要在SDR读取过程）
        ipmi_sensor_print_fc_discrete_verbose(intf, &sr, sensor_raw, sdr_record_type);
    } else {
        /*
            // 使用与原始 ipmitool 一致的简单格式
            if sr.s_reading_valid {
                println!(
                    "{:<16} | {:<17} | {:<6}",
                    sensor_name, "discrete", "ok"
                );
            } else {
                // 无读数的离散传感器
                println!(
                    "{:<16} | {:<17} | {:<6}",
                    sensor_name, "no reading", "ns"
                );
            }
        */

        //bgz  上面if else替换为下面内容
        // 在ipmi_sensor_print_fc_discrete函数中 - 使用10列格式匹配ipmitool
        if sr.s_reading_valid {
            // 状态值格式完全匹配C版本：0x{data2:02x}{data3:02x}
            let status_str = format!("0x{:02x}{:02x}", sr.s_data2, sr.s_data3);

            println!(
                "{:<16} | 0x{:<8x} | {:<10} | {:<6}| {:<9} | {:<9} | {:<9} | {:<9} | {:<9} | {:<9} ",
                sensor_name,
                sr.s_reading,
                "discrete",
                status_str,
                "na",
                "na",
                "na",
                "na",
                "na",
                "na"
            );
        } else {
            // 无读数的离散传感器也使用10列格式
            println!(
                "{:<16} | 0x{:<8x} | {:<10} | {:<6}| {:<9} | {:<9} | {:<9} | {:<9} | {:<9} | {:<9}",
                sensor_name,
                0,
                "discrete",
                "na", // 匹配ipmitool的na状态
                "na",
                "na",
                "na",
                "na",
                "na",
                "na"
            );
        }
    }

    true
}

/*
pub fn ipmi_sensor_print_eventonly(intf: &mut dyn IpmiIntf, sensor_raw: &[u8]) -> bool {
    let _ctx = intf.context().output_config().clone();
    let sensor = match SdrRecordEventonlySensor::from_le_bytes(sensor_raw) {
        Ok(s) => s,
        Err(e) => {
            debug5!("Error: Failed to parse event-only sensor record: {}", e);
            return false;
        }
    };

    // 增强的传感器名称提取逻辑
    let mut id_len = (sensor.id_code & 0x1f) as usize;

    // 第一次尝试：使用id_code指定的长度
    if id_len == 0 {
        // 第二次尝试：扫描整个id_string数组找到实际的字符串长度
        for i in 0..16.min(sensor.id_string.len()) {
            if sensor.id_string[i] == 0 {
                id_len = i;
                break;
            }
            // 检查是否为可打印ASCII字符
            if sensor.id_string[i] < 0x20 || sensor.id_string[i] > 0x7e {
                id_len = i;
                break;
            }
            if i == 15 {
                // 到了末尾，检查所有字符是否都是可打印的
                if sensor.id_string.iter().all(|&b| (0x20..=0x7e).contains(&b)) {
                    id_len = 16;
                } else {
                    id_len = 0;
                }
            }
        }
    }

    // 第三次尝试：从传感器原始数据中直接提取名称
    let sensor_name = if id_len > 0 && id_len <= sensor.id_string.len() {
        let extracted_name = String::from_utf8_lossy(&sensor.id_string[..id_len])
            .trim_matches('\0')
            .trim()
            .to_string();

        // 验证提取的名称是否有效
        if !extracted_name.is_empty()
            && extracted_name
                .chars()
                .all(|c| c.is_ascii() && !c.is_control())
        {
            extracted_name
        } else {
            // 第四次尝试：尝试从原始数据的不同偏移位置提取名称
            extract_sensor_name_from_raw_data(sensor_raw, sensor.keys.sensor_num)
        }
    } else {
        // 第四次尝试：尝试从原始数据的不同偏移位置提取名称
        extract_sensor_name_from_raw_data(sensor_raw, sensor.keys.sensor_num)
    };

    // 第五次尝试：使用传感器号生成fallback名称（仅当其他都失败时）
    let final_sensor_name = if sensor_name.is_empty() || sensor_name == "Unknown" {
        debug5!("Event-only sensor name extraction failed, using sensor number fallback");
        format!("Event_{:02X}", sensor.keys.sensor_num)
    } else {
        sensor_name
    };

    // Try to get sensor reading for event-only sensors
    let mut reading_value = 0u8;
    let mut status_value = 0u16;
    let mut has_reading = false;

    let rsp = ipmi_sdr_get_sensor_reading_ipmb(
        intf,
        sensor.keys.sensor_num,
        sensor.keys.owner_id,
        sensor.keys.lun(),
        sensor.keys.channel(),
    );

    if let Some(reading_rsp) = rsp {
        if reading_rsp.ccode == 0 && reading_rsp.data_len >= 1 {
            reading_value = reading_rsp.data[0];
            has_reading = true;
            if reading_rsp.data_len >= 2 {
                status_value = reading_rsp.data[1] as u16;
                if reading_rsp.data_len >= 3 {
                    status_value |= (reading_rsp.data[2] as u16) << 8;
                }
            }
        }
    }

    // Format output based on verbose mode
    if _ctx.verbose >= 1 {
        // -v和-vv都显示详细格式（调试信息差异主要在SDR读取过程）
        ipmi_sensor_print_eventonly_verbose(
            intf,
            &final_sensor_name,
            sensor_raw,
            reading_value,
            status_value,
            has_reading,
        );
    } else {
        // 默认表格格式
        if has_reading {
            println!(
                "{:<16} | 0x{:<8x} | {:<10} | 0x{:04x}| {:<9} | {:<9} | {:<9} | {:<9} | {:<9} | {:<9}",
                final_sensor_name,
                reading_value,
                "discrete",
                status_value,
                "na",
                "na",
                "na",
                "na",
                "na",
                "na"
            );
        } else {
            println!(
                "{:<16} | 0x{:<8x} | {:<10} | 0x{:04x}| {:<9} | {:<9} | {:<9} | {:<9} | {:<9} | {:<9}",
                final_sensor_name,
                0,
                "discrete",
                0x0080, // 匹配ipmitool的默认状态
                "na",
                "na",
                "na",
                "na",
                "na",
                "na"
            );
        }
    }

    true
}
*/

//bgz
pub fn ipmi_sensor_print_eventonly(intf: &mut dyn IpmiIntf, sensor_raw: &[u8]) -> bool {
    let _ctx = intf.context().output_config().clone();
    let sensor = match SdrRecordEventonlySensor::from_le_bytes(sensor_raw) {
        Ok(s) => s,
        Err(e) => {
            debug5!("Error: Failed to parse event-only sensor record: {}", e);
            return false;
        }
    };

    // 增强的传感器名称提取逻辑
    let mut id_len = (sensor.id_code & 0x1f) as usize;

    // 第一次尝试：使用id_code指定的长度
    if id_len == 0 {
        // 第二次尝试：扫描整个id_string数组找到实际的字符串长度
        for i in 0..16.min(sensor.id_string.len()) {
            if sensor.id_string[i] == 0 {
                id_len = i;
                break;
            }
            // 检查是否为可打印ASCII字符
            if sensor.id_string[i] < 0x20 || sensor.id_string[i] > 0x7e {
                id_len = i;
                break;
            }
            if i == 15 {
                // 到了末尾，检查所有字符是否都是可打印的
                if sensor.id_string.iter().all(|&b| (0x20..=0x7e).contains(&b)) {
                    id_len = 16;
                } else {
                    id_len = 0;
                }
            }
        }
    }

    // 第三次尝试：从传感器原始数据中直接提取名称
    let sensor_name = if id_len > 0 && id_len <= sensor.id_string.len() {
        let extracted_name = String::from_utf8_lossy(&sensor.id_string[..id_len])
            .trim_matches('\0')
            .trim()
            .to_string();

        // 验证提取的名称是否有效
        if !extracted_name.is_empty()
            && extracted_name
                .chars()
                .all(|c| c.is_ascii() && !c.is_control())
        {
            extracted_name
        } else {
            // 第四次尝试：尝试从原始数据的不同偏移位置提取名称
            extract_sensor_name_from_raw_data(sensor_raw, sensor.keys.sensor_num)
        }
    } else {
        // 第四次尝试：尝试从原始数据的不同偏移位置提取名称
        extract_sensor_name_from_raw_data(sensor_raw, sensor.keys.sensor_num)
    };

    // 第五次尝试：使用传感器号生成fallback名称（仅当其他都失败时）
    let final_sensor_name = if sensor_name.is_empty() || sensor_name == "Unknown" {
        debug5!("Event-only sensor name extraction failed, using sensor number fallback");
        format!("Event_{:02X}", sensor.keys.sensor_num)
    } else {
        sensor_name
    };

    // 新增：传感器名称过滤 - 跳过特定名称的传感器
    let skip_sensors = [
        "BIOSFRB2",
        "BIOSPOST",
        "OSLoad",
        "SMSOS",
        "OEM",
        "Boot_Up",
        "POWER_ON",
        "POWER_OFF",
        "POWER_CYCLE",
        "POWER_RESET",
    ];

    if skip_sensors.iter().any(|&name| final_sensor_name == name) {
        debug5!(
            "Skipping event-only sensor with filtered name: {}",
            final_sensor_name
        );
        return true; // 返回true表示处理成功，但不显示
    }

    // 新增：传感器类型过滤 - 跳过特定类型的传感器
    let skip_sensor_types = [0x12, 0x1D, 0x1F, 0x20]; // 系统事件，系统启动等类型

    if skip_sensor_types.contains(&sensor.sensor_type) {
        debug5!(
            "Skipping event-only sensor with filtered type: 0x{:02x}",
            sensor.sensor_type
        );
        return true;
    }

    // Try to get sensor reading for event-only sensors
    let mut reading_value = 0u8;
    let mut status_value = 0u16;
    let mut has_reading = false;

    let rsp = ipmi_sdr_get_sensor_reading_ipmb(
        intf,
        sensor.keys.sensor_num,
        sensor.keys.owner_id,
        sensor.keys.lun(),
        sensor.keys.channel(),
    );

    if let Some(reading_rsp) = rsp {
        if reading_rsp.ccode == 0 && reading_rsp.data_len >= 1 {
            reading_value = reading_rsp.data[0];
            has_reading = true;
            if reading_rsp.data_len >= 2 {
                // 修正Event-Only传感器状态值计算：与ipmitool兼容，data[1]在高位，data[2]在低位 (0x{data[1]}{data[2]})
                status_value = (reading_rsp.data[1] as u16) << 8;
                if reading_rsp.data_len >= 3 {
                    status_value |= reading_rsp.data[2] as u16;
                }
            }
        }
    }

    // Format output based on verbose mode
    if _ctx.verbose >= 1 {
        // -v和-vv都显示详细格式（调试信息差异主要在SDR读取过程）
        ipmi_sensor_print_eventonly_verbose(
            intf,
            &final_sensor_name,
            sensor_raw,
            reading_value,
            status_value,
            has_reading,
        );
    } else {
        // 使用与原始 ipmitool 一致的10列格式（匹配离散传感器格式）
        if has_reading {
            println!(
                "{:<16} | 0x{:<8x} | {:<10} | 0x{:04x}| {:<9} | {:<9} | {:<9} | {:<9} | {:<9} | {:<9}",
                final_sensor_name,
                reading_value,
                "discrete",
                status_value,
                "na",
                "na",
                "na",
                "na",
                "na",
                "na"
            );
        } else {
            println!(
                "{:<16} | 0x{:<8x} | {:<10} | 0x{:04x}| {:<9} | {:<9} | {:<9} | {:<9} | {:<9} | {:<9}",
                final_sensor_name,
                0,
                "discrete",
                0x0080, // 匹配ipmitool的默认状态
                "na",
                "na",
                "na",
                "na",
                "na",
                "na"
            );
        }
    }

    true
}

// ================================
// Verbose模式的输出函数实现
// ================================

/// Verbose模式下的阈值传感器输出（匹配ipmitool风格）
pub fn ipmi_sensor_print_fc_threshold_verbose(
    intf: &mut dyn IpmiIntf,
    sr: &SensorReading,
    thresh_available: bool,
    _rsp: &IpmiRs,
    sensor_raw: &[u8],
    sdr_record_type: u8,
) {
    let binding = String::from_utf8_lossy(&sr.s_id);
    let sensor_name = binding.trim_matches('\0').trim();

    // 获取传感器信息
    let (sensor_num, entity_id, entity_instance, sensor_type) =
        extract_sensor_info(sensor_raw, sdr_record_type);

    // 打印传感器ID行
    println!(
        "Sensor ID              : {} (0x{:x})",
        sensor_name, sensor_num
    );
    println!(" Entity ID             : {}.{}", entity_id, entity_instance);

    // 获取传感器类型描述
    let sensor_type_desc = get_sensor_type_description(sensor_type);
    println!(" Sensor Type (Threshold)  : {}", sensor_type_desc);

    // 打印传感器读数
    if sr.s_reading_valid {
        if sr.s_has_analog_value {
            // 模拟值传感器
            println!(
                " Sensor Reading        : {} (+/- 0) {}",
                if sr.s_a_val.fract() == 0.0 {
                    format!("{}", sr.s_a_val as i32)
                } else {
                    format!("{:.3}", sr.s_a_val)
                },
                sr.s_a_units
            );

            // 获取状态
            let ctx = intf.context().output_config().clone();
            let status = sr.ipmi_sdr_get_thresh_status("na", &ctx);
            println!(" Status                : {}", status);
        } else {
            // 离散传感器值
            println!(" Sensor Reading        : 0x{:x}", sr.s_reading);
        }
    } else {
        // 无效读数
        println!(" Sensor Reading        :  Unable to read sensor: Device Not Present");
        println!();
        println!(" Event Status          : Unavailable");
    }

    // 打印阈值信息
    if thresh_available && sr.s_reading_valid {
        // 只有当传感器读数有效时才显示阈值信息
        if let Ok(sensor) = SdrRecordCommonSensor::from_le_bytes(sensor_raw) {
            let thresh_rsp = ipmi_sdr_get_sensor_thresholds(
                intf,
                sensor.keys.sensor_num,
                sensor.keys.owner_id,
                sensor.keys.lun(),
                sensor.keys.channel(),
            );

            if let Some(rsp) = thresh_rsp {
                if rsp.ccode == 0 && rsp.data_len > 0 {
                    // 定义阈值映射 (使用原来的逻辑但修复格式)
                    let thresholds = [
                        (LOWER_NON_RECOV_SPECIFIED, 3, "Lower Non-Recoverable"),
                        (LOWER_CRIT_SPECIFIED, 2, "Lower Critical"),
                        (LOWER_NON_CRIT_SPECIFIED, 1, "Lower Non-Critical"),
                        (UPPER_NON_CRIT_SPECIFIED, 4, "Upper Non-Critical"),
                        (UPPER_CRIT_SPECIFIED, 5, "Upper Critical"),
                        (UPPER_NON_RECOV_SPECIFIED, 6, "Upper Non-Recoverable"),
                    ];

                    for (bit_mask, data_idx, name) in thresholds {
                        let is_avail = rsp.data[0] & bit_mask != 0;
                        if is_avail && data_idx < rsp.data.len() {
                            let data = rsp.data[data_idx];
                            if let Some(ref full) = sr.full {
                                let thresh_val = full.print_thresh_setting(true, data, "");
                                println!(" {:<21} : {}", name, thresh_val.trim());
                            } else {
                                println!(" {:<21} : na", name);
                            }
                        } else {
                            println!(" {:<21} : na", name);
                        }
                    }
                }
            }
        }

        // 打印滞回信息（只有在阈值可用时）
        println!(" {:<21} : Unspecified", "Positive Hysteresis");
        println!(" {:<21} : Unspecified", "Negative Hysteresis");
        println!(" {:<21} : ", "Assertion Events");
    }

    // 打印断言使能状态
    if sr.s_reading_valid {
        // 根据传感器实际支持的阈值显示断言状态
        if let Ok(sensor) = SdrRecordCommonSensor::from_le_bytes(sensor_raw) {
            let thresh_rsp = ipmi_sdr_get_sensor_thresholds(
                intf,
                sensor.keys.sensor_num,
                sensor.keys.owner_id,
                sensor.keys.lun(),
                sensor.keys.channel(),
            );

            if let Some(rsp) = thresh_rsp {
                if rsp.ccode == 0 && rsp.data_len > 0 {
                    let mut assertions = Vec::new();
                    let mut deassertions = Vec::new();

                    // 只显示传感器实际支持的阈值断言状态
                    if rsp.data[0] & LOWER_NON_CRIT_SPECIFIED != 0 {
                        assertions.push("lnc-");
                        deassertions.push("lnc-");
                    }
                    if rsp.data[0] & LOWER_CRIT_SPECIFIED != 0 {
                        assertions.push("lcr-");
                        deassertions.push("lcr-");
                    }
                    if rsp.data[0] & LOWER_NON_RECOV_SPECIFIED != 0 {
                        assertions.push("lnr-");
                        deassertions.push("lnr-");
                    }
                    if rsp.data[0] & UPPER_NON_CRIT_SPECIFIED != 0 {
                        assertions.push("unc+");
                        deassertions.push("unc+");
                    }
                    if rsp.data[0] & UPPER_CRIT_SPECIFIED != 0 {
                        assertions.push("ucr+");
                        deassertions.push("ucr+");
                    }
                    if rsp.data[0] & UPPER_NON_RECOV_SPECIFIED != 0 {
                        assertions.push("unr+");
                        deassertions.push("unr+");
                    }

                    println!(" {:<21} : {} ", "Assertions Enabled", assertions.join(" "));
                    println!(
                        " {:<21} : {} ",
                        "Deassertions Enabled",
                        deassertions.join(" ")
                    );
                } else {
                    println!(" {:<21} : ", "Assertions Enabled");
                }
            } else {
                println!(" {:<21} : ", "Assertions Enabled");
            }
        } else {
            println!(" {:<21} : ", "Assertions Enabled");
        }
    } else {
        // 对于无效读数的传感器，不显示所有阈值，而是查询实际支持的阈值
        if let Ok(sensor) = SdrRecordCommonSensor::from_le_bytes(sensor_raw) {
            let thresh_rsp = ipmi_sdr_get_sensor_thresholds(
                intf,
                sensor.keys.sensor_num,
                sensor.keys.owner_id,
                sensor.keys.lun(),
                sensor.keys.channel(),
            );

            if let Some(rsp) = thresh_rsp {
                if rsp.ccode == 0 && rsp.data_len > 0 {
                    let mut assertions = Vec::new();
                    let mut deassertions = Vec::new();

                    // 显示传感器支持的阈值
                    if rsp.data[0] & LOWER_NON_CRIT_SPECIFIED != 0 {
                        assertions.push("lnc-");
                        deassertions.push("lnc-");
                    }
                    if rsp.data[0] & LOWER_CRIT_SPECIFIED != 0 {
                        assertions.push("lcr-");
                        deassertions.push("lcr-");
                    }
                    if rsp.data[0] & LOWER_NON_RECOV_SPECIFIED != 0 {
                        assertions.push("lnr-");
                        deassertions.push("lnr-");
                    }
                    if rsp.data[0] & UPPER_NON_CRIT_SPECIFIED != 0 {
                        assertions.push("unc+");
                        deassertions.push("unc+");
                    }
                    if rsp.data[0] & UPPER_CRIT_SPECIFIED != 0 {
                        assertions.push("ucr+");
                        deassertions.push("ucr+");
                    }
                    if rsp.data[0] & UPPER_NON_RECOV_SPECIFIED != 0 {
                        assertions.push("unr+");
                        deassertions.push("unr+");
                    }

                    println!(" {:<21} : {} ", "Assertions Enabled", assertions.join(" "));
                    println!(
                        " {:<21} : {} ",
                        "Deassertions Enabled",
                        deassertions.join(" ")
                    );
                } else {
                    println!(" {:<21} : ", "Assertions Enabled");
                }
            } else {
                println!(" {:<21} : ", "Assertions Enabled");
            }
        } else {
            println!(" {:<21} : ", "Assertions Enabled");
        }
    }

    println!(); // 空行分隔
}

/// Verbose模式下的离散传感器输出（匹配ipmitool风格）
pub fn ipmi_sensor_print_fc_discrete_verbose(
    intf: &mut dyn IpmiIntf,
    sr: &SensorReading,
    sensor_raw: &[u8],
    sdr_record_type: u8,
) {
    let binding = String::from_utf8_lossy(&sr.s_id);
    let sensor_name = binding.trim_matches('\0').trim();

    // 获取传感器信息
    let (sensor_num, entity_id, entity_instance, sensor_type) =
        extract_sensor_info(sensor_raw, sdr_record_type);

    // 打印传感器ID行
    println!(
        "Sensor ID              : {} (0x{:x})",
        sensor_name, sensor_num
    );
    println!(" Entity ID             : {}.{}", entity_id, entity_instance);

    // 获取传感器类型描述
    let sensor_type_desc = get_sensor_type_description(sensor_type);
    println!(" Sensor Type (Discrete): {}", sensor_type_desc);

    // 打印离散传感器状态断言 - 这是与ipmitool兼容的关键功能
    if sr.s_reading_valid {
        // 获取实际的传感器读数来显示States Asserted
        if let Ok(sensor) = SdrRecordCommonSensor::from_le_bytes(sensor_raw) {
            match ipmi_sdr_get_sensor_reading_ipmb(
                intf,
                sensor.keys.sensor_num,
                sensor.keys.owner_id,
                sensor.keys.lun(),
                sensor.keys.channel(),
            ) {
                Some(rsp) if rsp.ccode == 0 && rsp.data_len >= 2 => {
                    let status_data2 = rsp.data[1]; // data2包含离散状态
                    let status_data3 = if rsp.data_len >= 3 { rsp.data[2] } else { 0 };

                    // 匹配ipmitool逻辑：如果status_data2为0且status_data3的低7位为0，则不显示States Asserted
                    if status_data2 != 0 || (status_data3 & 0x7f) != 0 {
                        let status_desc = get_discrete_sensor_status_description(
                            sensor_type,
                            status_data2,
                            status_data3,
                        );
                        if !status_desc.is_empty() {
                            println!(" States Asserted       : {}", sensor_type_desc);
                            println!("                         {}", status_desc);
                        }
                    }
                }
                _ => {
                    // 使用SensorReading中已有的数据作为备选
                    // 匹配ipmitool逻辑：如果s_data2为0且s_data3的低7位为0，则不显示States Asserted
                    if sr.s_data2 != 0 || (sr.s_data3 & 0x7f) != 0 {
                        let status_desc = get_discrete_sensor_status_description(
                            sensor_type,
                            sr.s_data2,
                            sr.s_data3,
                        );
                        if !status_desc.is_empty() {
                            println!(" States Asserted       : {}", sensor_type_desc);
                            println!("                         {}", status_desc);
                        }
                    }
                }
            }
        }
    }

    println!(); // 空行分隔
}

/// Verbose模式下的Event-Only传感器输出（匹配ipmitool风格）
pub fn ipmi_sensor_print_eventonly_verbose(
    _intf: &mut dyn IpmiIntf,
    sensor_name: &str,
    sensor_raw: &[u8],
    _reading_value: u8,
    status_value: u16,
    has_reading: bool,
) {
    // 使用extract_sensor_info获取正确的传感器信息，而不是依赖解析的结构体
    let (sensor_num, entity_id, entity_instance, sensor_type) =
        extract_sensor_info(sensor_raw, 0x03); // Event-Only类型

    // 打印传感器ID行
    println!(
        "Sensor ID              : {} (0x{:x})",
        sensor_name, sensor_num
    );
    println!(" Entity ID             : {}.{}", entity_id, entity_instance);

    // 获取传感器类型描述
    let sensor_type_desc = get_sensor_type_description(sensor_type);
    println!(" Sensor Type (Discrete): {}", sensor_type_desc);

    // 对于Event-Only传感器，显示状态断言（如果有读数值）
    if has_reading {
        let status_desc = get_discrete_sensor_status_description(
            sensor_type,
            (status_value & 0xff) as u8,
            (status_value >> 8) as u8,
        );
        if !status_desc.is_empty() {
            println!(" States Asserted       : {}", sensor_type_desc);
            println!("                         {}", status_desc);
        }
    }

    println!(); // 空行分隔
}

// ================================
// 辅助函数实现
// ================================

/// 从传感器原始数据中提取基本信息
fn extract_sensor_info(sensor_raw: &[u8], sdr_record_type: u8) -> (u8, u8, u8, u8) {
    debug5!(
        "extract_sensor_info: record_type=0x{:02x}, data_len={}",
        sdr_record_type,
        sensor_raw.len()
    );

    match sdr_record_type {
        SDR_RECORD_TYPE_FULL_SENSOR => {
            if let Ok(full_sensor) = SdrRecordFullSensor::from_le_bytes(sensor_raw) {
                let result = (
                    full_sensor.cmn.keys.sensor_num,
                    full_sensor.cmn.entity.id,
                    full_sensor.cmn.entity.instance(),
                    full_sensor.cmn.sensor.sensor_type,
                );
                debug5!("extract_sensor_info: Full sensor parsed successfully: sensor_num=0x{:02x}, entity={}.{}, type=0x{:02x}",result.0,result.1,result.2,result.3);
                result
            } else {
                // 手动解析前8字节
                if sensor_raw.len() >= 8 {
                    let sensor_num = sensor_raw[2];
                    let entity_id = sensor_raw[3];
                    let entity_instance = sensor_raw[4] & 0x7F;
                    let sensor_type = sensor_raw[7];
                    debug5!("extract_sensor_info: Full sensor manual fallback: sensor_num=0x{:02x}, entity={}.{}, type=0x{:02x}",sensor_num,entity_id,entity_instance,sensor_type);
                    (sensor_num, entity_id, entity_instance, sensor_type)
                } else {
                    debug5!("extract_sensor_info: Failed to parse Full sensor");
                    (0, 0, 0, 0)
                }
            }
        }
        SDR_RECORD_TYPE_COMPACT_SENSOR => {
            if let Ok(compact_sensor) = SdrRecordCompactSensor::from_le_bytes(sensor_raw) {
                let result = (
                    compact_sensor.cmn.keys.sensor_num,
                    compact_sensor.cmn.entity.id,
                    compact_sensor.cmn.entity.instance(),
                    compact_sensor.cmn.sensor.sensor_type,
                );
                debug5!("extract_sensor_info: Compact sensor parsed successfully: sensor_num=0x{:02x}, entity={}.{}, type=0x{:02x}",result.0,result.1,result.2,result.3);
                result
            } else if sensor_raw.len() >= 8 {
                let sensor_num = sensor_raw[2];
                let entity_id = sensor_raw[3];
                let entity_instance = sensor_raw[4] & 0x7F;
                let sensor_type = sensor_raw[7];
                debug5!("extract_sensor_info: Compact sensor manual fallback: sensor_num=0x{:02x}, entity={}.{}, type=0x{:02x}",sensor_num,entity_id,entity_instance,sensor_type);
                (sensor_num, entity_id, entity_instance, sensor_type)
            } else {
                debug5!("extract_sensor_info: Failed to parse Compact sensor");
                (0, 0, 0, 0)
            }
        }
        0x03 => {
            // SDR_RECORD_TYPE_EVENTONLY_SENSOR
            debug5!("extract_sensor_info: Attempting to parse Event-Only sensor...");

            // 首先尝试使用RAWDATA宏生成的方法
            match SdrRecordEventonlySensor::from_le_bytes(sensor_raw) {
                Ok(eventonly_sensor) => {
                    let result = (
                        eventonly_sensor.keys.sensor_num,
                        eventonly_sensor.entity.id,
                        eventonly_sensor.entity.instance(),
                        eventonly_sensor.sensor_type,
                    );
                    debug5!(
                        "extract_sensor_info: Event-Only sensor parsed successfully: sensor_num=0x{:02x}, entity={}.{}, type=0x{:02x}",
                        result.0, result.1, result.2, result.3
                    );
                    result
                }
                Err(e) => {
                    debug5!(
                        "extract_sensor_info: RAWDATA parsing failed: {}, trying manual parsing",
                        e
                    );

                    // 手动解析Event-Only传感器
                    if sensor_raw.len() >= 12 {
                        // EventonlySensorKeys (3 bytes) + EntityId (2 bytes) + sensor_type (1 byte) + event_type (1 byte) + others...
                        let sensor_num = sensor_raw[2]; // EventonlySensorKeys.sensor_num
                        let entity_id = sensor_raw[3]; // EntityId.id
                        let entity_instance_logical = sensor_raw[4]; // EntityId.instance_logical
                        let entity_instance = entity_instance_logical & 0x7F; // 取低7位
                        let sensor_type = sensor_raw[5]; // sensor_type字段

                        let result = (sensor_num, entity_id, entity_instance, sensor_type);
                        debug5!(
                            "extract_sensor_info: Event-Only sensor manual parsing successful: sensor_num=0x{:02x}, entity={}.{}, type=0x{:02x}",
                            result.0, result.1, result.2, result.3
                        );
                        result
                    } else {
                        debug5!(
                            "extract_sensor_info: Event-Only sensor data too short: {} bytes",
                            sensor_raw.len()
                        );
                        debug5!(
                            "extract_sensor_info: Raw data hex: {:02x?}",
                            &sensor_raw[..sensor_raw.len().min(32)]
                        );
                        (0, 0, 0, 0)
                    }
                }
            }
        }
        _ => {
            debug5!(
                "extract_sensor_info: Unknown record type 0x{:02x}",
                sdr_record_type
            );
            (0, 0, 0, 0)
        }
    }
}

/// 获取传感器类型的描述
fn get_sensor_type_description(sensor_type: u8) -> &'static str {
    match sensor_type {
        0x01 => "Temperature",
        0x02 => "Voltage",
        0x03 => "Current",
        0x04 => "Fan",
        0x05 => "Physical Security",
        0x06 => "Platform Security",
        0x07 => "Processor",
        0x08 => "Power Supply",
        0x09 => "Power Unit",
        0x0a => "Cooling Device",
        0x0b => "Other",
        0x0c => "Memory",
        0x0d => "Drive Slot / Bay",
        0x0e => "POST Memory Resize",
        0x0f => "System Firmware",
        0x10 => "Event Logging Disabled",
        0x11 => "Watchdog1",
        0x12 => "System Event",
        0x13 => "Critical Interrupt",
        0x14 => "Button",
        0x15 => "Module / Board",
        0x16 => "Microcontroller",
        0x17 => "Add-in Card",
        0x18 => "Chassis",
        0x19 => "Chip Set",
        0x1a => "Other FRU",
        0x1b => "Cable / Interconnect",
        0x1c => "Terminator",
        0x1d => "System Boot Initiated",
        0x1e => "Boot Error",
        0x1f => "OS Boot",
        0x20 => "OS Critical Stop",
        0x21 => "Slot / Connector",
        0x22 => "System ACPI Power State",
        0x23 => "Watchdog2",
        0x24 => "Platform Alert",
        0x25 => "Entity Presence",
        0x26 => "Monitor ASIC",
        0x27 => "LAN",
        0x28 => "Management Subsystem Health",
        0x29 => "Battery",
        0x2a => "Session Audit",
        0x2b => "Version Change",
        0x2c => "FRU State",
        _ => "Unknown",
    }
}

/// 获取离散传感器状态的描述
fn get_discrete_sensor_status_description(sensor_type: u8, data2: u8, data3: u8) -> String {
    match sensor_type {
        0x08 => {
            // Power Supply
            if data2 & 0x01 != 0 {
                "[Presence detected]".to_string()
            } else if data2 & 0x02 != 0 {
                "[Power Supply Failure detected]".to_string()
            } else if data2 & 0x04 != 0 {
                "[Predictive Failure]".to_string()
            } else if data2 & 0x08 != 0 {
                "[Power Supply input lost (AC/DC)]".to_string()
            } else if data2 & 0x10 != 0 {
                "[Power Supply input lost or out-of-range]".to_string()
            } else if data2 & 0x20 != 0 {
                "[Power Supply input out-of-range, but present]".to_string()
            } else if data2 & 0x40 != 0 {
                "[Configuration error]".to_string()
            } else {
                String::new()
            }
        }
        0x07 => {
            // Processor
            if data2 & 0x01 != 0 {
                "[Presence detected]".to_string()
            } else if data2 & 0x02 != 0 {
                "[Processor disabled]".to_string()
            } else if data2 & 0x04 != 0 {
                "[Terminator presence detected]".to_string()
            } else if data2 & 0x08 != 0 {
                "[Processor automatically throttled]".to_string()
            } else if data2 & 0x10 != 0 {
                "[Machine Check Exception]".to_string()
            } else if data2 & 0x20 != 0 {
                "[Correctable Machine Check Error]".to_string()
            } else {
                String::new()
            }
        }
        0x0c => {
            // Memory
            if data2 & 0x01 != 0 {
                "[Correctable ECC / other correctable memory error]".to_string()
            } else if data2 & 0x02 != 0 {
                "[Uncorrectable ECC / other uncorrectable memory error]".to_string()
            } else if data2 & 0x04 != 0 {
                "[Parity]".to_string()
            } else if data2 & 0x08 != 0 {
                "[Memory Scrub Failed]".to_string()
            } else if data2 & 0x10 != 0 {
                "[Memory Device Disabled]".to_string()
            } else if data2 & 0x20 != 0 {
                "[Correctable ECC / other correctable memory error logging limit reached]"
                    .to_string()
            } else if data2 & 0x40 != 0 {
                "[Presence Detected]".to_string()
            } else if data2 & 0x80 != 0 {
                "[Configuration Error]".to_string()
            } else {
                String::new()
            }
        }
        0x0d => {
            // Drive Slot / Bay
            if data2 & 0x01 != 0 {
                "[Drive Present]".to_string()
            } else if data2 & 0x02 != 0 {
                "[Drive Fault]".to_string()
            } else if data2 & 0x04 != 0 {
                "[Predictive Failure]".to_string()
            } else if data2 & 0x08 != 0 {
                "[Hot Spare]".to_string()
            } else if data2 & 0x10 != 0 {
                "[Consistency Check / Parity Check in progress]".to_string()
            } else if data2 & 0x20 != 0 {
                "[In Critical Array]".to_string()
            } else if data2 & 0x40 != 0 {
                "[In Failed Array]".to_string()
            } else if data2 & 0x80 != 0 {
                "[Rebuild/Remap in progress]".to_string()
            } else {
                String::new()
            }
        }
        0x04 => {
            // Fan
            if data2 & 0x01 != 0 {
                "[Lower Critical - going low]".to_string()
            } else if data2 & 0x02 != 0 {
                "[Lower Critical - going high]".to_string()
            } else if data2 & 0x04 != 0 {
                "[Lower Non-recoverable - going low]".to_string()
            } else if data2 & 0x08 != 0 {
                "[Lower Non-recoverable - going high]".to_string()
            } else if data2 & 0x10 != 0 {
                "[Upper Critical - going low]".to_string()
            } else if data2 & 0x20 != 0 {
                "[Upper Critical - going high]".to_string()
            } else if data2 & 0x40 != 0 {
                "[Upper Non-recoverable - going low]".to_string()
            } else if data2 & 0x80 != 0 {
                "[Upper Non-recoverable - going high]".to_string()
            } else if data3 & 0x01 != 0 {
                "[Device Present]".to_string()
            } else if data3 & 0x02 != 0 {
                "[Device Absent]".to_string()
            } else {
                String::new()
            }
        }
        0x22 => {
            // System ACPI Power State
            if data2 & 0x01 != 0 {
                "[S0/G0: working]".to_string()
            } else if data2 & 0x02 != 0 {
                "[S1: sleeping - processor context maintained]".to_string()
            } else if data2 & 0x04 != 0 {
                "[S2: sleeping - processor context lost]".to_string()
            } else if data2 & 0x08 != 0 {
                "[S3: sleeping - processor & hw context lost, memory retained]".to_string()
            } else if data2 & 0x10 != 0 {
                "[S4: non-volatile sleep/suspend-to disk]".to_string()
            } else if data2 & 0x20 != 0 {
                "[S5/G2: soft-off]".to_string()
            } else if data2 & 0x40 != 0 {
                "[S4/S5 soft-off, particular S4/S5 state cannot be determined]".to_string()
            } else if data2 & 0x80 != 0 {
                "[G3: mechanical off]".to_string()
            } else if data3 & 0x01 != 0 {
                "[Sleeping in an S1, S2, or S3 states]".to_string()
            } else if data3 & 0x02 != 0 {
                "[G1: sleeping]".to_string()
            } else if data3 & 0x04 != 0 {
                "[S5: entered by override]".to_string()
            } else if data3 & 0x08 != 0 {
                "[Legacy ON state]".to_string()
            } else if data3 & 0x10 != 0 {
                "[Legacy OFF state]".to_string()
            } else if data3 & 0x20 != 0 {
                "[Unknown]".to_string()
            } else {
                String::new()
            }
        }
        0x10 => {
            // Event Logging Disabled
            if data2 & 0x01 != 0 {
                "[Correctable Machine Check Error Logging Disabled]".to_string()
            } else if data2 & 0x02 != 0 {
                "[Event 'Type' Logging Disabled]".to_string()
            } else if data2 & 0x04 != 0 {
                "[Log Area Reset/Cleared]".to_string()
            } else if data2 & 0x08 != 0 {
                "[All Event Logging Disabled]".to_string()
            } else if data2 & 0x10 != 0 {
                "[SEL Full]".to_string()
            } else if data2 & 0x20 != 0 {
                "[SEL Almost Full]".to_string()
            } else {
                String::new()
            }
        }
        0x23 => {
            // Watchdog2 - 严格匹配ipmitool逻辑
            if data2 & 0x01 != 0 {
                "[Timer expired]".to_string()
            } else if data2 & 0x02 != 0 {
                "[Hard reset]".to_string()
            } else if data2 & 0x04 != 0 {
                "[Power down]".to_string()
            } else if data2 & 0x08 != 0 {
                "[Power cycle]".to_string()
            } else if data3 & 0x01 != 0 {
                "[Timer interrupt]".to_string()
            } else {
                // 对于reserved位(data2 bits 4-7)，不返回任何描述
                // 这与ipmitool保持一致：只有在有意义的状态位被设置时才显示
                String::new()
            }
        }
        _ => String::new(),
    }
}

// 从原始SDR数据中尝试提取传感器名称的辅助函数
fn extract_sensor_name_from_raw_data(sensor_raw: &[u8], _sensor_num: u8) -> String {
    // Event-Only传感器的结构大致为：
    // EventonlySensorKeys (3) + EntityId (2) + sensor_type (1) + event_type (1)
    // + EventonlyShareInfo (2) + reserved (1) + oem (1) + id_code (1) + id_string (最多16)

    // 计算id_string的起始位置：3+2+1+1+2+1+1+1 = 12
    const ID_STRING_OFFSET: usize = 12;

    if sensor_raw.len() > ID_STRING_OFFSET {
        // 尝试从这个位置读取id_code
        if sensor_raw.len() > ID_STRING_OFFSET - 1 {
            let id_code = sensor_raw[ID_STRING_OFFSET - 1];
            let id_len = (id_code & 0x1f) as usize;

            if id_len > 0 && sensor_raw.len() >= ID_STRING_OFFSET + id_len {
                let name_bytes = &sensor_raw[ID_STRING_OFFSET..ID_STRING_OFFSET + id_len.min(16)];
                let extracted_name = String::from_utf8_lossy(name_bytes)
                    .trim_matches('\0')
                    .trim()
                    .to_string();

                if !extracted_name.is_empty()
                    && extracted_name
                        .chars()
                        .all(|c| c.is_ascii() && !c.is_control())
                {
                    return extracted_name;
                }
            }
        }

        // 如果上面的方法失败，尝试扫描整个可能的名称区域
        let start_pos = ID_STRING_OFFSET;
        let end_pos = (sensor_raw.len()).min(ID_STRING_OFFSET + 16);

        if start_pos < end_pos {
            // 查找第一个null字符或非打印字符
            let actual_len = sensor_raw
                .iter()
                .skip(start_pos)
                .take(end_pos - start_pos)
                .take_while(|&&b| b != 0 && (0x20..=0x7e).contains(&b))
                .count();

            if actual_len > 0 {
                let name_bytes = &sensor_raw[start_pos..start_pos + actual_len];
                let extracted_name = String::from_utf8_lossy(name_bytes).trim().to_string();

                if !extracted_name.is_empty() {
                    return extracted_name;
                }
            }
        }
    }

    // 如果所有尝试都失败，返回空字符串
    String::new()
}

// 从Full和Compact传感器的原始SDR数据中尝试提取传感器名称的辅助函数
fn extract_sensor_name_from_full_compact_data(sensor_raw: &[u8], sdr_record_type: u8) -> String {
    // Full和Compact传感器的结构：
    // SdrRecordCommonSensor (18) + 其他字段 + id_code (1) + id_string (最多16)

    let id_string_offset = match sdr_record_type {
        SDR_RECORD_TYPE_FULL_SENSOR => {
            // Full传感器：CommonSensor(18) + linearization(1) + mtol(2) + bacc(4) + analog_flag(1)
            // + nominal_read(1) + normal_max(1) + normal_min(1) + sensor_max(1) + sensor_min(1)
            // + threshold(8) + reserved(2) + oem(1) + id_code(1) = 42
            42
        }
        SDR_RECORD_TYPE_COMPACT_SENSOR => {
            // Compact传感器：CommonSensor(18) + share(2) + threshold(2) + reserved(3) + oem(1) + id_code(1) = 27
            27
        }
        _ => return String::new(),
    };

    if sensor_raw.len() > id_string_offset {
        // 尝试从计算出的位置读取id_code
        if sensor_raw.len() > id_string_offset - 1 {
            let id_code = sensor_raw[id_string_offset - 1];
            let id_len = (id_code & 0x1f) as usize;

            if id_len > 0 && sensor_raw.len() >= id_string_offset + id_len {
                let name_bytes = &sensor_raw[id_string_offset..id_string_offset + id_len.min(16)];
                let extracted_name = String::from_utf8_lossy(name_bytes)
                    .trim_matches('\0')
                    .trim()
                    .to_string();

                if !extracted_name.is_empty()
                    && extracted_name
                        .chars()
                        .all(|c| c.is_ascii() && !c.is_control())
                {
                    return extracted_name;
                }
            }
        }

        // 如果id_code方法失败，尝试扫描可能的名称区域
        let start_pos = id_string_offset;
        let end_pos = (sensor_raw.len()).min(id_string_offset + 16);

        if start_pos < end_pos {
            // 查找连续的可打印ASCII字符
            let actual_len = sensor_raw
                .iter()
                .skip(start_pos)
                .take(end_pos - start_pos)
                .take_while(|&&b| b != 0 && (0x20..=0x7e).contains(&b))
                .count();

            if actual_len > 0 {
                let name_bytes = &sensor_raw[start_pos..start_pos + actual_len];
                let extracted_name = String::from_utf8_lossy(name_bytes).trim().to_string();

                if !extracted_name.is_empty() {
                    return extracted_name;
                }
            }
        }
    }

    // 最后尝试：在整个数据中寻找可能的传感器名称
    // 查找长度为3-16字符的连续ASCII字符串
    let mut best_name = String::new();
    let mut i = 18; // 跳过CommonSensor部分

    while i < sensor_raw.len().saturating_sub(3) {
        let mut len = 0;
        let start = i;

        // 查找连续的可打印字符
        while i + len < sensor_raw.len() && len < 16 {
            let byte = sensor_raw[i + len];
            if byte == 0 {
                break; // null终止符
            }
            if !(0x20..=0x7e).contains(&byte) {
                break; // 非打印字符
            }
            len += 1;
        }

        // 如果找到了合理长度的字符串
        if (3..=16).contains(&len) {
            let candidate = String::from_utf8_lossy(&sensor_raw[start..start + len])
                .trim()
                .to_string();

            // 检查是否看起来像传感器名称（包含常见的传感器名称模式）
            if is_likely_sensor_name(&candidate)
                && (best_name.is_empty() || candidate.len() > best_name.len())
            {
                best_name = candidate;
            }
        }

        i += 1;
    }

    best_name
}

// 判断字符串是否看起来像传感器名称
fn is_likely_sensor_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // 常见的传感器名称模式
    let sensor_patterns = [
        "Temp",
        "Temperature",
        "Voltage",
        "Current",
        "Power",
        "Fan",
        "Speed",
        "CPU",
        "PSU",
        "Status",
        "Present",
        "VCore",
        "VSoc",
        "VDDIO",
        "AUX",
        "Inlet",
        "Outlet",
        "Ambient",
        "VR",
        "MEM",
        "PCIe",
        "GPU",
        "HDD",
        "SSD",
        "RAID",
        "BIOS",
        "Boot",
        "WatchDog",
        "SEL",
        "_",
        "0",
        "1",
        "2",
    ];

    // 检查是否包含常见模式
    for pattern in &sensor_patterns {
        if name.contains(pattern) {
            return true;
        }
    }

    // 检查是否全部是大写字母和数字的组合
    if name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '+' || c == '-')
    {
        return true;
    }

    false
}

pub fn ipmi_sensor_get_sensor_reading_factors(
    intf: &mut dyn IpmiIntf,
    sensor: &mut SdrRecordFullSensor,
    reading: u8,
) -> bool {
    // let id = std::str::from_utf8(&sensor.id_string[..16])
    //     .unwrap_or("")
    //     .trim_matches('\0')
    //     .to_string();

    let mut req_data = [sensor.cmn.keys.sensor_num, reading];

    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_SE);
    req.msg.lun_mut(sensor.cmn.keys.lun());
    req.msg.cmd = GET_SENSOR_FACTORS;
    req.msg.data = req_data.as_mut_ptr();
    req.msg.data_len = req_data.len() as u16;

    match intf.sendrecv(&req) {
        Some(rsp) => {
            if rsp.ccode != 0 {
                log::error!("Error getting reading factor for sensor");
                return false;
            }

            // Update SDR copy with updated Reading Factors
            // Note: data[0] points to next valid entry in sampling table
            // Using raw copy from response data
            sensor.mtol = u16::from_le_bytes(rsp.data[1..3].try_into().unwrap());
            sensor.bacc = u32::from_le_bytes(rsp.data[3..7].try_into().unwrap());
            true
        }
        None => {
            eprintln!(
                "Error updating reading factor for sensor {} (#{})",
                String::from_utf8_lossy(&sensor.id_string).trim_matches('\0'),
                sensor.cmn.keys.sensor_num
            );
            false
        }
    }
}

impl SensorReading {
    /// 生成CSV格式的传感器阈值数据
    pub fn dump_sensor_fc_threshold_csv(
        &self,
        thresh_available: bool,
        rsp: &IpmiRs,
        ctx: &OutputContext,
    ) -> String {
        let mut output = String::new();
        let thresh_status = self.ipmi_sdr_get_thresh_status("na", ctx);
        // 基本输出格式
        let binding = String::from_utf8_lossy(&self.s_id);
        let sensor_name = binding.trim_matches('\0').trim();
        output.push_str(sensor_name);

        // 读数部分
        output.push_str(&match (self.s_reading_valid, self.s_has_analog_value) {
            (true, true) => format!(",{:.3},{},{}", self.s_a_val, self.s_a_units, thresh_status),
            (true, false) => format!(
                ",0x{:x},{},{}",
                self.s_reading, self.s_a_units, thresh_status
            ),
            (false, _) => format!(",na,{},na", self.s_a_units),
        });

        // 阈值处理部分
        match &self.full {
            Some(full) if thresh_available => {
                // Define threshold parameters array (bit_mask, data_index)
                let thresholds = [
                    (LOWER_NON_RECOV_SPECIFIED, 3),
                    (LOWER_CRIT_SPECIFIED, 2),
                    (LOWER_NON_CRIT_SPECIFIED, 1),
                    (UPPER_NON_CRIT_SPECIFIED, 4),
                    (UPPER_CRIT_SPECIFIED, 5),
                    (UPPER_NON_RECOV_SPECIFIED, 6),
                ];
                //输出6个阈值
                for (bit_mask, data_idx) in thresholds {
                    let data = rsp.data.get(data_idx).copied().unwrap_or(0);
                    let is_avail = rsp.data[0] & bit_mask != 0;
                    output.push_str(&full.print_thresh_setting(is_avail, data, ","));
                }
            }
            _ => {
                output.push_str(",na,na,na,na,na,na");
            }
        }
        output
    }

    //ipmi_sensor_print_fc_discrete verbose == 0
    pub fn dump_sensor_fc_thredshold(
        &self,
        thresh_available: bool,
        rsp: &IpmiRs,
        ctx: &OutputContext,
    ) -> String {
        let mut output = String::new();
        let thresh_status = self.ipmi_sdr_get_thresh_status("na", ctx);

        // 传感器名称（16字符宽度，左对齐）
        let binding = String::from_utf8_lossy(&self.s_id);
        let sensor_name = binding.trim_matches('\0').trim();
        output.push_str(&format!("{:<16} ", sensor_name));

        // 读数部分（匹配C版本格式）
        if self.s_reading_valid {
            if self.s_has_analog_value {
                // 模拟值传感器：显示数值（保留3位小数）
                output.push_str(&format!("| {:<10.3} ", self.s_a_val));
            } else {
                // 离散传感器：显示十六进制值
                output.push_str(&format!("| 0x{:<8x} ", self.s_reading));
            }
        } else {
            // 无效读数
            output.push_str(&format!("| {:<10} ", "na"));
        }

        // 单位（匹配C版本格式）
        output.push_str(&format!("| {:<10} ", self.s_a_units));

        // 状态（匹配C版本格式）
        output.push_str(&format!("| {:<5} ", thresh_status));

        // 阈值处理部分（匹配C版本的6个阈值顺序）
        match &self.full {
            Some(full) if thresh_available => {
                // C版本的阈值顺序：lnr, lcr, lnc, unc, ucr, unr
                let thresholds = [
                    (LOWER_NON_RECOV_SPECIFIED, 3), // lnr
                    (LOWER_CRIT_SPECIFIED, 2),      // lcr
                    (LOWER_NON_CRIT_SPECIFIED, 1),  // lnc
                    (UPPER_NON_CRIT_SPECIFIED, 4),  // unc
                    (UPPER_CRIT_SPECIFIED, 5),      // ucr
                    (UPPER_NON_RECOV_SPECIFIED, 6), // unr
                ];

                for (bit_mask, data_idx) in thresholds {
                    let is_avail = rsp.data[0] & bit_mask != 0;
                    let data = rsp.data.get(data_idx).copied().unwrap_or(0);
                    let thresh_val = full.print_thresh_setting(is_avail, data, "");
                    output.push_str(&format!("| {:<9} ", thresh_val.trim()));
                }
            }
            _ => {
                // 没有阈值数据时显示na
                for _ in 0..6 {
                    output.push_str("| na        ");
                }
            }
        }

        output
    }

    /// 简单格式输出（匹配C版本verbose=0的格式）
    /// 只显示基本传感器信息，不显示阈值
    pub fn dump_sensor_fc_threshold_simple(
        &self,
        _thresh_available: bool,
        _rsp: &IpmiRs,
        ctx: &OutputContext,
    ) -> String {
        let mut output = String::new();
        let thresh_status = self.ipmi_sdr_get_thresh_status("na", ctx);

        // 基本输出格式（完全匹配C版本）
        let binding = String::from_utf8_lossy(&self.s_id);
        let sensor_name = binding.trim_matches('\0').trim();
        output.push_str(&format!("{:<16} ", sensor_name));

        // 读数部分（完全匹配C版本格式）
        if self.s_reading_valid {
            /*
            if self.s_has_analog_value {
                // 模拟值传感器：显示数值和单位（匹配C版本格式）
                // C版本格式：| 49 degrees C      | ok
                // 保留小数精度以匹配C版本
                let value_str = if self.s_a_val.fract() == 0.0 {
                    format!("{} {}", self.s_a_val as i32, self.s_a_units)
                } else {
                    format!("{:.2} {}", self.s_a_val, self.s_a_units)
                };
                output.push_str(&format!("| {:<17} | {:<6}", value_str, thresh_status));
            } else {
                // 离散传感器：显示十六进制值
                output.push_str(&format!("| 0x{:<15x} | {:<6}", self.s_data2, thresh_status));
            }
            */
            //bgz里面内容全部替换为下面：
            if self.s_has_analog_value {
                // 对于温度传感器，当值为0时显示为"disabled"
                if self.s_a_val == 0.0 && self.s_a_units.contains("degrees") {
                    output.push_str(&format!("| {:<17} | {:<6}", "disabled", "ns"));
                } else {
                    // 正常显示模拟值
                    let value_str = if self.s_a_val.fract() == 0.0 {
                        format!("{} {}", self.s_a_val as i32, self.s_a_units)
                    } else {
                        format!("{:.2} {}", self.s_a_val, self.s_a_units)
                    };
                    output.push_str(&format!("| {:<17} | {:<6}", value_str, thresh_status));
                }
            }
        } else {
            // 无效读数（匹配C版本的"no reading"格式）
            output.push_str(&format!("| {:<17} | {:<6}", "no reading", "na"));
        }

        output
    }

    /// 扩展格式输出（匹配C版本sdr_extended=1的格式）
    /// 显示传感器名称、传感器号、状态、实体ID、读数
    pub fn dump_sensor_fc_threshold_extended(
        &self,
        _thresh_available: bool,
        _rsp: &IpmiRs,
        ctx: &OutputContext,
        sensor_raw: &[u8],
        sdr_record_type: u8,
    ) -> String {
        let mut output = String::new();
        let thresh_status = self.ipmi_sdr_get_thresh_status("na", ctx);

        // 基本输出格式（匹配C版本扩展格式）
        let binding = String::from_utf8_lossy(&self.s_id);
        let sensor_name = binding.trim_matches('\0').trim();
        output.push_str(&format!("{:<16} | ", sensor_name));

        // 获取传感器号和实体信息
        let (sensor_num, entity_id, entity_instance) = match sdr_record_type {
            SDR_RECORD_TYPE_FULL_SENSOR => {
                if let Ok(full_sensor) = SdrRecordFullSensor::from_le_bytes(sensor_raw) {
                    (
                        full_sensor.cmn.keys.sensor_num,
                        full_sensor.cmn.entity.id,
                        full_sensor.cmn.entity.instance(),
                    )
                } else {
                    (0, 0, 0)
                }
            }
            SDR_RECORD_TYPE_COMPACT_SENSOR => {
                if let Ok(compact_sensor) = SdrRecordCompactSensor::from_le_bytes(sensor_raw) {
                    (
                        compact_sensor.cmn.keys.sensor_num,
                        compact_sensor.cmn.entity.id,
                        compact_sensor.cmn.entity.instance(),
                    )
                } else {
                    (0, 0, 0)
                }
            }
            _ => (0, 0, 0),
        };

        // 传感器号（十六进制格式）
        output.push_str(&format!("{:02X}h | ", sensor_num));

        // 状态
        output.push_str(&format!("{:<3} | ", thresh_status));

        // 实体ID.实例
        output.push_str(&format!("{}.{} | ", entity_id, entity_instance));

        // 读数部分（匹配C版本扩展格式）
        if self.s_reading_valid {
            if self.s_has_analog_value {
                // 模拟值传感器：显示数值和单位
                let value_str = if self.s_a_val.fract() == 0.0 {
                    format!("{} {}", self.s_a_val as i32, self.s_a_units)
                } else {
                    format!("{:.2} {}", self.s_a_val, self.s_a_units)
                };
                output.push_str(&value_str);
            } else {
                // 离散传感器：显示十六进制值
                output.push_str(&format!("0x{:02x}", self.s_data2));
            }
        } else {
            // 无效读数
            output.push_str("No Reading");
        }

        output
    }

    pub fn ipmi_sdr_get_thresh_status(
        &self,
        invalidstr: &'static str,
        ctx: &OutputContext,
    ) -> &'static str {
        if !self.s_reading_valid {
            return invalidstr;
        }

        let stat = self.s_data2;
        if stat & SDR_SENSOR_STAT_LO_NR != 0 {
            if ctx.verbose > 0 {
                "Lower Non-Recoverable"
            } else if ctx.extended {
                "lnr"
            } else {
                "nr"
            }
        } else if stat & SDR_SENSOR_STAT_HI_NR != 0 {
            if ctx.verbose > 0 {
                "Upper Non-Recoverable"
            } else if ctx.extended {
                "unr"
            } else {
                "nr"
            }
        } else if stat & SDR_SENSOR_STAT_LO_CR != 0 {
            if ctx.verbose > 0 {
                "Lower Critical"
            } else if ctx.extended {
                "lcr"
            } else {
                "cr"
            }
        } else if stat & SDR_SENSOR_STAT_HI_CR != 0 {
            if ctx.verbose > 0 {
                "Upper Critical"
            } else if ctx.extended {
                "ucr"
            } else {
                "cr"
            }
        } else if stat & SDR_SENSOR_STAT_LO_NC != 0 {
            if ctx.verbose > 0 {
                "Lower Non-Critical"
            } else if ctx.extended {
                "lnc"
            } else {
                "nc"
            }
        } else if stat & SDR_SENSOR_STAT_HI_NC != 0 {
            if ctx.verbose > 0 {
                "Upper Non-Critical"
            } else if ctx.extended {
                "unc"
            } else {
                "nc"
            }
        } else {
            "ok"
        }
    }
}

/// 获取SDR仓库信息，用于显示调试信息
fn get_sdr_repository_info(
    intf: &mut dyn IpmiIntf,
) -> Result<crate::commands::sdr::types::SdrRepositoryInfo, crate::error::IpmiError> {
    debug5!("Sending Get SDR Repository Info command");

    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_STORAGE);
    req.msg.cmd = 0x20; // IPMI_GET_SDR_REPOSITORY_INFO

    let rsp = intf
        .sendrecv(&req)
        .ok_or(crate::error::IpmiError::ResponseError)?;

    if rsp.ccode != 0 {
        debug5!(
            "Get SDR Repository Info failed with completion code: 0x{:02x}",
            rsp.ccode
        );
        return Err(crate::error::IpmiError::CompletionCode(rsp.ccode));
    }

    debug5!("Got SDR Repository Info response: {} bytes", rsp.data_len);

    crate::commands::sdr::types::SdrRepositoryInfo::from_response_data(
        &rsp.data[..rsp.data_len as usize],
    )
}
