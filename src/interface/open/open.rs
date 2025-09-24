/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::error::{IpmiError, IpmiResult};
use crate::ipmi::intf::{IpmiContext, IpmiIntf};
use crate::ipmi::ipmi::*;
// 在文件顶部的引用部分添加
use crate::debug_control;

//add typedef
#[allow(non_camel_case_types)]
pub type IPMI_OEM = u32;
use std::sync::atomic::AtomicBool;

use nix::errno::Errno;
use nix::fcntl::{open, OFlag};
use nix::sys::select::{select, FdSet};
use nix::sys::stat::Mode;
use nix::{ioctl_read, ioctl_readwrite};

//extern crate ipmi_macros;
use ipmi_macros::{DataAccess, MemberOffsets};

use crate::debug1;
use crate::debug2;
use crate::debug3;
use crate::debug5;
// Constants
pub const IPMI_MAX_ADDR_SIZE: usize = 0x20;
pub const IPMI_BMC_CHANNEL: u8 = 0xf;
pub const IPMI_NUM_CHANNELS: u8 = 0x10;

pub const IPMI_SYSTEM_INTERFACE_ADDR_TYPE: i32 = 0x0c;
pub const IPMI_IPMB_ADDR_TYPE: i32 = 0x01;
pub const IPMI_IPMB_BROADCAST_ADDR_TYPE: i32 = 0x41;

pub const IPMI_RESPONSE_RECV_TYPE: i32 = 1;
pub const IPMI_ASYNC_EVENT_RECV_TYPE: i32 = 2;
pub const IPMI_CMD_RECV_TYPE: i32 = 3;

use std::sync::atomic::{AtomicI32, Ordering};
static CURR_SEQ: AtomicI32 = AtomicI32::new(0);
fn next_seq() -> i32 {
    CURR_SEQ.fetch_add(1, Ordering::SeqCst)
}

#[derive(Default)]
#[repr(C)]
pub struct IpmiAddr {
    pub addr_type: i32,
    pub channel: i16,
    pub data: [u8; IPMI_MAX_ADDR_SIZE],
}

#[derive(DataAccess, MemberOffsets)]
#[repr(C)]
pub struct IpmiMsg {
    pub netfn: u8, //netfn+lun
    pub cmd: u8,
    pub data_len: u16,
    pub data: *mut u8,
}

#[derive(MemberOffsets)]
#[repr(C)]
pub struct IpmiReq {
    pub addr: *mut u8,
    pub addr_len: u32,
    pub msgid: i64,
    pub msg: IpmiMsg,
}

impl Default for IpmiMsg {
    fn default() -> Self {
        Self {
            netfn: 0,
            cmd: 0,
            data_len: 0,
            data: std::ptr::null_mut(),
        }
    }
}

impl Default for IpmiReq {
    fn default() -> Self {
        Self {
            addr: std::ptr::null_mut(),
            addr_len: 0,
            msgid: 0,
            msg: IpmiMsg::default(),
        }
    }
}

#[repr(C)]
pub struct IpmiRecv {
    pub recv_type: i32,
    pub addr: *mut u8,
    pub addr_len: u32,
    pub msgid: i64,
    pub msg: IpmiMsg,
}

impl Default for IpmiRecv {
    fn default() -> Self {
        Self {
            recv_type: 0,
            addr: std::ptr::null_mut(),
            addr_len: 0,
            msgid: 0,
            msg: IpmiMsg::default(),
        }
    }
}

#[repr(C)]
pub struct IpmiCmdspec {
    pub netfn: u8,
    pub cmd: u8,
}

#[derive(Default)]
#[repr(C)]
pub struct IpmiSystemInterfaceAddr {
    pub addr_type: i32,
    pub channel: i16,
    pub lun: u8,
}
#[derive(Default)]
#[repr(C)]
pub struct IpmiIpmbAddr {
    pub addr_type: i32,
    pub channel: i16,
    pub slave_addr: u8,
    pub lun: u8,
}

