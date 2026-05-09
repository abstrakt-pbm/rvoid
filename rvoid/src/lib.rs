#![no_std]

pub use rvoid_core::SystemInfo;

pub use rvoid_startup::entry;

pub use rvoid_startup as startup;

pub mod prelude {
    pub use crate::SystemInfo;
}
