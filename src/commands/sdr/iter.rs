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
#![allow(dead_code)]

use crate::commands::mc::{IpmDevidRsp, BMC_GET_DEVICE_ID};
use crate::commands::sdr::sdr::*;
use crate::debug3;
use crate::debug4;
use crate::debug5;

use crate::ipmi::ipmi::*;

//use crate::commands::sdr::sdradd::IpmiSdrInterface;
use crate::error::IpmiError;
use crate::ipmi::intf::IpmiIntf;
use rand::Rng;
use std::thread;

static mut USE_BUILT_IN: bool = false; /* Uses DeviceSDRs instead of SDRR */
static mut SDR_MAX_READ_LEN_STATIC: u8 = 0;
static mut SDRIANA: i64 = 0;

const IPMI_NETFN_STORAGE: u8 = 0x0A;
const IPMI_NETFN_SE: u8 = 0x04;
const GET_SDR: u8 = 0x23;
const GET_DEVICE_SDR: u8 = 0x23;
const IPMI_CC_CANT_RET_NUM_REQ_BYTES: u8 = 0xCA;
const IPMI_CC_RES_CANCELED: u8 = 0xC5;

use std::sync::atomic::AtomicUsize;
static SDR_MAX_READ_LEN: AtomicUsize = AtomicUsize::new(0);
pub struct SdrIterator<'a> {
    //intf: Rc<RefCell<Box<dyn IpmiIntf>>>,
    //pub intf: &mut Box<dyn IpmiIntf>,
    //pub intf: Box<dyn IpmiIntf> 类型，不可复制，只能移动
    //intf: &'a mut Box<dyn IpmiIntf>, // 更通用的方式
    pub intf: &'a mut dyn IpmiIntf,
    reservation_id: u16,
    pub next_id: u16, // 设为公共，以便外部访问
    total: i32,
    finished: bool,
    use_builtin: bool,
}

