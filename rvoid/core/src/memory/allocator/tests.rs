use super::*;
use core::mem::MaybeUninit;

fn memory_region(left_border: usize, right_border: usize) -> MemoryRegion {
    MemoryRegion {
        left_border,
        right_border,
    }
}

fn physical_memory_region(start_addr: usize, end_addr: usize) -> PhysicalMemoryRegion {
    PhysicalMemoryRegion {
        start_addr,
        end_addr,
    }
}

fn assert_allocated_region(
    allocated: &AllocatedPhysicalMemoryRegion,
    start_addr: usize,
    end_addr: usize,
) {
    assert_eq!(
        allocated.region(),
        physical_memory_region(start_addr, end_addr)
    );
}

fn create_regions_array<'a, const N: usize>(
    storage: &'a mut [MaybeUninit<MemoryRegion>; N],
) -> RegionsArray<'a> {
    unsafe { RegionsArray::initialize(storage.as_mut_ptr().cast::<MemoryRegion>(), N).unwrap() }
}

fn create_allocator<'a, const FREE_N: usize, const USED_N: usize>(
    free_storage: &'a mut [MaybeUninit<MemoryRegion>; FREE_N],
    used_storage: &'a mut [MaybeUninit<MemoryRegion>; USED_N],
) -> PhysicalMemoryAllocator<'a> {
    PhysicalMemoryAllocator {
        free_regions: create_regions_array(free_storage),
        used_regions: create_regions_array(used_storage),
    }
}

#[test]
fn allocate_region_rejects_zero_size() {
    let mut free_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut used_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut allocator = create_allocator(&mut free_storage, &mut used_storage);

    let result = allocator.allocate_region(0, 1);

    assert!(matches!(
        result,
        Err(PhysicalMemoryAllocatorError::ZeroSize)
    ));
}

#[test]
fn allocate_region_rejects_one_byte_region() {
    let mut free_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut used_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut allocator = create_allocator(&mut free_storage, &mut used_storage);

    let result = allocator.allocate_region(1, 1);

    assert!(matches!(
        result,
        Err(PhysicalMemoryAllocatorError::InvalidRegionSize)
    ));
}

#[test]
fn allocate_region_rejects_zero_alignment() {
    let mut free_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut used_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut allocator = create_allocator(&mut free_storage, &mut used_storage);

    let result = allocator.allocate_region(16, 0);

    assert!(matches!(
        result,
        Err(PhysicalMemoryAllocatorError::InvalidAlignment)
    ));
}

#[test]
fn allocate_region_rejects_non_power_of_two_alignment() {
    let mut free_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut used_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut allocator = create_allocator(&mut free_storage, &mut used_storage);

    let result = allocator.allocate_region(16, 3);

    assert!(matches!(
        result,
        Err(PhysicalMemoryAllocatorError::InvalidAlignment)
    ));
}

#[test]
fn allocate_region_returns_out_of_memory_when_no_free_regions() {
    let mut free_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut used_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut allocator = create_allocator(&mut free_storage, &mut used_storage);

    let result = allocator.allocate_region(16, 1);

    assert!(matches!(
        result,
        Err(PhysicalMemoryAllocatorError::OutOfMemory)
    ));
}

#[test]
fn allocate_region_allocates_from_single_free_region() {
    let mut free_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut used_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut allocator = create_allocator(&mut free_storage, &mut used_storage);

    allocator
        .free_regions
        .insert_region(memory_region(100, 199))
        .unwrap();

    let allocated = allocator.allocate_region(20, 1).unwrap();

    assert_allocated_region(&allocated, 100, 119);

    assert_eq!(allocator.used_regions.len(), 1);
    assert_eq!(allocator.used_regions[0], memory_region(100, 119));

    assert_eq!(allocator.free_regions.len(), 1);
    assert_eq!(allocator.free_regions[0], memory_region(120, 199));
}

