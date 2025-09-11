/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use unpack::{RawSize, RawDeserialize};

#[test]
fn test_primitive_sizes() {
    assert_eq!(u32::RAW_SIZE, 4);
    assert_eq!(f64::RAW_SIZE, 8);
}

#[test]
fn test_struct_size() {
    #[derive(RawSize)]
    #[repr(C)]
    struct Test {
        a: u32,
        b: u16,
    }

    assert_eq!(Test::RAW_SIZE, 6);
}

#[test]
fn test_deserialize() {
    #[derive(RawSize, RawDeserialize, Debug, PartialEq)]
    #[repr(C)]
    struct Packet {
        id: u32,
        flags: u8,
    }

    let data = [0x78u8, 0x56, 0x34, 0x12, 0xFF]; // Little-endian
    let pkt = Packet::from_bytes(&data).unwrap();

    assert_eq!(pkt.id, 0x12345678);
    assert_eq!(pkt.flags, 0xFF);
}

#[test]
fn test_packed_struct() {
    #[derive(RawSize, RawDeserialize)]
    #[repr(packed)]
    struct Packed {
        a: u8,
        b: u32,
    }

    assert_eq!(Packed::RAW_SIZE, 5);
    let data = [0x01, 0x11, 0x22, 0x33, 0x44];
    let p = Packed::from_bytes(&data).unwrap();
    assert_eq!(p.b, 0x44332211); // 注意字节序
}