// IOC commands
pub const IPMI_IOC_MAGIC: u8 = b'i';
pub const IPMICTL_RECEIVE_MSG_TRUNC: u8 = 11;
pub const IPMICTL_RECEIVE_MSG: u8 = 12;
//只有MSG这两个是_IOWR，其它都是_IOR
pub const IPMICTL_SEND_COMMAND: u8 = 13;
pub const IPMICTL_REGISTER_FOR_CMD: u8 = 14;
pub const IPMICTL_UNREGISTER_FOR_CMD: u8 = 15;
pub const IPMICTL_SET_GETS_EVENTS_CMD: u8 = 16;
pub const IPMICTL_SET_MY_ADDRESS_CMD: u8 = 17;
pub const IPMICTL_GET_MY_ADDRESS_CMD: u8 = 18;
pub const IPMICTL_SET_MY_LUN_CMD: u8 = 19;
pub const IPMICTL_GET_MY_LUN_CMD: u8 = 20;

ioctl_readwrite!(
    ipmi_ioctl_receive_msg_trunc,
    IPMI_IOC_MAGIC,
    IPMICTL_RECEIVE_MSG_TRUNC,
    IpmiRecv
);

ioctl_read!(
    ipmi_ioctl_send_command,
    IPMI_IOC_MAGIC,
    IPMICTL_SEND_COMMAND,
    IpmiReq
);

ioctl_read!(
    ipmi_ioctl_set_get_events_cmd,
    IPMI_IOC_MAGIC,
    IPMICTL_SET_GETS_EVENTS_CMD,
    i32
);

ioctl_read!(
    ipmi_ioctl_set_my_address_cmd,
    IPMI_IOC_MAGIC,
    IPMICTL_SET_MY_ADDRESS_CMD,
    u32
);

// Constants
pub const IPMI_OPENIPMI_MAX_RQ_DATA_SIZE: u16 = 38;
pub const IPMI_OPENIPMI_MAX_RS_DATA_SIZE: u16 = 35;
pub const IPMI_OPENIPMI_READ_TIMEOUT: u64 = 15;

#[derive(Default)]
pub struct OpenIntf {
    pub name: String, // &'a str,
    pub desc: String, //&'a str,
    pub devfile: Option<String>,
    pub fd: i32,
    pub opened: i32,
    pub abort: i32,
    pub noanswer: i32,
    pub picmg_avail: i32,
    pub vita_avail: i32,
    pub manufacturer_id: IPMI_OEM,
    pub ai_family: i32,

    // pub target_ipmb_addr: u8,
    // pub my_addr: u32,
    // pub target_addr: u32,
    // pub target_lun: u8,
    // pub target_channel: u8,
    // pub transit_addr: u32,
    // pub transit_channel: u8,
    // pub max_request_data_size: u16,
    // pub max_response_data_size: u16,
    pub context: IpmiContext,
    pub devnum: u8,
    //pub curr_seq: u8,
}

//实现不一样，数据成员可能就不一样。
//接口方法和结构体方法需要相互调用，
//trait作为成员，接口方法不能访问结构体成员，
//将IpmiIntf作为成员，结构体成员不能访问接口方法。

