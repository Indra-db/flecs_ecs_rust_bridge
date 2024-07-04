//! Provides functionality for working with various IDs in an ECS framework, including entity IDs, component IDs, tag IDs, and pair IDs.

use crate::core::*;
use crate::sys;

/// Class for working with entity, component, tag and pair ids.
/// This wraps an [`Id`].
///
/// A flecs id is an identifier that can be added to entities. Ids can be:
///
/// * entities (including components, tags)
/// * pair ids
/// * entities with id flags set (like `flecs::AutoOverride`, `flecs::Toggle`)
///
/// # See also
///
/// * [flecs C++ documentation](https://www.flecs.dev/flecs/structflecs_1_1id.html#details)
/// * [flecs C documentation](https://www.flecs.dev/flecs/group__ids.html)
#[derive(Debug, Clone, Copy, Eq)]
pub struct IdView<'a> {
    pub(crate) world: WorldRef<'a>,
    pub(crate) id: Id,
}

impl<'a> PartialEq<IdView<'a>> for u64 {
    fn eq(&self, other: &IdView<'a>) -> bool {
        *self == other.id.0
    }
}

impl<'a> PartialEq<u64> for IdView<'a> {
    fn eq(&self, other: &u64) -> bool {
        self.id == *other
    }
}

impl<'a> PartialEq<Id> for IdView<'a> {
    fn eq(&self, other: &Id) -> bool {
        self.id == *other
    }
}

impl<'a> PartialEq<Entity> for IdView<'a> {
    fn eq(&self, other: &Entity) -> bool {
        self.id == *other
    }
}

impl<'a> PartialEq<EntityView<'a>> for IdView<'a> {
    fn eq(&self, other: &EntityView<'a>) -> bool {
        self.id == other.id
    }
}

impl<'a> PartialEq for IdView<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<'a, T> PartialEq<Component<'a, T>> for IdView<'a>
where
    T: ComponentId,
{
    fn eq(&self, other: &Component<'a, T>) -> bool {
        self.id == other.base.entity.id
    }
}

impl<'a> PartialEq<UntypedComponent<'a>> for IdView<'a> {
    fn eq(&self, other: &UntypedComponent<'a>) -> bool {
        self.id == other.entity.id
    }
}

impl<'a> PartialOrd<IdView<'a>> for u64 {
    fn partial_cmp(&self, other: &IdView<'a>) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other.id.0))
    }
}

impl<'a> PartialOrd<u64> for IdView<'a> {
    fn partial_cmp(&self, other: &u64) -> Option<std::cmp::Ordering> {
        Some(self.id.0.cmp(other))
    }
}

impl<'a> PartialOrd<Entity> for IdView<'a> {
    fn partial_cmp(&self, other: &Entity) -> Option<std::cmp::Ordering> {
        Some(self.id.0.cmp(&other.0))
    }
}

impl<'a> PartialOrd<Id> for IdView<'a> {
    fn partial_cmp(&self, other: &Id) -> Option<std::cmp::Ordering> {
        Some(self.id.0.cmp(other))
    }
}

impl<'a> PartialOrd<EntityView<'a>> for IdView<'a> {
    fn partial_cmp(&self, other: &EntityView<'a>) -> Option<std::cmp::Ordering> {
        Some(self.id.0.cmp(&other.id.0))
    }
}

impl<'a> PartialOrd for IdView<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl<'a, T> PartialOrd<Component<'a, T>> for IdView<'a>
where
    T: ComponentId,
{
    fn partial_cmp(&self, other: &Component<'a, T>) -> Option<std::cmp::Ordering> {
        Some(self.id.0.cmp(&other.base.entity.id.0))
    }
}

impl<'a> PartialOrd<UntypedComponent<'a>> for IdView<'a> {
    fn partial_cmp(&self, other: &UntypedComponent<'a>) -> Option<std::cmp::Ordering> {
        Some(self.id.0.cmp(&other.entity.id.0))
    }
}

