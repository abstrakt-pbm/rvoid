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
            image_handle: ::rvoid::startup::uefi::uefi::Handle,
            system_table: *mut ::core::ffi::c_void,
        ) -> ::rvoid::startup::uefi::uefi::Status {
            unsafe {
                ::rvoid::startup::uefi::uefi::boot::set_image_handle(image_handle);

                let system_table = system_table as *const ::core::ffi::c_void;

                ::rvoid::startup::uefi::uefi::table::set_system_table(
                    system_table.cast(),
                );
            }

            ::rvoid::startup::uefi::uefi::helpers::init().unwrap();

            let system = ::rvoid::startup::uefi::startup();

            #user_fn_name(system);

            ::rvoid::startup::uefi::uefi::Status::SUCCESS
        }
    };

    expanded.into()
}
