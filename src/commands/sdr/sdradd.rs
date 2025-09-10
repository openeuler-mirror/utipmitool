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
use ipmi_macros::AsBytes;

use crate::commands::sdr::iter::SdrIterator;
use crate::commands::sdr::sdr::SdrRecordHeader;
use crate::commands::sdr::sdr::*;
use crate::commands::sdr::types::{SDR_RECORD_TYPE_COMPACT_SENSOR, SDR_RECORD_TYPE_FULL_SENSOR};
use crate::commands::sdr::SdrRecordCommonSensor;
use crate::ipmi::intf::IpmiIntf;
use crate::ipmi::ipmi::*;
use std::sync::atomic::{AtomicUsize, Ordering};

//sdr_record_list
//sdrr_get_records，用list存储SdrRecord，

pub const ADD_PARTIAL_SDR: u8 = 0x25;
pub const PARTIAL_ADD: u8 = 0x00;
pub const LAST_RECORD: u8 = 0x01;
static SDR_MAX_WRITE_LEN: AtomicUsize = AtomicUsize::new(0); // 初始默认值

#[derive(Clone)]
pub struct SdrRecord {
    // id: u16,
    // version: u8,
    // record_type: u8,
    // length: u8,
    pub header: SdrRecordHeader,
    pub raw: Vec<u8>,
    //record: SdrRecordType,
} //应该用方法返回结recored

impl SdrRecord {
    /// 实现IPMI原始记录名称打印功能
    pub fn ipmi_sdr_print_name_from_rawentry(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 函数保留用于兼容性，但不再执行打印操作以避免在 sel elist 中产生噪音
        Ok(())
    }
}

//#[derive(Debug)]
pub enum SdrRecordType {
    Common(Box<SdrRecordCommonSensor>),
    Full(Box<SdrRecordFullSensor>),
    Compact(Box<SdrRecordCompactSensor>),
    // 其他类型...
}

//先用type将成员枚举初始化，在根据枚举类型来解析
impl SdrRecordType {
    //根据record_type转换成对应的类型
    pub fn from_le_bytes(data: &[u8], record_type: u8) -> Option<Self> {
        match record_type {
            SDR_RECORD_TYPE_FULL_SENSOR => SdrRecordFullSensor::from_le_bytes(data)
                .ok()
                .map(|s| Self::Full(Box::new(s))),
            SDR_RECORD_TYPE_COMPACT_SENSOR => SdrRecordCompactSensor::from_le_bytes(data)
                .ok()
                .map(|s| Self::Compact(Box::new(s))),
            _ => None, // 明确处理未知类型
        }
    }

    pub fn get_id_info(&self) -> Option<(u8, &[u8])> {
        match self {
            SdrRecordType::Full(full) => Some((full.id_code, &full.id_string)),
            SdrRecordType::Compact(compact) => Some((compact.id_code, &compact.id_string)),
            // 其他类型处理...
            _ => None,
        }
    }
}

impl<'a> SdrIterator<'a> {
    /// 从迭代器填充 SDR 列表
    pub fn sdrr_get_records(&mut self) -> Result<Vec<SdrRecord>, Box<dyn std::error::Error>> {
        let mut queue: Vec<SdrRecord> = Vec::new();
        //遍历header,根据header的读取数据
        while let Some(header) = self.next() {
            let raw_data = self.ipmi_sdr_get_record(&header).unwrap_or_default();

            let record = SdrRecord {
                header: SdrRecordHeader {
                    id: header.id,
                    version: 0x51,
                    record_type: header.record_type,
                    length: header.length,
                },
                raw: raw_data, // ✅ 修复：使用实际的SDR数据而不是空Vec
            };
            let _ = record.ipmi_sdr_print_name_from_rawentry();
            queue.push(record);
        }

        Ok(queue)
    }
}