impl<'a> SdrIterator<'a> {
    //IpmiError
    //&mut dyn ipmi_intf::IpmiIntf
    //intf: &mut dyn IpmiIntf
    //pub fn new(intf: &mut dyn IpmiIntf, mut use_builtin: bool) -> Option<Self> {
    //intf: &mut Box<dyn IpmiIntf>
    pub fn new(intf: &'a mut dyn IpmiIntf, mut use_builtin: bool) -> Option<Self> {
        // Get device ID first to check SDR capabilities
        let mut req = IpmiRq::default();
        let mut iter = SdrIterator {
            intf, //引用不能为空
            reservation_id: 0,
            next_id: 0,
            total: 0,
            finished: false,
            use_builtin,
        };

        req.msg.netfn_mut(IPMI_NETFN_APP);
        req.msg.cmd = BMC_GET_DEVICE_ID;
        req.msg.data_len = 0;

        // 借用给itr
        let rsp = match iter.intf.sendrecv(&req) {
            Some(r) => r,
            None => {
                log::error!("Get Device ID command failed");
                return None;
            }
        };

        if rsp.ccode != 0 {
            println!(
                "Get Device ID command failed: {}",
                IpmiError::CompletionCode(rsp.ccode)
            );
            return None;
        }
        let devid: IpmDevidRsp = unsafe { std::ptr::read(rsp.data.as_ptr() as *const _) };
        //IPM_DEV_MANUFACTURER_ID
        //let sdriana = ipmi24toh(&devid.manufacturer_id);

        if !use_builtin && (devid.device_revision & IPM_DEV_DEVICE_ID_SDR_MASK) != 0 {
            if (devid.adtl_device_support & 0x02) == 0 {
                if devid.adtl_device_support & 0x01 != 0 {
                    // 使用设备SDR（静默）
                    use_builtin = true;
                } else {
                    log::error!("Error obtaining SDR info");
                    return None;
                }
            } else {
                // 使用SDR仓库（静默）
            }
        }
        iter.use_builtin = use_builtin;

        // Get SDR repository/device info
        if !iter.use_builtin {
            //主要流程
            // Get repository info
            let mut req = IpmiRq::default();
            req.msg.netfn_mut(IPMI_NETFN_STORAGE);
            req.msg.cmd = GET_SDR_REPO_INFO;

            let rsp: IpmiRs = match iter.intf.sendrecv(&req) {
                Some(r) => r,
                None => {
                    log::error!("Error obtaining SDR info");
                    return None;
                }
            };

            if rsp.fail() {
                println!(
                    "Error obtaining SDR info: {}",
                    IpmiError::CompletionCode(rsp.ccode)
                );
                return None;
            }
            let sdr_info = SdrRepoInfoRs::from_le_bytes(rsp.data.as_slice()).unwrap();

            debug4!("sdr_info      = {:02x?}", sdr_info.as_bytes());

            // IPMIv1.0 == 0x01, IPMIv1.5 == 0x51, IPMIv2.0 == 0x02
            if ![0x51, 0x01, 0x02].contains(&sdr_info.version) {
                log::warn!(
                    "WARNING: Unknown SDR repository version 0x{:02x}",
                    sdr_info.version
                );
            }

            iter.total = sdr_info.count as i32;
            iter.next_id = 0;
            debug3!("SDR free space: {}", sdr_info.free);
            debug3!("SDR records   : {}", sdr_info.count);

            //Rebuild repository if empty
            if sdr_info.count == 0 {
                debug3!("Rebuilding SDRR...");
                // 仓库重建（静默模式）
                //将借用解引
                if !ipmi_sdr_add_from_sensors(iter.intf, 0) {
                    debug3!("Could not build SDRR!");
                    log::error!("Could not build SDRR!");
                    return None;
                }
            }
        } else {
            // Get device SDR info
            let mut req = IpmiRq::default();
            req.msg.netfn_mut(IPMI_NETFN_SE);
            req.msg.cmd = GET_DEVICE_SDR_INFO;

            let rsp = match iter.intf.sendrecv(&req) {
                Some(r) if !r.fail() && r.data_len > 0 => r,
                _ => {
                    log::error!("Error in cmd get sensor SDR info");
                    return None;
                }
            };

            let mut sdr_info = SdrRepoInfoRs::default();
            let bytes = sdr_info.as_bytes_mut();

            bytes.copy_from_slice(&rsp.data[..bytes.len()]);
            iter.total = sdr_info.count as i32;
            iter.next_id = 0;
            debug3!("SDR records   :{}", sdr_info.count);
        };

        // 单独Get reservation ID
        iter.reservation_id = match ipmi_sdr_get_reservation(iter.intf, use_builtin) {
            Some(id) => id,
            None => return None,
        };

        Some(iter)
    }

    /*
    ipmi_sdr_get_next_header,sdr_list_itr.reservation在getheader过程中会更新
    while ((header = ipmi_sdr_get_next_header(intf, sdr_list_itr))) {
        uint8_t *rec;
        struct sdr_record_list *sdrr;

        rec = ipmi_sdr_get_record(intf, header, sdr_list_itr);
    */

