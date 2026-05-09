#![no_std]
#![no_main]

use core::panic::PanicInfo;
use rvoid::prelude::*;

#[rvoid::entry]
fn main(system: SystemInfo) -> ! {
    let _ = system;

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
