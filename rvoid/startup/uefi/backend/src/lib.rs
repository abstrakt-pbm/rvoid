#![no_std]

use core::ffi::c_void;

pub type EfiHandle = uefi::Handle;
pub type EfiStatus = uefi::Status;
pub type EfiSystemTable = RawEfiSystemTable;

#[repr(C)]
pub struct EfiTableHeader {
    pub signature: u64,
    pub revision: u32,
    pub header_size: u32,
    pub crc32: u32,
    pub reserved: u32,
}

#[repr(C)]
pub struct RawEfiSystemTable {
    pub hdr: EfiTableHeader,

    pub firmware_vendor: *mut u16,
    pub firmware_revision: u32,

    pub console_in_handle: EfiHandle,
    pub con_in: *mut c_void,

    pub console_out_handle: EfiHandle,
    pub con_out: *mut EfiSimpleTextOutputProtocol,

    pub standard_error_handle: EfiHandle,
    pub std_err: *mut EfiSimpleTextOutputProtocol,

    pub runtime_services: *mut c_void,
    pub boot_services: *mut c_void,

    pub number_of_table_entries: usize,
    pub configuration_table: *mut c_void,
}

#[repr(C)]
pub struct EfiSimpleTextOutputProtocol {
    pub reset: usize,

    pub output_string:
        extern "efiapi" fn(this: *mut EfiSimpleTextOutputProtocol, string: *const u16) -> EfiStatus,

    pub test_string: usize,
    pub query_mode: usize,
    pub set_mode: usize,
    pub set_attribute: usize,
    pub clear_screen: usize,
    pub set_cursor_position: usize,
    pub enable_cursor: usize,
    pub mode: *mut c_void,
}

unsafe fn uefi_puts(system_table: *mut EfiSystemTable, string: *const u16) {
    if system_table.is_null() {
        return;
    }

    let con_out = unsafe { (*system_table).con_out };

    if con_out.is_null() {
        return;
    }

    unsafe {
        let _ = ((*con_out).output_string)(con_out, string);
    }
}

pub unsafe fn startup(
    image_handle: EfiHandle,
    system_table: *mut EfiSystemTable,
) -> rvoid_core::SystemInfo {
    let _ = image_handle;

    const MSG: &[u16] = &[
        b'R' as u16,
        b'V' as u16,
        b'O' as u16,
        b'I' as u16,
        b'D' as u16,
        b':' as u16,
        b' ' as u16,
        b'U' as u16,
        b'E' as u16,
        b'F' as u16,
        b'I' as u16,
        b' ' as u16,
        b's' as u16,
        b't' as u16,
        b'a' as u16,
        b'r' as u16,
        b't' as u16,
        b'u' as u16,
        b'p' as u16,
        b'\r' as u16,
        b'\n' as u16,
        0,
    ];

    unsafe {
        uefi_puts(system_table, MSG.as_ptr());
    }

    rvoid_core::SystemInfo::empty()
}
