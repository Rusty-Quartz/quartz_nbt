#![warn(missing_docs)]

//! This crate contains the function-like procedural macro which parses `quartz_nbt`'s compact
//! compound format.

use syn::{parse::Parse, parse_macro_input};

extern crate proc_macro;

mod gen;
mod parse;

#[allow(missing_docs)]
#[proc_macro]
pub fn compound(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    struct Wrapper(parse::Compound);

    impl Parse for Wrapper {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            parse::Compound::parse_terminated(input).map(|punctuated| Wrapper(punctuated))
        }
    }

    let input = parse_macro_input!(item as Wrapper).0;
    gen::gen_compound_expr(&input).into()
}
