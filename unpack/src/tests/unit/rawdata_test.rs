/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use unpack::traits::RawSize;
use unpack::Endianness;

// 基础嵌套结构体
#[derive(Debug, PartialEq, RAWDATA)]
#[repr(C)]
struct Inner {
    a: u16,
    b: u8,
}

// 包含数组和嵌套结构体的主结构体
#[derive(Debug, PartialEq, RAWDATA)]
#[repr(C)]
#[raw_data(endian = "little")]
struct TestStruct {
    id: u8,
    values: [u16; 3],  // 数组类型
    inner: Inner,      // 嵌套结构体
    flag: u8,
}

#[test]
fn test_rawdata_derive() {
    // 准备测试数据 (小端字节序)
    let data = vec![
        0x01,                   // id: 0x01
        0x34, 0x12,             // values[0]: 0x1234
        0x78, 0x56,             // values[1]: 0x5678
        0xBC, 0x9A,             // values[2]: 0x9ABC
        0xEF, 0xCD,             // inner.a: 0xCDEF
        0x11,                   // inner.b: 0x11
        0xFF,                   // flag: 0xFF
    ];

    // 反序列化
    let result = TestStruct::from_bytes_with_endian(&data, Endianness::Little).unwrap();

    // 验证结果
    assert_eq!(result.id, 0x01);
    assert_eq!(result.values, [0x1234, 0x5678, 0x9ABC]);
    assert_eq!(result.inner, Inner { a: 0xCDEF, b: 0x11 });
    assert_eq!(result.flag, 0xFF);

    // 验证RAW_SIZE常量
    assert_eq!(TestStruct::RAW_SIZE, 1 + 6 + 3 + 1); // id + values + inner + flag
    assert_eq!(Inner::RAW_SIZE, 2 + 1); // a + b
}

#[test]
fn test_rawdata_derive_big_endian() {
    // 准备测试数据 (大端字节序)
    let data = vec![
        0x01,                   // id: 0x01
        0x12, 0x34,             // values[0]: 0x1234
        0x56, 0x78,             // values[1]: 0x5678
        0x9A, 0xBC,             // values[2]: 0x9ABC
        0xCD, 0xEF,             // inner.a: 0xCDEF
        0x11,                   // inner.b: 0x11
        0xFF,                   // flag: 0xFF
    ];

    // 反序列化 (结构体标注为小端，但传入大端数据会失败)
    let result = TestStruct::from_bytes_with_endian(&data, Endianness::Big);
    assert!(result.is_err());
}