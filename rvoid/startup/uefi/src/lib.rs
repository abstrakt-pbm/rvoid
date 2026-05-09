#![no_std]
#![no_std]

mod startup;
// later:
// mod memory;
// mod framebuffer;
// mod acpi;

pub type EfiHandle = uefi::Handle;
pub type EfiStatus = uefi::Status;
pub type EfiSystemTable = uefi::table::SystemTable<uefi::table::Boot>;

pub use startup::startup;
pub mod startup
