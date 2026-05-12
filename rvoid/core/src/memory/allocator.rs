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
    RegionWasNotAllocated,
    RegionsArray(RegionsArrayError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicalMemoryRegion {
    pub start_addr: usize,
    pub end_addr: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AllocatedPhysicalMemoryRegion {
    region: PhysicalMemoryRegion,
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
    ) -> Result<AllocatedPhysicalMemoryRegion, PhysicalMemoryAllocatorError> {
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
                let internal_region = MemoryRegion {
                    left_border: start_addr,
                    right_border: end_addr,
                };

                self.free_regions
                    .delete_region(internal_region)
                    .map_err(PhysicalMemoryAllocatorError::RegionsArray)?;

                self.used_regions
                    .insert_region(internal_region)
                    .map_err(PhysicalMemoryAllocatorError::RegionsArray)?;

                return Ok(AllocatedPhysicalMemoryRegion {
                    region: PhysicalMemoryRegion {
                        start_addr,
                        end_addr,
                    },
                });
            }
        }

        Err(PhysicalMemoryAllocatorError::OutOfMemory)
    }
    pub fn free_region(
        &mut self,
        allocated_region: AllocatedPhysicalMemoryRegion,
    ) -> Result<(), PhysicalMemoryAllocatorError> {
        let region = allocated_region.region();

        let internal_region = MemoryRegion {
            left_border: region.start_addr,
            right_border: region.end_addr,
        };

        if !self.used_regions.contains_region(internal_region) {
            return Err(PhysicalMemoryAllocatorError::RegionWasNotAllocated);
        }

        self.used_regions
            .delete_region(internal_region)
            .map_err(PhysicalMemoryAllocatorError::RegionsArray)?;

        self.free_regions
            .insert_region(internal_region)
            .map_err(PhysicalMemoryAllocatorError::RegionsArray)?;

        Ok(())
    }
}

impl AllocatedPhysicalMemoryRegion {
    pub fn region(&self) -> PhysicalMemoryRegion {
        self.region
    }

    pub fn start_addr(&self) -> usize {
        self.region.start_addr
    }

    pub fn end_addr(&self) -> usize {
        self.region.end_addr
    }

    pub fn size(&self) -> usize {
        self.region.end_addr - self.region.start_addr
    }
}

impl PhysicalMemoryRegion {
    pub fn new_inclusive(
        start_addr: usize,
        end_addr: usize,
    ) -> Result<Self, PhysicalMemoryAllocatorError> {
        if start_addr > end_addr {
            return Err(PhysicalMemoryAllocatorError::InvalidRegionSize);
        }

        Ok(Self {
            start_addr,
            end_addr,
        })
    }

    pub fn size(&self) -> Result<usize, PhysicalMemoryAllocatorError> {
        self.end_addr
            .checked_sub(self.start_addr)
            .and_then(|size_minus_one| size_minus_one.checked_add(1))
            .ok_or(PhysicalMemoryAllocatorError::AddressOverflow)
    }

    pub fn contains_addr(&self, addr: usize) -> bool {
        self.start_addr <= addr && addr <= self.end_addr
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        self.start_addr <= other.end_addr && other.start_addr <= self.end_addr
    }

    pub fn is_adjacent_to(&self, other: &Self) -> bool {
        self.end_addr.checked_add(1) == Some(other.start_addr)
            || other.end_addr.checked_add(1) == Some(self.start_addr)
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
