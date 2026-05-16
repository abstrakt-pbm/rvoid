#![no_std]

pub use uefi;

pub fn startup() -> rvoid_core::SystemInfo {
    uefi::println!("RVOID: UEFI startup | new");

    rvoid_core::SystemInfo::empty()
}
