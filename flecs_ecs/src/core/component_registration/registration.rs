#![doc(hidden)]
use std::ffi::c_char;

use crate::core::*;
use crate::sys;

#[doc(hidden)]
pub fn internal_register_component<'a, const IS_NAMED: bool, T>(
    world: impl IntoWorld<'a>,
    name: *const i8,
) -> u64
where
    T: ComponentId,
{
    let world_ptr = world.world_ptr_mut();

    let id = if IS_NAMED {
        register_component_data_named::<T>(world_ptr, name)
    } else {
        register_component_data::<T>(world_ptr)
    };

    if T::IS_ENUM {
        register_enum_data::<T>(world_ptr, id);
    }
    id
}

#[doc(hidden)]
pub(crate) fn external_register_component<'a, T>(
    world: impl IntoWorld<'a>,
    name: *const i8,
) -> u64 {
    external_register_component_data::<T>(world.world_ptr_mut(), name)
}

#[inline(always)]
/// attempts to register the component with the world. If it's already registered, it does nothing.
pub(crate) fn try_register_component<'a, T>(world: impl IntoWorld<'a>) -> EntityT
where
    T: ComponentId,
{
    internal_register_component::<false, T>(world, std::ptr::null())
}

#[inline(always)]
pub(crate) fn try_register_component_named<'a, T>(world: impl IntoWorld<'a>, name: &str) -> EntityT
where
    T: ComponentId,
{
    let name = compact_str::format_compact!("{}\0", name);

    internal_register_component::<true, T>(world, name.as_ptr() as *const c_char)
}

/// registers enum fields with the world.
pub(crate) fn register_enum_data<T>(world: *mut WorldT, id: EntityT)
where
    T: ComponentId,
{
    //TODO we should convert this ecs_cpp functions to rust so if it ever changes, our solution won't break
    unsafe { sys::ecs_cpp_enum_init(world, id) };
    let enum_array_ptr = T::UnderlyingEnumType::__enum_data_mut();

    for (index, enum_item) in T::UnderlyingEnumType::iter().enumerate() {
        let name = enum_item.name_cstr();
        let entity_id: EntityT = unsafe {
            sys::ecs_cpp_enum_constant_register(
                world,
                id,
                T::UnderlyingEnumType::id_variant_of_index_unchecked(enum_item.enum_index()),
                name.as_ptr(),
                index as i32,
            )
        };
        if !T::UnderlyingEnumType::is_index_registered_as_entity(index) {
            unsafe { *enum_array_ptr.add(index) = entity_id };
        }
    }
}

/// registers the component with the world.
pub(crate) fn register_component_data_named<T>(world: *mut WorldT, name: *const c_char) -> EntityT
where
    T: ComponentId,
{
    let prev_scope = if !name.is_null() && unsafe { sys::ecs_get_scope(world) != 0 } {
        unsafe { sys::ecs_set_scope(world, 0) }
    } else {
        0
    };
    let prev_with = unsafe { sys::ecs_set_with(world, 0) };

    let id = register_componment_data_explicit::<T>(world, name);

    if prev_with != 0 {
        unsafe { sys::ecs_set_with(world, prev_with) };
    }
    if prev_scope != 0 {
        unsafe { sys::ecs_set_scope(world, prev_scope) };
    }
    id
}

/// registers the component with the world.
pub(crate) fn register_component_data<T>(world: *mut WorldT) -> EntityT
where
    T: ComponentId,
{
    let prev_scope = unsafe { sys::ecs_set_scope(world, 0) };
    let prev_with = unsafe { sys::ecs_set_with(world, 0) };

    let id = register_componment_data_explicit::<T>(world, std::ptr::null());

    if prev_with != 0 {
        unsafe { sys::ecs_set_with(world, prev_with) };
    }
    if prev_scope != 0 {
        unsafe { sys::ecs_set_scope(world, prev_scope) };
    }
    id
}

pub(crate) fn external_register_component_data<T>(
    world: *mut WorldT,
    name: *const c_char,
) -> EntityT {
    let prev_scope = unsafe { sys::ecs_set_scope(world, 0) };
    let prev_with = unsafe { sys::ecs_set_with(world, 0) };

    let id = external_register_componment_data_explicit::<T>(world, name);

    if prev_with != 0 {
        unsafe { sys::ecs_set_with(world, prev_with) };
    }
    if prev_scope != 0 {
        unsafe { sys::ecs_set_scope(world, prev_scope) };
    }
    id
}

/// registers the component with the world.
pub(crate) fn register_componment_data_explicit<T>(
    world: *mut WorldT,
    name: *const c_char,
) -> EntityT
where
    T: ComponentId,
{
    let only_type_name = crate::core::get_only_type_name::<T>();
    let only_type_name = compact_str::format_compact!("{}\0", only_type_name);

    // If no name was provided first check if a type with the provided
    // symbol was already registered.
    let id = if name.is_null() {
        let prev_scope = unsafe { sys::ecs_set_scope(world, 0) };
        let id = unsafe {
            sys::ecs_lookup_symbol(world, only_type_name.as_ptr() as *const i8, false, false)
        };
        unsafe { sys::ecs_set_scope(world, prev_scope) };
        id
    } else {
        0
    };
    if id != 0 {
        return id;
    }

    let type_name = crate::core::type_name_cstring::<T>();
    let type_name_ptr = type_name.as_ptr();

    let name = if name.is_null() { type_name_ptr } else { name };

    let entity_desc = create_entity_desc(name, type_name_ptr);

    let entity = unsafe { flecs_ecs_sys::ecs_entity_init(world, &entity_desc) };

    let type_info = create_type_info::<T>();

    let component_desc = create_component_desc(entity, type_info);

    let entity = unsafe { flecs_ecs_sys::ecs_component_init(world, &component_desc) };

    ecs_assert!(
        entity != 0 && unsafe { sys::ecs_exists(world, entity) },
        FlecsErrorCode::InternalError
    );

    entity
}

/// registers the component with the world.
pub(crate) fn external_register_componment_data_explicit<T>(
    world: *mut WorldT,
    name: *const c_char,
) -> EntityT {
    let only_type_name = crate::core::get_only_type_name::<T>();
    let only_type_name = compact_str::format_compact!("{}\0", only_type_name);

    // If no name was provided first check if a type with the provided
    // symbol was already registered.
    let id = if name.is_null() {
        let prev_scope = unsafe { sys::ecs_set_scope(world, 0) };
        let id = unsafe {
            sys::ecs_lookup_symbol(world, only_type_name.as_ptr() as *const i8, false, false)
        };
        unsafe { sys::ecs_set_scope(world, prev_scope) };
        id
    } else {
        0
    };
    if id != 0 {
        return id;
    }

    let type_name = crate::core::type_name_cstring::<T>();
    let type_name_ptr = type_name.as_ptr();

    let name = if name.is_null() { type_name_ptr } else { name };

    let entity_desc = create_entity_desc(name, type_name_ptr);

    let entity = unsafe { flecs_ecs_sys::ecs_entity_init(world, &entity_desc) };

    let type_info = external_create_type_info::<T>();

    let component_desc = create_component_desc(entity, type_info);

    let entity = unsafe { flecs_ecs_sys::ecs_component_init(world, &component_desc) };

    ecs_assert!(
        entity != 0 && unsafe { sys::ecs_exists(world, entity) },
        FlecsErrorCode::InternalError
    );

    entity
}
