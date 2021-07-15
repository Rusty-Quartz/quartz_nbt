use syn::{
    braced,
    bracketed,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{Brace, Bracket},
    Error,
    Expr,
    Ident,
    LitStr,
    Token,
};

pub type Compound = Punctuated<KeyValuePair, Token![,]>;

pub enum Value {
    Compound(Compound),
    ByteArray(Punctuated<Expr, Token![,]>),
    IntArray(Punctuated<Expr, Token![,]>),
    LongArray(Punctuated<Expr, Token![,]>),
    List(Punctuated<Self, Token![,]>),
    Expr(Expr),
}

impl Parse for Value {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Brace) {
            let content;
            braced!(content in input);
            Ok(Value::Compound(Compound::parse_terminated(&content)?))
        } else if input.peek(Bracket) {
            let content;
            bracketed!(content in input);

            if content.peek(Ident) && content.peek2(Token![;]) {
                let type_specifier_ident: Ident = content.parse()?;
                let type_specifier = type_specifier_ident.to_string();
                let _semicolon: Token![;] = content.parse()?;

                match type_specifier.as_str() {
                    "B" | "b" => Ok(Value::ByteArray(Punctuated::parse_terminated(&content)?)),
                    "I" | "i" => Ok(Value::IntArray(Punctuated::parse_terminated(&content)?)),
                    "L" | "l" => Ok(Value::LongArray(Punctuated::parse_terminated(&content)?)),
                    _ => Err(Error::new_spanned(
                        type_specifier_ident,
                        "Invalid type specifier, expected B, I, or L",
                    )),
                }
            } else {
                Ok(Value::List(Punctuated::parse_terminated(&content)?))
            }
        } else {
            Ok(Value::Expr(input.parse()?))
        }
    }
}

pub struct KeyValuePair {
    pub key: LitStr,
    pub colon: Token![:],
    pub value: Value,
}

impl Parse for KeyValuePair {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let key = input.parse()?;
        let colon = input.parse()?;
        let value = input.parse()?;

        Ok(KeyValuePair { key, colon, value })
    }
}
