extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, LitStr, parse_macro_input};

#[proc_macro]
pub fn define_id_type(input: TokenStream) -> TokenStream {
    let x = parse_macro_input!(input as Ident);

    let x_str = x.to_string();

    let id_struct = Ident::new(&format!("{}Id", x_str), x.span());

    let error_enum = Ident::new(&format!("{}IdError", x_str), x.span());

    let x_lower = x_str.to_lowercase();

    let error_msg = LitStr::new(
        &format!("Invalid negative value for {} id", x_lower),
        x.span(),
    );

    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct #id_struct(u64);

        #[derive(Debug, thiserror::Error)]
        pub enum #error_enum {
            #[error(#error_msg)]
            NegativeValue,
        }

        impl std::fmt::Display for #id_struct {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl TryFrom<i32> for #id_struct {
            type Error = #error_enum;

            fn try_from(value: i32) -> Result<Self, Self::Error> {
                if value < 0 {
                    Err(#error_enum::NegativeValue)
                } else {
                    Ok(#id_struct(value as u64))
                }
            }
        }

        impl TryFrom<i64> for #id_struct {
            type Error = #error_enum;

            fn try_from(value: i64) -> Result<Self, Self::Error> {
                if value < 0 {
                    Err(#error_enum::NegativeValue)
                } else {
                    Ok(#id_struct(value as u64))
                }
            }
        }

        impl From<u64> for #id_struct {
            fn from(value: u64) -> Self {
                #id_struct(value)
            }
        }

        impl From<#id_struct> for u64 {
            fn from(id: #id_struct) -> Self {
                id.0
            }
        }

        // 假设用户已定义Identifier trait
        impl Identifier for #id_struct {}
    }
    .into()
}
