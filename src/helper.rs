/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
pub fn ipmi24toh(data: &[u8; 3]) -> u32 {
    //u32::from(data[0]) | (u32::from(data[1]) << 8) | (u32::from(data[2]) << 16)
    u32::from_le_bytes([data[0], data[1], data[2], 0])
}

// fn val2str(val: u32, map: &HashMap<u32, &'static str>) -> &'static str {
//     map.get(&val).copied().unwrap_or("Unknown value")
// }

pub fn buf2str(data: &[u8], len: usize) -> String {
    data.iter()
        .take(len)
        .map(|byte| format!("{:02x}", byte))
        .collect::<Vec<_>>()
        .join(" ")
}