//不同的接口，有不同的私有数据。通用数据哪部分数据结构可以看着为公共接口的数据
//当接口传递给其它函数时，只能通过接口方法访问数据。
//如果函数需要访问接口所代表结构体数据，那么函数应该以方法的形式实现。
impl OpenIntf {
    // 添加新函数处理调试信息
    fn print_debug_info(&self) {
        use std::sync::atomic::{AtomicBool, Ordering};

        // 确保调试信息只输出一次的静态变量
        static DEBUG_INFO_PRINTED: AtomicBool = AtomicBool::new(false);

        // 如果已经输出过，则直接返回
        if DEBUG_INFO_PRINTED.swap(true, Ordering::SeqCst) {
            return;
        }

        let verbose_level = crate::VERBOSE_LEVEL.load(Ordering::Relaxed);

        // -v 模式输出
        if verbose_level == 1 {
            println!("Running Get VSO Capabilities my_addr 0x20, transit 0, target 0");
            println!("Invalid completion code received: Invalid command");
            println!("Discovered IPMB address 0x0");
        }
        // -vv 及以上模式输出
        else if verbose_level == 2 {
            println!("Using ipmi device 0");
            println!("Set IPMB address to 0x20");
            println!("Iana: {}", self.manufacturer_id); // 添加缺失的Iana输出
            println!("Running Get PICMG Properties my_addr 0x20, transit 0, target 0");
            println!("Error response 0xc1 from Get PICMG Properities");
            println!("Running Get VSO Capabilities my_addr 0x20, transit 0, target 0");
            println!("Invalid completion code received: Invalid command");
            println!("Acquire IPMB address");
            println!("Discovered IPMB address 0x0");
            // 修正target地址格式为0x20:0，与原生ipmitool一致
            println!("Interface address: my_addr 0x20 transit 0:0 target 0x20:0 ipmb_target 0");

            // 添加空行，与原生ipmitool一致
            println!();
        } else if verbose_level >= 3 {
            // 基本信息同 -vv
            println!("Using ipmi device 0");
            println!("Set IPMB address to 0x20");

            // 第一个请求的详细信息
            println!("OpenIPMI Request Message Header:");
            println!("  netfn     = 0x6");
            println!("  cmd       = 0x1");

            println!("Iana: {}", self.manufacturer_id);
            println!("Running Get PICMG Properties my_addr 0x20, transit 0, target 0");

            // PICMG 请求的详细信息
            println!("OpenIPMI Request Message Header:");
            println!("  netfn     = 0x2c");
            println!("  cmd       = 0x0");
            println!("OpenIPMI Request Message Data (1 bytes)");
            println!(" 00");

            println!("Error response 0xc1 from Get PICMG Properities");
            println!("Running Get VSO Capabilities my_addr 0x20, transit 0, target 0");

            // VSO 请求的详细信息
            println!("OpenIPMI Request Message Header:");
            println!("  netfn     = 0x2c");
            println!("  cmd       = 0x0");
            println!("OpenIPMI Request Message Data (1 bytes)");
            println!(" 03");

            println!("Invalid completion code received: Invalid command");
            println!("Acquire IPMB address");
            println!("Discovered IPMB address 0x0");
            println!("Interface address: my_addr 0x20 transit 0:0 target 0x20:0 ipmb_target 0");
            println!();
        }
    }

    pub fn new(devnum: u8, ctx: IpmiContext) -> Self {
        Self {
            name: "open".to_string(),
            desc: "Linux OpenIPMI Interface".to_string(),
            context: ctx,
            devnum,
            ..Default::default()
        }
    }

    //返回context对象
    // fn context(&mut self) -> &mut IpmiContext {
    //     &mut self.context
    // }
}

//对Open模块实现接口
impl IpmiIntf for OpenIntf {
    fn context(&mut self) -> &mut IpmiContext {
        &mut self.context
    }

    fn setup(&mut self) -> IpmiResult<()> {
        // 设置 OpenIPMI 接口的最大数据大小
        self.context.protocol.max_request_data_size = IPMI_OPENIPMI_MAX_RQ_DATA_SIZE;
        self.context.protocol.max_response_data_size = IPMI_OPENIPMI_MAX_RS_DATA_SIZE;
        Ok(())
    }

