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
/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
// 对应C结构体的Rust实现
struct SelOemMsgRec {
    values: [i32; 14],    // 对应value[14]
    strings: [Option<String>; 14], // 对应string[14]
    text: String,         // 对应*text
}

// 新的消息处理结构
struct OemMessageContext<'a> {
    evt: &'a [u8],         // 原始事件数据字节流
    csv_output: bool,      // 输出模式标志
    oem_records: &'a [SelOemMsgRec], // OEM消息记录集合
}

impl<'a> OemMessageContext<'a> {
    // Rust风格的核心匹配逻辑
    fn oem_match(&self, rec: &SelOemMsgRec) -> bool {
        // 对应C的SEL_BYTE宏
        let sel_byte = |n: usize| n - 3;

        // 逐个字节匹配条件
        self.evt[2] == rec.values[sel_byte(3)] as u8
            && self.check_byte(3, rec.values[sel_byte(4)])
            && self.check_byte(4, rec.values[sel_byte(5)])
            && self.check_byte(5, rec.values[sel_byte(6)])
            && self.check_byte(6, rec.values[sel_byte(7)])
            && self.check_byte(10, rec.values[sel_byte(11)])
            && self.check_byte(11, rec.values[sel_byte(12)])
    }

    // 带通配符的字节检查
    fn check_byte(&self, idx: usize, val: i32) -> bool {
        val < 0 || (idx < self.evt.len() && self.evt[idx] == val as u8)
    }

    // 主处理函数
    pub fn process_messages(&self) -> String {
        let mut output = String::new();

        for rec in self.oem_records {
            if self.oem_match(rec) {
                // 基础文本输出
                let base = if self.csv_output {
                    format!(",\"{}\"", rec.text)
                } else {
                    format!(" | {}", rec.text)
                };
                output.push_str(&base);

                // 动态字段处理
                for j in 4..17 {
                    let byte_idx = j - 3;
                    if byte_idx < rec.values.len()
                        && rec.values[byte_idx] == -3
                        && byte_idx < self.evt.len()
                    {
                        let value_str = if self.csv_output {
                            format!(",{}={:#04x}",
                                rec.strings[byte_idx].as_deref().unwrap_or(""),
                                self.evt[byte_idx])
                        } else {
                            format!(" {} = {:#04x}",
                                rec.strings[byte_idx].as_deref().unwrap_or(""),
                                self.evt[byte_idx])
                        };
                        output.push_str(&value_str);
                    }
                }
            }
        }
        output
    }
}