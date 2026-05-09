#![no_std]

pub use rvoid_core::SystemInfo;
pub use rvoid_entry::entry;

pub mod prelude {
    pub use crate::SystemInfo;
}

pub mod startup {
    #[cfg(feature = "uefi")]
    pub mod uefi {
        pub use rvoid_uefi_backend::{EfiHandle, EfiStatus, EfiSystemTable, startup};
    }
}