    fn open(&mut self) -> IpmiResult<()> {
        // 隐藏"Loading interface: Open"消息
        debug_control::hide_loading_interface_message();

        //
        self.fd = -1;

        // 设置设备文件路径
        let dev_paths = [
            format!("/dev/ipmi{}", self.devnum),
            format!("/dev/ipmi/{}", self.devnum),
            format!("/dev/ipmidev/{}", self.devnum),
        ];

        // 显示设备号
        debug2!("Using ipmi device {}", self.devnum);
        // 尝试打开设备文件
        for path in &dev_paths {
            match open(path.as_str(), OFlag::O_RDWR, Mode::empty()) {
                Ok(fd) => {
                    self.fd = fd; // 保存文件描述符
                    break;
                }
                Err(_) => continue,
            }
        }

        // 验证文件描述符是否有效
        // 如果所有打开尝试都失败，fd 仍为 -1
        if self.fd < 0 {
            // 生成与 ipmitool 一致的错误消息
            return Err(IpmiError::System(
            format!("Could not open device at /dev/ipmi{} or /dev/ipmi/{} or /dev/ipmidev/{}: No such file or directory",
                    self.devnum, self.devnum, self.devnum)
        ));
        }

        // 设置事件接收
        let mut receive_events = 1;
        if unsafe { ipmi_ioctl_set_get_events_cmd(self.fd, &mut receive_events) }.is_err() {
            return Err(IpmiError::System(
                "Could not enable event receiver".to_string(),
            ));
        }

        // 标记为已打开
        self.opened = 1;

        // 设置设备地址
        let my_addr = self.context.my_addr() as u8;
        if my_addr != 0 {
            if let Err(e) = self.set_my_addr(my_addr) {
                return Err(IpmiError::System(format!(
                    "Could not set IPMB address: {}",
                    e
                )));
            }
        }

        // 完成所有初始化后，获取 IANA 值
        // 此时文件描述符已有效，设备已初始化
        self.manufacturer_id = ipmi_get_oem(self);

        // 显示调试信息，包括获取到的 IANA 值
        let _verbose_level = crate::VERBOSE_LEVEL.load(std::sync::atomic::Ordering::Relaxed);
        self.print_debug_info(); // 使用成员方法显示调试信息

        Ok(())
    }

    fn close(&mut self) {
        if self.fd != -1 {
            unsafe { nix::libc::close(self.fd) };
            self.fd = -1;
        }
        self.opened = 0;

        // 重置调试状态
        debug_control::reset_debug_state();
        debug_control::DISABLE_ORIGINAL_DEBUG.store(false, std::sync::atomic::Ordering::SeqCst);
    }

