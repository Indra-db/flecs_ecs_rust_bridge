use flecs_ecs::prelude::*;

#[macro_export]
macro_rules! impl_component_traits_primitive_type {
    ($name:ident) => {
        impl NotEmptyComponent for $name {}

        impl ComponentType<flecs_ecs::core::Struct> for $name {}

        impl ComponentInfo for $name {
            const IS_ENUM: bool = false;
            const IS_TAG: bool = false;
            type TagType = FlecsFirstIsNotATag;
            const IMPLS_CLONE: bool = true;
            const IMPLS_DEFAULT: bool = true;
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
            fn __register_default_hooks(type_hooks: &mut TypeHooksT) {
                register_ctor_lifecycle_actions::<$name>(type_hooks);
            }
            fn __register_clone_hooks(type_hooks: &mut TypeHooksT) {
                register_copy_lifecycle_action::<$name>(type_hooks);
            }
        }
    };
}

impl_component_traits_primitive_type!(bool);
impl_component_traits_primitive_type!(char);
impl_component_traits_primitive_type!(u8);
impl_component_traits_primitive_type!(u16);
impl_component_traits_primitive_type!(u32);
impl_component_traits_primitive_type!(u64);
impl_component_traits_primitive_type!(usize); //TODO
impl_component_traits_primitive_type!(i8);
impl_component_traits_primitive_type!(i16);
impl_component_traits_primitive_type!(i32);
impl_component_traits_primitive_type!(i64);
impl_component_traits_primitive_type!(isize); //TODO
impl_component_traits_primitive_type!(f32);
impl_component_traits_primitive_type!(f64);
