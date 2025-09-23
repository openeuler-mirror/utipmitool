/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::commands::mc::{IpmDevidRsp, BMC_GET_DEVICE_ID};
use crate::error::IpmiError;
use crate::helper::ipmi24toh;
use crate::ipmi::intf::IpmiIntf;
use crate::ipmi::ipmi::*;

pub struct OemValStr {
    #[allow(dead_code)]
    oem: IPMI_OEM,
    #[allow(dead_code)]
    code: u16,
    #[allow(dead_code)]
    desc: &'static str,
}

pub const IPMI_OEM_PRODUCT_INFO: &[OemValStr] = &[
    /* Keep OEM grouped together */

    /* For ipmitool debugging */
    OemValStr {
        oem: IPMI_OEM::Debug,
        code: 0x1234,
        desc: "Great Debuggable BMC",
    },
    /* Intel stuff, thanks to Tim Bell */
    OemValStr {
        oem: IPMI_OEM::Intel,
        code: 0x000C,
        desc: "TSRLT2",
    },
    OemValStr {
        oem: IPMI_OEM::Intel,
        code: 0x001B,
        desc: "TIGPR2U",
    },
    OemValStr {
        oem: IPMI_OEM::Intel,
        code: 0x0022,
        desc: "TIGI2U",
    },
    OemValStr {
        oem: IPMI_OEM::Intel,
        code: 0x0026,
        desc: "Bridgeport",
    },
    OemValStr {
        oem: IPMI_OEM::Intel,
        code: 0x0028,
        desc: "S5000PAL",
    },
    OemValStr {
        oem: IPMI_OEM::Intel,
        code: 0x0029,
        desc: "S5000PSL",
    },
    OemValStr {
        oem: IPMI_OEM::Intel,
        code: 0x0100,
        desc: "Tiger4",
    },
    OemValStr {
        oem: IPMI_OEM::Intel,
        code: 0x0103,
        desc: "McCarran",
    },
    OemValStr {
        oem: IPMI_OEM::Intel,
        code: 0x0800,
        desc: "ZT5504",
    },
    OemValStr {
        oem: IPMI_OEM::Intel,
        code: 0x0808,
        desc: "MPCBL0001",
    },
    OemValStr {
        oem: IPMI_OEM::Intel,
        code: 0x0811,
        desc: "TIGW1U",
    },
    OemValStr {
        oem: IPMI_OEM::Intel,
        code: 0x4311,
        desc: "NSI2U",
    },
    /* Kontron */
    OemValStr {
        oem: IPMI_OEM::Kontron,
        code: 4000,
        desc: "AM4000 AdvancedMC",
    },
    OemValStr {
        oem: IPMI_OEM::Kontron,
        code: 4001,
        desc: "AM4001 AdvancedMC",
    },
    OemValStr {
        oem: IPMI_OEM::Kontron,
        code: 4002,
        desc: "AM4002 AdvancedMC",
    },
    OemValStr {
        oem: IPMI_OEM::Kontron,
        code: 4010,
        desc: "AM4010 AdvancedMC",
    },
    OemValStr {
        oem: IPMI_OEM::Kontron,
        code: 5503,
        desc: "AM4500/4520 AdvancedMC",
    },
    OemValStr {
        oem: IPMI_OEM::Kontron,
        code: 5504,
        desc: "AM4300 AdvancedMC",
    },
    OemValStr {
        oem: IPMI_OEM::Kontron,
        code: 5507,
        desc: "AM4301 AdvancedMC",
    },
    OemValStr {
        oem: IPMI_OEM::Kontron,
        code: 5508,
        desc: "AM4330 AdvancedMC",
    },
    // ... (continuing with similarly formatted entries)

    // YADRO
    OemValStr {
        oem: IPMI_OEM::YADRO,
        code: 0x0001,
        desc: "VESNIN BMC",
    },
    OemValStr {
        oem: IPMI_OEM::YADRO,
        code: 0x000A,
        desc: "TATLIN.UNIFIED Storage Controller BMC",
    },
    OemValStr {
        oem: IPMI_OEM::YADRO,
        code: 0x0014,
        desc: "VEGMAN Series BMC",
    },
    OemValStr {
        oem: IPMI_OEM::YADRO,
        code: 0x0015,
        desc: "TATLIN.ARCHIVE/xS BMC",
    },
    //OemValStr { oem: 0xffffff, code: 0xffff, desc: "" }
];

//业务层面的接口,sel.c
pub fn ipmi_get_oem(intf: &mut dyn IpmiIntf) -> IPMI_OEM {
    // 检查文件描述符有效性
    // if intf.fd == 0 {
    //     //TODO
    //     // if( sel_iana != IPMI_OEM_UNKNOWN ){
    //     //     return sel_iana;
    //     // }
    //     return IPMI_OEM::Unknown;
    // }

    // 返回已缓存的制造商 ID
    // if intf.opened != 0 && self.manufacturer_id != IPMI_OEM::Unknown {
    //     return self.manufacturer_id;
    // }

    // 准备请求
    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_APP);
    req.msg.cmd = BMC_GET_DEVICE_ID;
    req.msg.data_len = 0;

    // 发送请求并获取响应
    let rsp = match intf.sendrecv(&req) {
        Some(r) => r,
        None => {
            log::error!("Get Device ID command failed");
            return IPMI_OEM::Unknown;
        }
    };

    // 检查返回码
    if rsp.ccode != 0 {
        println!(
            "Get Device ID command failed: {:#x} {}",
            rsp.ccode,
            IpmiError::CompletionCode(rsp.ccode)
        );
        return IPMI_OEM::Unknown;
    }

    // 解析设备 ID 响应
    //let devid =
    match IpmDevidRsp::from_le_bytes(rsp.data.as_slice()) {
        Ok(devid) => {
            IPMI_OEM::try_from(ipmi24toh(&devid.manufacturer_id)).unwrap_or(IPMI_OEM::Unknown)
        }
        Err(_) => {
            println!("Invalid device ID response");
            IPMI_OEM::Unknown
        }
    }
}

pub fn ipmi_get_oem_id(intf: &mut dyn IpmiIntf) -> IPMI_OEM {
    // Execute a Get Board ID command to determine the board
    let mut req = IpmiRq::default();
    req.msg.netfn_mut(IPMI_NETFN_TSOL);
    req.msg.cmd = 0x21;
    req.msg.data = std::ptr::null_mut();
    req.msg.data_len = 0;

    let rsp = match intf.sendrecv(&req) {
        Some(rsp) => rsp,
        None => {
            log::error!("Get Board ID command failed");
            return IPMI_OEM::Unknown;
        }
    };
    if rsp.ccode != 0 {
        // 静默处理：某些系统不支持Board ID命令，这是正常的
        log::debug!(
            "Get Board ID command failed: {}",
            IpmiError::CompletionCode(rsp.ccode)
        );
        return IPMI_OEM::Unknown;
    }
    if rsp.data_len < 2 {
        eprintln!("Get Board ID response too short");
        return IPMI_OEM::Unknown;
    }
    let oem_id = u32::from_le_bytes([rsp.data[0], rsp.data[1], 0, 0]);
    //eprintln!("Board ID: {:x}", oem_id);
    IPMI_OEM::try_from(oem_id).unwrap_or(IPMI_OEM::Unknown)
}

// struct ipmi_oem_handle {
// 	const char * name;
// 	const char * desc;
// 	int (*setup)(struct ipmi_intf * intf);
// };
