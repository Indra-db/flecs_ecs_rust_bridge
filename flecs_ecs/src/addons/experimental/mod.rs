#![doc(hidden)]

#[cfg(feature = "flecs_script")]
pub mod flecs_script;

use crate::core::{ComponentId, Entity, World};

/// Script mixin implementation
#[cfg(feature = "flecs_script")]
impl World {
    pub fn to_expr_id(
        &self,
        id_of_value: impl Into<Entity>,
        value: *const std::ffi::c_void,
    ) -> String {
        use crate::prelude::experimental::flecs_script::*;
        Script::to_expr_id(self, id_of_value, value)
    }

    pub fn to_expr<T: ComponentId>(&self, value: &T) -> String {
        use crate::prelude::experimental::flecs_script::*;
        Script::to_expr(self, value)
    }
}
