//! An [`Archetype`] type can be used to describe what types of components an entity has.

use std::{
    ffi::CStr,
    fmt::{Debug, Display},
    ptr::NonNull,
};

use crate::core::*;
use crate::sys;

/// Archetype type.
///
/// A type is a vector of component [ids](Id) which can be requested from [entities] or [tables].
///
/// # See also
///
/// * [`EntityView::archetype()`]
/// * [`Table::archetype()`]
/// * C++ API: `type`
///
/// [entities]: EntityView::archetype
/// [tables]: Table::archetype
#[doc(alias = "type")]
pub struct Archetype<'a> {
    world: WorldRef<'a>,
    type_vec: &'a [Id],
    lock: Option<TableLock<'a>>,
}

impl<'a> Display for Archetype<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(s) = self.to_string() {
            write!(f, "{}", s)
        } else {
            write!(f, "empty archetype")
        }
    }
}

impl<'a> Debug for Archetype<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(s) = self.to_string() {
            write!(f, "{}", s)
        } else {
            write!(f, "empty archetype")
        }
    }
}

impl<'a> Archetype<'a> {
    pub fn new(world: impl IntoWorld<'a>, type_vec: &'a [Id]) -> Self {
        Archetype {
            world: world.world(),
            type_vec,
            lock: None,
        }
    }

    pub fn new_locked(world: impl IntoWorld<'a>, type_vec: &'a [Id], lock: TableLock<'a>) -> Self {
        Archetype {
            world: world.world(),
            type_vec,
            lock: Some(lock),
        }
    }

    /// Convert type to comma-separated string
    ///
    /// # Returns
    ///
    /// `Some(String)` - if the type is not null. `None` - if the type is null.
    ///
    /// # See also
    ///
    /// * C++ API: `Type::str()`
    #[doc(alias = "Type::str()")]
    pub fn to_string(&self) -> Option<String> {
        NonNull::new(unsafe {
            sys::ecs_type_str(
                self.world.world_ptr_mut(),
                &sys::ecs_type_t {
                    array: self.type_vec.as_ptr() as *mut _,
                    count: self.type_vec.len() as i32,
                },
            )
        })
        .map(|s| unsafe {
            let len = CStr::from_ptr(s.as_ptr()).to_bytes().len();
            // Convert the C string to a Rust String without any new heap allocation.
            // The String will de-allocate the C string when it goes out of scope.
            String::from_utf8_unchecked(Vec::from_raw_parts(s.as_ptr() as *mut u8, len, len))
        })
    }

    /// Return the number of elements in the type.
    ///
    /// # See also
    ///
    /// * C++ API: `Type::count()`
    #[doc(alias = "Type::count()")]
    pub fn count(&self) -> usize {
        self.type_vec.len()
    }

    /// Return a slice to the array of types.
    ///
    /// # Returns
    ///
    /// `Some(&[Id])` - A slice to the array if the type is not null and has elements.
    /// `None` - If the type is null or has no elements.
    ///
    /// # Safety
    ///
    /// This method is safe as long as the underlying array pointed to by `type_vec` does not change
    /// while the slice is in use and the elements are valid [`Id`] instances. The caller must ensure
    /// that the `EcsType` instance (or the underlying `type_vec` data it points to) lives at least as
    /// long as the returned slice to avoid dangling references.
    ///
    /// # See also
    ///
    /// * C++ API: `type::array()`
    #[doc(alias = "type::array()")]
    pub fn as_slice(&self) -> &[Id] {
        self.type_vec
    }

    /// Get [id](IdView) at specified index in type
    ///
    /// # Returns
    ///
    /// Result returned as `IdView`.
    ///
    /// - [`Some(IdView)`] - if the type is not null and the index is within bounds.
    /// - `None` - if the type is null or the index is out of bounds.
    ///
    /// # See also
    ///
    /// * C++ API: `type::get`
    ///
    /// [`Some(IdView)`]: IdView
    #[doc(alias = "type::get")]
    pub fn get(&self, index: usize) -> Option<IdView> {
        if index < self.count() {
            Some(IdView::new_from(self.world, self.type_vec[index]))
        } else {
            None
        }
    }
}