#[test]
fn allocate_region_respects_alignment() {
    let mut free_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut used_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut allocator = create_allocator(&mut free_storage, &mut used_storage);

    allocator
        .free_regions
        .insert_region(memory_region(100, 199))
        .unwrap();

    let allocated = allocator.allocate_region(16, 64).unwrap();

    assert_allocated_region(&allocated, 128, 143);

    assert_eq!(allocator.used_regions.len(), 1);
    assert_eq!(allocator.used_regions[0], memory_region(128, 143));

    assert_eq!(allocator.free_regions.len(), 2);
    assert_eq!(allocator.free_regions[0], memory_region(100, 127));
    assert_eq!(allocator.free_regions[1], memory_region(144, 199));
}

#[test]
fn allocate_region_allocates_exact_whole_free_region() {
    let mut free_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut used_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut allocator = create_allocator(&mut free_storage, &mut used_storage);

    allocator
        .free_regions
        .insert_region(memory_region(100, 199))
        .unwrap();

    let allocated = allocator.allocate_region(100, 1).unwrap();

    assert_allocated_region(&allocated, 100, 199);

    assert_eq!(allocator.used_regions.len(), 1);
    assert_eq!(allocator.used_regions[0], memory_region(100, 199));

    assert_eq!(allocator.free_regions.len(), 0);
}

#[test]
fn allocate_region_skips_too_small_free_region() {
    let mut free_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut used_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut allocator = create_allocator(&mut free_storage, &mut used_storage);

    allocator
        .free_regions
        .insert_region(memory_region(10, 19))
        .unwrap();

    allocator
        .free_regions
        .insert_region(memory_region(100, 199))
        .unwrap();

    let allocated = allocator.allocate_region(20, 1).unwrap();

    assert_allocated_region(&allocated, 100, 119);

    assert_eq!(allocator.used_regions.len(), 1);
    assert_eq!(allocator.used_regions[0], memory_region(100, 119));

    assert_eq!(allocator.free_regions.len(), 2);
    assert_eq!(allocator.free_regions[0], memory_region(10, 19));
    assert_eq!(allocator.free_regions[1], memory_region(120, 199));
}

#[test]
fn allocate_region_returns_out_of_memory_when_alignment_prevents_fit() {
    let mut free_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut used_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut allocator = create_allocator(&mut free_storage, &mut used_storage);

    allocator
        .free_regions
        .insert_region(memory_region(100, 120))
        .unwrap();

    let result = allocator.allocate_region(16, 64);

    assert!(matches!(
        result,
        Err(PhysicalMemoryAllocatorError::OutOfMemory)
    ));

    assert_eq!(allocator.free_regions.len(), 1);
    assert_eq!(allocator.free_regions[0], memory_region(100, 120));
    assert_eq!(allocator.used_regions.len(), 0);
}

#[test]
fn allocate_region_returns_used_regions_storage_full_when_used_storage_is_full() {
    let mut free_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut used_storage = [MaybeUninit::<MemoryRegion>::uninit(); 1];
    let mut allocator = create_allocator(&mut free_storage, &mut used_storage);

    allocator
        .free_regions
        .insert_region(memory_region(100, 199))
        .unwrap();

    allocator
        .used_regions
        .insert_region(memory_region(1000, 1099))
        .unwrap();

    let result = allocator.allocate_region(20, 1);

    assert!(matches!(
        result,
        Err(PhysicalMemoryAllocatorError::UsedRegionsStorageFull)
    ));

    assert_eq!(allocator.free_regions.len(), 1);
    assert_eq!(allocator.free_regions[0], memory_region(100, 199));
    assert_eq!(allocator.used_regions.len(), 1);
    assert_eq!(allocator.used_regions[0], memory_region(1000, 1099));
}

#[test]
fn allocate_region_returns_regions_array_error_when_free_regions_cannot_split() {
    let mut free_storage = [MaybeUninit::<MemoryRegion>::uninit(); 1];
    let mut used_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut allocator = create_allocator(&mut free_storage, &mut used_storage);

    allocator
        .free_regions
        .insert_region(memory_region(100, 199))
        .unwrap();

    let result = allocator.allocate_region(20, 128);

    assert!(matches!(
        result,
        Err(PhysicalMemoryAllocatorError::RegionsArray(
            RegionsArrayError::StorageFull
        ))
    ));

    assert_eq!(allocator.free_regions.len(), 1);
    assert_eq!(allocator.free_regions[0], memory_region(100, 199));
    assert_eq!(allocator.used_regions.len(), 0);
}

