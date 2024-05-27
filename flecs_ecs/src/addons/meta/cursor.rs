use crate::core::*;
use flecs_ecs::sys;

/// Class for reading/writing dynamic values
pub struct Cursor<'a> {
    cursor: sys::ecs_meta_cursor_t,
    phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Cursor<'a> {
    /// Creates a new cursor instance
    pub fn new(
        world: impl IntoWorld<'a>,
        type_id: impl Into<Entity>,
        ptr: *mut std::ffi::c_void,
    ) -> Self {
        let world = world.world_ptr();
        let type_id = *type_id.into();
        let cursor = unsafe { sys::ecs_meta_cursor(world, type_id, ptr) };
        Self {
            cursor,
            phantom: std::marker::PhantomData,
        }
    }

    /// Push value scope (such as a nested struct)
    pub fn push(&mut self) -> i32 {
        unsafe { sys::ecs_meta_push(&mut self.cursor) }
    }

    /// Pop value scope
    pub fn pop(&mut self) -> i32 {
        unsafe { sys::ecs_meta_pop(&mut self.cursor) }
    }

    /// Move to next member/element
    pub fn next(&mut self) -> i32 {
        unsafe { sys::ecs_meta_next(&mut self.cursor) }
    }

    /// Move to member by name
    pub fn member(&mut self, name: &str) -> i32 {
        let name = compact_str::format_compact!("{}\0", name);
        unsafe { sys::ecs_meta_member(&mut self.cursor, name.as_ptr() as *const i8) }
    }

    /// Move to element by index
    pub fn elem(&mut self, elem: i32) -> i32 {
        unsafe { sys::ecs_meta_elem(&mut self.cursor, elem) }
    }

    /// Test if current scope is a collection type
    pub fn is_collection(&self) -> bool {
        unsafe { sys::ecs_meta_is_collection(&self.cursor) }
    }

    /// Get member name
    pub fn get_member(&self) -> &str {
        unsafe {
            std::ffi::CStr::from_ptr(sys::ecs_meta_get_member(&self.cursor))
                .to_str()
                .unwrap()
        }
    }

    /// Get type of value
    pub fn get_type(&self) -> EntityView {
        unsafe {
            EntityView::new_from(
                WorldRef::from_ptr(self.cursor.world as *mut WorldT),
                sys::ecs_meta_get_type(&self.cursor),
            )
        }
    }

    /// Get unit of value
    pub fn get_unit(&self) -> EntityView {
        unsafe {
            EntityView::new_from(
                WorldRef::from_ptr(self.cursor.world as *mut WorldT),
                sys::ecs_meta_get_unit(&self.cursor),
            )
        }
    }

    /// Get untyped pointer to value
    pub fn get_ptr(&mut self) -> *mut std::ffi::c_void {
        unsafe { sys::ecs_meta_get_ptr(&mut self.cursor) }
    }

    /// Set boolean value
    pub fn set_bool(&mut self, value: bool) -> i32 {
        unsafe { sys::ecs_meta_set_bool(&mut self.cursor, value) }
    }

    /// Set char value
    pub fn set_char(&mut self, value: char) -> i32 {
        unsafe { sys::ecs_meta_set_char(&mut self.cursor, value as i8) }
    }

    /// Set signed int value
    pub fn set_int(&mut self, value: i64) -> i32 {
        unsafe { sys::ecs_meta_set_int(&mut self.cursor, value) }
    }

    /// Set unsigned int value
    pub fn set_uint(&mut self, value: u64) -> i32 {
        unsafe { sys::ecs_meta_set_uint(&mut self.cursor, value) }
    }

    /// Set float value
    pub fn set_float(&mut self, value: f64) -> i32 {
        unsafe { sys::ecs_meta_set_float(&mut self.cursor, value) }
    }

    /// Set string value
    pub fn set_string(&mut self, value: &str) -> i32 {
        let value = compact_str::format_compact!("{}\0", value);
        unsafe { sys::ecs_meta_set_string(&mut self.cursor, value.as_ptr() as *const i8) }
    }

    /// Set string literal value
    pub fn set_string_literal(&mut self, value: &str) -> i32 {
        let value = compact_str::format_compact!("{}\0", value);
        unsafe { sys::ecs_meta_set_string_literal(&mut self.cursor, value.as_ptr() as *const i8) }
    }

    /// Set entity value
    pub fn set_entity(&mut self, value: impl Into<Entity>) -> i32 {
        unsafe { sys::ecs_meta_set_entity(&mut self.cursor, *value.into()) }
    }

    /// Set (component) id value
    pub fn set_id(&mut self, value: impl IntoId) -> i32 {
        unsafe { sys::ecs_meta_set_id(&mut self.cursor, *value.into()) }
    }

    /// Set null value
    pub fn set_null(&mut self) -> i32 {
        unsafe { sys::ecs_meta_set_null(&mut self.cursor) }
    }

    /// Get boolean value
    pub fn get_bool(&self) -> bool {
        unsafe { sys::ecs_meta_get_bool(&self.cursor) }
    }

    /// Get char value
    pub fn get_char(&self) -> char {
        unsafe { sys::ecs_meta_get_char(&self.cursor) as u8 as char }
    }

    /// Get signed int value
    pub fn get_int(&self) -> i64 {
        unsafe { sys::ecs_meta_get_int(&self.cursor) }
    }

    /// Get unsigned int value
    pub fn get_uint(&self) -> u64 {
        unsafe { sys::ecs_meta_get_uint(&self.cursor) }
    }

    /// Get float value
    pub fn get_float(&self) -> f64 {
        unsafe { sys::ecs_meta_get_float(&self.cursor) }
    }

    /// Get string value
    pub fn get_string(&self) -> *const i8 {
        // TODO: Rustify this to return &str
        unsafe { sys::ecs_meta_get_string(&self.cursor) }
    }

    /// Get entity value
    pub fn get_entity(&self) -> EntityView {
        unsafe {
            EntityView::new_from(
                WorldRef::from_ptr(self.cursor.world as *mut WorldT),
                sys::ecs_meta_get_entity(&self.cursor),
            )
        }
    }
}
