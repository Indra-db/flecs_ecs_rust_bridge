use std::ffi::c_char;
use std::ffi::c_void;

use crate::core::*;
use crate::sys::*;

/// Serializer object, used for serializing opaque types
pub type Serializer = ecs_serializer_t;

/// Serializer function, used to serialize opaque types
pub type SerializeT = ecs_meta_serialize_t;

/// Type safe variant of serializer function
pub type SerializeFn<T> = extern "C" fn(&Serializer, &T) -> i32;

pub type AssignBoolFn<T> = extern "C" fn(&mut T, bool);
pub type AssignCharFn<T> = extern "C" fn(&mut T, i8);
pub type AssignIntFn<T> = extern "C" fn(&mut T, i64);
pub type AssignUIntFn<T> = extern "C" fn(&mut T, u64);
pub type AssignFloatFn<T> = extern "C" fn(&mut T, f32);
// TODO: Replace with idiomatic Rust equivalent of c_char. Might need changes to flecs.
pub type AssignStringFn<T> = extern "C" fn(&mut T, *const c_char);
pub type AssignEntityFn<T> = extern "C" fn(&mut T, WorldRef, Entity);
pub type AssignNullFn<T> = extern "C" fn(&mut T);
pub type ClearFn<T> = extern "C" fn(&mut T);
// TODO: Implement the ensure_element function for collections.
pub type EnsureMemberFn<T, ElemType> = extern "C" fn(&mut T, *const c_char) -> &mut ElemType;
pub type CountFn<T> = extern "C" fn(&mut T) -> usize;
pub type ResizeFn<T> = extern "C" fn(&mut T, usize);

pub(crate) struct OpaqueFnPtrs<T: ComponentId, ElemType> {
    pub(crate) serialize: Option<fn(&Serializer, &T) -> i32>,
    pub(crate) assign_bool: Option<fn(&mut T, bool)>,
    pub(crate) assign_char: Option<fn(&mut T, i8)>,
    pub(crate) assign_int: Option<fn(&mut T, i64)>,
    pub(crate) assign_uint: Option<fn(&mut T, u64)>,
    pub(crate) assign_float: Option<fn(&mut T, f32)>,
    pub(crate) assign_string: Option<fn(&mut T, *const c_char)>,
    pub(crate) assign_entity: Option<fn(&mut T, &mut WorldRef, Entity)>,
    pub(crate) assign_null: Option<fn(&mut T)>,
    pub(crate) clear: Option<fn(&mut T)>,
    pub(crate) ensure_member: Option<fn(&mut T, *const c_char) -> &mut ElemType>,
    pub(crate) count: Option<fn(&mut T) -> usize>,
    pub(crate) resize: Option<fn(&mut T, usize)>,
}

