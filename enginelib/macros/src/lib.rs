use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};
#[proc_macro_attribute]
pub fn metadata(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = proc_macro2::TokenStream::from(item);
    quote! {
        #[unsafe(export_name="metadata")]
        #item
    }
    .into()
}
#[proc_macro_attribute]
pub fn module(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = proc_macro2::TokenStream::from(item);
    quote! {
        #[unsafe(export_name="run")]
        #item
    }
    .into()
}
#[proc_macro_derive(Verifiable)]
pub fn derive_verifiable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl Verifiable for #name {
            fn verify(&self, b: Vec<u8>) -> bool {
                let k: Result<#name, _> = bincode::deserialize(b.as_slice());
                k.is_ok()
            }
        }
    };

    TokenStream::from(expanded)
}
