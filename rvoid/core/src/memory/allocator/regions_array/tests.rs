
use super::*;
use core::mem::MaybeUninit;

fn region(left_border: usize, right_border: usize) -> MemoryRegion {
    MemoryRegion {
        left_border,
        right_border,
    }
}

fn create_array<'a, const N: usize>(
    storage: &'a mut [MaybeUninit<MemoryRegion>; N],
) -> RegionsArray<'a> {
    unsafe { RegionsArray::initialize(storage.as_mut_ptr().cast::<MemoryRegion>(), N).unwrap() }
}

#[test]
fn initialize_creates_empty_array() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];

    let array = create_array(&mut storage);

    assert_eq!(array.len, 0);
    assert_eq!(array.capacity, 4);
    assert!(array.get(0).is_none());
}

#[test]
fn initialize_rejects_null_pointer() {
    let result = unsafe { RegionsArray::initialize(core::ptr::null_mut(), 4) };

    assert!(matches!(result, Err(RegionsArrayError::NullPointer)));
}

#[test]
fn initialize_rejects_zero_capacity() {
    let ptr = NonNull::<MemoryRegion>::dangling().as_ptr();

    let result = unsafe { RegionsArray::initialize(ptr, 0) };

    assert!(matches!(result, Err(RegionsArrayError::ZeroCapacity)));
}

#[test]
fn insert_into_empty_array() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 20)).unwrap();

    assert_eq!(array.len, 1);
    assert_eq!(array[0], region(10, 20));
}

#[test]
fn insert_before_existing_region_keeps_order() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut array = create_array(&mut storage);

    array.insert_region(region(30, 40)).unwrap();
    array.insert_region(region(10, 20)).unwrap();

    assert_eq!(array.len, 2);
    assert_eq!(array[0], region(10, 20));
    assert_eq!(array[1], region(30, 40));
}

#[test]
fn insert_between_existing_regions_keeps_order() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 20)).unwrap();
    array.insert_region(region(50, 60)).unwrap();
    array.insert_region(region(30, 40)).unwrap();

    assert_eq!(array.len, 3);
    assert_eq!(array[0], region(10, 20));
    assert_eq!(array[1], region(30, 40));
    assert_eq!(array[2], region(50, 60));
}

#[test]
fn insert_after_existing_region_keeps_order() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 20)).unwrap();
    array.insert_region(region(30, 40)).unwrap();

    assert_eq!(array.len, 2);
    assert_eq!(array[0], region(10, 20));
    assert_eq!(array[1], region(30, 40));
}

#[test]
fn insert_region_inside_existing_region_does_nothing() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 100)).unwrap();
    array.insert_region(region(20, 30)).unwrap();

    assert_eq!(array.len, 1);
    assert_eq!(array[0], region(10, 100));
}

#[test]
fn insert_region_expands_existing_region_to_right() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 20)).unwrap();
    array.insert_region(region(15, 30)).unwrap();

    assert_eq!(array.len, 1);
    assert_eq!(array[0], region(10, 30));
}

#[test]
fn insert_region_expands_existing_region_to_left() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 20)).unwrap();
    array.insert_region(region(0, 15)).unwrap();

    assert_eq!(array.len, 1);
    assert_eq!(array[0], region(0, 20));
}

#[test]
fn insert_region_merges_multiple_overlapping_regions() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 8];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 20)).unwrap();
    array.insert_region(region(30, 40)).unwrap();
    array.insert_region(region(50, 60)).unwrap();

    array.insert_region(region(15, 55)).unwrap();

    assert_eq!(array.len, 1);
    assert_eq!(array[0], region(10, 60));
}

#[test]
fn insert_region_covering_multiple_regions_merges_them() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 8];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 20)).unwrap();
    array.insert_region(region(30, 40)).unwrap();

    array.insert_region(region(0, 50)).unwrap();

    assert_eq!(array.len, 1);
    assert_eq!(array[0], region(0, 50));
}

#[test]
fn adjacent_regions_are_not_merged() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 20)).unwrap();
    array.insert_region(region(21, 30)).unwrap();

    assert_eq!(array.len, 2);
    assert_eq!(array[0], region(10, 20));
    assert_eq!(array[1], region(21, 30));
}

