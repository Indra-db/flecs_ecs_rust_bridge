use std::ops::Deref;

use crate::core::*;

/// Untyped component class.
#[derive(Clone, Copy, Debug)]
pub struct UntypedComponent<'a> {
    pub entity: EntityView<'a>,
}

impl<'a> Deref for UntypedComponent<'a> {
    type Target = EntityView<'a>;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl<'a> UntypedComponent<'a> {
    /// Create a new untyped component.
    ///
    /// # Arguments
    ///
    /// * `world`: the world.
    /// * `id`: the id of the component to reference.
    ///
    /// # See also
    ///
    /// * C++ API: `untyped_component::untyped_component`
    #[doc(alias = "untyped_component::untyped_component")]
    pub fn new(world: impl IntoWorld<'a>, id: impl Into<Entity>) -> Self {
        UntypedComponent {
            entity: EntityView::new_from(world, id),
        }
    }

    /// Get the id of the component.
    ///
    /// # See also
    ///
    /// * C++ API: `untyped_component::entity`
    #[doc(alias = "untyped_component::entity")]
    pub fn as_entity(&self) -> EntityView<'a> {
        self.entity
    }
}

#[cfg(feature = "flecs_meta")]
impl<'a> UntypedComponent<'a> {}

#[cfg(feature = "flecs_metrics")]
impl<'a> UntypedComponent<'a> {}

mod eq_operations {
    use super::*;

    impl<'a> PartialEq<UntypedComponent<'a>> for u64 {
        #[inline]
        fn eq(&self, other: &UntypedComponent<'a>) -> bool {
            *self == other.entity.id
        }
    }

    impl<'a> PartialEq<u64> for UntypedComponent<'a> {
        #[inline]
        fn eq(&self, other: &u64) -> bool {
            self.entity.id == *other
        }
    }

    impl<'a> PartialEq<Entity> for UntypedComponent<'a> {
        #[inline]
        fn eq(&self, other: &Entity) -> bool {
            self.entity.id == *other
        }
    }

    impl<'a> PartialEq<Id> for UntypedComponent<'a> {
        #[inline]
        fn eq(&self, other: &Id) -> bool {
            self.entity.id == *other
        }
    }

    impl<'a> PartialEq<EntityView<'a>> for UntypedComponent<'a> {
        #[inline]
        fn eq(&self, other: &EntityView<'a>) -> bool {
            self.entity == *other
        }
    }

    impl<'a> PartialEq<IdView<'a>> for UntypedComponent<'a> {
        #[inline]
        fn eq(&self, other: &IdView<'a>) -> bool {
            self.entity == other.id
        }
    }

    impl<'a, T> PartialEq<Component<'a, T>> for UntypedComponent<'a>
    where
        T: ComponentId,
    {
        #[inline]
        fn eq(&self, other: &Component<'a, T>) -> bool {
            self.entity == other.base.entity
        }
    }

    impl<'a> PartialEq for UntypedComponent<'a> {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.entity == other.entity
        }
    }

    impl<'a> Eq for UntypedComponent<'a> {}
}

mod ord_operations {
    use super::*;

    impl<'a> PartialOrd<UntypedComponent<'a>> for u64 {
        #[inline]
        fn partial_cmp(&self, other: &UntypedComponent<'a>) -> Option<std::cmp::Ordering> {
            self.partial_cmp(&other.entity.id)
        }
    }

    impl<'a> PartialOrd<u64> for UntypedComponent<'a> {
        #[inline]
        fn partial_cmp(&self, other: &u64) -> Option<std::cmp::Ordering> {
            self.entity.id.partial_cmp(other)
        }
    }

    impl<'a> PartialOrd<Entity> for UntypedComponent<'a> {
        #[inline]
        fn partial_cmp(&self, other: &Entity) -> Option<std::cmp::Ordering> {
            self.entity.id.partial_cmp(other)
        }
    }

    impl<'a> PartialOrd<Id> for UntypedComponent<'a> {
        #[inline]
        fn partial_cmp(&self, other: &Id) -> Option<std::cmp::Ordering> {
            self.entity.id.partial_cmp(other)
        }
    }

    impl<'a> PartialOrd<EntityView<'a>> for UntypedComponent<'a> {
        #[inline]
        fn partial_cmp(&self, other: &EntityView<'a>) -> Option<std::cmp::Ordering> {
            self.entity.partial_cmp(other)
        }
    }

    impl<'a> PartialOrd<IdView<'a>> for UntypedComponent<'a> {
        #[inline]
        fn partial_cmp(&self, other: &IdView<'a>) -> Option<std::cmp::Ordering> {
            self.entity.partial_cmp(&other.id)
        }
    }

    impl<'a> PartialOrd for UntypedComponent<'a> {
        #[inline]
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.entity.cmp(&other.entity))
        }
    }

    impl<'a> Ord for UntypedComponent<'a> {
        #[inline]
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.entity.cmp(&other.entity)
        }
    }
}
