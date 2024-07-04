//! Periodically tracks statistics for the world and systems.
use crate::core::{EntityT, IntoWorld, World};
use crate::sys;

#[cfg(feature = "flecs_module")]
use super::module::Module;

/// Component that stores world statistics
pub type WorldStats = sys::EcsWorldStats;
/// Component that stores system/pipeline statistics
pub type PipelineStats = sys::EcsPipelineStats;
/// Component with world summary stats
pub type WorldSummary = sys::EcsWorldSummary;
/// Component with system stats
pub type SystemStats = sys::EcsSystemStats;

#[derive(Debug, Clone, Copy, Default)]
pub struct Stats;

#[cfg(feature = "flecs_module")]
impl Module for Stats {
    fn module(world: &World) {
        unsafe { sys::FlecsStatsImport(world.ptr_mut()) };
        world.component::<WorldSummary>();
        world.component::<WorldStats>();
        world.component::<SystemStats>();
        world.component::<PipelineStats>();
    }
}

///////////////////////////
/// trait implementations
///////////////////////////

impl flecs_ecs::core::DataComponent for sys::EcsWorldStats {}

impl flecs_ecs::core::ComponentType<flecs_ecs::core::Struct> for sys::EcsWorldStats {}

impl flecs_ecs::core::component_registration::registration_traits::ComponentInfo
    for sys::EcsWorldStats
{
    const IS_GENERIC: bool = false;
    const IS_ENUM: bool = false;
    const IS_TAG: bool = false;
    type TagType =
        flecs_ecs::core::component_registration::registration_traits::FlecsFirstIsNotATag;
    const IMPLS_CLONE: bool = true;
    const IMPLS_DEFAULT: bool = false;
    const IS_REF: bool = false;
    const IS_MUT: bool = false;
}
impl flecs_ecs::core::component_registration::registration_traits::ComponentId
    for sys::EcsWorldStats
where
    Self: 'static,
{
    type UnderlyingType = sys::EcsWorldStats;
    type UnderlyingEnumType = flecs_ecs::core::component_registration::NoneEnum;
    #[inline(always)]
    fn index() -> u32 {
        static INDEX: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(u32::MAX);
        Self::get_or_init_index(&INDEX)
    }
    fn __register_lifecycle_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
        flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<sys::EcsWorldStats>(
            type_hooks,
        );
    }
    fn __register_default_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
        use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
        const IMPLS_DEFAULT: bool = sys::EcsWorldStats::IMPLS_DEFAULT;
        if IMPLS_DEFAULT {
            flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,sys::EcsWorldStats>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type, >(type_hooks);
        }
    }
    fn __register_clone_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
        use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
        const IMPLS_CLONE: bool = sys::EcsWorldStats::IMPLS_CLONE;
        if IMPLS_CLONE {
            flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,sys::EcsWorldStats>as flecs_ecs::core::component_registration::registration_traits::FlecsCloneType> ::Type, >(type_hooks);
        } else {
            flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<
                sys::EcsWorldStats,
            >(type_hooks);
        }
    }
}

///////////////////////////////////////////////
///////////////////////////////////////////////

impl flecs_ecs::core::DataComponent for sys::EcsPipelineStats {}

impl flecs_ecs::core::ComponentType<flecs_ecs::core::Struct> for sys::EcsPipelineStats {}

impl flecs_ecs::core::component_registration::registration_traits::ComponentInfo
    for sys::EcsPipelineStats
{
    const IS_GENERIC: bool = false;
    const IS_ENUM: bool = false;
    const IS_TAG: bool = false;
    type TagType =
        flecs_ecs::core::component_registration::registration_traits::FlecsFirstIsNotATag;
    const IMPLS_CLONE: bool = true;
    const IMPLS_DEFAULT: bool = false;
    const IS_REF: bool = false;
    const IS_MUT: bool = false;
}
impl flecs_ecs::core::component_registration::registration_traits::ComponentId
    for sys::EcsPipelineStats