#[test]
fn insert_rejects_invalid_region() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut array = create_array(&mut storage);

    let result = array.insert_region(region(20, 10));

    assert!(matches!(result, Err(RegionsArrayError::InvalidRegion)));
}

#[test]
fn insert_returns_storage_full_when_capacity_exhausted() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 1];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 20)).unwrap();

    let result = array.insert_region(region(30, 40));

    assert!(matches!(result, Err(RegionsArrayError::StorageFull)));
}

#[test]
fn delete_by_index_removes_region_and_shifts_tail_left() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 20)).unwrap();
    array.insert_region(region(30, 40)).unwrap();
    array.insert_region(region(50, 60)).unwrap();

    array.delete_by_index(1).unwrap();

    assert_eq!(array.len, 2);
    assert_eq!(array[0], region(10, 20));
    assert_eq!(array[1], region(50, 60));
}

#[test]
fn delete_by_index_rejects_out_of_bounds_index() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 20)).unwrap();

    let result = array.delete_by_index(1);

    assert!(matches!(result, Err(RegionsArrayError::IndexOutOfBounds)));
}
#[test]
fn subtract_keeps_region_when_there_is_no_overlap() {
    let existing = region(10, 20);
    let deleted = region(30, 40);

    let result = existing.subtract(&deleted).unwrap();

    assert_eq!(result, SubtractResult::One(region(10, 20)));
}

#[test]
fn subtract_keeps_region_when_deleted_region_is_adjacent_on_right() {
    let existing = region(10, 20);
    let deleted = region(21, 30);

    let result = existing.subtract(&deleted).unwrap();

    assert_eq!(result, SubtractResult::One(region(10, 20)));
}

#[test]
fn subtract_keeps_region_when_deleted_region_is_adjacent_on_left() {
    let existing = region(10, 20);
    let deleted = region(0, 9);

    let result = existing.subtract(&deleted).unwrap();

    assert_eq!(result, SubtractResult::One(region(10, 20)));
}

#[test]
fn subtract_returns_none_when_deleted_region_fully_covers_existing_region() {
    let existing = region(10, 20);
    let deleted = region(0, 30);

    let result = existing.subtract(&deleted).unwrap();

    assert_eq!(result, SubtractResult::None);
}

#[test]
fn subtract_returns_none_when_deleted_region_equals_existing_region() {
    let existing = region(10, 20);
    let deleted = region(10, 20);

    let result = existing.subtract(&deleted).unwrap();

    assert_eq!(result, SubtractResult::None);
}

#[test]
fn subtract_cuts_left_part_of_region() {
    let existing = region(10, 50);
    let deleted = region(0, 20);

    let result = existing.subtract(&deleted).unwrap();

    assert_eq!(result, SubtractResult::One(region(21, 50)));
}

#[test]
fn subtract_cuts_right_part_of_region() {
    let existing = region(10, 50);
    let deleted = region(40, 100);

    let result = existing.subtract(&deleted).unwrap();

    assert_eq!(result, SubtractResult::One(region(10, 39)));
}

#[test]
fn subtract_splits_region_when_deleted_region_is_inside_existing_region() {
    let existing = region(10, 50);
    let deleted = region(20, 30);

    let result = existing.subtract(&deleted).unwrap();

    assert_eq!(result, SubtractResult::Two(region(10, 19), region(31, 50)));
}

#[test]
fn subtract_drops_one_byte_left_fragment() {
    let existing = region(10, 20);
    let deleted = region(11, 15);

    let result = existing.subtract(&deleted).unwrap();

    assert_eq!(result, SubtractResult::One(region(16, 20)));
}

#[test]
fn subtract_drops_one_byte_right_fragment() {
    let existing = region(10, 20);
    let deleted = region(15, 19);

    let result = existing.subtract(&deleted).unwrap();

    assert_eq!(result, SubtractResult::One(region(10, 14)));
}

#[test]
fn subtract_drops_both_one_byte_fragments() {
    let existing = region(10, 13);
    let deleted = region(11, 12);

    let result = existing.subtract(&deleted).unwrap();

    assert_eq!(result, SubtractResult::None);
}

