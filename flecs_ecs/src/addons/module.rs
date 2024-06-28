//! Modules organize components, systems and more in reusable units of code.

use crate::core::{flecs, ComponentId, EntityView, World};

pub trait Module: ComponentId {
    fn module(world: &World);
}

impl World {
    pub fn import<T: Module>(&self) -> EntityView {
        let module = self.component::<T>();
        // If we have already registered this type don't re-create the module
        if module.has::<flecs::Module>() {
            return module.entity;
        }

        // Reset scope
        let prev_scope = self.set_scope_id(0);

        // Initialise component for the module and add Module tag
        module.add::<flecs::Module>();

        // Set scope to our module
        self.set_scope_id(module.entity);

        // Build the module
        T::module(self);

        // Return out scope to the previous scope
        self.set_scope_id(prev_scope);

        module.entity
    }
}
