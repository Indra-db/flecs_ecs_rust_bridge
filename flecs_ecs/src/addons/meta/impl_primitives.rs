use flecs_ecs::prelude::*;

#[macro_export]
macro_rules! impl_component_traits_primitive_type {
    ($name:ident, $id:ident) => {
        impl FlecsConstantId for $name {
            const ID: u64 = $id;
        }

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

            fn register_explicit<'a>(_world: impl IntoWorld<'a>) {}

            fn register_explicit_named<'a>(_world: impl IntoWorld<'a>, _name: &str) -> EntityT {
                $id
            }

            fn is_registered() -> bool {
                true
            }

            fn is_registered_with_world<'a>(_: impl IntoWorld<'a>) -> bool {
                true
            }

            fn get_id<'a>(_world: impl IntoWorld<'a>) -> IdT {
                $id
            }

            unsafe fn get_id_unchecked() -> IdT {
                $id
            }
        }
    };
}

impl_component_traits_primitive_type!(bool, ECS_BOOL_T);
impl_component_traits_primitive_type!(char, ECS_CHAR_T);
impl_component_traits_primitive_type!(u8, ECS_U8_T);
impl_component_traits_primitive_type!(u16, ECS_U16_T);
impl_component_traits_primitive_type!(u32, ECS_U32_T);
impl_component_traits_primitive_type!(u64, ECS_U64_T);
impl_component_traits_primitive_type!(usize, ECS_UPTR_T);
impl_component_traits_primitive_type!(i8, ECS_I8_T);
impl_component_traits_primitive_type!(i16, ECS_I16_T);
impl_component_traits_primitive_type!(i32, ECS_I32_T);
impl_component_traits_primitive_type!(i64, ECS_I64_T);
impl_component_traits_primitive_type!(isize, ECS_IPTR_T);
impl_component_traits_primitive_type!(f32, ECS_F32_T);
impl_component_traits_primitive_type!(f64, ECS_F64_T);
impl_component_traits_primitive_type!(String, ECS_STRING_T);

impl FlecsConstantId for EntityView<'static> {
    const ID: u64 = ECS_ENTITY_T;
}

impl NotEmptyComponent for EntityView<'static> {}

impl ComponentType<flecs_ecs::core::Struct> for EntityView<'static> {}

impl ComponentInfo for EntityView<'static> {
    const IS_ENUM: bool = false;
    const IS_TAG: bool = false;
    type TagType = FlecsFirstIsNotATag;
    const IMPLS_CLONE: bool = true;
    const IMPLS_DEFAULT: bool = true;
    const IS_REF: bool = false;
    const IS_MUT: bool = false;
}
impl ComponentId for EntityView<'static> {
    type UnderlyingType = EntityView<'static>;
    type UnderlyingEnumType = NoneEnum;
    fn __get_once_lock_data() -> &'static std::sync::OnceLock<IdComponent> {
        static ONCE_LOCK: std::sync::OnceLock<IdComponent> = std::sync::OnceLock::new();
        &ONCE_LOCK
    }
    fn __register_lifecycle_hooks(type_hooks: &mut TypeHooksT) {
        register_lifecycle_actions::<EntityView<'static>>(type_hooks);
    }
    fn __register_default_hooks(_type_hooks: &mut TypeHooksT) {}

    fn __register_clone_hooks(type_hooks: &mut TypeHooksT) {
        register_copy_lifecycle_action::<EntityView<'static>>(type_hooks);
    }

    fn register_explicit<'a>(_world: impl IntoWorld<'a>) {}

    fn register_explicit_named<'a>(_world: impl IntoWorld<'a>, _name: &str) -> EntityT {
        ECS_ENTITY_T
    }

    fn is_registered() -> bool {
        true
    }

    fn is_registered_with_world<'a>(_: impl IntoWorld<'a>) -> bool {
        true
    }

    fn get_id<'a>(_world: impl IntoWorld<'a>) -> IdT {
        ECS_ENTITY_T
    }

    unsafe fn get_id_unchecked() -> IdT {
        ECS_ENTITY_T
    }
}
