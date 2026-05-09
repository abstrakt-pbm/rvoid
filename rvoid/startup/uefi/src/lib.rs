#![no_std]

pub use rvoid_uefi_entry::entry;

pub mod startup {
    pub use rvoid_uefi_backend::{EfiHandle, EfiStatus, EfiSystemTable, startup};
}
