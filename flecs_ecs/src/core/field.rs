//! Table column API.

use std::{
    ops::{Deref, DerefMut},
    os::raw::c_void,
};

use crate::core::*;

/// Wrapper class around a table column.
///
/// # Type parameters
///
/// * `T`: The type of the column.

pub struct Field<'a, T> {
    pub(crate) slice_components: &'a mut [T],
    pub(crate) is_shared: bool,
}

impl<'a, T> Field<'a, T> {
    /// Create a new column from component array.
    ///
    /// # Arguments
    ///
    /// * `slice_components`: pointer to the component array.
    /// * `is_shared`: whether the component is shared.
    ///
    /// # See also
    ///
    /// * C++ API: `field::field`
    #[doc(alias = "field::field")]
    pub fn new(slice_components: &'a mut [T], is_shared: bool) -> Self {
        Self {
            slice_components,
            is_shared,
        }
    }

    /// whether the column / component is shared.
    pub fn is_shared(&self) -> bool {
        self.is_shared
    }
}

impl<'a, T: ComponentId> Deref for Field<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.slice_components
    }
}

impl<'a, T: ComponentId> DerefMut for Field<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.slice_components
    }
}

/// Wrapper class around an untyped table column.
/// This class is used primarily for dynamic component types.
pub struct FieldUntyped {
    pub(crate) array: *mut c_void,
    pub(crate) size: usize,
    pub(crate) count: usize,
    pub(crate) is_shared: bool,
}

/// Unsafe wrapper class around a column.
/// This class can be used when a system does not know the type of a column at
/// compile time.
///
/// # Arguments
///
/// * `array`: pointer to the component array.
/// * `size`: size of the component type.
/// * `count`: number of elements in the array.
/// * `is_shared`: whether the component is shared.
///
/// # See also
///
/// * C++ API: `untyped_field::untyped_column`
impl FieldUntyped {
    pub(crate) fn new(array: *mut c_void, size: usize, count: usize, is_shared: bool) -> Self {
        Self {
            array,
            size,
            count,
            is_shared,
        }
    }

    pub fn at(&self, index: usize) -> *const c_void {
        ecs_assert!(
            index < self.count,
            FlecsErrorCode::OutOfRange,
            "Index {} is out of range {}",
            index,
            self.count
        );

        ecs_assert!(
            !self.is_shared,
            FlecsErrorCode::InvalidParameter,
            "Column is shared, cannot index"
        );

        unsafe { self.array.add(index * self.size) }
    }

    pub fn at_mut(&mut self, index: usize) -> *mut c_void {
        ecs_assert!(
            index < self.count,
            FlecsErrorCode::OutOfRange,
            "Index {} is out of range {}",
            index,
            self.count
        );

        ecs_assert!(
            !self.is_shared,
            FlecsErrorCode::InvalidParameter,
            "Column is shared, cannot index"
        );

        unsafe { self.array.add(index * self.size) }
    }
}

// no impl Index/IndexMut for FieldUntyped because it's untyped and it does not support returning ptrs well
