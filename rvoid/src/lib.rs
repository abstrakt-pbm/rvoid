#![no_std]

pub use rvoid_core::SystemInfo;

pub mod prelude {
    pub use crate::SystemInfo;
}

#[cfg(feature = "uefi")]
pub use rvoid_uefi_entry::entry;

#[cfg(not(any(feature = "uefi")))]
pub use rvoid_stub_entry::entry;

pub mod startup {
    #[cfg(feature = "uefi")]
    pub mod uefi {
        pub use rvoid_uefi_backend::{EfiHandle, EfiStatus, EfiSystemTable, startup};
    }
}
