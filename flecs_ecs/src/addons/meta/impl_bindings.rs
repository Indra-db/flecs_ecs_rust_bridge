use flecs_ecs::prelude::*;
use flecs_ecs::sys::*;

use flecs_ecs::addons::meta::declarations::*;

#[macro_export]
macro_rules! impl_component_traits_binding_type {
    ($name:ident) => {
        impl NotEmptyComponent for $name {}

        impl ComponentType<flecs_ecs::core::Struct> for $name {}

        impl ComponentInfo for $name {
            const IS_ENUM: bool = false;
            const IS_TAG: bool = false;
            type TagType = FlecsFirstIsNotATag;
            const IMPLS_CLONE: bool = true;
            const IMPLS_DEFAULT: bool = false;
            const IS_REF: bool = false;
            const IS_MUT: bool = false;
        }
        impl ComponentId for $name {
            type UnderlyingType = $name;
            type UnderlyingEnumType = NoneEnum;
            fn __get_once_lock_data() -> &'static std::sync::OnceLock<IdComponent> {
                static ONCE_LOCK: std::sync::OnceLock<IdComponent> = std::sync::OnceLock::new();
                &ONCE_LOCK
            }
            fn __register_lifecycle_hooks(type_hooks: &mut TypeHooksT) {
                register_lifecycle_actions::<$name>(type_hooks);
            }
            fn __register_default_hooks(_type_hooks: &mut TypeHooksT) {}

            fn __register_clone_hooks(type_hooks: &mut TypeHooksT) {
                register_copy_lifecycle_action::<$name>(type_hooks);
            }
        }
    };
}

impl_component_traits_binding_type!(Type);
impl_component_traits_binding_type!(TypeSerializer);
impl_component_traits_binding_type!(Primitive);
impl_component_traits_binding_type!(EcsEnum);
impl_component_traits_binding_type!(Bitmask);
impl_component_traits_binding_type!(Member);
impl_component_traits_binding_type!(MemberRanges);
impl_component_traits_binding_type!(EcsStruct);
impl_component_traits_binding_type!(Array);
impl_component_traits_binding_type!(Vector);
impl_component_traits_binding_type!(Unit);

impl_component_traits_binding_type!(MemberT);
impl_component_traits_binding_type!(EnumConstantT);
impl_component_traits_binding_type!(BitmaskConstantT);
