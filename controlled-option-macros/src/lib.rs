// -*- coding: utf-8 -*-
// ------------------------------------------------------------------------------------------------
// Copyright © 2021, Douglas Creager.
// Licensed under either of Apache License, Version 2.0, or MIT license, at your option.
// Please see the LICENSE-APACHE or LICENSE-MIT files in this distribution for license details.
// ------------------------------------------------------------------------------------------------

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::Field;
use syn::Fields;
use syn::Item;
use syn::Member;

fn field_is_niche(field: &&Field) -> bool {
    for attr in &field.attrs {
        if attr.path.is_ident("niche") {
            return true;
        }
    }
    false
}

#[proc_macro_derive(Niche, attributes(niche))]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Item);
    match &item {
        Item::Struct(item) => {
            let ty_name = &item.ident;

            // Find the field that is marked #[niche].  In a regular struct, extract its name; in a
            // tuple struct, extract its index.  In both cases, that can be converted into a
            // `Member`, which is the type needed down below in the field access expression.
            let niche_field_name: Option<Member> = match &item.fields {
                Fields::Named(fields) => fields
                    .named
                    .iter()
                    .find(field_is_niche)
                    .and_then(|field| field.ident.as_ref())
                    .cloned()
                    .map(Member::from),
                Fields::Unnamed(fields) => fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .find(|(_, field)| field_is_niche(field))
                    .map(|(idx, _)| idx.into()),
                Fields::Unit => {
                    let msg = "#[derive(Niche)] cannot be used on an empty tuple struct";
                    return syn::parse::Error::new_spanned(item, msg)
                        .to_compile_error()
                        .into();
                }
            };
            let niche_field_name = match niche_field_name {
                Some(field) => field,
                None => {
                    let msg = "#[derive(Niche)] requires a field marked #[niche]";
                    return syn::parse::Error::new_spanned(item, msg)
                        .to_compile_error()
                        .into();
                }
            };

            let output = quote! {
                impl ::controlled_option::Niche for #ty_name {
                    type Output = ::std::mem::MaybeUninit<Self>;

                    #[inline]
                    fn none() -> Self::Output {
                        let mut value = Self::Output::uninit();
                        let ptr = value.as_mut_ptr();
                        ::controlled_option::fill_struct_field_with_none(
                            unsafe { ::std::ptr::addr_of_mut!((*ptr).#niche_field_name) }
                        );
                        value
                    }

                    #[inline]
                    fn is_none(value: &Self::Output) -> bool {
                        let ptr = value.as_ptr();
                        ::controlled_option::struct_field_is_none(
                            unsafe { ::std::ptr::addr_of!((*ptr).#niche_field_name) }
                        )
                    }

                    #[inline]
                    fn into_some(value: Self) -> Self::Output {
                        ::std::mem::MaybeUninit::new(value)
                    }

                    #[inline]
                    fn from_some(value: Self::Output) -> Self {
                        unsafe { value.assume_init() }
                    }
                }
            };
            output.into()
        }
        _ => {
            let msg = "#[derive(Niche)] is only supported on struct types";
            syn::parse::Error::new_spanned(item, msg)
                .to_compile_error()
                .into()
        }
    }
}
