use std::ffi::c_void;

use crate::core::*;
use crate::sys::*;

use super::meta_functions::*;

/// Serializer object, used for serializing opaque types
pub type Serializer = ecs_serializer_t;

/// Serializer function, used to serialize opaque types
pub type SerializeT = ecs_meta_serialize_t;

/// Type safe interface for opaque types
pub struct Opaque<'a, T, ElemType = c_void>
where
    T: ComponentId,
{
    world: WorldRef<'a>,
    pub desc: ecs_opaque_desc_t,
    //opaque_fn_ptrs: Box<OpaqueFnPtrs<T, ElemType>>,
    phantom: std::marker::PhantomData<T>,
    phantom2: std::marker::PhantomData<ElemType>,
}

impl<'a, T, ElemType> Opaque<'a, T, ElemType>
where
    T: ComponentId + Sized,
{
    /// Creates a new Opaque instance
    pub fn new(world: impl IntoWorld<'a>) -> Self {
        Self {
            world: world.world(),
            desc: ecs_opaque_desc_t {
                entity: T::id(world),
                type_: Default::default(),
            },
            phantom: std::marker::PhantomData,
            phantom2: std::marker::PhantomData,
            //opaque_fn_ptrs: Default::default(),
        }
    }

    /// Type that describes the type kind/structure of the opaque type
    pub fn as_type(&mut self, func: impl Into<Entity>) -> &mut Self {
        self.desc.type_.as_type = *func.into();
        self
    }

    /// Serialize function
    /// Fn(&Serializer, &T) -> i32
    pub fn serialize(&mut self, func: impl SerializeFn<T>) -> &mut Self {
        self.desc.type_.serialize = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&flecs_ecs_sys::ecs_serializer_t, &T) -> i32,
                unsafe extern "C" fn(
                    *const flecs_ecs_sys::ecs_serializer_t,
                    *const std::ffi::c_void,
                ) -> i32,
            >(func.to_extern_fn())
        });
        self
    }

    /// Assign bool value
    pub fn assign_bool(&mut self, func: impl AssignBoolFn<T>) -> &mut Self {
        self.desc.type_.assign_bool = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&mut T, bool),
                unsafe extern "C" fn(*mut std::ffi::c_void, bool),
            >(func.to_extern_fn())
        });
        self
    }

    /// Assign char value
    pub fn assign_char(&mut self, func: impl AssignCharFn<T>) -> &mut Self {
        self.desc.type_.assign_char = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&mut T, i8),
                unsafe extern "C" fn(*mut std::ffi::c_void, i8),
            >(func.to_extern_fn())
        });
        self
    }

    /// Assign int value
    pub fn assign_int(&mut self, func: impl AssignIntFn<T>) -> &mut Self {
        self.desc.type_.assign_int = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&mut T, i64),
                unsafe extern "C" fn(*mut std::ffi::c_void, i64),
            >(func.to_extern_fn())
        });
        self
    }

    /// Assign unsigned int value
    pub fn assign_uint(&mut self, func: impl AssignUIntFn<T>) -> &mut Self {
        self.desc.type_.assign_uint = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&mut T, u64),
                unsafe extern "C" fn(*mut std::ffi::c_void, u64),
            >(func.to_extern_fn())
        });
        self
    }

    /// Assign float value
    pub fn assign_float(&mut self, func: impl AssignFloatFn<T>) -> &mut Self {
        self.desc.type_.assign_float = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&mut T, f32),
                unsafe extern "C" fn(*mut std::ffi::c_void, f64),
            >(func.to_extern_fn())
        });
        self
    }

    /// Assign string value
    pub fn assign_string(&mut self, func: impl AssignStringFn<T>) -> &mut Self {
        self.desc.type_.assign_string = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&mut T, *const i8),
                unsafe extern "C" fn(*mut std::ffi::c_void, *const i8),
            >(func.to_extern_fn())
        });
        self
    }

    /// Assign entity value
    pub fn assign_entity(&mut self, func: impl AssignEntityFn<T>) -> &mut Self {
        self.desc.type_.assign_entity = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&mut T, WorldRef, Entity),
                unsafe extern "C" fn(*mut std::ffi::c_void, *mut flecs_ecs_sys::ecs_world_t, u64),
            >(func.to_extern_fn())
        });
        self
    }

    /// Assign null value
    pub fn assign_null(&mut self, func: impl AssignNullFn<T>) -> &mut Self {
        self.desc.type_.assign_null = Some(unsafe {
            std::mem::transmute::<extern "C" fn(&mut T), unsafe extern "C" fn(*mut std::ffi::c_void)>(
                func.to_extern_fn(),
            )
        });
        self
    }

    /// Clear collection elements
    pub fn clear(&mut self, func: impl ClearFn<T>) -> &mut Self {
        self.desc.type_.clear = Some(unsafe {
            std::mem::transmute::<extern "C" fn(&mut T), unsafe extern "C" fn(*mut std::ffi::c_void)>(
                func.to_extern_fn(),
            )
        });
        self
    }

    /// Ensure & get element
    pub fn ensure_member(&mut self, func: impl EnsureMemberFn<T, ElemType>) -> &mut Self {
        self.desc.type_.ensure_member = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&mut T, *const i8) -> &mut ElemType,
                unsafe extern "C" fn(*mut std::ffi::c_void, *const i8) -> *mut std::ffi::c_void,
            >(func.to_extern_fn())
        });
        self
    }

    /// Return number of elements
    pub fn count(&mut self, func: impl CountFn<T>) -> &mut Self {
        self.desc.type_.count = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&mut T) -> usize,
                unsafe extern "C" fn(*const std::ffi::c_void) -> usize,
            >(func.to_extern_fn())
        });
        self
    }

    /// Resize to number of elements
    pub fn resize(&mut self, func: impl ResizeFn<T>) -> &mut Self {
        self.desc.type_.resize = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&mut T, usize),
                unsafe extern "C" fn(*mut std::ffi::c_void, usize),
            >(func.to_extern_fn())
        });
        self
    }
}

impl<'a, T, ElemType> Drop for Opaque<'a, T, ElemType>
where
    T: ComponentId,
{
    /// Finalizes the opaque type descriptor when it is dropped
    fn drop(&mut self) {
        unsafe {
            ecs_opaque_init(self.world.world_ptr_mut(), &self.desc);
        }
    }
}
