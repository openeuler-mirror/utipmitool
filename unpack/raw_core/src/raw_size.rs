/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
use crate::Endianness;
use std::error::Error;
use std::mem::MaybeUninit;

//Sized 表示类型的大小是已知的，即类型的大小在编译时就可以确定
pub trait RawSize: Sized {
    /// 编译期确定的静态大小,用来指示读取的字节数来填充
    const RAW_SIZE: usize;
    const ENDIAN: Endianness = Endianness::Native;
    /// 运行时计算大小（默认使用静态算出的值）
    fn raw_size(&self) -> usize {
        Self::RAW_SIZE
    }

    /// 安全反序列化入口（自动检查长度）
    fn from_bytes_with_endian(bytes: &[u8], endian: Endianness) -> Result<Self, Box<dyn Error>>;
}

// 为基本类型实现 RawSize
macro_rules! impl_raw_size_for_numeric {
    ($($ty:ty),*) => {
        $(
            impl RawSize for $ty {
                const RAW_SIZE: usize = std::mem::size_of::<$ty>();

                fn from_bytes_with_endian(bytes: &[u8], endian: Endianness) -> Result<Self, Box<dyn Error>> {
                    if bytes.len() < Self::RAW_SIZE {
                       return Err("Insufficient data".into());
                    }
                    let bytes_array = bytes[..Self::RAW_SIZE].try_into().unwrap();
                    Ok(match endian {
                        Endianness::Little => Self::from_le_bytes(bytes_array),
                        Endianness::Big => Self::from_be_bytes(bytes_array),
                        Endianness::Native => Self::from_ne_bytes(bytes_array),
                    })
                }
            }
        )*
    };
}

impl RawSize for bool {
    const RAW_SIZE: usize = 1;

    fn from_bytes_with_endian(bytes: &[u8], _endian: Endianness) -> Result<Self, Box<dyn Error>> {
        if bytes.len() < Self::RAW_SIZE {
            return Err("Insufficient data".into());
        }
        Ok(bytes[0] != 0)
    }
}

impl RawSize for char {
    const RAW_SIZE: usize = 1; // C char通常是1字节
                               // 可以新增注解解析功能，定义的时候可以使用注解#(raw_data,size=4)
    fn from_bytes_with_endian(bytes: &[u8], _endian: Endianness) -> Result<Self, Box<dyn Error>> {
        if bytes.len() < Self::RAW_SIZE {
            return Err("Insufficient data".into());
        }
        // 将C char(ASCII)转换为Rust char
        let c = bytes[0] as char;
        if c.is_ascii() {
            Ok(c)
        } else {
            Err("Invalid ASCII character".into())
        }
    }
}

impl_raw_size_for_numeric!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);

// 为数组实现 RawSize
impl<T, const N: usize> RawSize for [T; N]
where
    T: RawSize,
{
    const RAW_SIZE: usize = if N == 0 { 0 } else { T::RAW_SIZE * N };
    fn from_bytes_with_endian(bytes: &[u8], endian: Endianness) -> Result<Self, Box<dyn Error>> {
        if bytes.len() < Self::RAW_SIZE {
            return Err("Insufficient data".into());
        }

        let mut result = MaybeUninit::<Self>::uninit();
        let ptr = result.as_mut_ptr() as *mut T;
        let mut i = 0;
        let offset = 0;

        while i < N {
            match T::from_bytes_with_endian(&bytes[offset..offset + T::RAW_SIZE], endian) {
                Ok(value) => unsafe { ptr.add(i).write(value) },
                Err(e) => {
                    // 清理 [0..i) 的已初始化元素
                    while i > 0 {
                        i -= 1;
                        unsafe { ptr.add(i).drop_in_place() }
                    }
                    return Err(e);
                }
            }
            i += 1;
        }

        Ok(unsafe { result.assume_init() })
    }
}

// 为Vec实现 RawSize（动态大小）

impl<T> RawSize for Vec<T>
where
    T: RawSize,
{
    const RAW_SIZE: usize = 0; // Vec本身没有固定大小

    // fn raw_size(&self) -> usize {
    //     self.iter().map(|x| x.raw_size()).sum()
    // }

    fn from_bytes_with_endian(bytes: &[u8], endian: Endianness) -> Result<Self, Box<dyn Error>> {
        if T::RAW_SIZE == 0 {
            return Err("Cannot deserialize Vec of zero-sized types".into());
        }

        if bytes.len() % T::RAW_SIZE != 0 {
            return Err("Input bytes length must be multiple of element size".into());
        }

        let count = bytes.len() / T::RAW_SIZE;
        let mut vec = Vec::with_capacity(count);
        let mut offset = 0;

        for _ in 0..count {
            let elem = T::from_bytes_with_endian(&bytes[offset..offset + T::RAW_SIZE], endian)?;
            vec.push(elem);
            offset += T::RAW_SIZE;
        }

        Ok(vec)
    }
}

// 为String实现
impl RawSize for String {
    const RAW_SIZE: usize = 0;

    //应该限定使用多少字节完成字符串的反序列化
    fn from_bytes_with_endian(bytes: &[u8], _endian: Endianness) -> Result<Self, Box<dyn Error>> {
        String::from_utf8(bytes.to_vec()).map_err(|e| e.into())
    } //字节序无大小端问题
}

// 为Option实现
impl<T> RawSize for Option<T>
where
    T: RawSize,
{
    const RAW_SIZE: usize = 0; // 大小取决于Some/None

    fn raw_size(&self) -> usize {
        match self {
            Some(x) => x.raw_size(),
            None => 0,
        }
    }

    fn from_bytes_with_endian(bytes: &[u8], endian: Endianness) -> Result<Self, Box<dyn Error>> {
        if bytes.is_empty() {
            Ok(None)
        } else {
            T::from_bytes_with_endian(bytes, endian).map(Some)
        }
    }
}