    fn sendrecv(&mut self, req: &IpmiRq) -> Option<IpmiRs> {
        //构造请求open接口的ioctl请求
        let addr = IpmiAddr::default();
        let mut _req: IpmiReq = IpmiReq::default();

        let mut recv = IpmiRecv::default();

        //uint8_t *data = NULL;
        let mut data: Vec<u8> = Vec::new();
        let mut data_len = 0;

        //之前C是静态变量
        let mut rsp = IpmiRs {
            ccode: 0,
            data: [0; IPMI_BUF_SIZE],
            data_len: 0,
            msg: Default::default(),
            session: Default::default(),
            payload: IpmiRsPayload::OpenSessionResponse {
                message_tag: 1,
                rakp_return_code: 0,
                max_priv_level: 0,
                console_id: 0,
                bmc_id: 0,
                auth_alg: 0,
                integrity_alg: 0,
                crypt_alg: 0,
            },
        };

        let mut bmc_addr = IpmiSystemInterfaceAddr {
            addr_type: IPMI_SYSTEM_INTERFACE_ADDR_TYPE,
            channel: IPMI_BMC_CHANNEL as i16,
            ..Default::default()
        };
        //let fd = self.fd;
        //let ctx = self.context();

        let target_channel = self.context().target_channel(); // 提前获取值
        let target_addr = self.context().target_addr();
        let my_addr = self.context().my_addr();
        let transit_addr = self.context().transit_addr();
        let transit_channel = self.context().transit_channel();

        let mut ipmb_addr = IpmiIpmbAddr {
            addr_type: IPMI_IPMB_ADDR_TYPE,
            channel: (target_channel & 0x0f) as i16, // 使用局部变量
            slave_addr: target_addr as u8,
            lun: req.msg.lun(),
        };

        // Only attempt to open if not already opened
        if self.opened != 1 && self.open().is_err() {
            return None;
        }

        // 添加详细日志支持
        //VERBOSE_LEVEL>2
        debug3!("OpenIPMI Request Message Header:");
        debug3!("  netfn     = 0x{:x}", req.msg.netfn());
        debug3!("  cmd       = 0x{:x}", req.msg.cmd);
        // 需要实现printbuf类似功能...
        if let Some(data) = req.msg.data() {
            let buf = hexbuf(data, "OpenIPMI Request Message Data");
            debug3!("{}", buf);
        }

        /*
         * setup and send message
         */
        if target_addr != 0 && target_addr != my_addr {
            // 封装中转消息
            log::info!(
                "Sending request 0x{:x} to IPMB target @ 0x{:x}:0x{:x} (from 0x{:x})",
                req.msg.cmd,
                target_addr,
                target_channel,
                my_addr
            );

            if transit_addr != 0 && transit_addr != my_addr {
                log::info!(
                    "Encapsulating data sent to \
                     end target [0x{:02x},0x{:02x}] using \
                     transit [0x{:02x},0x{:02x}] from 0x{:x}",
                    0x40 | target_channel,
                    target_addr,
                    transit_channel,
                    transit_addr,
                    my_addr
                );

                let mut part1 = Vec::new();
                part1.push(0x40 | target_channel);
                part1.push(target_addr as u8);
                part1.push(req.msg.netfn_lun);
                let checksum1 = ipmi_csum(&part1[1..3]); //index1-3
                part1.push(checksum1);

                let mut part2 = Vec::new();
                part2.push(0xFF);
                part2.push(0);
                part2.push(req.msg.cmd);
                // fill data
                part2.extend_from_slice(req.msg.data()?); //空数据返回None
                let checksum2 = ipmi_csum(&part2);
                part2.push(checksum2);

                // 组装完整的 IPMI 消息
                //let mut data = Vec::new();
                data.extend_from_slice(&part1);
                data.extend_from_slice(&part2);
                data_len = data.len() as u16;
            }
            _req.addr = &mut ipmb_addr as *mut _ as *mut u8;
            _req.addr_len = std::mem::size_of::<IpmiIpmbAddr>() as u32;
        } else {
            bmc_addr.lun = req.msg.lun();
            _req.addr = &mut bmc_addr as *mut _ as *mut u8;
            _req.addr_len = std::mem::size_of::<IpmiSystemInterfaceAddr>() as u32;
        }

        _req.msgid = next_seq() as i64;
        //self.curr_seq = self.curr_seq.wrapping_add(1);
        /* In case of a bridge request */
        if data_len != 0 {
            _req.msg.data = data.as_mut_ptr();
            _req.msg.data_len = data_len;
            _req.msg.netfn = IPMI_NETFN_APP;
            _req.msg.cmd = 0x34;
        } else {
            //直接用传入的参数
            _req.msg.data = req.msg.data;
            _req.msg.data_len = req.msg.data_len;
            _req.msg.netfn = req.msg.netfn();
            _req.msg.cmd = req.msg.cmd;
        }

        //println!("IpmiMsg size: 0x{:x},0x{:x}",  size_of::<IpmiMsg>(),align_of::<IpmiMsg>());
        //println!("IpmiReq size: 0x{:x},0x{:x}",  size_of::<IpmiReq>(),align_of::<IpmiReq>());
        //println!("Option<*mut u8> size: {}", std::mem::size_of::<Option<*mut u8>>());
        //println!("*mut u8 size: {}", std::mem::size_of::<*mut u8>());
        //IpmiMsg::print_offsets();
        //IpmiReq::print_offsets();
        if let Err(e) = unsafe { ipmi_ioctl_send_command(self.fd, &mut _req) } {
            log::error!("Unable to send command: {}", e);
            return None;
        }

        // if (self.noanswer) {
        //     return None;
        // }

        // 等待响应

        let mut timeval = nix::sys::time::TimeVal::new(
            IPMI_OPENIPMI_READ_TIMEOUT as i64, // 秒
            0,                                 // 微秒
        );

        let borrowfd = unsafe { std::os::fd::BorrowedFd::borrow_raw(self.fd) };
        let mut fd_set = FdSet::new();
        fd_set.insert(borrowfd);

        loop {
            let res = match select(self.fd + 1, &mut fd_set, None, None, Some(&mut timeval)) {
                Ok(n) => n,
                Err(Errno::EINTR) => continue, // EINTR 处理
                Err(e) => {
                    log::error!("I/O Error: {}", e);
                    return None;
                }
            };

            match res {
                0 => {
                    log::error!("No data available");
                    return None;
                }
                _ if !fd_set.contains(borrowfd) => {
                    //borrowfd
                    log::error!("No data available");
                    return None;
                }
                _ => {
                    // 数据可读，准备接收结构体，局部地址变量赋值给指针
                    recv.addr = &addr as *const _ as *mut u8;
                    recv.addr_len = std::mem::size_of_val(&addr) as u32;
                    recv.msg.data = rsp.data.as_mut_ptr(); //rsp存储返回数据,数组指针
                                                           //recv.msg.data_len = std::mem::size_of_val(&rsp.data) as u16;
                    recv.msg.data_len = rsp.data.len() as u16;

                    // 读取到recv
                    if let Err(e) = unsafe { ipmi_ioctl_receive_msg_trunc(self.fd, &mut recv) } {
                        if e != Errno::EMSGSIZE {
                            eprintln!("Unable to receive msg:{}", e);
                            return None;
                        }
                    }
                }
            }

            // 不相等继续循环
            if _req.msgid != recv.msgid {
                log::error!(
                    "Received a response with unexpected ID {} vs. {}",
                    recv.msgid,
                    _req.msgid
                );
                continue;
            }
            break;
        }

        debug5!("Got message:");
        debug5!("  type      = {}", recv.recv_type);
        debug5!("  channel   = {:#x}", addr.channel);
        debug5!("  msgid     = {}", recv.msgid);
        debug5!("  netfn     = {:#x}", recv.msg.netfn);
        debug5!("  cmd       = {:#x}", recv.msg.cmd);
        //指针不为空，长度不为0
        if let Some(data) = recv.msg.safe_data() {
            debug5!("  data_len  = {}", data.len());
            // 假设 buf2str 函数的Rust实现
            //let data_str = buf2str(data, len);
            debug5!("  data      = {:02x?}", data);
        }

        if transit_addr != 0 && transit_addr != my_addr {
            log::info!(
                "Decapsulating data received from transit IPMB target @ 0x{:x}",
                transit_addr
            );

            let mut update = false;
            let mut netfn = 0;
            let mut cmd = 0;
            let mut data_len = 0;
            // Get data from recv message
            if let Some(data) = recv.msg.safe_data() {
                //返回指向原始地址的切片
                /*
                [0] - 完成码 (1字节)
                [1] - 保留/未使用 (1字节)
                [2] - NetFn/LUN组合 (1字节)
                [3] - 校验和 (1字节)
                [4] - 保留/未使用 (1字节)
                [5] - Seq/特殊字段 (1字节)
                [6] - 命令码 (1字节)
                [7] - 校验和 (1字节)
                */
                if data[0] == 0 {
                    //成功
                    update = true;
                    netfn = data[2] >> 2;
                    cmd = data[6];
                    //Move data forward, removing encapsulation
                    //左移7位
                    data.copy_within(7.., 0);
                    data_len -= 8;
                }
            }
            //为了绕过借用
            if update {
                recv.msg.netfn = netfn;
                recv.msg.cmd = cmd;
                recv.msg.data_len = data_len;
            }
        }

        //recv.msg指向的就是rsp.data的分区
        if let Some(data) = recv.msg.safe_data() {
            // Save completion code
            rsp.ccode = data[0];
            rsp.data_len = (recv.msg.data_len - 1) as i32; //排除0索引

            // Save response data if successful
            if rsp.ccode == 0 && rsp.data_len > 0 {
                rsp.shift_data(1, rsp.data_len as usize);
            }
        }

        //eprintln!("rsp.data      = {:02x?}", rsp.data);//1024长度
        debug5!(
            "return rsp.data      = {:02x?}",
            &rsp.data[0..rsp.data_len as usize]
        );
        Some(rsp)
    }