#[test]
fn free_region_returns_allocated_region_to_free_regions() {
    let mut free_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut used_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut allocator = create_allocator(&mut free_storage, &mut used_storage);

    allocator
        .free_regions
        .insert_region(memory_region(100, 199))
        .unwrap();

    let allocated = allocator.allocate_region(20, 1).unwrap();

    assert_eq!(allocator.used_regions.len(), 1);
    assert_eq!(allocator.used_regions[0], memory_region(100, 119));
    assert_eq!(allocator.free_regions.len(), 1);
    assert_eq!(allocator.free_regions[0], memory_region(120, 199));

    let result = allocator.free_region(allocated);

    assert!(result.is_ok());

    assert_eq!(allocator.used_regions.len(), 0);

    assert_eq!(allocator.free_regions.len(), 1);
    assert_eq!(allocator.free_regions[0], memory_region(100, 199));
}

#[test]
fn free_region_merges_with_left_and_right_free_regions() {
    let mut free_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut used_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut allocator = create_allocator(&mut free_storage, &mut used_storage);

    allocator
        .free_regions
        .insert_region(memory_region(100, 199))
        .unwrap();

    let allocated = allocator.allocate_region(16, 64).unwrap();

    assert_allocated_region(&allocated, 128, 143);

    assert_eq!(allocator.used_regions.len(), 1);
    assert_eq!(allocator.used_regions[0], memory_region(128, 143));

    assert_eq!(allocator.free_regions.len(), 2);
    assert_eq!(allocator.free_regions[0], memory_region(100, 127));
    assert_eq!(allocator.free_regions[1], memory_region(144, 199));

    let result = allocator.free_region(allocated);

    assert!(result.is_ok());

    assert_eq!(allocator.used_regions.len(), 0);

    assert_eq!(allocator.free_regions.len(), 1);
    assert_eq!(allocator.free_regions[0], memory_region(100, 199));
}

#[test]
fn free_region_returns_error_when_region_is_not_in_used_regions() {
    let mut free_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut used_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut allocator = create_allocator(&mut free_storage, &mut used_storage);

    allocator
        .free_regions
        .insert_region(memory_region(100, 199))
        .unwrap();

    let allocated = allocator.allocate_region(20, 1).unwrap();

    allocator
        .used_regions
        .delete_region(memory_region(100, 119))
        .unwrap();

    let result = allocator.free_region(allocated);

    assert!(matches!(
        result,
        Err(PhysicalMemoryAllocatorError::RegionWasNotAllocated)
    ));

    assert_eq!(allocator.used_regions.len(), 0);

    assert_eq!(allocator.free_regions.len(), 1);
    assert_eq!(allocator.free_regions[0], memory_region(120, 199));
}

#[test]
fn free_region_does_not_change_state_when_free_regions_storage_is_full() {
    let mut free_storage = [MaybeUninit::<MemoryRegion>::uninit(); 1];
    let mut used_storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut allocator = create_allocator(&mut free_storage, &mut used_storage);

    allocator
        .free_regions
        .insert_region(memory_region(100, 199))
        .unwrap();

    let allocated = allocator.allocate_region(100, 1).unwrap();

    assert_eq!(allocator.free_regions.len(), 0);
    assert_eq!(allocator.used_regions.len(), 1);
    assert_eq!(allocator.used_regions[0], memory_region(100, 199));

    allocator
        .free_regions
        .insert_region(memory_region(300, 399))
        .unwrap();

    let result = allocator.free_region(allocated);

    assert!(matches!(
        result,
        Err(PhysicalMemoryAllocatorError::RegionsArray(
            RegionsArrayError::StorageFull
        ))
    ));

    assert_eq!(allocator.used_regions.len(), 1);
    assert_eq!(allocator.used_regions[0], memory_region(100, 199));

    assert_eq!(allocator.free_regions.len(), 1);
    assert_eq!(allocator.free_regions[0], memory_region(300, 399));
}
