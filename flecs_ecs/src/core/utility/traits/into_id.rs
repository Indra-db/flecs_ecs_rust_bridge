use crate::core::*;

/// Extracts the Ecs ID from a type.
/// Extension trait from [`Into<Entity>`] for tuples that implement `Into<Entity>`.
/// These types can be [`Id`], [`IdView`], [`Entity`], [`EntityView`], [`Component`], [`UntypedComponent`].
pub trait IntoId: Into<Id> {
    /// # Safety
    /// This is used to determine if the type is a pair or not.
    /// not if the underlying id represents a pair.
    /// This should generally not be used by the user.
    const IS_PAIR: bool;

    /// This will return the id of the first part of a pair.
    /// If this is called on a non_pair, it will return the same as get_id.
    #[doc(hidden)] // not meant to be used by the user
    #[inline]
    fn get_id_first(&self) -> Entity {
        Entity(0)
    }

    /// This will return the id of the second part of a pair.
    /// If this is called on a non_pair, it will return the same as get_id.
    #[doc(hidden)]
    #[inline]
    fn get_id_second(&self) -> Entity {
        Entity::new(0)
    }
}

impl<T, U> From<(T, U)> for Id
where
    T: Into<Entity>,
    U: Into<Entity>,
{
    fn from(pair: (T, U)) -> Self {
        ecs_pair(*pair.0.into(), *pair.1.into()).into()
    }
}

impl<T, U> IntoId for (T, U)
where
    T: Into<Entity> + Copy,
    U: Into<Entity> + Copy,
{
    const IS_PAIR: bool = true;

    #[doc(hidden)] // not meant to be used by the user
    #[inline]
    fn get_id_first(&self) -> Entity {
        self.0.into()
    }

    #[doc(hidden)] // not meant to be used by the user
    #[inline]
    fn get_id_second(&self) -> Entity {
        self.1.into()
    }
}

// // We can not implement for T where T : `Into<Entity>`, because it would essentially extend the trait, which we don't want
// // so we have to implement for each type that implements `Into<Entity>` separately.

impl IntoId for IdT {
    const IS_PAIR: bool = false;
}

impl IntoId for Entity {
    const IS_PAIR: bool = false;
}

impl IntoId for Id {
    const IS_PAIR: bool = false;
}

impl<'a> IntoId for IdView<'a> {
    const IS_PAIR: bool = false;
}

impl<'a> IntoId for EntityView<'a> {
    const IS_PAIR: bool = false;
}

impl<'a, T> IntoId for Component<'a, T>
where
    T: ComponentId,
{
    const IS_PAIR: bool = false;
}

impl<'a> IntoId for UntypedComponent<'a> {
    const IS_PAIR: bool = false;
}