#[test]
fn subtract_rejects_invalid_deleted_region() {
    let existing = region(10, 20);
    let deleted = region(20, 10);

    let result = existing.subtract(&deleted);

    assert!(matches!(result, Err(RegionsArrayError::InvalidRegion)));
}

#[test]
fn delete_region_rejects_invalid_region() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 20)).unwrap();

    let result = array.delete_region(region(20, 10));

    assert!(matches!(result, Err(RegionsArrayError::InvalidRegion)));
}

#[test]
fn delete_region_does_nothing_when_region_does_not_overlap() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 20)).unwrap();
    array.insert_region(region(30, 40)).unwrap();

    array.delete_region(region(50, 60)).unwrap();

    assert_eq!(array.len, 2);
    assert_eq!(array[0], region(10, 20));
    assert_eq!(array[1], region(30, 40));
}

#[test]
fn delete_region_does_nothing_when_region_is_adjacent() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 20)).unwrap();

    array.delete_region(region(21, 30)).unwrap();

    assert_eq!(array.len, 1);
    assert_eq!(array[0], region(10, 20));
}

#[test]
fn delete_region_removes_exact_region() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 20)).unwrap();

    array.delete_region(region(10, 20)).unwrap();

    assert_eq!(array.len, 0);
    assert!(array.get(0).is_none());
}

#[test]
fn delete_region_removes_region_when_fully_covered() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 20)).unwrap();

    array.delete_region(region(0, 30)).unwrap();

    assert_eq!(array.len, 0);
    assert!(array.get(0).is_none());
}

#[test]
fn delete_region_cuts_left_part_of_existing_region() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 50)).unwrap();

    array.delete_region(region(0, 20)).unwrap();

    assert_eq!(array.len, 1);
    assert_eq!(array[0], region(21, 50));
}

#[test]
fn delete_region_cuts_right_part_of_existing_region() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 50)).unwrap();

    array.delete_region(region(40, 100)).unwrap();

    assert_eq!(array.len, 1);
    assert_eq!(array[0], region(10, 39));
}

#[test]
fn delete_region_splits_existing_region() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 50)).unwrap();

    array.delete_region(region(20, 30)).unwrap();

    assert_eq!(array.len, 2);
    assert_eq!(array[0], region(10, 19));
    assert_eq!(array[1], region(31, 50));
}

#[test]
fn delete_region_spanning_multiple_regions_trims_edges_and_removes_middle() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 8];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 20)).unwrap();
    array.insert_region(region(30, 40)).unwrap();
    array.insert_region(region(50, 60)).unwrap();

    array.delete_region(region(15, 55)).unwrap();

    assert_eq!(array.len, 2);
    assert_eq!(array[0], region(10, 14));
    assert_eq!(array[1], region(56, 60));
}

#[test]
fn delete_region_removes_multiple_fully_covered_regions() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 8];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 20)).unwrap();
    array.insert_region(region(30, 40)).unwrap();
    array.insert_region(region(50, 60)).unwrap();

    array.delete_region(region(0, 100)).unwrap();

    assert_eq!(array.len, 0);
    assert!(array.get(0).is_none());
}

#[test]
fn delete_region_drops_one_byte_fragments_after_split() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 4];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 13)).unwrap();

    array.delete_region(region(11, 12)).unwrap();

    assert_eq!(array.len, 0);
    assert!(array.get(0).is_none());
}

#[test]
fn delete_region_split_returns_storage_full_when_capacity_is_exhausted() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 1];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 100)).unwrap();

    let result = array.delete_region(region(30, 40));

    assert!(matches!(result, Err(RegionsArrayError::StorageFull)));

    assert_eq!(array.len, 1);
    assert_eq!(array[0], region(10, 100));
}

#[test]
fn delete_region_can_remove_from_full_storage_when_no_split_is_needed() {
    let mut storage = [MaybeUninit::<MemoryRegion>::uninit(); 2];
    let mut array = create_array(&mut storage);

    array.insert_region(region(10, 20)).unwrap();
    array.insert_region(region(30, 40)).unwrap();

    array.delete_region(region(0, 25)).unwrap();

    assert_eq!(array.len, 1);
    assert_eq!(array[0], region(30, 40));
}
