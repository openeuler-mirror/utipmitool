/*
 * SPDX-FileCopyrightText: 2025 UnionTech Software Technology Co., Ltd.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */
// 新增数据结构

use proc_macro::TokenStream;  // 修改为proc_macro
use proc_macro2::TokenStream as TokenStream2;  // 添加别名
use quote::quote;
use syn::{parse_macro_input,DataStruct, Fields, DeriveInput, Data,Type,FieldsNamed, Ident};

pub trait RawSize {
    fn raw_size(&self) -> usize {
        Self::static_raw_size()
    }

    fn static_raw_size() -> usize;
}

impl RawSize for u8 {
    fn static_raw_size() -> usize { std::mem::size_of::<u8>() }
}

impl RawSize for i32 {
    fn static_raw_size() -> usize { std::mem::size_of::<i32>() }
}


pub struct StructField {
    pub name: &'static str,//结构体字段名
    pub type_name: &'static str,//字段类型名称
    pub offset: usize,//字段在结构体中的偏移量
    pub size: usize,  // 原始大小（含对齐填充）
    //pub raw_size: usize,      // 实际数据占用（不含对齐填充）
    //pub children: Vec<StructField>,//如果是字段结构体类型，存储子结构的信息
}

pub trait StructMeta {
    fn struct_info() -> Vec<StructField>;
}

/*
pub trait StructMeta {
    fn raw_size() -> usize {
        0
    }
}
/// 适用于纯数据结构的场景
fn raw_size() -> usize {
    #( std::mem::size_of::<#fields>() )+*
}

impl StructMeta for u8 { fn raw_size() -> usize { 1 } }
impl StructMeta for u16 { fn raw_size() -> usize { 2 } }
impl StructMeta for u32 { fn raw_size() -> usize { 4 } }

let expanded = quote! {
    impl StructMeta for #name {
        fn raw_size() -> usize {
            #( <#fields as StructMeta>::raw_size() )+*
        }
    }
};

*/
// 核心递归计算逻辑
pub fn calculate_raw_size(ty: &Type) -> TokenStream2 {
    if let Type::Path(path) = ty {
        let type_name = path.path.segments.last().unwrap().ident.to_string();
        match type_name.as_str() {
            "u8" | "i8" | "bool" => quote! {1},
            "u16" | "i16" => quote! {2},
            "u32" | "i32" | "f32" => quote! {4},
            "u64" | "i64" | "f64" => quote! {8},
            //_ => quote! { <#ty as StructMeta>::raw_size() }//非基础类型
            _=> quote! {
                calculate_raw_size(#ty);
            }
        }
    } else if let Type::Array(arr) = ty {
        let elem_ty = &arr.elem;
        let len = &arr.len;
        let elem_size = calculate_raw_size(elem_ty);
        quote! {
            (#elem_size) * (#len as usize) // 正确生成嵌套表达式
        }
        //quote! { (#calculate_raw_size(#elem_ty)) * (#len as usize) }
        // 关键修复点：添加引用符号 & 并正确调用函数
        //quote! { ( #(calculate_raw_size(&#elem_ty)) ) * ( #len as usize ) }
    } else {
        panic!("Unsupported type: {}", quote! {#ty});
    }
}

