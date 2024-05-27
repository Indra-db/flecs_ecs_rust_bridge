#![doc(hidden)]
mod cursor;
mod declarations;
mod impl_bindings;
mod impl_primitives;
mod opaque;

use std::ffi::c_void;

pub use cursor::*;
pub use declarations::*;
pub use opaque::*;

use crate::core::{
    ecs_assert, Component, ComponentId, Entity, EntityView, FlecsErrorCode, IntoWorld, World,
    WorldRef,
};
use crate::sys;

impl World {
    /// Return meta cursor to value
    pub fn cursor_id(&self, type_id: impl Into<Entity>, ptr: *mut c_void) -> Cursor {
        Cursor::new(self, type_id, ptr)
    }

    /// Return meta cursor to value
    pub fn cursor<T: ComponentId>(&self, ptr: *mut c_void) -> Cursor {
        let type_id = T::get_id(self.world());
        Cursor::new(self, type_id, ptr)
    }

    /// Create primitive type
    pub fn primitive(&self, kind: EcsPrimitiveKind) -> EntityView {
        let desc = sys::ecs_primitive_desc_t {
            kind: kind as u32,
            entity: 0u64,
        };

        let eid = unsafe { sys::ecs_primitive_init(self.ptr_mut(), &desc) };
        ecs_assert!(
            eid != 0,
            FlecsErrorCode::InvalidOperation,
            "failed to create primitive type"
        );
        EntityView::new_from(self, eid)
    }

    /// Create array type
    pub fn array_id(&self, elem_id: impl Into<Entity>, array_count: i32) -> EntityView {
        let desc = sys::ecs_array_desc_t {
            type_: *elem_id.into(),
            count: array_count,
            entity: 0u64,
        };

        let eid = unsafe { sys::ecs_array_init(self.ptr_mut(), &desc) };
        ecs_assert!(
            eid != 0,
            FlecsErrorCode::InvalidOperation,
            "failed to create array type"
        );
        EntityView::new_from(self, eid)
    }

    /// Create array type
    pub fn array<T: ComponentId>(&self, array_count: i32) -> EntityView {
        self.array_id(T::get_id(self.world()), array_count)
    }

    /// Create vector type
    pub fn vector_id(&self, elem_id: impl Into<Entity>) -> EntityView {
        let desc = sys::ecs_vector_desc_t {
            entity: 0u64,
            type_: *elem_id.into(),
        };

        let eid = unsafe { sys::ecs_vector_init(self.ptr_mut(), &desc) };

        ecs_assert!(
            eid != 0,
            FlecsErrorCode::InvalidOperation,
            "failed to create vector type"
        );

        EntityView::new_from(self, eid)
    }
}

pub trait EcsSerializer {
    fn value_id(&self, type_id: impl Into<Entity>, value: *const c_void) -> i32;
    fn value<T: ComponentId>(&self, value: &T) -> i32;
    fn member(&self, name: &str) -> i32;
}

impl EcsSerializer for sys::ecs_serializer_t {
    fn value_id(&self, type_id: impl Into<Entity>, value: *const c_void) -> i32 {
        if let Some(value_func) = self.value {
            unsafe { value_func(self, *type_id.into(), value) }
        } else {
            0
        }
    }

    fn value<T: ComponentId>(&self, value: &T) -> i32 {
        self.value_id(
            T::get_id(unsafe { WorldRef::from_ptr(self.world as *mut _) }),
            value as *const T as *const c_void,
        )
    }

    fn member(&self, name: &str) -> i32 {
        let name = compact_str::format_compact!("{}\0", name);
        if let Some(member_func) = self.member {
            unsafe { member_func(self, name.as_ptr() as *const i8) }
        } else {
            0
        }
    }
}

/// Register opaque type interface
impl<'a, T: ComponentId> Component<'a, T> {
    /// # See also
    ///
    /// * C++ API: `component::opaque`
    #[doc(alias = "component::opaque")]
    pub fn opaque_func<Func>(&self, func: Func) -> &Self
    where
        Func: FnOnce(WorldRef<'a>) -> Opaque<'a, T>,
    {
        let mut opaque = func(self.world());
        opaque.desc.entity = T::get_id(self.world());
        unsafe { sys::ecs_opaque_init(self.world_ptr_mut(), &opaque.desc) };
        self
    }

    /// # See also
    ///
    /// * C++ API: `component::opaque`
    #[doc(alias = "component::opaque")]
    pub fn opaque_id(&self, type_id: impl Into<Entity>) -> Opaque<'a, T> {
        let mut opaque = Opaque::<T>::new(self.world());
        opaque.as_type(type_id);
        opaque
    }

    // TODO
    /*
        /** Return opaque type builder for collection type */
    template <typename ElemType>
    flecs::opaque<T, ElemType> opaque(flecs::id_t as_type) {
        return flecs::opaque<T, ElemType>(world_).as_type(as_type);
    }

    /** Add constant. */
    component<T>& constant(const char *name, T value) {
        int32_t v = static_cast<int32_t>(value);
        untyped_component::constant(name, v);
        return *this;
    }

         */
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use super::*;

    // pub type SerializeFn<T> = extern "C" fn(*const Serializer, *const T) -> i32;

    extern "C" fn int_serializer(s: *const meta::Serializer, i: *const Int) -> i32 {
        unsafe { (*s).value::<Int>(&*i) }
    }

    #[derive(Debug, Clone, Component)]
    struct Int {
        value: i32,
    }

    #[test]

    fn test_opaque() {
        let world = World::new();
        let opaque = world
            .component::<Int>()
            .opaque_id(meta::I32)
            .serialize(int_serializer);

        let int_type = Int { value: 42 };

        let json: *mut i8 = unsafe {
            sys::ecs_ptr_to_json(
                world.ptr_mut(),
                <Int as ComponentId>::id(&world),
                &int_type as *const Int as *const c_void,
            )
        };

        let json_str = unsafe { std::ffi::CStr::from_ptr(json).to_str().unwrap() };
        println!("{}", json_str);
    }
}
