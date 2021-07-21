use std::collections::HashSet;

use crate::parse::{Compound, KeyValuePair, Value};
use proc_macro2::{Literal, TokenStream};
use quote::{quote, ToTokens};
use syn::Error;

pub fn gen_compound_expr(compound: &Compound) -> TokenStream {
    let mut used_keys: HashSet<String> = HashSet::new();
    let inserts = compound.iter().map(|KeyValuePair { key, value, .. }| {
        let key_string = key.value();

        if used_keys.contains(&key_string) {
            let error = Error::new_spanned(key, "Duplicate key").to_compile_error();
            quote! { #error }
        } else {
            used_keys.insert(key_string);

            quote! {
                __compound.insert(#key, #value);
            }
        }
    });

    let capacity = Literal::usize_unsuffixed(compound.len());

    quote! {
        {
            let mut __compound = ::quartz_nbt::NbtCompound::with_capacity(#capacity);
            #( #inserts )*
            __compound
        }
    }
}

impl ToTokens for Value {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Value::Compound(compound) => tokens.extend(gen_compound_expr(compound)),
            Value::ByteArray(array) => {
                let iter = array.iter();
                tokens.extend(quote! {
                    { ::quartz_nbt::NbtTag::ByteArray(::std::vec![#( (#iter) as i8 ),*]) }
                })
            }
            Value::IntArray(array) => {
                let iter = array.iter();
                tokens.extend(quote! {
                    { ::quartz_nbt::NbtTag::IntArray(::std::vec![#( (#iter) as i32 ),*]) }
                })
            }
            Value::LongArray(array) => {
                let iter = array.iter();
                tokens.extend(quote! {
                    { ::quartz_nbt::NbtTag::LongArray(::std::vec![#( (#iter) as i64 ),*]) }
                })
            }
            Value::List(list) =>
                if list.is_empty() {
                    tokens.extend(quote! { ::quartz_nbt::NbtList::new() })
                } else {
                    tokens.extend(quote! {
                        { ::quartz_nbt::NbtList::from(::std::vec![#list]) }
                    })
                },
            Value::Expr(expr) => expr.to_tokens(tokens),
        }
    }
}