impl<'a> Ord for IdView<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl<'a> std::ops::Deref for IdView<'a> {
    type Target = u64;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl<'a> IdView<'a> {
    /// checks if the id is a pair
    ///
    /// # See also
    ///
    /// * C++ API: `id::is_pair`
    #[doc(alias = "id::is_pair")]
    /// * C API: `ecs_id_is_pair`
    #[doc(alias = "ecs_id_is_pair")]
    pub fn is_pair(self) -> bool {
        unsafe { sys::ecs_id_is_pair(*self.id) }
    }

    /// checks if the id is a entity
    ///
    /// # See also
    ///
    /// * C++ API: `id::is_entity`
    #[doc(alias = "id::is_entity")]
    pub fn is_entity(self) -> bool {
        self.id & RUST_ecs_id_FLAGS_MASK == 0
    }

    /// checks if entity is valid
    ///
    /// # See also
    ///
    /// * C++ API: `entity_view::is_valid`
    #[doc(alias = "entity_view::is_valid")]
    pub fn is_valid(self) -> bool {
        unsafe { sys::ecs_is_valid(self.world.world_ptr_mut(), *self.id) }
    }

    /// Test if id has specified first
    ///
    /// # Arguments
    ///
    /// * `first` - The first id to test
    ///
    /// # See also
    ///
    /// * C++ API: `id::has_relationship`
    #[doc(alias = "id::has_relationship")]
    #[inline(always)]
    pub fn has_relationship(self, first: impl Into<Entity>) -> bool {
        if !self.is_pair() {
            return false;
        }

        ecs_first(self.id) == first.into()
    }

    /// Get first element from a pair.
    ///
    /// If the id is not a pair, this operation will fail. When the id has a
    /// world, the operation will ensure that the returned id has the correct generation count.
    ///
    /// # See also
    ///
    /// * C++ API: `id::first`
    #[doc(alias = "id::first")]
    #[inline(always)]
    pub fn first_id(&self) -> EntityView {
        ecs_assert!(self.is_pair(), FlecsErrorCode::InvalidOperation);

        let entity = ecs_first(self.id);
        self.world.get_alive(entity)
    }

    /// Get first element from a pair.
    ///
    /// If the id is not a pair, this operation will fail. When the id has a
    /// world, the operation will ensure that the returned id has the correct generation count.
    ///
    /// # See also
    ///
    /// * C++ API: `id::first`
    #[doc(alias = "id::first")]
    #[inline(always)]
    pub fn get_first_id(&self) -> Option<EntityView> {
        if !self.is_pair() {
            None
        } else {
            let entity = ecs_first(self.id);
            self.world.try_get_alive(entity)
        }
    }

    /// Get second element from a pair.
    ///
    /// If the id is not a pair, this operation will fail. When the id has a
    /// world, the operation will ensure that the returned id has the correct generation count.
    ///
    /// # See also
    ///
    /// * C++ API: `id::second`
    #[doc(alias = "id::second")]
    pub fn second_id(&self) -> EntityView {
        ecs_assert!(self.is_pair(), FlecsErrorCode::InvalidOperation);

        let entity = ecs_second(self.id);
        self.world.get_alive(entity)
    }

    /// Get second element from a pair.
    ///
    /// If the id is not a pair, this operation will fail. When the id has a
    /// world, the operation will ensure that the returned id has the correct generation count.
    ///
    /// # See also
    ///
    /// * C++ API: `id::second`
    #[doc(alias = "id::second")]
    pub fn get_second_id(&self) -> Option<EntityView> {
        if !self.is_pair() {
            None
        } else {
            let entity = ecs_second(self.id);
            self.world.try_get_alive(entity)
        }
    }

    /// Return id as entity (only allowed when id is valid entity)
    ///
    /// # See also
    ///
    /// * C++ API: `id::entity`
    #[doc(alias = "id::entity")]
    #[inline(always)]
    pub fn entity_view(self) -> EntityView<'a> {
        ecs_assert!(!self.is_pair(), FlecsErrorCode::InvalidOperation);
        ecs_assert!(self.flags().id == 0, FlecsErrorCode::InvalidOperation);

        EntityView::new_from(self.world, Entity(*self.id))
    }

    /// Return id as entity (only allowed when id is valid entity)
    ///
    /// # See also
    ///
    /// * C++ API: `id::entity`
    #[doc(alias = "id::entity")]
    #[inline(always)]
    pub fn get_entity_view(self) -> Option<EntityView<'a>> {
        if self.is_pair() || self.flags().id != 0 {
            None
        } else {
            Some(EntityView::new_from(self.world, Entity(*self.id)))
        }
    }

    /// Get the component type for the id.
    ///
    /// This operation returns the component id for an id,
    /// if the id is associated with a type. For a regular component with a non-zero size
    /// (an entity with the `EcsComponent` component) the operation will return the entity itself.
    /// For an entity that does not have the `EcsComponent` component, or with an `EcsComponent`
    /// value with size 0, the operation will return an Entity wrapping 0
    ///
    /// For a pair id the operation will return the type associated with the pair, by applying the following rules in order:
    ///
    /// * The first pair element is returned if it is a component
    /// * Entity wrapping 0 is returned if the relationship entity has the Tag property
    /// * The second pair element is returned if it is a component
    /// * Entity wrapping 0 is returned
    ///
    /// # Returns
    ///
    /// The type id of the id
    ///
    /// # See also
    ///
    /// * C++ API: `id::type_id`
    #[doc(alias = "id::type_id")]
    /// * C API: `ecs_get_typeid`
    #[doc(alias = "ecs_get_typeid")]
    #[inline(always)]
    pub fn get_type_id(self) -> Option<EntityView<'a>> {
        let type_id = unsafe { sys::ecs_get_typeid(self.world.world_ptr(), *self.id) };
        if type_id == 0 {
            None
        } else {
            Some(EntityView::new_from(self.world, Entity(type_id)))
        }
    }

    /// Get the component type for the id.
    ///
    /// This operation returns the component id for an id,
    /// if the id is associated with a type. For a regular component with a non-zero size
    /// (an entity with the `EcsComponent` component) the operation will return the entity itself.
    /// For an entity that does not have the `EcsComponent` component, or with an `EcsComponent`
    /// value with size 0, the operation will return an Entity wrapping 0
    ///
    /// For a pair id the operation will return the type associated with the pair, by applying the following rules in order:
    ///
    /// * The first pair element is returned if it is a component
    /// * Entity wrapping 0 is returned if the relationship entity has the Tag property
    /// * The second pair element is returned if it is a component
    /// * Entity wrapping 0 is returned
    ///
    /// # Returns
    ///
    /// The type id of the id
    ///
    /// # See also
    ///
    /// * C++ API: `id::type_id`
    #[doc(alias = "id::type_id")]
    /// * C API: `ecs_get_typeid`
    #[doc(alias = "ecs_get_typeid")]
    #[inline(always)]
    pub fn type_id(self) -> EntityView<'a> {
        let type_id = unsafe { sys::ecs_get_typeid(self.world.world_ptr(), *self.id) };
        EntityView::new_from(self.world, Entity(type_id))
    }
}

impl<'a> IdOperations<'a> for IdView<'a> {
    type IdType = Id;

    fn id(&self) -> Self::IdType {
        self.id
    }

    /// Wraps an id or pair
    ///
    /// # Arguments
    ///
    /// * `world` - The optional world to the id belongs to
    /// * `with` - The id or pair to wrap
    ///
    /// # See also
    ///
    /// * C++ API: `Id::Id`
    #[doc(alias = "Id::Id")]
    /// * C API: `ecs_id_t`
    #[doc(alias = "ecs_id_t")]
    fn new_from(world: impl IntoWorld<'a>, id: impl IntoId) -> Self {
        Self {
            world: world.world(),
            id: id.into(),
        }
    }
}
