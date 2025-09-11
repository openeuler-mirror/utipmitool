/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
#![allow(dead_code)]
#![allow(unused_imports)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, DeriveInput, Meta};
/// 原始数据对齐注解
// #[derive(Copy, Clone)]
// pub enum DataAlignment {
//     C,      // 数据按C标准对齐,Aligned
//     Packed  // 数据无对齐保证,Unaligned
// }

// pub enum DataAlignment {
//     Aligned,    // 原始数据已对齐
//     Unaligned   // 原始数据未对齐
// }

pub enum StructLayout {
    C,      // #[repr(C)]
    Packed, // #[repr(packed)]
}
/*
/// 结构体注解示例
#[raw_data(C)] // 表示原始数据按C对齐
#[repr(C)]
struct NetworkHeader {
    magic: u32,
    length: u16
}

/// packed注解示例
#[raw_data(packed)]
#[repr(C)]
struct PackedData {
    timestamp: u64,
    value: u16
}
 */

//解析[raw_data(endian="little")]
pub fn parse_endian_anno(attrs: &[Attribute]) -> Option<String> {
    let mut endian = None;

    for attr in attrs {
        if attr.path().is_ident("raw_data") {
            if let Ok(meta) = attr.parse_args_with(
                syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
            ) {
                for m in meta {
                    if let Meta::NameValue(nv) = m {
                        if nv.path.is_ident("endian") {
                            if let syn::Expr::Lit(lit) = nv.value {
                                if let syn::Lit::Str(s) = lit.lit {
                                    endian = Some(s.value());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    endian
}

//解析[raw_data(packed)]
pub fn parse_layout_anno(attrs: &[Attribute]) -> (StructLayout, StructLayout) {
    let mut data_layout = StructLayout::Packed; //默认原始数据未对齐
    let mut target_layout = StructLayout::C; //默认目标布局为C

    for attr in attrs {
        if attr.path().is_ident("raw_data") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("C") {
                    data_layout = StructLayout::C;
                    return Ok(());
                } else if meta.path.is_ident("packed") {
                    data_layout = StructLayout::Packed;
                    return Ok(());
                }
                Err(syn::Error::new_spanned(
                    &meta.path,
                    "Unsupported raw_data attribute",
                ))
            })
            .unwrap(); // 处理可能的错误
        } else if attr.path().is_ident("repr") {
            attr.parse_nested_meta(|meta| {
                // #[repr(C)]
                if meta.path.is_ident("C") {
                    target_layout = StructLayout::C;
                    return Ok(());
                } else if meta.path.is_ident("packed") {
                    target_layout = StructLayout::Packed;
                    return Ok(());
                }
                Err(syn::Error::new_spanned(
                    &meta.path,
                    "Unsupported raw_data attribute",
                ))
            })
            .unwrap();
        }
    }
    (data_layout, target_layout)
}

// //放结构体生成序列化方案
// fn generate_impl(struct_name: &Ident, data_align: DataAlignment, fields: &Fields) -> TokenStream {
//     let strategy = match data_align {
//         DataAlignment::C => quote! { AlignedStrategy },
//         DataAlignment::Packed => quote! { PackedStrategy }
//     };

//     quote! {
//         impl #struct_name {
//             //原始数据类型
//             pub const DATA_ALIGNMENT: DataAlignment = DataAlignment::#data_align;
//             //
//             pub fn deserialize(bytes: &[u8]) -> Result<Self> {
//                 #strategy::deserialize(bytes)
//             }
//         }
//     }
// }

// fn generate_deserialize_impl(struct_name: &Ident, fields: &Fields) -> TokenStream {
//     let field_parsers = fields.iter().enumerate().map(|(i, f)| {
//         let name = f.ident.as_ref().unwrap_or(&format_ident!("_{}", i));
//         let ty = &f.ty;
//         quote! {
//             #name: <#ty as RawDeserialize>::deserialize(
//                 bytes.get(Self::#name_OFFSET..)
//                     .ok_or(Error::InsufficientData)?,
//                 is_data_aligned && (Self::#name_OFFSET % std::mem::align_of::<#ty>()) == 0
//             )?
//         }
//     });

//     quote! {
//         unsafe impl RawDeserialize for #struct_name {
//             fn deserialize(bytes: &[u8], is_data_aligned: bool) -> Result<Self> {
//                 Ok(Self {
//                     #(#field_parsers),*
//                 })
//             }
//         }
//     }
// }

// let (data_layout, target_layout) = parse_layout_anno(&input.attrs);
// // 选择合适的策略
// let strategy = match (data_layout, target_layout) {
//     (StructLayout::C, StructLayout::C) => quote! { strategy::AlignedToAligned },
//     (StructLayout::C, StructLayout::Packed) => quote! { strategy::AlignedToPacked },
//     (StructLayout::Packed, StructLayout::C) => quote! { strategy::UnalignedToAligned },
//     (StructLayout::Packed, StructLayout::Packed) => quote! { strategy::UnalignedToPacked },
// };
