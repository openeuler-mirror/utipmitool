/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
extern crate proc_macro;
//use proc_macro::TokenStream;

mod annotation;
//mod upack;

use annotation::parse_endian_anno;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

//结构体反序列化方案
// ... 前面的代码保持不变 ...

#[proc_macro_derive(RAWDATA, attributes(raw_data))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let repr_attr = input.attrs.iter().find(|a| a.path().is_ident("repr"));
    if repr_attr.is_none() {
        panic!("RAWDATA can only be derived for #[repr(C)] structs");
    }

    // 统一处理字段获取和大小计算
    // 修改第17行开始的匹配表达式
    let (_fields, sum) = match &input.data {
        // 改为引用匹配
        Data::Struct(data) => match &data.fields {
            // 改为引用匹配
            Fields::Named(fields) => {
                let field_sizes = fields.named.iter().map(|f| {
                    let ty = &f.ty;
                    quote! { <#ty as ::unpack::RawSize>::RAW_SIZE }
                });
                let sum = quote! { 0 #(+ #field_sizes)* };
                (fields.named.clone(), sum)
            }
            Fields::Unnamed(fields) => {
                let field_sizes = fields.unnamed.iter().map(|f| {
                    let ty = &f.ty;
                    quote! { <#ty as ::unpack::RawSize>::RAW_SIZE }
                });
                let sum = quote! { 0 #(+ #field_sizes)* };
                (fields.unnamed.clone(), sum) // 添加clone
            }
            Fields::Unit => {
                return TokenStream::from(quote! {
                    impl ::unpack::RawSize for #name {

                        const RAW_SIZE: usize = 0;
                        const ENDIAN: ::unpack::Endianness = ::unpack::Endianness::Native;

                        fn from_bytes_with_endian(
                            _bytes: &[u8],
                            _endian: ::unpack::Endianness
                        ) -> Result<Self, Box<dyn std::error::Error>> {
                            Ok(Self)
                        }
                    }
                });
            }
        },
        Data::Enum(_) => panic!("RAWDATA cannot be derived for enums"),
        Data::Union(_) => panic!("RAWDATA cannot be derived for unions"),
    };
    // 处理字节序注解
    let endian = match parse_endian_anno(&input.attrs) {
        Some(s) if s == "big" => quote! { ::unpack::Endianness::Big },
        Some(s) if s == "little" => quote! { ::unpack::Endianness::Little },
        _ => quote! { ::unpack::Endianness::Native },
    };

    let (_field_inits, struct_expr) = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields_named) => {
                let field_inits = fields_named.named.iter().map(|field| {
                    // 修改这里：忽略字段的可见性属性，只处理字段名和类型
                    let ident = field.ident.as_ref().unwrap();
                    let ty = &field.ty;
                    quote! {
                        #ident: {
                            let value = <#ty as ::unpack::RawSize>::from_bytes_with_endian(
                                &bytes[offset..offset + <#ty as ::unpack::RawSize>::RAW_SIZE],
                                endian
                            )?;
                            offset += <#ty as ::unpack::RawSize>::RAW_SIZE;
                            value
                        }
                    }
                });
                (quote! {}, quote! { Self { #(#field_inits),* } })
            }
            Fields::Unnamed(fields_unnamed) => {
                // 先收集到Vec中
                let field_inits: Vec<_> = fields_unnamed
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(_i, field)| {
                        let ty = &field.ty;
                        quote! {
                            {
                                let value = <#ty as RawSize>::from_bytes_with_endian(
                                    &bytes[offset..offset + <#ty as RawSize>::RAW_SIZE],
                                    endian
                                )?;
                                offset += <#ty as RawSize>::RAW_SIZE;
                                value
                            }
                        }
                    })
                    .collect(); // 关键修改：添加.collect()

                // 使用已收集的vec进行展开
                (
                    quote! { #(#field_inits),* },
                    quote! { Self ( #(#field_inits),* ) },
                )
            }
            Fields::Unit => (quote! {}, quote! { Self }),
        },
        _ => unreachable!(),
    };

    // 生成最终实现
    let expanded = quote! {
        impl ::unpack::RawSize for #name {

            const RAW_SIZE: usize = #sum;
            const ENDIAN: ::unpack::Endianness = #endian;

            fn from_bytes_with_endian(
                bytes: &[u8],
                endian: ::unpack::Endianness
            ) -> Result<Self, Box<dyn std::error::Error>> {
                if bytes.len() < Self::RAW_SIZE {
                    return Err("Insufficient data for deserialization".into());
                }

                let mut offset = 0;
                Ok(#struct_expr)
            }
        }
    };

    TokenStream::from(expanded)
}
