/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */


/*
struct valstr {
	uint32_t val;
	const char * str;
};
struct oemvalstr {
	uint32_t oem;
   uint16_t val;
	const char * str;
};
*/

    //str: *const i8,
struct valstr {
    val: u32,
    str: &'static str,
}

struct oemvalstr {
    oem: u32,
    val: u16,
    str: &'static str,
}