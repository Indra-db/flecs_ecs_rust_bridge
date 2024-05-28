use std::{ffi::c_void, marker::PhantomData, ops::Deref, ptr::NonNull};

use crate::core::*;
use crate::sys;

pub trait IntoWorld<'a> {
    #[doc(hidden)]
    #[inline(always)]
    fn world_ptr_mut(&self) -> *mut WorldT {
        self.world().raw_world.as_ptr()
    }
    #[doc(hidden)]
    #[inline(always)]
    fn world_ptr(&self) -> *const WorldT {
        self.world().raw_world.as_ptr()
    }

    fn world(&self) -> WorldRef<'a>;
}

impl<'a> IntoWorld<'a> for IdView<'a> {
    #[inline(always)]
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}

impl<'a> IntoWorld<'a> for EntityView<'a> {
    #[inline(always)]
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}

impl<'a> IntoWorld<'a> for &'a World {
    #[inline(always)]
    fn world(&self) -> WorldRef<'a> {
        (*self).into()
    }
}

#[repr(C)]
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct WorldRef<'a> {
    raw_world: NonNull<WorldT>,
    components: NonNull<FlecsIdMap>,
    pub(crate) components_array: NonNull<FlecsArray>,
    _marker: PhantomData<&'a ()>,
}

unsafe impl<'a> Send for WorldRef<'a> {}

impl<'a> WorldRef<'a> {
    #[inline(always)]
    pub fn real_world(&self) -> WorldRef<'a> {
        unsafe {
            WorldRef::from_ptr(
                sys::ecs_get_world(self.world_ptr_mut() as *const c_void) as *mut WorldT
            )
        }
    }

    /// # Safety
    /// Caller must ensure `raw_world` points to a valid `WorldT`
    #[inline(always)]
    pub unsafe fn from_ptr(raw_world: *mut WorldT) -> Self {
        WorldRef {
            raw_world: NonNull::new_unchecked(raw_world),
            components: NonNull::new_unchecked(World::get_components_map_ptr(raw_world)),
            components_array: NonNull::new_unchecked(World::get_components_array_ptr(raw_world)),
            _marker: PhantomData,
        }
    }
}

impl<'a> From<&'a World> for WorldRef<'a> {
    #[inline(always)]
    fn from(world: &'a World) -> Self {
        WorldRef {
            raw_world: world.raw_world,
            components: world.components,
            components_array: world.components_array,
            _marker: PhantomData,
        }
    }
}

impl<'a> Deref for WorldRef<'a> {
    type Target = World;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute::<&WorldRef, &World>(self) }
    }
}

impl<'a> IntoWorld<'a> for WorldRef<'a> {
    #[inline(always)]
    fn world(&self) -> WorldRef<'a> {
        *self
    }
}

impl<'a> IntoWorld<'a> for &WorldRef<'a> {
    #[inline(always)]
    fn world(&self) -> WorldRef<'a> {
        **self
    }
}
