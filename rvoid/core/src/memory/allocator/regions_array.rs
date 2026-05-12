use core::{
    marker::PhantomData,
    mem::MaybeUninit,
    ops::{Index, IndexMut},
    ptr::NonNull,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RegionsArrayError {
    NullPointer,
    ZeroCapacity,
    UnalignedPointer,
    StorageFull,
    IndexOutOfBounds,
    InvalidRegion,
    InvalidState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct OverlapRegionIds {
    pub(super) left_border_region_id: Option<usize>,
    pub(super) right_border_region_id: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MemoryRegion {
    pub left_border: usize,
    pub right_border: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SubtractResult {
    None,
    One(MemoryRegion),
    Two(MemoryRegion, MemoryRegion),
}

pub(super) struct RegionsArray<'metadata> {
    ptr: NonNull<MaybeUninit<MemoryRegion>>,
    capacity: usize,
    len: usize,

    _marker: PhantomData<&'metadata mut [MaybeUninit<MemoryRegion>]>,
}

impl<'metadata> RegionsArray<'metadata> {
    pub(super) unsafe fn initialize(
        ptr: *mut MemoryRegion,
        capacity: usize,
    ) -> Result<Self, RegionsArrayError> {
        if ptr.is_null() {
            return Err(RegionsArrayError::NullPointer);
        }

        if (ptr as usize) % core::mem::align_of::<MemoryRegion>() != 0 {
            return Err(RegionsArrayError::UnalignedPointer);
        }

        if capacity == 0 {
            return Err(RegionsArrayError::ZeroCapacity);
        }

        let ptr = NonNull::new(ptr.cast::<MaybeUninit<MemoryRegion>>())
            .ok_or(RegionsArrayError::NullPointer)?;

        Ok(Self {
            ptr,
            capacity,
            len: 0,
            _marker: PhantomData,
        })
    }

    pub(super) fn insert_region(&mut self, region: MemoryRegion) -> Result<(), RegionsArrayError> {
        if !region.is_valid() {
            return Err(RegionsArrayError::InvalidRegion);
        }

        let overlap_region_ids = self.find_overlap_regions(&region);

        match (
            overlap_region_ids.left_border_region_id,
            overlap_region_ids.right_border_region_id,
        ) {
            (None, None) => {
                let region_id = self.find_nearest_right(region.left_border);
                self.shift_right_by_one_step(region_id)?;
                self.write_region_unchecked(region_id, region);
                Ok(())
            }
            (Some(left_id), Some(right_id)) => {
                if left_id == right_id {
                    let region_id = left_id;
                    self[region_id].left_border =
                        core::cmp::min(self[region_id].left_border, region.left_border);

                    self[region_id].right_border =
                        core::cmp::max(self[region_id].right_border, region.right_border);
                } else {
                    let merged_region = MemoryRegion {
                        left_border: core::cmp::min(self[left_id].left_border, region.left_border),

                        right_border: core::cmp::max(
                            self[right_id].right_border,
                            region.right_border,
                        ),
                    };

                    self[left_id] = merged_region;

                    for index in ((left_id + 1)..=right_id).rev() {
                        self.delete_by_index(index)?;
                    }
                }
                Ok(())
            }
            _ => Err(RegionsArrayError::InvalidState),
        }
    }

    pub(super) fn find_nearest_right(&self, addr: usize) -> usize {
        for index in 0..self.len {
            let current_region = self[index];
            if current_region.left_border > addr {
                return index;
            }
        }
        self.len
    }

    pub(super) fn delete_region(&mut self, deleted: MemoryRegion) -> Result<(), RegionsArrayError> {
        if !deleted.is_valid() {
            return Err(RegionsArrayError::InvalidRegion);
        }

        let mut extra_slots_needed = 0usize;
        let mut slots_freed = 0usize;

        // Preflight pass.
        //
        // Пока ничего не меняем. Только проверяем, сколько слотов будет освобождено
        // и сколько дополнительных слотов понадобится для split-ов.
        for index in 0..self.len {
            let existing = self[index];

            match existing.subtract(&deleted)? {
                SubtractResult::None => {
                    slots_freed += 1;
                }

                SubtractResult::One(_) => {}

                SubtractResult::Two(_, _) => {
                    extra_slots_needed += 1;
                }
            }
        }

        // После удаления полностью покрытых регионов длина станет:
        //
        // self.len - slots_freed
        //
        // После split-ов длина увеличится на:
        //
        // extra_slots_needed
        //
        // Значит итоговая длина не должна превысить capacity.
        if self.len - slots_freed + extra_slots_needed > self.capacity {
            return Err(RegionsArrayError::StorageFull);
        }

        // Первый mutation pass.
        //
        // Сначала удаляем полностью покрытые регионы. Это освобождает место
        // для возможных split-ов.
        let mut index = 0;

        while index < self.len {
            let existing = self[index];

            match existing.subtract(&deleted)? {
                SubtractResult::None => {
                    self.delete_by_index(index)?;
                    // index не увеличиваем, потому что после сдвига на это место
                    // приехал следующий элемент.
                }

                _ => {
                    index += 1;
                }
            }
        }

        // Второй mutation pass.
        //
        // Теперь оставшиеся регионы либо не пересекаются, либо обрезаются,
        // либо split-ятся на два.
        let mut index = 0;

        while index < self.len {
            let existing = self[index];

            match existing.subtract(&deleted)? {
                SubtractResult::None => {
                    // Теоретически уже удалено на первом проходе.
                    // Если попали сюда, состояние неожиданное.
                    self.delete_by_index(index)?;
                }

                SubtractResult::One(region) => {
                    self[index] = region;
                    index += 1;
                }

                SubtractResult::Two(left, right) => {
                    self[index] = left;

                    self.shift_right_by_one_step(index + 1)?;
                    self.write_region_unchecked(index + 1, right);

                    index += 2;
                }
            }
        }

        Ok(())
    }

    pub(super) fn contains_region(&self, region: MemoryRegion) -> bool {
        for index in 0..self.len {
            if self[index] == region {
                return true;
            }
        }
        false
    }

    // find regions id overlaping with new created region
    pub(super) fn find_overlap_regions(&self, region: &MemoryRegion) -> OverlapRegionIds {
        let mut result = OverlapRegionIds {
            left_border_region_id: None,
            right_border_region_id: None,
        };

        if self.len == 0 {
            return result;
        }

        for index in 0..self.len {
            let current_region = self[index];
            if current_region.overlaps(region) {
                result.left_border_region_id = Some(index);
                break;
            }
        }

        for index in (0..self.len).rev() {
            let current_region = self[index];
            if current_region.overlaps(region) {
                result.right_border_region_id = Some(index);
                break;
            }
        }
        result
    }

    fn delete_by_index(&mut self, index: usize) -> Result<(), RegionsArrayError> {
        if index >= self.len {
            return Err(RegionsArrayError::IndexOutOfBounds);
        }

        let tail_count = self.len - index - 1;
        if tail_count > 0 {
            unsafe {
                core::ptr::copy(
                    self.ptr.as_ptr().add(index + 1),
                    self.ptr.as_ptr().add(index),
                    tail_count,
                );
            }
        }
        self.len -= 1;

        Ok(())
    }

    fn shift_right_by_one_step(&mut self, start_index: usize) -> Result<(), RegionsArrayError> {
        if start_index > self.len {
            return Err(RegionsArrayError::IndexOutOfBounds);
        }

        if self.len >= self.capacity {
            return Err(RegionsArrayError::StorageFull);
        }

        let count = self.len - start_index;

        if count > 0 {
            unsafe {
                core::ptr::copy(
                    self.ptr.as_ptr().add(start_index),
                    self.ptr.as_ptr().add(start_index + 1),
                    count,
                );
            }
        }
        self.len += 1;
        Ok(())
    }

    fn write_region_unchecked(&mut self, index: usize, region: MemoryRegion) {
        unsafe {
            self.ptr.as_ptr().add(index).write(MaybeUninit::new(region));
        }
    }

    pub(super) const fn len(&self) -> usize {
        self.len
    }

    pub(super) const fn capacity(&self) -> usize {
        self.capacity
    }

    pub(super) fn get(&self, index: usize) -> Option<&MemoryRegion> {
        if index >= self.len {
            return None;
        }
        unsafe { Some(&*self.ptr.as_ptr().add(index).cast::<MemoryRegion>()) }
    }

    pub(super) fn get_mut(&mut self, index: usize) -> Option<&mut MemoryRegion> {
        if index >= self.len {
            return None;
        }
        unsafe { Some(&mut *self.ptr.as_ptr().add(index).cast::<MemoryRegion>()) }
    }
}

impl MemoryRegion {
    pub(super) const fn is_valid(&self) -> bool {
        self.left_border < self.right_border
    }

    pub(super) const fn contains_addr(&self, addr: usize) -> bool {
        self.left_border <= addr && addr <= self.right_border
    }

    pub(super) const fn contains_region(&self, region: &MemoryRegion) -> bool {
        self.left_border <= region.left_border && region.right_border <= self.right_border
    }

    pub(super) const fn overlaps(&self, region: &MemoryRegion) -> bool {
        self.left_border <= region.right_border && region.left_border <= self.right_border
    }

    pub(super) const fn new(
        left_border: usize,
        right_border: usize,
    ) -> Result<Self, RegionsArrayError> {
        let region = Self {
            left_border,
            right_border,
        };

        if !region.is_valid() {
            return Err(RegionsArrayError::InvalidRegion);
        }

        Ok(region)
    }

    fn valid_or_none(left_border: usize, right_border: usize) -> Option<Self> {
        let region = Self {
            left_border,
            right_border,
        };

        if region.is_valid() {
            Some(region)
        } else {
            None
        }
    }

    pub(super) fn subtract(
        &self,
        deleted: &MemoryRegion,
    ) -> Result<SubtractResult, RegionsArrayError> {
        if !self.is_valid() || !deleted.is_valid() {
            return Err(RegionsArrayError::InvalidRegion);
        }

        if !self.overlaps(deleted) {
            return Ok(SubtractResult::One(*self));
        }

        if deleted.contains_region(self) {
            return Ok(SubtractResult::None);
        }

        let left = if deleted.left_border > self.left_border {
            MemoryRegion::valid_or_none(self.left_border, deleted.left_border - 1)
        } else {
            None
        };

        let right = if deleted.right_border < self.right_border {
            MemoryRegion::valid_or_none(deleted.right_border + 1, self.right_border)
        } else {
            None
        };

        Ok(SubtractResult::from_parts(left, right))
    }
}

impl<'metadata> Index<usize> for RegionsArray<'metadata> {
    type Output = MemoryRegion;
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("RegionsArray index out of bounds")
    }
}

impl<'metadata> IndexMut<usize> for RegionsArray<'metadata> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index)
            .expect("RegionsArray index out of bounds")
    }
}

impl SubtractResult {
    fn from_parts(left: Option<MemoryRegion>, right: Option<MemoryRegion>) -> Self {
        match (left, right) {
            (None, None) => Self::None,
            (Some(region), None) | (None, Some(region)) => Self::One(region),
            (Some(left), Some(right)) => Self::Two(left, right),
        }
    }
}

#[cfg(test)]
mod tests;
