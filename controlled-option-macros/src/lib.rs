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
use syn::parse_quote;
use syn::Field;
use syn::Fields;
use syn::Item;
use syn::Member;
use syn::Type;
use syn::WhereClause;

fn field_is_niche(field: &&Field) -> bool {
    for attr in &field.attrs {
        if attr.path.is_ident("niche") {
            return true;
        }
    }
    false
}

fn merge_where_clauses(lhs: Option<WhereClause>, rhs: WhereClause) -> WhereClause {
    match lhs {
        Some(mut lhs) => {
            lhs.predicates.extend(rhs.predicates);
            lhs
        }
        None => rhs,
    }
}

#[proc_macro_derive(Niche, attributes(niche))]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Item);
    match &item {
        Item::Struct(item) => {
            let ty_name = &item.ident;
            let ty_generics = &item.generics;
            let ty_where_clause = item.generics.where_clause.as_ref().cloned();

            // Find the field that is marked #[niche].  In a regular struct, extract its name; in a
            // tuple struct, extract its index.  In both cases, that can be converted into a
            // `Member`, which is the type needed down below in the field access expression.
            let niche_field_name: Member;
            let niche_field_type: &Type;
            match &item.fields {
                Fields::Named(fields) => {
                    let niche_field = match fields.named.iter().find(field_is_niche) {
                        Some(field) if field.ident.is_some() => field,
                        _ => {
                            let msg = "#[derive(Niche)] requires a field marked #[niche]";
                            return syn::parse::Error::new_spanned(item, msg)
                                .to_compile_error()
                                .into();
                        }
                    };
                    niche_field_name = niche_field.ident.as_ref().unwrap().clone().into();
                    niche_field_type = &niche_field.ty;
                }
                Fields::Unnamed(fields) => {
                    let (idx, niche_field) = match fields
                        .unnamed
                        .iter()
                        .enumerate()
                        .find(|(_, field)| field_is_niche(field))
                    {
                        Some((idx, field)) => (idx, field),
                        None => {
                            let msg = "#[derive(Niche)] requires a field marked #[niche]";
                            return syn::parse::Error::new_spanned(item, msg)
                                .to_compile_error()
                                .into();
                        }
                    };
                    niche_field_name = idx.into();
                    niche_field_type = &niche_field.ty;
                }
                Fields::Unit => {
                    let msg = "#[derive(Niche)] cannot be used on an empty tuple struct";
                    return syn::parse::Error::new_spanned(item, msg)
                        .to_compile_error()
                        .into();
                }
            }

            let where_clause = merge_where_clauses(
                ty_where_clause,
                parse_quote! { where #niche_field_type: ::controlled_option::Niche },
            );

            let output = quote! {
                impl #ty_generics ::controlled_option::Niche for #ty_name #ty_generics
                #where_clause
                {
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
