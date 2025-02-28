use proc_macro::TokenStream;
use quote::quote;
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
