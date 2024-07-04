//! Observers are systems that react to events. Observers let applications register callbacks for ECS events.
use std::ops::DerefMut;
use std::ptr::NonNull;
use std::{ops::Deref, os::raw::c_void};

use crate::core::*;
use crate::sys;

/// Observers are systems that react to events.
/// Observers let applications register callbacks for ECS events.
///
/// These are typically constructed via [`World::observer()`].
#[derive(Clone, Copy)]
pub struct Observer<'a> {
    entity: EntityView<'a>,
}

impl<'a> Deref for Observer<'a> {
    type Target = EntityView<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl<'a> DerefMut for Observer<'a> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entity
    }
}

impl<'a> IntoWorld<'a> for Observer<'a> {
    #[inline(always)]
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}

impl<'a> Observer<'a> {
    /// Create a new observer
    ///
    /// # See also
    ///
    /// * C++ API: `observer::observer`
    #[doc(alias = "observer::observer")]
    pub fn new(
        world: impl IntoWorld<'a>,
        mut desc: sys::ecs_observer_desc_t,
        is_instanced: bool,
    ) -> Self {
        if desc.query.flags & sys::EcsQueryIsInstanced == 0 {
            ecs_bit_cond(
                &mut desc.query.flags,
                sys::EcsQueryIsInstanced,
                is_instanced,
            );
        }

        let id = unsafe { sys::ecs_observer_init(world.world_ptr_mut(), &desc) };
        let entity = EntityView::new_from(world.world(), id);

        Self { entity }
    }

    /// Wrap an existing observer entity in an observer object
    pub fn new_from_existing(observer_entity: EntityView<'a>) -> Self {
        Self {
            entity: observer_entity,
        }
    }

    /// Set the context for the observer
    ///
    /// # See also
    ///
    /// * C++ API: `observer::ctx`
    #[doc(alias = "observer::ctx")]
    pub fn set_context(&mut self, context: *mut c_void) {
        let desc: sys::ecs_observer_desc_t = sys::ecs_observer_desc_t {
            entity: *self.id,
            ctx: context,
            ..Default::default()
        };

        unsafe {
            sys::ecs_observer_init(self.world.world_ptr_mut(), &desc);
        }
    }

    /// Get the context for the observer
    ///
    /// # See also
    ///
    /// * C++ API: `observer::ctx`
    #[doc(alias = "observer::ctx")]
    pub fn context(&self) -> *mut c_void {
        unsafe { (*sys::ecs_observer_get(self.world.world_ptr_mut(), *self.id)).ctx }
    }

    /// Get the query for the observer
    ///
    /// # See also
    ///
    /// * C++ API: `observer::query`
    #[doc(alias = "observer::query")]
    pub fn query(&mut self) -> Query<()> {
        unsafe {
            Query::<()>::new_from(NonNull::new_unchecked(
                (*sys::ecs_observer_get(self.world_ptr(), *self.entity.id())).query,
            ))
        }
    }

    /// Get the observer's entity
    ///
    /// # See also
    ///
    /// * C++ API: `observer::entity`
    #[doc(alias = "observer::entity")]
    pub fn entity(&self) -> EntityView<'a> {
        self.entity
    }
}
