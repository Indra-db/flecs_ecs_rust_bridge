use flecs_ecs::prelude::*;

#[macro_export]
macro_rules! impl_component_traits_primitive_type {
    ($name:ident, $id:ident) => {
        impl FlecsConstantId for $name {
            const ID: u64 = $id;
        }
        impl DataComponent for $name {}

        impl ComponentType<flecs_ecs::core::Struct> for $name {}

        impl ComponentInfo for $name {
            const IS_GENERIC: bool = false;
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

            #[inline(always)]
            fn index() -> u32 {
                static INDEX: std::sync::atomic::AtomicU32 =
                    std::sync::atomic::AtomicU32::new(u32::MAX);
                Self::get_or_init_index(&INDEX)
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

            fn __register_or_get_id<'a, const MANUAL_REGISTRATION_CHECK: bool>(
                _world: impl IntoWorld<'a>,
            ) -> EntityT {
                $id
            }

            fn __register_or_get_id_named<'a, const MANUAL_REGISTRATION_CHECK: bool>(
                _world: impl IntoWorld<'a>,
                _name: &str,
            ) -> EntityT {
                $id
            }

            fn is_registered_with_world<'a>(_: impl IntoWorld<'a>) -> bool {
                true
            }

            fn id<'a>(_world: impl IntoWorld<'a>) -> IdT {
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
//impl_component_traits_primitive_type!(String, ECS_STRING_T);

impl FlecsConstantId for EntityView<'static> {
    const ID: u64 = ECS_ENTITY_T;
}

impl DataComponent for EntityView<'static> {}

impl ComponentType<flecs_ecs::core::Struct> for EntityView<'static> {}

impl ComponentInfo for EntityView<'static> {
    const IS_ENUM: bool = false;
    const IS_TAG: bool = false;
    type TagType = FlecsFirstIsNotATag;
    const IMPLS_CLONE: bool = true;
    const IMPLS_DEFAULT: bool = true;
    const IS_REF: bool = false;
    const IS_MUT: bool = false;
    const IS_GENERIC: bool = false;
}

impl ComponentId for EntityView<'static> {
    type UnderlyingType = EntityView<'static>;
    type UnderlyingEnumType = NoneEnum;

    fn __register_lifecycle_hooks(type_hooks: &mut TypeHooksT) {
        register_lifecycle_actions::<EntityView<'static>>(type_hooks);
    }
    fn __register_default_hooks(_type_hooks: &mut TypeHooksT) {}

    fn __register_clone_hooks(type_hooks: &mut TypeHooksT) {
        register_copy_lifecycle_action::<EntityView<'static>>(type_hooks);
    }

    #[inline(always)]
    fn index() -> u32 {
        static INDEX: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(u32::MAX);
        Self::get_or_init_index(&INDEX)
    }

    fn __register_or_get_id<'a, const MANUAL_REGISTRATION_CHECK: bool>(
        _world: impl IntoWorld<'a>,
    ) -> EntityT {
        ECS_ENTITY_T
    }

    fn __register_or_get_id_named<'a, const MANUAL_REGISTRATION_CHECK: bool>(
        _world: impl IntoWorld<'a>,
        _name: &str,
    ) -> EntityT {
        ECS_ENTITY_T
    }

    fn is_registered_with_world<'a>(_: impl IntoWorld<'a>) -> bool {
        true
    }

    fn id<'a>(_world: impl IntoWorld<'a>) -> IdT {
        ECS_ENTITY_T
    }
}

// Recursive expansion of Component macro
// =======================================

impl flecs_ecs::core::DataComponent for String {}

impl flecs_ecs::core::ComponentType<flecs_ecs::core::Struct> for String {}

impl flecs_ecs::core::component_registration::registration_traits::ComponentInfo for String {
    const IS_ENUM: bool = false;
    const IS_TAG: bool = false;
    type TagType =
        flecs_ecs::core::component_registration::registration_traits::FlecsFirstIsNotATag;
    const IMPLS_CLONE: bool = { flecs_ecs::core::utility::types::ImplementsClone::<String>::IMPLS };
    const IMPLS_DEFAULT: bool =
        { flecs_ecs::core::utility::types::ImplementsDefault::<String>::IMPLS };
    const IS_REF: bool = false;
    const IS_MUT: bool = false;
    const IS_GENERIC: bool = false;
}
impl flecs_ecs::core::component_registration::registration_traits::ComponentId for String {
    type UnderlyingType = String;
    type UnderlyingEnumType = flecs_ecs::core::component_registration::NoneEnum;

    #[inline(always)]
    fn index() -> u32 {
        static INDEX: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(u32::MAX);
        Self::get_or_init_index(&INDEX)
    }

    fn __register_lifecycle_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
        flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<String>(type_hooks);
    }
    fn __register_default_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
        use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
        const IMPLS_DEFAULT: bool = String::IMPLS_DEFAULT;
        if IMPLS_DEFAULT {
            flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,String>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type, >(type_hooks);
        }
    }
    fn __register_clone_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
        use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
        const IMPLS_CLONE: bool = String::IMPLS_CLONE;
        if IMPLS_CLONE {
            flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,String>as flecs_ecs::core::component_registration::registration_traits::FlecsCloneType> ::Type, >(type_hooks);
        } else {
            flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<String>(
                type_hooks,
            );
        }
    }

    fn __register_or_get_id<'a, const MANUAL_REGISTRATION_CHECK: bool>(
        _world: impl IntoWorld<'a>,
    ) -> EntityT {
        ECS_ENTITY_T
    }

    fn __register_or_get_id_named<'a, const MANUAL_REGISTRATION_CHECK: bool>(
        _world: impl IntoWorld<'a>,
        _name: &str,
    ) -> EntityT {
        ECS_ENTITY_T
    }

    fn is_registered_with_world<'a>(_: impl IntoWorld<'a>) -> bool {
        true
    }

    fn id<'a>(_world: impl IntoWorld<'a>) -> IdT {
        ECS_ENTITY_T
    }
}