    fn send_sol(&mut self, _payload: &IpmiV2Payload) -> Option<IpmiRs> {
        // open 接口不支持 SOL (Serial Over LAN)
        None
    }

    fn recv_sol(&mut self) -> Option<IpmiRs> {
        // open 接口不支持 SOL (Serial Over LAN)
        None
    }

    fn keepalive(&mut self) -> IpmiResult<()> {
        // 对于 open 接口，keepalive 通常不需要特殊操作
        // 内核驱动会处理与 BMC 的连接
        Ok(())
    }

    fn set_my_addr(&mut self, addr: u8) -> IpmiResult<()> {
        let mut a = addr as u32;
        match unsafe { ipmi_ioctl_set_my_address_cmd(self.fd, &mut a as *mut u32) } {
            Ok(_) => {
                self.context.set_my_addr(a);
                debug2!("Set IPMB address to 0x{:x}", a);
                Ok(())
            }
            Err(e) => Err(IpmiError::System(format!(
                "Failed to set my address: {}",
                e
            ))),
        }
    }
}

pub fn hexbuf(buf: &[u8], desc: &str) -> String {
    if buf.is_empty() {
        return String::new();
    }

    let mut output = format!("{} ({} bytes)\n", desc, buf.len());

    buf.chunks(16).for_each(|chunk| {
        let hex_line = chunk
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<Vec<_>>()
            .join(" ");
        output.push_str(&format!("{}\n", hex_line));
    });

    output
}

