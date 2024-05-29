use crate::core::{flecs, ComponentId, EntityView, World};

pub trait Module: ComponentId {
    fn module(world: &World);
}

impl World {
    pub fn import<T: Module>(&self) -> EntityView {
        // If we have already registered this type don't re-create the module
        if T::is_registered_with_world(self) {
            return self.component::<T>().entity;
        }

        // Reset scope
        let prev_scope = self.set_scope_id(0);

        // Set scope to our module
        self.set_scope::<T>();

        // Initialise component for the module and add Module tag
        let module = self.component::<T>();
        module.add::<flecs::Module>();

        // Build the module
        T::module(self);

        // Return out scope to the previous scope
        self.set_scope_id(prev_scope);

        module.entity
    }
}
