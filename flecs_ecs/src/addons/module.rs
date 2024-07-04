//! Modules organize components, systems and more in reusable units of code.
//!
//! * To define a module, see [`Module`].
//! * To import a module, see [`World::import()`].
//! * To override the name of a module, see [`World::module()`].

use crate::core::{flecs, ComponentId, EntityView, IdOperations, World, SEPARATOR};
use crate::sys;

/// Define a module
///
/// # See also
///
/// * [`addons::module`](crate::addons::module)
/// * [`World::import()`]
/// * [`World::module()`]
pub trait Module: ComponentId {
    /// Perform the module definition.
    ///
    /// This is invoked via [`World::import()`].
    ///
    /// This method should configure the components, systems, observers, and
    /// whatever else is needed for the proper functioning of this module.
    fn module(world: &World);
}

/// Module mixin implementation
impl World {
    /// Import a module.
    ///
    /// This operation will load a module. The module name will be used to verify if
    /// the module was already loaded, in which case it won't be reimported.
    ///
    /// Module contents will be stored as children of the module entity. This
    /// prevents modules from accidentally defining conflicting identifiers. This is
    /// enforced by setting the scope before and after loading the module to the
    /// module entity id.
    ///
    /// # See also
    ///
    /// * [`addons::module`](crate::addons::module)
    /// * [`Module`]
    /// * [`World::module()`]
    /// * C++ API: `world::import`
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

    /// Define a module.
    ///
    /// This operation is not mandatory, but can be called inside the module ctor to
    /// obtain the entity associated with the module, or override the module name.
    ///
    /// # Type Parameters
    ///
    /// * `M` - The type of the module.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to give the module.
    ///
    /// # Returns
    ///
    /// The module entity.
    ///
    /// # See also
    ///
    /// * [`addons::module`](crate::addons::module)
    /// * [`Module`]
    /// * [`World::import()`]
    /// * C++ API: `world::module`
    pub fn module<M: ComponentId>(&self, name: &str) -> EntityView {
        let id = self.component_named::<M>(name).id();

        let name = compact_str::format_compact!("{}\0", name);
        unsafe {
            sys::ecs_add_path_w_sep(
                self.raw_world.as_ptr(),
                *id,
                0,
                name.as_ptr() as *const _,
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
            );
        }
        self.set_scope_id(id);
        EntityView::new_from(self, *id)
    }
}
