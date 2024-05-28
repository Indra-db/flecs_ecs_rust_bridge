use flecs_ecs::prelude::*;

use flecs_ecs::addons::meta::declarations::*;

use super::Opaque;

flecs_ecs::impl_component_traits_binding_type!(MemberT);
flecs_ecs::impl_component_traits_binding_type!(EnumConstantT);
flecs_ecs::impl_component_traits_binding_type!(BitmaskConstantT);

impl<T: ComponentId> FlecsConstantId for Opaque<'static, T> {
    const ID: u64 = ECS_OPAQUE;
}

impl<T: ComponentId> NotEmptyComponent for Opaque<'static, T> {}

impl<T: ComponentId> ComponentType<flecs_ecs::core::Struct> for Opaque<'static, T> {}

impl<T: ComponentId> ComponentInfo for Opaque<'static, T> {
    const IS_ENUM: bool = false;
    const IS_TAG: bool = false;
    type TagType = FlecsFirstIsNotATag;
    const IMPLS_CLONE: bool = true;
    const IMPLS_DEFAULT: bool = true;
    const IS_REF: bool = false;
    const IS_MUT: bool = false;
}

impl<T: ComponentId> ComponentId for Opaque<'static, T> {
    type UnderlyingType = Opaque<'static, T>;
    type UnderlyingEnumType = NoneEnum;
    fn __get_once_lock_data() -> &'static std::sync::OnceLock<IdComponent> {
        static ONCE_LOCK: std::sync::OnceLock<IdComponent> = std::sync::OnceLock::new();
        &ONCE_LOCK
    }
    fn __register_lifecycle_hooks(type_hooks: &mut TypeHooksT) {
        register_lifecycle_actions::<Opaque<'static, T>>(type_hooks);
    }
    fn __register_default_hooks(_type_hooks: &mut TypeHooksT) {}

    fn __register_clone_hooks(_type_hooks: &mut TypeHooksT) {}

    fn register_explicit<'a>(_world: impl IntoWorld<'a>) {}

    fn register_explicit_named<'a>(_world: impl IntoWorld<'a>, _name: &str) -> EntityT {
        ECS_OPAQUE
    }

    fn is_registered() -> bool {
        true
    }

    fn is_registered_with_world<'a>(_: impl IntoWorld<'a>) -> bool {
        true
    }

    fn get_id<'a>(_world: impl IntoWorld<'a>) -> IdT {
        ECS_OPAQUE
    }

    unsafe fn get_id_unchecked() -> IdT {
        ECS_OPAQUE
    }
}
