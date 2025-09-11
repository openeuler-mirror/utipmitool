/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
pub use raw_core::raw_size::RawSize;
pub use raw_core::Endianness;
//traits 模块，并重新导出 RawSize trait
// pub mod traits {
//     pub use crate::raw_size::RawSize as RawSize;
// }

// pub enum Endianness {
//     Little,
//     Big,
//     Native
// }

// 重新导出 macro 模块
// pub mod macros {
//     // 重导出 RawSize 宏
//     pub use raw_derive::{RAWDATA};
// }

//pub use raw_derive::RAWDATA
//
