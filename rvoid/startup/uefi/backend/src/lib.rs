#![no_std]

pub type EfiHandle = uefi::Handle;
pub type EfiStatus = uefi::Status;
pub type EfiSystemTable = core::ffi::c_void;

pub unsafe fn startup(
    image_handle: EfiHandle,
    system_table: *mut EfiSystemTable,
) -> rvoid_core::SystemInfo {
    let _ = image_handle;
    let _ = system_table;

    rvoid_core::SystemInfo::empty()
}
