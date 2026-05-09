#![no_std]

#[cfg(feature = "uefi")]
pub use rvoid_uefi::entry;

#[cfg(not(any(feature = "uefi")))]
pub use rvoid_stub_entry::entry;

#[cfg(feature = "uefi")]
pub mod uefi {
    pub use rvoid_uefi::startup::{EfiHandle, EfiStatus, EfiSystemTable, startup};
}