    //0xa,0x23
    fn ipmi_sdr_get_header(&mut self) -> Option<SdrGetRs> {
        let mut req: IpmiRq = IpmiRq::default();
        if !self.use_builtin {
            req.msg.netfn_mut(IPMI_NETFN_STORAGE);
            req.msg.cmd = GET_SDR;
        } else {
            req.msg.netfn_mut(IPMI_NETFN_SE);
            req.msg.cmd = GET_DEVICE_SDR;
        }
        //next_id是上一次成功获取header后更新的值
        let mut sdr_rq: SdrGetRq = SdrGetRq {
            reserve_id: self.reservation_id,
            id: self.next_id,
            offset: 0,
            length: 5,
        };

        req.msg.data_len = std::mem::size_of::<SdrGetRq>() as u16;
        req.msg.data = sdr_rq.as_mut_ptr() as *mut u8;
        for _ in 0..5 {
            sdr_rq.reserve_id = self.reservation_id;
            match self.intf.sendrecv(&req) {
                Some(rsp) => {
                    match rsp.ccode {
                        0xc5 => {
                            // Lost reservation
                            println!(
                                "SDR reservation {:04x} cancelled. Retrying...",
                                self.reservation_id
                            );

                            // Random sleep between 0-3 seconds
                            let sleep_time = (rand::random::<u8>() & 3) as u64;
                            std::thread::sleep(std::time::Duration::from_secs(sleep_time));

                            // Renew reservation
                            // TODO
                            match ipmi_sdr_get_reservation(self.intf, self.use_builtin) {
                                Some(id) => self.reservation_id = id,
                                None => {
                                    println!("Unable to renew SDR reservation");
                                    return None;
                                }
                            }
                            continue;
                        }
                        0 => {
                            // Success
                            if rsp.data.len() < 5 {
                                println!("Invalid SDR response length");
                                return None;
                            }

                            //let mut sdr_rs: SdrGetRs = SdrGetRs::default();
                            //提取返回的header
                            let mut sdr_rs: SdrGetRs = match SdrGetRs::from_le_bytes(&rsp.data) {
                                Ok(r) => r,
                                Err(e) => {
                                    log::error!("transfrom to SdrGetRs Failed:{}", e);
                                    return None;
                                }
                            };

                            //请求headerself.next_id不等于返回header,id Handle record ID mismatch
                            if self.next_id != 0 && self.next_id != sdr_rs.header.id {
                                debug5!("SDR record id mismatch: {:04x}", sdr_rs.header.id);
                                sdr_rs.header.id = self.next_id;
                                //返回当前的请求id，方便继续使用这个id请求请求
                            }

                            debug5!("SDR record ID   : 0x{:04x}", self.next_id);
                            debug5!("SDR record type : 0x{:02x}", sdr_rs.header.record_type);
                            debug5!("SDR record next : 0x{:04x}", sdr_rs.next);
                            debug5!("SDR record bytes: {}", sdr_rs.header.length);
                            //self.next_id = sdr_rs.header.id;//更新下个请求id
                            return Some(sdr_rs);
                        }
                        _ => {
                            println!(
                                "Get SDR {:04x} command failed: {}",
                                self.next_id,
                                IpmiError::CompletionCode(rsp.ccode)
                            );
                            continue;
                        }
                    }
                }
                None => {
                    log::error!("Get SDR {:04x} command failed", self.next_id);
                    continue;
                }
            }
        }
        None
    }
    //迭代器里获取的记录,其实header作为迭代器的参数，就不用传入了。
    pub fn ipmi_sdr_get_record(&mut self, header: &SdrRecordHeader) -> Option<Vec<u8>> {
        let len = header.length;
        if len < 1 {
            return None;
        }
        //在堆里分配内存，用来存储返回数据
        let mut data = vec![0u8; len as usize + 1];
        //迭代器的reservation_id信息，
        let mut sdr_rq = SdrGetRq {
            reserve_id: self.reservation_id,
            id: header.id,
            offset: 0,
            length: 0, //临时赋值
        };

        debug4!("Getting {} bytes from SDR at offset {}", len, 0);

        //let x = if condition { 5 } else { "hello" };
        let mut req = IpmiRq {
            msg: {
                if !self.use_builtin {
                    IpmiMessage::new(IPMI_NETFN_STORAGE, GET_SDR)
                } else {
                    IpmiMessage::new(IPMI_NETFN_SE, GET_DEVICE_SDR)
                }
            },
        };

        req.msg.data = sdr_rq.as_mut_ptr() as *mut u8;
        req.msg.data_len = std::mem::size_of::<SdrGetRq>() as u16;

        //未初始化,则初始化
        unsafe {
            if SDR_MAX_READ_LEN_STATIC == 0 {
                //ipmi_intf_get_max_response_data_size
                SDR_MAX_READ_LEN_STATIC =
                    (self.intf.context().get_max_response_data_size() as u8).saturating_sub(2);
                //sdr_max_read_len = self.intf.get_max_response_data_size() as u8 - 2 ;
                //SDR_MAX_READ_LEN.store(sdr_max_read_len.min(0xFE).into(), Ordering::Relaxed);
                SDR_MAX_READ_LEN_STATIC = SDR_MAX_READ_LEN_STATIC.min(0xFE);
            }
        }
        //对同一个headerid进行多次请求，直到获取所有数据
        let mut i: u8 = 0;
        while i < len {
            //header->length
            unsafe {
                //sdr_max_read_len = SDR_MAX_READ_LEN.load(Ordering::Relaxed);
                //sdr_max_read_len = std::cmp::max(sdr_max_read_len, 1);
                //剩余的小于sdr_max_read_len则，下次去剩下的长度
                let remaining = len - i;
                sdr_rq.length = std::cmp::min(remaining, SDR_MAX_READ_LEN_STATIC);

                let current_max = SDR_MAX_READ_LEN_STATIC;
                debug5!(
                    "total:{},index:{},remaining:{} sdr_max_read_len:{}",
                    len,
                    i,
                    remaining,
                    current_max
                );
            } //初始化sdr_rq.length

            sdr_rq.offset = i + 5; // skip 5 header bytes，只区数据部分

            debug5!(
                "Getting {} bytes from SDR at offset {}",
                sdr_rq.length,
                sdr_rq.offset
            );

            //还需要优化
            let rsp: IpmiRs = self.intf.sendrecv(&req)?;

            if rsp.ccode == IPMI_CC_CANT_RET_NUM_REQ_BYTES {
                unsafe {
                    SDR_MAX_READ_LEN_STATIC = sdr_rq.length - 1;
                    if SDR_MAX_READ_LEN_STATIC > 0 {
                        //SDR_MAX_READ_LEN.store(new_max_len, Ordering::Relaxed);
                        continue;
                    }
                }
                return None;
            } else if rsp.ccode == IPMI_CC_RES_CANCELED {
                //log::debug!("SDR reservation cancelled. Sleeping a bit and retrying...");
                thread::sleep(std::time::Duration::from_secs(
                    rand::rng().random_range(0..4),
                ));
                sdr_rq.reserve_id = match ipmi_sdr_get_reservation(self.intf, self.use_builtin) {
                    Some(r) => {
                        self.reservation_id = r;
                        r
                    }
                    None => return None,
                };
                continue;
            }

            if rsp.ccode != 0 || rsp.data.is_empty() {
                return None;
            }
            //从返回数据的下标2开始取,复制到data
            let next_index = i + sdr_rq.length;
            let real_data = &rsp.data[2..2 + sdr_rq.length as usize];
            data[i as usize..next_index as usize].copy_from_slice(real_data);

            debug5!(
                "respone sdr data: {:02x?},len: {}",
                &rsp.data[..sdr_rq.length as usize],
                sdr_rq.length
            );

            //剩余的数据还要分批读取，继续下次循环
            i += sdr_rq.length;
        }
        Some(data)
        // SdrRecordCommonSensor::into_box(&data)
    }
}

//实现next接口，返回header
impl<'a> Iterator for SdrIterator<'a> {
    type Item = SdrRecordHeader;
    fn next(&mut self) -> Option<SdrRecordHeader> {
        if self.finished {
            debug5!("SDR iterator finished");
            return None;
        }
        // 获取header的时候会更新reservation_id，
        // ipmi_sdr_get_record的时候要使用reservation_id
        // 因为next要和reservation_id一起用，不能先next，在用ipmi_sdr_get_record读取。

        match self.ipmi_sdr_get_header() {
            //header记录 SDR_RECORD_TYPE_xxx
            Some(rs) => {
                if rs.next == 0xFFFF {
                    self.finished = true;
                }
                self.next_id = rs.next; // 更新下一个ID
                Some(rs.header)
            }
            None => {
                // 返回None，其实处理错误
                self.finished = true;
                None
            }
        }
    }
}
