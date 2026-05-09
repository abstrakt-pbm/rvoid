use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn entry(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let user_fn = parse_macro_input!(item as ItemFn);
    let user_fn_name = &user_fn.sig.ident;

    let expanded = quote! {
        #user_fn

        #[unsafe(no_mangle)]
        pub extern "efiapi" fn efi_main(
            image_handle: ::rvoid::startup::uefi::EfiHandle,
            system_table: *mut ::rvoid::startup::uefi::EfiSystemTable,
        ) -> ::rvoid::startup::uefi::EfiStatus {
            let system = unsafe {
                ::rvoid::startup::uefi::startup(
                    image_handle,
                    system_table,
                )
            };

            #user_fn_name(system)
        }
    };

    expanded.into()
}
