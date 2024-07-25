#![doc(hidden)]
//! (internal) utility functions for dealing with ECS identifiers. This module is mostly used internally by the library.
//! but can be used by the user if needed.
use std::{ffi::CString, os::raw::c_char};

use crate::core::*;
use crate::sys;

/// Combines two 32 bit integers into a 64 bit integer.
///
/// # Arguments
///
/// * `lo`: The lower 32 bit integer.
/// * `hi`: The higher 32 bit integer.
///
/// # Returns
///
/// The combined 64 bit integer.
#[inline(always)]
pub fn ecs_entity_id_combine(lo: u64, hi: u64) -> u64 {
    (hi << 32) + lo
}

/// Combines two 32 bit integers into a 64 bit integer and adds the `ECS_PAIR` flag.
///
/// # Arguments
///
/// * `rel`: The first 32 bit integer.
/// * `target`: The second 32 bit integer.
///
/// # Returns
///
/// The combined 64 bit integer with the `ECS_PAIR` flag set.
#[inline(always)]
pub fn ecs_pair(rel: u64, target: u64) -> u64 {
    ECS_PAIR | ecs_entity_id_combine(target, rel)
}

/// Checks if given entity is a pair
pub fn ecs_is_pair(entity: impl Into<Id>) -> bool {
    entity.into() & RUST_ecs_id_FLAGS_MASK == ECS_PAIR
}

/// Set the `ECS_DEPENDS_ON` flag for the given entity.
///
/// # Arguments
///
/// * `entity`: The entity to set the `ECS_DEPENDS_ON` flag for.
///
/// # Returns
///
/// The entity with the `ECS_DEPENDS_ON` flag set.
#[inline(always)]
pub fn ecs_dependson(entity: u64) -> u64 {
    ecs_pair(ECS_DEPENDS_ON, entity)
}

