mod regions_array;

use self::regions_array::{MemoryRegion, RegionsArray, RegionsArrayError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhysicalMemoryAllocatorError {
    ZeroSize,
    InvalidRegionSize,
    InvalidAlignment,
    AddressOverflow,
    OutOfMemory,
    UsedRegionsStorageFull,
    RegionsArray(RegionsArrayError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicalMemoryRegion {
    pub start_addr: usize,
    pub end_addr: usize,
}

pub struct PhysicalMemoryAllocator<'metadata> {
    free_regions: RegionsArray<'metadata>,
    used_regions: RegionsArray<'metadata>,
}

impl<'metadata> PhysicalMemoryAllocator<'metadata> {
    pub fn initialize() {
        todo!("PhysicalMemoryAllocator::initialize is not implemented yet")
    }

    pub fn allocate_region(
        &mut self,
        size: usize,
        align: usize,
    ) -> Result<PhysicalMemoryRegion, PhysicalMemoryAllocatorError> {
        if size == 0 {
            return Err(PhysicalMemoryAllocatorError::ZeroSize);
        }

        if size < 2 {
            return Err(PhysicalMemoryAllocatorError::InvalidRegionSize);
        }

        if align == 0 || !align.is_power_of_two() {
            return Err(PhysicalMemoryAllocatorError::InvalidAlignment);
        }

        if self.used_regions.len() >= self.used_regions.capacity() {
            return Err(PhysicalMemoryAllocatorError::UsedRegionsStorageFull);
        }

        for index in 0..self.free_regions.len() {
            let free = self.free_regions[index];

            let start_addr = align_up(free.left_border, align)?;

            let end_addr = start_addr
                .checked_add(size - 1)
                .ok_or(PhysicalMemoryAllocatorError::AddressOverflow)?;

            if end_addr <= free.right_border {
                let allocated_region = MemoryRegion {
                    left_border: start_addr,
                    right_border: end_addr,
                };

                self.free_regions
                    .delete_region(allocated_region)
                    .map_err(PhysicalMemoryAllocatorError::RegionsArray)?;

                self.used_regions
                    .insert_region(allocated_region)
                    .map_err(PhysicalMemoryAllocatorError::RegionsArray)?;

                return Ok(PhysicalMemoryRegion {
                    start_addr,
                    end_addr,
                });
            }
        }

        Err(PhysicalMemoryAllocatorError::OutOfMemory)
    }
}

fn align_up(value: usize, align: usize) -> Result<usize, PhysicalMemoryAllocatorError> {
    if align == 0 || !align.is_power_of_two() {
        return Err(PhysicalMemoryAllocatorError::InvalidAlignment);
    }

    value
        .checked_next_multiple_of(align)
        .ok_or(PhysicalMemoryAllocatorError::AddressOverflow)
}

#[cfg(test)]
mod tests;
