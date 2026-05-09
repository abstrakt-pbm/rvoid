use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn entry(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let user_fn = parse_macro_input!(item as ItemFn);

    let expanded = quote! {
        #user_fn

        compile_error!(
            "rvoid: no startup backend selected. Enable one startup backend feature, for example `uefi`."
        );
    };

    expanded.into()
}