/// Returns true if the entity has the given pair.
///
/// # Arguments
///
/// * `world`: The world to check in.
/// * `entity`: The entity to check.
/// * `first`: The first entity of the pair.
/// * `second`: The second entity of the pair.
///
/// # Returns
///
/// True if the entity has the given pair, false otherwise.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[inline(always)]
pub fn ecs_has_pair(
    world: *const WorldT,
    entity: impl Into<Entity>,
    first: impl Into<Entity>,
    second: impl Into<Entity>,
) -> bool {
    unsafe {
        sys::ecs_has_id(
            world,
            *entity.into(),
            ecs_pair(*first.into(), *second.into()),
        )
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[inline(always)]
pub fn ecs_add_pair(
    world: *mut WorldT,
    entity: impl Into<Entity>,
    first: impl Into<Entity>,
    second: impl Into<Entity>,
) {
    unsafe {
        sys::ecs_add_id(
            world,
            *entity.into(),
            ecs_pair(*first.into(), *second.into()),
        );
    };
}

/// Get the first entity from a pair.
///
/// # Arguments
///
/// * `e`: The pair to get the first entity from.
///
/// # Returns
///
/// The first entity from the pair.
#[inline(always)]
pub fn ecs_first(e: impl IntoId) -> Entity {
    Entity(ecs_entity_id_high(e.into() & RUST_ECS_COMPONENT_MASK))
}

/// Get the second entity from a pair.
///
/// # Arguments
///
/// * `e`: The pair to get the second entity from.
///
/// # Returns
///
/// The second entity from the pair.
#[inline(always)]
pub fn ecs_second(e: impl IntoId) -> Entity {
    Entity(ecs_entity_id_low(e.into()))
}

/// Get the lower 32 bits of an entity id.
///
/// # Arguments
///
/// * `value`: The entity id to get the lower 32 bits from.
///
/// # Returns
///
/// The lower 32 bits of the entity id.
#[inline(always)]
pub fn ecs_entity_id_low(value: impl IntoId) -> u64 {
    *value.into() as u32 as u64
}

/// Get the higher 32 bits of an entity id.
///
/// # Arguments
///
/// * `value`: The entity id to get the higher 32 bits from.
///
/// # Returns
///
/// The higher 32 bits of the entity id.
#[inline(always)]
pub fn ecs_entity_id_high(value: impl IntoId) -> u64 {
    (*value.into()) >> 32
}

pub fn type_name_cstring<T>() -> CString {
    CString::new(std::any::type_name::<T>()).unwrap()
}

/// Get the type name of the given type.
///
/// # Type Parameters
///
/// * `T`: The type to get the name of.
///
/// # Returns
///
/// `[Type]` string slice.
///
/// # Example
///
/// ```
/// use flecs_ecs::core::get_only_type_name;
///
/// pub mod Bar {
///     pub struct Foo;
/// }
///
/// let name = get_only_type_name::<Bar::Foo>();
/// assert_eq!(name, "Foo");
/// ```
#[inline(always)]
pub fn get_only_type_name<T>() -> &'static str {
    use std::any::type_name;
    let name = type_name::<T>();
    name.split("::").last().unwrap_or(name)
}

/// Returns true if the given type is an empty type.
///
/// # Type Parameters
///
/// * `T`: The type to check.
#[inline(always)]
pub fn is_empty_type<T>() -> bool {
    std::mem::size_of::<T>() == 0
}

/// Extracts a row index from an ECS record identifier.
///
/// Applies a bitwise AND with `ECS_ROW_MASK` to `row`, isolating relevant bits,
/// and returns the result as an `i32`. This function is typically used to decode
/// information about an entity's components or position encoded within the record identifier.
///
/// # Arguments
///
/// * `row`: A `u32` representing an ECS record identifier.
///
/// # Returns
///
/// * `i32`: The decoded row index.
pub fn ecs_record_to_row(row: u32) -> i32 {
    (row & sys::ECS_ROW_MASK) as i32
}

/// Internal helper function to set a component for an entity.
///
/// This function sets the given value for an entity in the ECS world, ensuring
/// that the type of the component is valid.
///
/// # Type Parameters
///
/// * `T`: The type of the component data. Must implement `ComponentId`.
///
/// # Arguments
///
/// * `entity`: The ID of the entity.
/// * `value`: The value to set for the component.
/// * `id`: The ID of the component type.
pub(crate) fn set_helper<T: ComponentId>(world: *mut WorldT, entity: u64, value: T, id: u64) {
    ecs_assert!(
        std::mem::size_of::<T>() != 0,
        FlecsErrorCode::InvalidParameter,
        "invalid type: {}",
        std::any::type_name::<T>()
    );

    let mut is_new = false;
    unsafe {
        if sys::ecs_is_deferred(world) {
            if T::NEEDS_DROP {
                if T::IMPLS_DEFAULT {
                    //use set batching //faster performance, no panic possible
                    let comp = sys::ecs_ensure_modified_id(world, entity, id) as *mut T;
                    //SAFETY: ecs_ensure_modified_id will default initialize the component
                    std::ptr::drop_in_place(comp);
                    std::ptr::write(comp, value);
                } else {
                    //when it has the component, we know it won't panic using set and impl drop.
                    if sys::ecs_has_id(world, entity, id) {
                        //use set batching //faster performance, no panic possible since it's already present
                        let comp = sys::ecs_ensure_modified_id(world, entity, id) as *mut T;
                        //SAFETY: ecs_ensure_modified_id will default initialize the component
                        std::ptr::drop_in_place(comp);
                        std::ptr::write(comp, value);
                        return;
                    }

                    // if does not impl default or not have the id
                    // use insert //slower performance
                    let ptr = sys::ecs_emplace_id(world, entity, id, &mut is_new) as *mut T;

                    if !is_new {
                        std::ptr::drop_in_place(ptr);
                    }
                    std::ptr::write(ptr, value);
                    sys::ecs_modified_id(world, entity, id);
                }
            } else {
                //if not needs drop, use set batching, faster performance
                let comp = sys::ecs_ensure_modified_id(world, entity, id) as *mut T;
                std::ptr::drop_in_place(comp);
                std::ptr::write(comp, value);
            }
        } else
        /* not deferred */
        {
            let ptr = sys::ecs_emplace_id(world, entity, id, &mut is_new) as *mut T;

            if !is_new {
                std::ptr::drop_in_place(ptr);
            }
            std::ptr::write(ptr, value);
            sys::ecs_modified_id(world, entity, id);
        }
    }
}

/// Remove generation from entity id.
///
/// # Arguments
///
/// * `entity`: The entity id to strip the generation from.
///
/// # Returns
///
/// * `IdT`: The entity id with the generation removed.
#[inline(always)]
pub fn strip_generation(entity: impl Into<Entity>) -> u64 {
    unsafe { sys::ecs_strip_generation(*entity.into()) }
}

/// Get the generation from an entity id.
///
/// # Arguments
///
/// * `entity`: The entity id to get the generation from.
///
/// # Returns
///
/// * `u32`: The generation of the entity id.
#[inline(always)]
pub fn get_generation(entity: impl Into<Entity>) -> u32 {
    (*(entity.into() & sys::ECS_GENERATION_MASK) >> 32) as u32
}

/// Gets the component data from the iterator.
/// Retrieves a pointer to the data array for a specified query field.
///
/// This function obtains a pointer to an array of data corresponding to the term in the query,
/// based on the given index. The index starts from 1, representing the first term in the query.
///
/// For instance, given a query "Position, Velocity", invoking this function with index 1 would
/// return a pointer to the "Position" data array, and index 2 would return the "Velocity" data array.
///
/// If the specified field is not owned by the entity being iterated (e.g., a shared component from a prefab,
/// a component from a parent, or a component from another entity), this function returns a direct pointer
/// instead of an array pointer. Use `ecs_field_is_self` to dynamically check if a field is owned.
///
/// The `size` of the type `T` must match the size of the data type of the returned array. Mismatches between
/// the provided type size and the actual data type size may cause the operation to assert. The size of the
/// field can be obtained dynamically using `ecs_field_size`.
///
/// # Safety
///
/// This function is unsafe because it dereferences the iterator and uses the index to get the component data.
/// Ensure that the iterator is valid and the index is valid.
///
/// # Arguments
///
/// - `it`: A pointer to the iterator.
/// - `index`: The index of the field in the iterator, starting from 1.
///
/// # Returns
///
/// A pointer to the data of the specified field. The pointer type is determined by the generic type `T`.
///
/// # Example
///
#[cfg_attr(doctest, doc = " ````no_test")]
/// ```
/// // Assuming `it` is a valid iterator pointer obtained from a query.
/// let position_ptr: *mut Position = ecs_field(it, 1);
/// let velocity_ptr: *mut Velocity = ecs_field(it, 2);
/// ```
#[inline(always)]
pub unsafe fn ecs_field<T>(it: *const IterT, index: i32) -> *mut T {
    let size = std::mem::size_of::<T>();
    sys::ecs_field_w_size(it, size, index) as *mut T
}

/// Get the `OperKind` for the given type.
///
/// # Type Parameters
///
/// * `T`: The type to get the `OperKind` for.
///
/// # See also
///
/// * C++ API: `type_to_oper`
pub(crate) fn type_to_oper<T: OperType>() -> OperKind {
    T::OPER
}

/// Sets the specified bit in the flags.
pub fn ecs_bit_set(flags: &mut u32, bit: u32) {
    *flags |= bit;
}

/// Clears the specified bit in the flags.
pub fn ecs_bit_clear(flags: &mut u32, bit: u32) {
    *flags &= !bit;
}

/// Conditionally sets or clears a bit in the flags based on a condition.
pub fn ecs_bit_cond(flags: &mut u32, bit: u32, cond: bool) {
    if cond {
        ecs_bit_set(flags, bit);
    } else {
        ecs_bit_clear(flags, bit);
    }
}

/// Copies the given Rust &str to a C string and returns a pointer to the C string.
/// this is intended to be used when the C code needs to take ownership of the string.
///
/// # Note
///
/// This function isn't being used anymore and might be removed in the future.
pub(crate) fn copy_and_allocate_c_char_from_rust_str(data: &str) -> *mut c_char {
    ecs_assert!(
        data.is_ascii(),
        FlecsErrorCode::InvalidParameter,
        "string must be ascii"
    );
    let bytes = data.as_bytes();
    let len = bytes.len() + 1; // +1 for the null terminator
    let memory_c_str = unsafe { sys::ecs_os_api.malloc_.unwrap()(len as i32) } as *mut u8;

    for (i, &byte) in bytes.iter().enumerate() {
        unsafe {
            memory_c_str.add(i).write(byte);
        }
    }

    // Write the null terminator to the end of the memory
    unsafe { memory_c_str.add(bytes.len()).write(0) };

    memory_c_str as *mut c_char
}

/// Prints the given C string to the console.
///
/// # Note
///
/// This function is for development purposes. It is not intended to be used in production code.
pub(crate) unsafe fn print_c_string(c_string: *const c_char) {
    // Ensure the pointer is not null
    assert!(!c_string.is_null(), "Null pointer passed to print_c_string");

    // Create a CStr from the raw pointer
    let c_str = std::ffi::CStr::from_ptr(c_string);

    // Convert CStr to a Rust string slice (&str)
    // This can fail if the C string is not valid UTF-8, so handle errors appropriately
    #[allow(clippy::print_stdout)]
    match c_str.to_str() {
        Ok(s) => println!("{}", s),
        Err(_) => println!("Failed to convert C string to Rust string"),
    }
}

/// Strips the given prefix from the given C string, returning a new C string with the prefix removed.
/// If the given C string does not start with the given prefix, returns `None`.
pub(crate) fn strip_prefix_str_raw<'a>(str: &'a str, prefix: &str) -> Option<&'a str> {
    let str_bytes = str.as_bytes();
    let prefix_bytes = prefix.as_bytes();

    if str_bytes.starts_with(prefix_bytes) {
        // SAFETY: We are slicing `cstr_bytes` which is guaranteed to be a valid
        // C string since it comes from a `&CStr`. We also check that it starts
        // with `prefix_bytes`, and we only slice off `prefix_bytes`, so the rest
        // remains a valid C string.
        Some(
            std::str::from_utf8(&str_bytes[prefix_bytes.len()..])
                .expect("error: pass valid utf strings"),
        )
    } else {
        None
    }
}
