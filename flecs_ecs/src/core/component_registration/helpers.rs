#![doc(hidden)]

use std::ffi::c_char;

use crate::core::*;

pub(crate) fn create_component_desc(
    entity: EntityT,
    type_info: flecs_ecs_sys::ecs_type_info_t,
) -> flecs_ecs_sys::ecs_component_desc_t {
    flecs_ecs_sys::ecs_component_desc_t {
        _canary: 0,
        entity,
        type_: type_info,
    }
}

pub(crate) fn create_type_info<T>() -> flecs_ecs_sys::ecs_type_info_t
where
    T: ComponentId,
{
    let size = std::mem::size_of::<T>();
    let alignment = if size != 0 {
        std::mem::align_of::<T>()
    } else {
        0
    };
    let mut hooks = Default::default();
    if size != 0 && T::NEEDS_DROP {
        // Register lifecycle callbacks, but only if the component has a
        // size and requires initialization of heap memory / needs drop.
        // Components that don't have a size are tags, and tags don't
        // require construction/destruction/copy/move's.
        T::__register_lifecycle_hooks(&mut hooks);
    }
    if T::IMPLS_DEFAULT {
        T::__register_default_hooks(&mut hooks);
    }
    if T::IMPLS_CLONE {
        T::__register_clone_hooks(&mut hooks);
    }

    let type_info: flecs_ecs_sys::ecs_type_info_t = flecs_ecs_sys::ecs_type_info_t {
        size: size as i32,
        alignment: alignment as i32,
        hooks,
        component: 0,
        name: std::ptr::null(),
    };
    type_info
}

pub(crate) fn external_create_type_info<T>() -> flecs_ecs_sys::ecs_type_info_t {
    let size = std::mem::size_of::<T>();
    let alignment = if size != 0 {
        std::mem::align_of::<T>()
    } else {
        0
    };
    let mut hooks = Default::default();
    if size != 0 && const { std::mem::needs_drop::<T>() } {
        // Register lifecycle callbacks, but only if the component has a
        // size and requires initialization of heap memory / needs drop.
        // Components that don't have a size are tags, and tags don't
        // require construction/destruction/copy/move's.
        flecs_ecs::core::lifecycle_traits::register_lifecycle_actions::<T>(&mut hooks);
    }

    let type_info: flecs_ecs_sys::ecs_type_info_t = flecs_ecs_sys::ecs_type_info_t {
        size: size as i32,
        alignment: alignment as i32,
        hooks,
        component: 0,
        name: std::ptr::null(),
    };
    type_info
}

pub(crate) fn create_entity_desc(
    name: *const c_char,
    symbol: *const c_char,
) -> flecs_ecs_sys::ecs_entity_desc_t {
    let entity_desc: flecs_ecs_sys::ecs_entity_desc_t = flecs_ecs_sys::ecs_entity_desc_t {
        _canary: 0,
        id: 0,
        parent: 0,
        name,
        sep: SEPARATOR.as_ptr(),
        root_sep: std::ptr::null(),
        symbol,
        use_low_id: true,
        add: std::ptr::null(),
        add_expr: std::ptr::null(),
        set: std::ptr::null(),
    };
    entity_desc
}