where
    Self: 'static,
{
    type UnderlyingType = sys::EcsPipelineStats;
    type UnderlyingEnumType = flecs_ecs::core::component_registration::NoneEnum;
    #[inline(always)]
    fn index() -> u32 {
        static INDEX: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(u32::MAX);
        Self::get_or_init_index(&INDEX)
    }
    fn __register_lifecycle_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
        flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<sys::EcsPipelineStats>(
            type_hooks,
        );
    }
    fn __register_default_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
        use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
        const IMPLS_DEFAULT: bool = sys::EcsPipelineStats::IMPLS_DEFAULT;
        if IMPLS_DEFAULT {
            flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,sys::EcsPipelineStats>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type, >(type_hooks);
        }
    }
    fn __register_clone_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
        use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
        const IMPLS_CLONE: bool = sys::EcsPipelineStats::IMPLS_CLONE;
        if IMPLS_CLONE {
            flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,sys::EcsPipelineStats>as flecs_ecs::core::component_registration::registration_traits::FlecsCloneType> ::Type, >(type_hooks);
        } else {
            flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<
                sys::EcsPipelineStats,
            >(type_hooks);
        }
    }
}

///////////////////////////////////////////////
///////////////////////////////////////////////

impl flecs_ecs::core::DataComponent for sys::EcsWorldSummary {}

impl flecs_ecs::core::ComponentType<flecs_ecs::core::Struct> for sys::EcsWorldSummary {}

impl flecs_ecs::core::component_registration::registration_traits::ComponentInfo
    for sys::EcsWorldSummary
{
    const IS_GENERIC: bool = false;
    const IS_ENUM: bool = false;
    const IS_TAG: bool = false;
    type TagType =
        flecs_ecs::core::component_registration::registration_traits::FlecsFirstIsNotATag;
    const IMPLS_CLONE: bool = true;
    const IMPLS_DEFAULT: bool = false;
    const IS_REF: bool = false;
    const IS_MUT: bool = false;
}
impl flecs_ecs::core::component_registration::registration_traits::ComponentId
    for sys::EcsWorldSummary
where
    Self: 'static,
{
    type UnderlyingType = sys::EcsWorldSummary;
    type UnderlyingEnumType = flecs_ecs::core::component_registration::NoneEnum;
    #[inline(always)]
    fn index() -> u32 {
        static INDEX: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(u32::MAX);
        Self::get_or_init_index(&INDEX)
    }
    fn __register_lifecycle_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
        flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<sys::EcsWorldSummary>(
            type_hooks,
        );
    }
    fn __register_default_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
        use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
        const IMPLS_DEFAULT: bool = sys::EcsWorldSummary::IMPLS_DEFAULT;
        if IMPLS_DEFAULT {
            flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,sys::EcsWorldSummary>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type, >(type_hooks);
        }
    }
    fn __register_clone_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
        use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
        const IMPLS_CLONE: bool = sys::EcsWorldSummary::IMPLS_CLONE;
        if IMPLS_CLONE {
            flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,sys::EcsWorldSummary>as flecs_ecs::core::component_registration::registration_traits::FlecsCloneType> ::Type, >(type_hooks);
        } else {
            flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<
                sys::EcsWorldSummary,
            >(type_hooks);
        }
    }
}

///////////////////////////////////////////////
///////////////////////////////////////////////

impl flecs_ecs::core::DataComponent for sys::EcsSystemStats {}

impl flecs_ecs::core::ComponentType<flecs_ecs::core::Struct> for sys::EcsSystemStats {}

impl flecs_ecs::core::component_registration::registration_traits::ComponentInfo
    for sys::EcsSystemStats
{
    const IS_GENERIC: bool = false;
    const IS_ENUM: bool = false;
    const IS_TAG: bool = false;
    type TagType =
        flecs_ecs::core::component_registration::registration_traits::FlecsFirstIsNotATag;
    const IMPLS_CLONE: bool = true;
    const IMPLS_DEFAULT: bool = false;
    const IS_REF: bool = false;
    const IS_MUT: bool = false;
}
impl flecs_ecs::core::component_registration::registration_traits::ComponentId
    for sys::EcsSystemStats