impl<T: ComponentId, ElemType> Default for OpaqueFnPtrs<T, ElemType> {
    fn default() -> Self {
        Self {
            serialize: None,
            assign_bool: None,
            assign_char: None,
            assign_int: None,
            assign_uint: None,
            assign_float: None,
            assign_string: None,
            assign_entity: None,
            assign_null: None,
            clear: None,
            ensure_member: None,
            count: None,
            resize: None,
        }
    }
}

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
    pub fn serialize(&mut self, func: SerializeFn<T>) -> &mut Self {
        self.desc.type_.serialize = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&flecs_ecs_sys::ecs_serializer_t, &T) -> i32,
                unsafe extern "C" fn(
                    *const flecs_ecs_sys::ecs_serializer_t,
                    *const std::ffi::c_void,
                ) -> i32,
            >(func)
        });
        self
    }

    // pub fn serialize2(&mut self, func: fn(&Serializer, &T) -> i32) -> &mut Self {
    //     self.desc.type_.serialize = Some(Self::serializer_dummy::<T>);
    //     self.opaque_fn_ptrs.serialize = Some(func);
    //     self
    // }

    // extern "C" fn serializer_dummy<Q: ComponentId>(
    //     serializer: *const ecs_serializer_t,
    //     value: *const c_void,
    // ) -> i32 {
    //     let func = unsafe { (*serializer).ctx as *mut OpaqueFnPtrs<Q, c_void> };
    //     let func = unsafe { (*func).serialize.unwrap() };
    //     unsafe { func(&*serializer, &*(value as *const Q)) }
    // }

    /// Assign bool value
    pub fn assign_bool(&mut self, func: AssignBoolFn<T>) -> &mut Self {
        self.desc.type_.assign_bool = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&mut T, bool),
                unsafe extern "C" fn(*mut std::ffi::c_void, bool),
            >(func)
        });
        self
    }

    /// Assign char value
    pub fn assign_char(&mut self, func: AssignCharFn<T>) -> &mut Self {
        self.desc.type_.assign_char = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&mut T, i8),
                unsafe extern "C" fn(*mut std::ffi::c_void, i8),
            >(func)
        });
        self
    }

    /// Assign int value
    pub fn assign_int(&mut self, func: AssignIntFn<T>) -> &mut Self {
        self.desc.type_.assign_int = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&mut T, i64),
                unsafe extern "C" fn(*mut std::ffi::c_void, i64),
            >(func)
        });
        self
    }

    /// Assign unsigned int value
    pub fn assign_uint(&mut self, func: AssignUIntFn<T>) -> &mut Self {
        self.desc.type_.assign_uint = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&mut T, u64),
                unsafe extern "C" fn(*mut std::ffi::c_void, u64),
            >(func)
        });
        self
    }

    /// Assign float value
    pub fn assign_float(&mut self, func: AssignFloatFn<T>) -> &mut Self {
        self.desc.type_.assign_float = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&mut T, f32),
                unsafe extern "C" fn(*mut std::ffi::c_void, f64),
            >(func)
        });
        self
    }

    /// Assign string value
    pub fn assign_string(&mut self, func: AssignStringFn<T>) -> &mut Self {
        self.desc.type_.assign_string = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&mut T, *const i8),
                unsafe extern "C" fn(*mut std::ffi::c_void, *const i8),
            >(func)
        });
        self
    }

    /// Assign entity value
    pub fn assign_entity(&mut self, func: AssignEntityFn<T>) -> &mut Self {
        self.desc.type_.assign_entity = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&mut T, WorldRef, Entity),
                unsafe extern "C" fn(*mut std::ffi::c_void, *mut flecs_ecs_sys::ecs_world_t, u64),
            >(func)
        });
        self
    }

    /// Assign null value
    pub fn assign_null(&mut self, func: AssignNullFn<T>) -> &mut Self {
        self.desc.type_.assign_null = Some(unsafe {
            std::mem::transmute::<extern "C" fn(&mut T), unsafe extern "C" fn(*mut std::ffi::c_void)>(
                func,
            )
        });
        self
    }

    /// Clear collection elements
    pub fn clear(&mut self, func: ClearFn<T>) -> &mut Self {
        self.desc.type_.clear = Some(unsafe {
            std::mem::transmute::<extern "C" fn(&mut T), unsafe extern "C" fn(*mut std::ffi::c_void)>(
                func,
            )
        });
        self
    }

    /// Ensure & get element
    pub fn ensure_member(&mut self, func: EnsureMemberFn<T, ElemType>) -> &mut Self {
        self.desc.type_.ensure_member = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&mut T, *const i8) -> &mut ElemType,
                unsafe extern "C" fn(*mut std::ffi::c_void, *const i8) -> *mut std::ffi::c_void,
            >(func)
        });
        self
    }

    /// Return number of elements
    pub fn count(&mut self, func: CountFn<T>) -> &mut Self {
        self.desc.type_.count = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&mut T) -> usize,
                unsafe extern "C" fn(*const std::ffi::c_void) -> usize,
            >(func)
        });
        self
    }

    /// Resize to number of elements
    pub fn resize(&mut self, func: ResizeFn<T>) -> &mut Self {
        self.desc.type_.resize = Some(unsafe {
            std::mem::transmute::<
                extern "C" fn(&mut T, usize),
                unsafe extern "C" fn(*mut std::ffi::c_void, usize),
            >(func)
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