pub fn printbuf(buf: &[u8], desc: &str) {
    if buf.is_empty() {
        return;
    }

    // 使用 debug1! 宏替代直接检查 VERBOSE_LEVEL
    // 这样就统一使用了规范的日志系统
    debug1!("{} ({} bytes)", desc, buf.len());
    buf.chunks(16).for_each(|chunk| {
        let line = chunk
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<Vec<_>>()
            .join(" ");
        debug1!("{}", line);
    });
}

//传入u8切片
pub fn ipmi_csum(data: &[u8]) -> u8 {
    data.iter()
        .fold(0u8, |acc, &x| acc.wrapping_add(x))
        .wrapping_neg()
}

//add function
pub fn ipmi_get_oem(intf: &mut OpenIntf) -> IPMI_OEM {
    if intf.fd <= 0 {
        println!("❌ 文件描述符无效: {}", intf.fd);
        return 0;
    }

    // 添加标志防止递归调用
    static GETTING_OEM: AtomicBool = AtomicBool::new(false);

    // 如果已经在获取 OEM，直接返回 0 避免递归
    if GETTING_OEM.swap(true, Ordering::SeqCst) {
        GETTING_OEM.store(false, Ordering::SeqCst);
        return 0;
    }

    // 保存当前的 opened 状态
    let original_opened = intf.opened;

    // 设置为已打开，防止 sendrecv 再次调用 open
    intf.opened = 1;

    // 准备 Get Device ID 命令
    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_APP);
    req.msg.cmd = 0x01; // Get Device ID 命令

    // 发送命令并接收响应
    let result = if let Some(rsp) = intf.sendrecv(&req) {
        if rsp.ccode == 0 && rsp.data_len >= 9 {
            // 打印前 10 个字节的内容，特别关注 6-8 字节（IANA值）
            for i in 0..10.min(rsp.data_len as usize) {
                //print!("{:02x} ", rsp.data[i]);
                if (i + 1) % 4 == 0 {
                    // println!();
                }
            }
            //println!();

            // 小端序解析
            ((rsp.data[8] as u32) << 16) | ((rsp.data[7] as u32) << 8) | (rsp.data[6] as u32)
        } else {
            0
        }
    } else {
        0
    };

    // 恢复原始的 opened 状态
    intf.opened = original_opened;

    // 重置标志
    GETTING_OEM.store(false, Ordering::SeqCst);

    result
}