where
    Self: 'static,
{
    type UnderlyingType = sys::EcsSystemStats;
    type UnderlyingEnumType = flecs_ecs::core::component_registration::NoneEnum;
    #[inline(always)]
    fn index() -> u32 {
        static INDEX: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(u32::MAX);
        Self::get_or_init_index(&INDEX)
    }
    fn __register_lifecycle_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
        flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<sys::EcsSystemStats>(
            type_hooks,
        );
    }
    fn __register_default_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
        use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
        const IMPLS_DEFAULT: bool = sys::EcsSystemStats::IMPLS_DEFAULT;
        if IMPLS_DEFAULT {
            flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,sys::EcsSystemStats>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type, >(type_hooks);
        }
    }
    fn __register_clone_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
        use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
        const IMPLS_CLONE: bool = sys::EcsSystemStats::IMPLS_CLONE;
        if IMPLS_CLONE {
            flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,sys::EcsSystemStats>as flecs_ecs::core::component_registration::registration_traits::FlecsCloneType> ::Type, >(type_hooks);
        } else {
            flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<
                sys::EcsSystemStats,
            >(type_hooks);
        }
    }
}

///////////////////////////////////////////////
///////////////////////////////////////////////

impl flecs_ecs::core::DataComponent for Stats {}

impl flecs_ecs::core::ComponentType<flecs_ecs::core::Struct> for Stats {}

impl flecs_ecs::core::component_registration::registration_traits::ComponentInfo for Stats {
    const IS_GENERIC: bool = false;
    const IS_ENUM: bool = false;
    const IS_TAG: bool = false;
    type TagType =
        flecs_ecs::core::component_registration::registration_traits::FlecsFirstIsNotATag;
    const IMPLS_CLONE: bool = true;
    const IMPLS_DEFAULT: bool = true;
    const IS_REF: bool = false;
    const IS_MUT: bool = false;
}
impl flecs_ecs::core::component_registration::registration_traits::ComponentId for Stats
where
    Self: 'static,
{
    type UnderlyingType = Stats;
    type UnderlyingEnumType = flecs_ecs::core::component_registration::NoneEnum;
    #[inline(always)]
    fn index() -> u32 {
        static INDEX: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(u32::MAX);
        Self::get_or_init_index(&INDEX)
    }
    fn __register_lifecycle_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
        flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<Stats>(type_hooks);
    }
    fn __register_default_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
        use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
        const IMPLS_DEFAULT: bool = Stats::IMPLS_DEFAULT;
        if IMPLS_DEFAULT {
            flecs_ecs::core::lifecycle_traits::register_ctor_lifecycle_actions:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_DEFAULT,Stats>as flecs_ecs::core::component_registration::registration_traits::FlecsDefaultType> ::Type, >(type_hooks);
        }
    }
    fn __register_clone_hooks(type_hooks: &mut flecs_ecs::core::TypeHooksT) {
        use flecs_ecs::core::component_registration::registration_traits::ComponentInfo;
        const IMPLS_CLONE: bool = Stats::IMPLS_CLONE;
        if IMPLS_CLONE {
            flecs_ecs::core::lifecycle_traits::register_copy_lifecycle_action:: <<flecs_ecs::core::component_registration::registration_types::ConditionalTypeSelector<IMPLS_CLONE,Stats>as flecs_ecs::core::component_registration::registration_traits::FlecsCloneType> ::Type, >(type_hooks);
        } else {
            flecs_ecs::core::lifecycle_traits::register_copy_panic_lifecycle_action::<Stats>(
                type_hooks,
            );
        }
    }

    fn __register_or_get_id<'a, const MANUAL_REGISTRATION_CHECK: bool>(
        world: impl IntoWorld<'a>,
    ) -> EntityT {
        Self::__register_or_get_id_named::<MANUAL_REGISTRATION_CHECK>(world, "flecs::stats")
    }
}