#[derive(AsBytes)]
#[repr(C)]
pub struct SdrAddRq {
    pub reserve_id: u16, /* reservation ID */
    pub id: u16,         /* record ID */
    pub offset: u8,      /* offset into SDR */
    pub in_progress: u8, /* 0=partial, 1=last */
                         //    data: [u8; 1],        /* SDR record data */
}

/// 添加 SDR 记录到仓库
pub fn ipmi_sdr_add_record(intf: &mut dyn IpmiIntf, sdr: &SdrRecord) -> bool {
    // 检查有效记录
    if sdr.raw.is_empty() || sdr.header.length == 0 {
        return false;
    }

    // 获取 SDR 保留 ID
    let reserve_id = match ipmi_sdr_get_reservation(intf, false) {
        Some(id) => id,
        None => {
            println!("Unable to get SDR reservation ID");
            return false;
        }
    };

    let max_write_len = SDR_MAX_WRITE_LEN.load(Ordering::Relaxed);

    //生成sdr_rq的buffer，可能要求数据要连续
    let mut buffer = vec![0u8; std::mem::size_of::<SdrAddRq>() + max_write_len];
    //let sdr_rq = buffer.as_mut_slice();

    let mut sdr_rq = SdrAddRq {
        reserve_id,
        id: 0,
        offset: 0,
        in_progress: PARTIAL_ADD,
    };
    // 初始化请求结构
    // let mut req = IpmiRq::default();
    let mut req = IpmiRq::default();
    // 首次发送（单独处理header）

    {
        //作用域用来释放as_bytes引用
        let rq = sdr_rq.as_bytes();
        let header = sdr.header.as_bytes();

        // 将 sdr_rq 复制到 buffer 的起始位置
        buffer[..rq.len()].copy_from_slice(rq);
        // 将 data 复制到 buffer 的后续位置
        buffer[rq.len()..rq.len() + header.len()].copy_from_slice(header);

        req.msg.netfn_mut(IPMI_NETFN_STORAGE);
        req.msg.cmd = ADD_PARTIAL_SDR;

        req.msg.data = buffer.as_mut_ptr();
        req.msg.data_len = (rq.len() + header.len()) as u16;
    }
    match partial_send(intf, &req) {
        Ok(next_id) => {
            sdr_rq.id = next_id;
        }
        Err(_) => {
            return false;
        }
    }

    let mut i = 0;
    let len = sdr.header.length;
    let max_write_len = SDR_MAX_WRITE_LEN.load(Ordering::Relaxed) as u8;
    while i < len {
        let data_len;
        if (len - i) <= max_write_len {
            /* last crunch */
            data_len = len - i;
            sdr_rq.in_progress = LAST_RECORD;
        } else {
            data_len = max_write_len;
        }

        sdr_rq.offset = i + 5;
        //rq变化后要重新复制到buffer
        {
            // 每次循环生成新的rq引用
            let rq = sdr_rq.as_bytes();
            buffer[..rq.len()].copy_from_slice(rq);
            let index = i as usize;
            buffer[rq.len()..rq.len() + data_len as usize]
                .copy_from_slice(&sdr.raw[index..index + data_len as usize]);
            //memcpy(sdr_rq->data, sdrr->raw + i, data_len);
            req.msg.data_len = data_len as u16 + rq.len() as u16;
        }
        match partial_send(intf, &req) {
            Ok(next_id) => {
                sdr_rq.id = next_id;
            }
            Err(_) => {
                log::error!("ipmitool: partial add failed");
                return false;
            }
        }
        i += data_len;
    }
    true
}

pub fn partial_send(intf: &mut dyn IpmiIntf, req: &IpmiRq) -> Result<u16, i32> {
    let rsp = match intf.sendrecv(req) {
        Some(r) => r,
        None => return Err(-1),
    };

    if rsp.ccode != 0 || rsp.data_len < 2 {
        return Err(-1);
    }
    Ok(u16::from_le_bytes([rsp.data[0], rsp.data[1]]))
}
