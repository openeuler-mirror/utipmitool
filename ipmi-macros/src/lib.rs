/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
extern crate proc_macro;
//pub mod structmeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Type};

//pub mod structmeta;

//use crate::structmeta::StructField;
//crate::structmeta::StructField;
//use self::procmacro::impl_struct_info;
//mod procmacro;

#[proc_macro_derive(DataAccess)]
pub fn derive_data_access(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // 新增校验逻辑,还需要增加字段类型检查
    let fields = if let Data::Struct(data) = &input.data {
        &data.fields
    } else {
        return syn::Error::new_spanned(name, "DataAccess only support struct")
            .to_compile_error()
            .into();
    };

    let has_data = fields
        .iter()
        .any(|f| f.ident.as_ref().map_or(false, |i| i == "data"));
    let has_data_len = fields
        .iter()
        .any(|f| f.ident.as_ref().map_or(false, |i| i == "data_len"));

    if !has_data || !has_data_len {
        let msg = format!(
            "struct must contains 'data' and 'data_len' filed (missing {})",
            match (has_data, has_data_len) {
                (false, false) => "data, data_len",
                (false, _) => "data",
                (_, false) => "data_len",
                (true, true) => "",
            }
        );
        return syn::Error::new_spanned(name, msg).to_compile_error().into();
    }

    let expanded = quote! {
        impl #name {

            pub fn data(&self) -> Option<&[u8]> {
                if self.data.is_null() || self.data_len == 0 {
                    None
                } else {
                    unsafe { Some(std::slice::from_raw_parts(self.data, self.data_len as usize)) }
                }

            }

            pub fn data_mut(&mut self) -> Option<&mut [u8]> {
                if self.data.is_null() || self.data_len == 0 {
                    None
                } else {
                    unsafe { Some(std::slice::from_raw_parts_mut(self.data, self.data_len as usize)) }
                }
            }

            /// 带生命周期标注的版本（更安全）
            pub fn safe_data<'a>(&'a mut self) -> Option<&'a mut [u8]> {
                self.data_mut()
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(MemberOffsets)]
pub fn derive_member_offsets(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let fields = if let Data::Struct(data) = &input.data {
        &data.fields
    } else {
        return syn::Error::new_spanned(name, "MemberOffsets only support struct")
            .to_compile_error()
            .into();
    };

    let field_info: Vec<_> = fields
        .iter()
        .filter_map(|f| {
            let ident = f.ident.as_ref()?;
            let ty = &f.ty;
            Some(quote! {
                (
                    stringify!(#ident),
                    std::any::type_name::<#ty>(),
                    std::mem::offset_of!(#name, #ident),
                    //std::mem::size_of_val(&self.#ident),
                    std::mem::size_of::<#ty>()
                )
            })
        })
        .collect();

    let expanded = quote! {
        impl #name {
            pub fn member_offsets() -> Vec<(&'static str, &'static str, usize, usize)> {
                vec![
                    #(#field_info),*
                ]
            }
            //只需要返回成员的大小
            pub fn member_size() -> Vec<usize> {
                vec![
                    #(#field_info.3),*
                ]
            }
            pub fn print_offsets() {
                println!("Struct: {}, Size: {}", stringify!(#name),std::mem::size_of::<#name>());
                for (name, type_name, offset,size) in Self::member_offsets() {
                    println!(
                        "Offset: 0x{:04x} Size: {:2} Field: {:12} Type: {:20}",
                        offset, size, name, type_name,
                    );
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(AsBytes)]
pub fn derive_as_bytes(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let fields = match &input.data {
        Data::Struct(data) => &data.fields,
        _ => panic!("AsBytes only supports structs"),
    };

    let field_parsers = fields.iter().map(|f| {
        let ident = f.ident.as_ref().expect("结构体必须使用命名字段"); // 确保字段有名称
        let ty = &f.ty;

        // 判断是否为基本类型
        let is_primitive = match ty {
            Type::Path(path) => path.path.segments.last().map_or(false, |seg| {
                let ident = seg.ident.to_string();
                matches!(
                    ident.as_str(),
                    "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64"
                )
            }),
            _ => false,
        };
        if is_primitive {
            quote! {
                #ident: {
                    let bytes = bytes.get(offset..offset + std::mem::size_of::<#ty>())
                        .ok_or("字段字节不足")?;
                    offset += std::mem::size_of::<#ty>();
                    #ty::from_le_bytes(bytes.try_into().unwrap()) // 基本类型无需错误传播
                },
            }
        }
        // 数组处理逻辑
        else if let Type::Array(arr) = &f.ty {
            let elem_ty = &arr.elem;
            let len = &arr.len;
            quote! {
                #ident: {
                    let elem_size = std::mem::size_of::<#elem_ty>();
                    let total_bytes = elem_size * #len;
                    let bytes_part = bytes.get(offset..offset + total_bytes)
                        .ok_or("数组字节不足")?;
                    offset += total_bytes;

                    let mut arr = [<#elem_ty>::default(); #len];
                    for i in 0..#len {
                        let elem_bytes = &bytes_part[i*elem_size..(i+1)*elem_size];
                        //arr[i] = <#elem_ty>::from_le_bytes(elem_bytes)?;
                        arr[i] = <#elem_ty>::from_le_bytes(elem_bytes.try_into().unwrap());
                    }
                    arr
                },
            }
        }
        // 结构体字段处理
        else {
            quote! {
                #ident: {
                    let field_size = std::mem::size_of::<#ty>();
                    let bytes_part = bytes.get(offset..offset + field_size)
                        .ok_or("字段字节不足")?;
                    //eprintln!("@{}@\noffset: {} field_size: {}, bytes_part: {:02x?}",stringify!(#ident), offset, field_size, bytes_part);
                    offset += field_size;
                    <#ty>::from_le_bytes(bytes_part)?
                    //<#ty>::from_le_bytes(bytes_part.try_into().map_err(|_| "字节转换失败")?)
                },
            }
        }
    });

    let expanded = quote! {
        impl #name {
            pub fn from_le_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
                use std::mem::size_of;
                let mut offset = 0;

                if bytes.len() < size_of::<Self>() {
                    return Err("Input bytes too short");
                }

                Ok(Self {
                    #(#field_parsers)*
                })
            }

            // 不可变切片方法
            pub fn as_bytes(&self) -> &[u8] {
                unsafe {
                    std::slice::from_raw_parts(
                        self as *const _ as *const u8,
                        std::mem::size_of_val(self)
                    )
                }
            }

            // 可变切片方法
            pub fn as_bytes_mut(&mut self) -> &mut [u8] {
                unsafe {
                    std::slice::from_raw_parts_mut(
                        self as *mut _ as *mut u8,
                        std::mem::size_of_val(self)
                    )
                }
            }
            /// 获取结构体的原始指针
            pub fn as_ptr(&self) -> *const #name {
                self as *const _
            }

            /// 获取结构体的可变指针
            pub fn as_mut_ptr(&mut self) -> *mut #name {
                self as *mut _
            }

        }
    };
    TokenStream::from(expanded)
}

// pub fn calculate_raw_size(ty: &Type) -> TokenStream {
//     match ty {
//         Type::Path(path) => {
//             let type_name = path.path.segments.last().unwrap().ident.to_string();
//             match type_name.as_str() {
//                 "u8" | "i8" | "bool" => quote! {1},
//                 "u16" | "i16" => quote! {2},
//                 "u32" | "i32" | "f32" => quote! {4},
//                 "u64" | "i64" | "f64" => quote! {8},
//                 _ => quote! { calculate_raw_size(ty) } // 使用trait方法处理嵌套结构
//             }
//         },
//         Type::Array(arr) => {
//             let elem_ty = &arr.elem;
//             let len = &arr.len;
//             let elem_size = calculate_raw_size(elem_ty);
//             quote! {
//                 (#elem_size) * (#len as usize)
//             }
//         },
//         _ => panic!("Unsupported type: {}", quote! {#ty})
//     }
// }

//派生宏实现修改
#[proc_macro_derive(StructMeta)]
pub fn struct_meta_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => &fields.named,
        _ => panic!("Only structs with named fields are supported"),
    };

    let field_info = fields.iter().map(|f| {
        let ident = f.ident.as_ref().unwrap();
        let ty = &f.ty;

        // 计算原始数据大小
        //let raw_size = structmeta::calculate_raw_size(ty);

        let _is_primitive = matches!(&ty,
            Type::Path(p) if {
                let type_name = p.path.segments.last().unwrap().ident.to_string();
                type_name.matches("u8|i8|u16|i16|u32|i32|u64|i64|f32|f64|bool").count() > 0
            }
        );
        // 递归处理子结构
        // let children = if is_primitive {
        //     quote! { Vec::new() }
        // } else {
        //     quote! { <#ty>::struct_info() }
        // };

        quote! {
            StructField {
                name: stringify!(#ident),
                type_name: std::any::type_name::<#ty>(),
                offset: std::mem::offset_of!(#name, #ident),
                size: std::mem::size_of::<#ty>(),
                //raw_size: #raw_size,
                //children: #children
            }
        }
    });

    let expanded = quote! {
        impl StructMeta for #name {
        //impl  #name {
            fn struct_info() -> Vec<structmeta::StructField> {
                vec![#(#field_info),*]
            }

            // fn raw_size() -> usize {
            //     0 #(+ <#fields as StructMeta>::raw_size())*
            // }

            // fn is_composite() -> bool {
            //    false
            // }

        }
    };
    TokenStream::from(expanded)
    //expanded.into()
    //let expanded1 = quote! {};
    //expanded1.into()
}
