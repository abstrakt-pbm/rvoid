use core::{cell::UnsafeCell, mem::MaybeUninit};

use super::{AllocatedPhysicalMemoryRegion, PhysicalMemoryAllocator, PhysicalMemoryAllocatorError};

pub struct GlobalPhysicalMemoryAllocator {
    inner: UnsafeCell<MaybeUninit<PhysicalMemoryAllocator<'static>>>,
    initialized: UnsafeCell<bool>,
}

// SAFETY:
// This is sound only while access to the allocator is externally synchronized.
// For early single-core boot code this can be acceptable.
// Before using this from interrupts or multiple CPUs, replace this with an
// IRQ-safe lock / spinlock-based implementation.
unsafe impl Sync for GlobalPhysicalMemoryAllocator {}

impl GlobalPhysicalMemoryAllocator {
    pub const fn uninit() -> Self {
        Self {
            inner: UnsafeCell::new(MaybeUninit::uninit()),
            initialized: UnsafeCell::new(false),
        }
    }

    pub fn init(&self, allocator: PhysicalMemoryAllocator<'static>) {
        unsafe {
            if *self.initialized.get() {
                panic!("physical memory allocator already initialized");
            }

            (*self.inner.get()).write(allocator);
            *self.initialized.get() = true;
        }
    }

    fn with<R>(&self, f: impl FnOnce(&mut PhysicalMemoryAllocator<'static>) -> R) -> R {
        unsafe {
            if !*self.initialized.get() {
                panic!("physical memory allocator is not initialized");
            }

            let allocator = (*self.inner.get()).assume_init_mut();
            f(allocator)
        }
    }
}

static PHYSICAL_MEMORY_ALLOCATOR: GlobalPhysicalMemoryAllocator =
    GlobalPhysicalMemoryAllocator::uninit();

pub fn init_physical_allocator(allocator: PhysicalMemoryAllocator<'static>) {
    PHYSICAL_MEMORY_ALLOCATOR.init(allocator);
}

pub fn allocate_physical_region(
    size: usize,
    align: usize,
) -> Result<AllocatedPhysicalMemoryRegion, PhysicalMemoryAllocatorError> {
    PHYSICAL_MEMORY_ALLOCATOR.with(|allocator| allocator.allocate_region(size, align))
}

pub fn free_physical_region(
    region: AllocatedPhysicalMemoryRegion,
) -> Result<(), PhysicalMemoryAllocatorError> {
    PHYSICAL_MEMORY_ALLOCATOR.with(|allocator| allocator.free_region(region))
}
