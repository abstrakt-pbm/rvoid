use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

#[proc_macro_attribute]
pub fn entry(attr: TokenStream, item: TokenStream) -> TokenStream {
    let _ = attr;

    let user_fn = parse_macro_input!(item as ItemFn);

    #[cfg(feature = "uefi")]
    {
        return expand_uefi_entry(user_fn).into();
    }

    #[cfg(not(feature = "uefi"))]
    {
        return expand_missing_backend(user_fn).into();
    }
}

// transfer to rvoid-uefi crate in future
#[cfg(feature = "uefi")]
fn expand_uefi_entry(user_fn: ItemFn) -> proc_macro2::TokenStream {
    let user_fn_name = &user_fn.sig.ident;

    quote! {
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
    }
}

#[cfg(not(feature = "uefi"))]
fn expand_missing_backend(user_fn: ItemFn) -> proc_macro2::TokenStream {
    quote! {
        #user_fn

        compile_error!(
            "rvoid: no startup backend selected. Enable a backend feature, for example `uefi`."
        );
    }
}
