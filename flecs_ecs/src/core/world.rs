//! World operations.

use std::ffi::CStr;
use std::{os::raw::c_void, ptr::NonNull};

#[cfg(feature = "flecs_app")]
use crate::addons::app::App;

#[cfg(feature = "flecs_system")]
use crate::addons::system::{System, SystemBuilder};

#[cfg(feature = "flecs_pipeline")]
use crate::addons::pipeline::PipelineBuilder;

use crate::core::*;

use crate::prelude::meta::Opaque;
use crate::sys;

#[derive(Debug, Eq, PartialEq)]
pub struct World {
    pub(crate) raw_world: NonNull<WorldT>,
}

unsafe impl Send for World {}

impl Clone for World {
    fn clone(&self) -> Self {
        unsafe { sys::flecs_poly_claim_(self.raw_world.as_ptr() as *mut c_void) };
        Self {
            raw_world: self.raw_world,
        }
    }
}

impl Default for World {
    fn default() -> Self {
        let world = Self {
            raw_world: unsafe { NonNull::new_unchecked(sys::ecs_init()) },
        };
        let ctx = Box::leak(Box::new(WorldCtx::new()));
        unsafe {
            sys::ecs_set_binding_ctx(
                world.raw_world.as_ptr(),
                ctx as *mut WorldCtx as *mut c_void,
                Some(world_ctx_destruct),
            );
        }

        world.init_builtin_components();
        world
    }
}

impl Drop for World {
    fn drop(&mut self) {
        let world_ptr = self.raw_world.as_ptr();
        if unsafe { sys::flecs_poly_release_(world_ptr as *mut c_void) } == 0 {
            if unsafe { sys::ecs_stage_get_id(world_ptr) } == -1 {
                unsafe { sys::ecs_stage_free(world_ptr) };
            } else {
                if !self.world_ctx().is_ref_count_zero() {
                    panic!("The code base still has lingering references to `Query` objects. This is a bug in the user code. 
                    Please ensure that all `Query` objects are out of scope before the world is destroyed.");
                }
                unsafe { sys::ecs_fini(self.raw_world.as_ptr()) };
            }
        }
    }
}

impl World {
    /// Creates a new world, same as `default()`
    pub fn new() -> Self {
        Self::default()
    }

    fn init_builtin_components(&self) {
        self.component::<()>();

        #[cfg(feature = "flecs_meta")]
        {
            self.component_named::<crate::prelude::meta::EcsTypeKind>("flecs::meta::type_kind");
            self.component_named::<crate::prelude::meta::EcsPrimitiveKind>(
                "flecs::meta::primitive_kind",
            );
            self.component_named::<crate::prelude::meta::EcsMember>("flecs::meta::member_t");
            self.component_named::<crate::prelude::meta::EcsEnumConstant>(
                "flecs::meta::enum_constant",
            );
            self.component_named::<crate::prelude::meta::EcsBitmaskConstant>(
                "flecs::meta::bitmask_constant",
            );

            let entity = self.entity_named("::flecs::rust").add::<flecs::Module>();

            entity.scope(|world| {
                let comp = world.component::<Entity>();
                comp.opaque_func(crate::prelude::meta::flecs_entity_support);
            });
        }
    }

    /// deletes and recreates the world
    ///
    /// # Safety
    /// This function panics if lingering references to `Query` and `World` objects are still present.
    ///
    /// # See also
    ///
    /// * C++ API: `world::reset`
    #[doc(alias = "world::reset")]
    pub fn reset(self) -> Self {
        if unsafe { sys::flecs_poly_refcount(self.raw_world.as_ptr() as *mut c_void) } > 1 {
            panic!("Reset would invalidate other world handles that are still lingering in the user's code base. 
            This is a bug in the user code. Please ensure that all world handles are out of scope before calling `reset`.");
        }
        unsafe { sys::ecs_fini(self.raw_world.as_ptr()) };
        World::new()
    }

    /// obtain pointer to C world object
    ///
    /// # Returns
    ///
    /// Returns a pointer to the C world object.
    ///
    /// # See also
    ///
    /// * C++ API: `world::c_ptr`
    #[doc(alias = "world::c_ptr")]
    #[inline(always)]
    pub fn ptr_mut(&self) -> *mut WorldT {
        self.raw_world.as_ptr()
    }

    /// Get the world's info.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_info`
    ///
    /// # Example
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// world.progress();
    ///
    /// let world_info = world.info();
    ///
    /// assert!(world_info.delta_time > 0.0);
    /// //assert!(world_info.world_time_total_raw > 0.0); //BUG TODO
    /// //assert!(world_info.systems_ran_frame == 0);
    /// ```
    #[doc(alias = "world::get_info")]
    pub fn info(&self) -> sys::WorldInfo {
        // SAFETY: The pointer is valid for the lifetime of the world.
        unsafe { *sys::ecs_get_world_info(self.raw_world.as_ptr()) }
    }

    /// Gets the last `delta_time`.
    ///
    /// After calling this function, the next call to [`World::progress()`] returns false.
    ///
    /// # See also
    ///
    /// * C++ API: `world::quit`
    ///
    /// # Example
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let mut count = 0;
    /// while world.progress() {
    ///     count += 1;
    ///     if count == 5 {
    ///         world.quit();
    ///     }
    /// }
    /// assert!(count == 5);
    /// ```
    #[doc(alias = "world::quit")]
    pub fn quit(&self) {
        unsafe {
            sys::ecs_quit(self.raw_world.as_ptr());
        }
    }

    /// Registers an action to be executed when the world is destroyed.
    ///
    /// # See also
    ///
    /// * C++ API: `world::atfini`
    #[doc(alias = "world::atfini")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // this doesn't actually deref the pointer
    pub fn on_destroyed(&self, action: sys::ecs_fini_action_t, ctx: *mut c_void) {
        unsafe {
            sys::ecs_atfini(self.raw_world.as_ptr(), action, ctx);
        }
    }

    /// Tests if `quit` has been called.
    ///
    /// # Returns
    ///
    /// True if quit has been called, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `world::should_quit`
    #[doc(alias = "world::should_quit")]
    pub fn should_quit(&self) -> bool {
        unsafe { sys::ecs_should_quit(self.raw_world.as_ptr()) }
    }

    /// Begins a frame.
    ///
    /// When an application does not use `progress()` to control the main loop, it
    /// can still use Flecs features such as FPS limiting and time measurements.
    /// can still use Flecs features such as FPS limiting and time measurements processed.
    ///
    /// Calls to `frame_begin` must always be followed by `frame_end`.
    ///
    /// The function accepts a `delta_time` parameter, which will get passed to
    /// systems. This value is also used to compute the amount of time the
    /// function needs to sleep to ensure it does not exceed the `target_fps`, when
    /// it is set. When 0 is provided for `delta_time`, the time will be measured.
    ///
    /// # Safety
    /// This function should only be ran from the main thread.
    ///
    /// # Arguments
    /// * `delta_time`: Time elapsed since the last frame.
    ///
    /// # Returns
    /// The provided `delta_time`, or the measured time if 0 was provided.
    ///
    /// # See also
    ///
    /// * C++ API: `world::frame_begin`
    #[doc(alias = "world::frame_begin")]
    pub fn frame_begin(&self, delta_time: f32) -> f32 {
        unsafe { sys::ecs_frame_begin(self.raw_world.as_ptr(), delta_time) }
    }

    /// Ends a frame.
    ///
    /// This operation must be called at the end of the frame, and always after `frame_begin`
    ///
    /// # Safety
    /// The function should only be run from the main thread.
    ///
    /// # See also
    ///
    /// * C++ API: `world::frame_end`
    #[doc(alias = "world::frame_end")]
    pub fn frame_end(&self) {
        unsafe {
            sys::ecs_frame_end(self.raw_world.as_ptr());
        }
    }

    /// Begin readonly mode.
    ///
    /// When an application does not use [`World::progress()`] to control the main loop,
    /// it can still use Flecs features such as the defer queue. To stage changes, this function
    /// must be called after `sys::ecs_frame_begin`.
    ///
    /// A call to `sys::ecs_readonly_begin` must be followed by a call to `sys::ecs_readonly_end`.
    ///
    /// When staging is enabled, modifications to entities are stored to a stage.
    /// This ensures that arrays are not modified while iterating. Modifications are
    /// merged back to the "main stage" when `sys::ecs_readonly_end` is invoked.
    ///
    /// While the world is in staging mode, no structural changes (add/remove/...) can
    /// be made to the world itself. Operations must be executed on a stage instead (see `sys::ecs_get_stage`).
    ///
    /// Readonly mode is a stronger version of deferred mode. In deferred mode,
    /// ECS operations such as add/remove/set/delete etc. are added to a command
    /// queue to be executed later. In readonly mode, operations that could break
    /// scheduler logic (such as creating systems, queries) are also disallowed.
    ///
    /// Readonly mode itself has a single-threaded and a multi-threaded mode. In
    /// single-threaded mode certain mutations on the world are still allowed, for example:
    /// - Entity liveliness operations (such as new, `make_alive`), so that systems are
    ///   able to create new entities.
    /// - Implicit component registration, so that this works from systems.
    /// - Mutations to supporting data structures for the evaluation of uncached
    ///   queries (filters), so that these can be created on the fly.
    ///
    /// These mutations are safe in single-threaded applications, but for
    /// multi-threaded applications, the world needs to be entirely immutable. For this
    /// purpose, multi-threaded readonly mode exists, which disallows all mutations on
    /// the world.
    ///
    /// While in readonly mode, applications can still enqueue ECS operations on a
    /// stage. Stages are managed automatically when using the pipeline addon and
    /// [`World::progress()`], but they can also be configured manually.
    ///
    /// Number of stages typically corresponds with number of threads
    ///
    /// When an attempt is made to perform an operation on a world in readonly mode,
    /// the code will throw an assert saying that the world is in readonly mode.
    ///
    /// A call to `readonly_begin` must be followed up with `readonly_end()`.
    /// When `readonly_end()` is called, all enqueued commands from configured
    /// stages are merged back into the world. Calls to `readonly_begin()` and
    /// `readonly_end()` should always happen from a context where the code has
    /// exclusive access to the world. The functions themselves are not thread safe.
    ///
    /// ## Safety
    /// This function should only be run from the main thread.
    ///
    /// # Returns
    /// Whether the world is currently staged and whether it is in readonly mode.
    ///
    /// # See also
    ///
    /// * C++ API: `world::readonly_begin`
    #[doc(alias = "world::readonly_begin")]
    pub fn readonly_begin(&self, multi_threaded: bool) -> bool {
        unsafe { sys::ecs_readonly_begin(self.raw_world.as_ptr(), multi_threaded) }
    }

    /// End readonly mode.
    ///
    /// Leaves staging mode. After this operation, the world may be directly mutated again.
    /// By default, this operation also merges data back into the world, unless auto-merging
    /// was disabled explicitly.
    ///
    /// ## safety
    /// This function should only be run from the main thread.
    ///
    /// # Returns
    ///
    /// Whether the world is currently staged.
    ///
    /// # See also
    ///
    /// * C++ API: `world::readonly_end`
    ///
    /// # Example
    ///
    /// see [`World::readonly_begin()`]
    #[doc(alias = "world::readonly_end")]
    pub fn readonly_end(&self) {
        unsafe {
            sys::ecs_readonly_end(self.raw_world.as_ptr());
        }
    }

    /// Defers operations until the end of the frame.
    ///
    /// When this operation is invoked while iterating, the operations between `defer_begin`
    /// and `defer_end` are executed at the end of the frame.
    ///
    /// ## safety
    /// this operation is thread safe
    ///
    /// # Returns
    /// Whether the operation was successful.
    ///
    /// # See also
    ///
    /// * C++ API: `world::defer_begin`
    #[doc(alias = "world::defer_begin")]
    pub fn defer_begin(&self) -> bool {
        unsafe { sys::ecs_defer_begin(self.raw_world.as_ptr()) }
    }

    /// Ends a block of operations to defer.
    ///
    /// This should follow a `defer_begin` call.
    ///
    /// ## safety
    /// this operation is thread safe
    ///
    /// # Returns
    /// Whether the operation was successful.
    ///
    /// # See also
    ///
    /// * C++ API: `world::defer_end`
    #[doc(alias = "world::defer_end")]
    pub fn defer_end(&self) -> bool {
        unsafe { sys::ecs_defer_end(self.raw_world.as_ptr()) }
    }

    /// Test whether deferring is enabled.
    ///
    /// # Returns
    ///
    /// Whether deferring is enabled.
    ///
    /// # See also
    ///
    /// * C++ API: `world::is_deferred`
    #[doc(alias = "world::is_deferred")]
    pub fn is_deferred(&self) -> bool {
        unsafe { sys::ecs_is_deferred(self.raw_world.as_ptr()) }
    }

    /// Configure world to have N stages.
    ///
    /// This initializes N stages, which allows applications to defer operations to
    /// multiple isolated defer queues. This is typically used for applications with
    /// multiple threads, where each thread gets its own queue, and commands are
    /// merged when threads are synchronized.
    ///
    /// Note that `set_threads()` already creates the appropriate number of stages.
    /// The `set_stage_count()` operation is useful for applications that want to manage
    /// their own stages and/or threads.
    ///
    /// # Arguments
    ///
    /// * `stages`: The number of stages.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_stage_count`
    #[doc(alias = "world::set_stage_count")]
    pub fn set_stage_count(&self, stages: i32) {
        unsafe {
            sys::ecs_set_stage_count(self.raw_world.as_ptr(), stages);
        }
    }

    /// Get number of configured stages.
    ///
    /// Return number of stages set by `set_stage_count`.
    ///
    /// # Returns
    ///
    /// The number of stages used for threading.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_stage_count`
    #[doc(alias = "world::get_stage_count")]
    pub fn get_stage_count(&self) -> i32 {
        unsafe { sys::ecs_get_stage_count(self.raw_world.as_ptr()) }
    }

    /// Get current stage id.
    ///
    /// The stage id can be used by an application to learn about which stage it
    /// is using, which typically corresponds with the worker thread id.
    ///
    /// # Returns
    ///
    /// The stage id.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_stage_id`
    #[doc(alias = "world::get_stage_id")]
    pub fn get_stage_id(&self) -> i32 {
        unsafe { sys::ecs_stage_get_id(self.raw_world.as_ptr()) }
    }

    /// Test if is a stage.
    ///
    /// If this function returns `false`, it is guaranteed that this is a valid
    /// world object.
    ///
    /// # Returns
    ///
    /// True if the world is a stage, false if not.
    ///
    /// # See also
    ///
    /// * C++ API: `world::is_stage`
    #[doc(alias = "world::is_stage")]
    pub fn is_stage(&self) -> bool {
        unsafe {
            ecs_assert!(
                sys::flecs_poly_is_(
                    self.raw_world.as_ptr() as *const c_void,
                    sys::ecs_world_t_magic as i32
                ) || sys::flecs_poly_is_(
                    self.raw_world.as_ptr() as *const c_void,
                    sys::ecs_stage_t_magic as i32
                ),
                FlecsErrorCode::InvalidParameter,
                "flecs::world instance contains invalid reference to world or stage"
            );
            sys::flecs_poly_is_(
                self.raw_world.as_ptr() as *const c_void,
                sys::ecs_stage_t_magic as i32,
            )
        }
    }

    /// Merge world or stage.
    ///
    /// When automatic merging is disabled, an application can call this
    /// operation on either an individual stage, or on the world which will merge
    /// all stages. This operation may only be called when staging is not enabled
    /// (either after [`World::progress()`] or after `readonly_end()`).
    ///
    /// This operation may be called on an already merged stage or world.
    ///
    /// # See also
    ///
    /// * C++ API: `world::merge`
    #[doc(alias = "world::merge")]
    pub fn merge(&self) {
        unsafe { sys::ecs_merge(self.raw_world.as_ptr()) };
    }

    /// Get stage-specific world pointer.
    ///
    /// Flecs threads can safely invoke the API as long as they have a private
    /// context to write to, also referred to as the stage. This function returns a
    /// pointer to a stage, disguised as a world pointer.
    ///
    /// Note that this function does not(!) create a new world. It simply wraps the
    /// existing world in a thread-specific context, which the API knows how to
    /// unwrap. The reason the stage is returned as an `sys::ecs_world_t` is so that it
    /// can be passed transparently to the existing API functions, vs. having to
    /// create a dedicated API for threading.
    ///
    /// # Arguments
    ///
    /// * `stage_id` - The index of the stage to retrieve.
    ///
    /// # Returns
    ///
    /// A thread-specific pointer to the world.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_stage`
    #[doc(alias = "world::get_stage")]
    pub fn stage(&self, stage_id: i32) -> WorldRef {
        unsafe { WorldRef::from_ptr(sys::ecs_get_stage(self.raw_world.as_ptr(), stage_id)) }
    }

    /// Create asynchronous stage.
    ///
    /// An asynchronous stage can be used to asynchronously queue operations for
    /// later merging with the world. An asynchronous stage is similar to a regular
    /// stage, except that it does not allow reading from the world.
    ///
    /// Asynchronous stages are never merged automatically, and must therefore be
    /// manually merged with the `sys::ecs_merge` function. It is not necessary to call `defer_begin`
    /// or `defer_end` before and after enqueuing commands, as an
    /// asynchronous stage unconditionally defers operations.
    ///
    /// The application must ensure that no commands are added to the stage while the
    /// stage is being merged.
    ///
    /// An asynchronous stage must be cleaned up by `sys::ecs_async_stage_free`.
    ///
    /// # Returns
    ///
    /// The stage.
    ///
    /// # See also
    ///
    /// * C++ API: `world::async_stage`
    #[doc(alias = "world::async_stage")]
    pub fn create_async_stage(&self) -> WorldRef {
        unsafe { WorldRef::from_ptr(sys::ecs_stage_new(self.raw_world.as_ptr())) }
    }

    /// Get actual world.
    ///
    /// If the current object points to a stage, this operation will return the
    /// actual world.
    ///
    /// # Returns
    ///
    /// The actual world.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_world`
    #[doc(alias = "world::get_world")]
    pub fn get_world(&self) -> WorldRef {
        self.world().real_world()
    }

    /// Test whether the current world object is readonly.
    ///
    /// This function allows the code to test whether the currently used world
    /// object is readonly or whether it allows for writing.
    ///
    /// # Returns
    ///
    /// True if the world or stage is readonly.
    ///
    /// # See also
    ///
    /// * C++ API: `world::is_readonly`
    #[doc(alias = "world::is_readonly")]
    pub fn is_readonly(&self) -> bool {
        unsafe { sys::ecs_stage_is_readonly(self.raw_world.as_ptr()) }
    }

    /// Set world context.
    ///
    /// Set a context value that can be accessed by anyone that has a reference
    /// to the world.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The world context.
    /// * `ctx_free` - The free function for the context. Can pass `None` if no free function is needed.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_ctx`
    #[doc(alias = "world::set_ctx")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // this doesn't actually deref the pointer
    pub fn set_context(&self, ctx: *mut c_void, ctx_free: sys::ecs_ctx_free_t) {
        unsafe { sys::ecs_set_ctx(self.raw_world.as_ptr(), ctx, ctx_free) }
    }

    /// Get world context.
    ///
    /// # Returns
    ///
    /// The configured world context.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_ctx`
    #[doc(alias = "world::get_ctx")]
    pub fn context(&self) -> *mut c_void {
        unsafe { sys::ecs_get_ctx(self.raw_world.as_ptr()) }
    }

    /// Set world binding context
    /// Set a context value that can be accessed by anyone that has a reference to the world.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The world context.
    /// * `ctx_free` - The free function for the context. Can pass `None` if no free function is needed.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_binding_context`
    #[doc(alias = "world::set_binding_context")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn set_binding_context(&self, ctx: *mut c_void, ctx_free: sys::ecs_ctx_free_t) {
        unsafe { sys::ecs_set_ctx(self.raw_world.as_ptr(), ctx, ctx_free) }
    }

    /// Get world binding context.
    ///
    /// # Returns
    ///
    /// The configured world context.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_binding_context`
    #[doc(alias = "world::get_binding_context")]
    pub fn get_binding_context(&self) -> *mut c_void {
        unsafe { sys::ecs_get_ctx(self.raw_world.as_ptr()) }
    }

    /// Preallocate memory for a number of entities.
    ///
    /// This function preallocates memory for the entity index.
    ///
    /// # Arguments
    ///
    /// * `entity_count` - Number of entities to preallocate memory for.
    ///
    /// # See also
    ///
    /// * C++ API: `world::dim`
    #[doc(alias = "world::dim")]
    pub fn preallocate_entity_count(&self, entity_count: i32) {
        unsafe { sys::ecs_dim(self.raw_world.as_ptr(), entity_count) };
    }

    /// Set the entity range.
    ///
    /// This function limits the range of issued entity IDs between `min` and `max`.
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum entity ID issued.
    /// * `max` - Maximum entity ID issued.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_entity_range`
    #[doc(alias = "world::set_entity_range")]
    pub fn set_entity_range(&self, min: impl Into<Entity>, max: impl Into<Entity>) {
        unsafe { sys::ecs_set_entity_range(self.raw_world.as_ptr(), *min.into(), *max.into()) };
    }

    /// Enforce that operations cannot modify entities outside of the specified range.
    ///
    /// This function ensures that only entities within the specified range can
    /// be modified. Use this function if specific parts of the code are only allowed
    /// to modify a certain set of entities, as could be the case for networked applications.
    ///
    /// # Arguments
    ///
    /// * `enabled` - True if the range check should be enabled, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `world::enable_range_check`
    #[doc(alias = "world::enable_range_check")]
    pub fn enable_range_check(&self, enabled: bool) {
        unsafe { sys::ecs_enable_range_check(self.raw_world.as_ptr(), enabled) };
    }

    /// Get the current scope. Get the scope set by `sys::ecs_set_scope`. If no scope is set, this operation will return 0.
    ///
    /// # Returns
    ///
    /// Returns an `EntityView` representing the current scope.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_scope`
    #[doc(alias = "world::get_scope")]
    #[inline(always)]
    pub fn get_scope<T: ComponentId>(&self) -> EntityView {
        EntityView::new_from(self, unsafe { sys::ecs_get_scope(self.raw_world.as_ptr()) })
    }

    /// Set the current scope. This operation sets the scope of the current stage to the provided entity.
    /// As a result new entities will be created in this scope, and lookups will be relative to the provided scope.
    /// It is considered good practice to restore the scope to the old value.
    ///
    /// This method changes the current scope to the entity represented by the provided `id`.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the scope entity to set.
    ///
    /// # Returns
    ///
    /// Returns an `EntityView` representing the newly set scope.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_scope`
    #[doc(alias = "world::set_scope")]
    #[inline(always)]
    pub fn set_scope_id(&self, id: impl IntoId) -> EntityView {
        EntityView::new_from(self, unsafe {
            sys::ecs_set_scope(self.raw_world.as_ptr(), *id.into())
        })
    }

    /// Sets the current scope, but allows the scope type to be inferred from the type parameter.
    /// This operation sets the scope of the current stage to the provided entity.
    /// As a result new entities will be created in this scope, and lookups will be relative to the provided scope.
    /// It is considered good practice to restore the scope to the old value.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type that implements `ComponentId`.
    ///
    /// # Returns
    ///
    /// Returns an `EntityView` representing the newly set scope.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_scope`
    #[doc(alias = "world::set_scope")]
    #[inline(always)]
    pub fn set_scope<T: ComponentId>(&self) -> EntityView {
        self.set_scope_id(T::get_id(self))
    }

    /// Sets the search path for entity lookup operations.
    ///
    /// This function configures the search path used for looking up entities. The search path is an array of entity IDs that define the scopes within which lookup operations will search for entities.
    ///
    /// # Best Practices
    ///
    /// * It's advisable to restore the previous search path after making temporary changes.
    ///
    /// # Search Path Evaluation
    ///
    /// * The search path is evaluated starting from the last element of the array.
    ///
    /// # Default Behavior
    ///
    /// * The default search path includes `flecs.core`.
    ///
    /// # Overwriting
    ///
    /// * Providing a custom search path will overwrite the existing search path.
    ///
    /// # Considerations
    ///
    /// * If the custom search path doesn't include `flecs.core`, operations that rely on looking up names from `flecs.core` may fail.
    /// * The search path array is not managed by the Rust runtime. Ensure the array remains valid for as long as it is used as the search path.
    ///
    /// # Array Termination
    ///
    /// * The provided array must be terminated with a 0 element. This allows for pushing/popping elements onto/from an existing array without needing to call `sys::ecs_set_lookup_path` again.
    ///
    /// # Arguments
    ///
    /// * `search_path` - A 0-terminated array of entity IDs defining the new search path.
    ///
    /// # Returns
    ///
    /// Returns the current search path after the operation.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_lookup_path`
    /// * C API: `sys::ecs_set_lookup_path`
    #[doc(alias = "world::set_lookup_path")]
    #[doc(alias = "wsys::ecs_set_lookup_path")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // this doesn't actually deref the pointer
    pub fn set_lookup_path(&self, search_path: impl Into<Entity>) -> *mut EntityT {
        unsafe { sys::ecs_set_lookup_path(self.raw_world.as_ptr(), &*search_path.into()) }
    }

    /// Lookup an entity by name.
    /// The entity is searched recursively recursively traversing
    /// up the tree until found.
    ///
    /// # Safety
    ///
    /// Panics: Ensure that the entity exists before using it.
    /// Use `try_lookup` variant otherwise.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to lookup.
    ///
    /// # Returns
    ///
    /// The entity, if not found, the entity will have an id of 0.
    ///
    /// # See also
    ///
    /// * C++ API: `world::lookup`
    #[doc(alias = "world::lookup")]
    #[inline(always)]
    pub fn lookup_recursively(&self, name: &str) -> EntityView {
        self.try_lookup_recursive(name)
            .expect("Entity not found, when unsure, use try_lookup_recursive")
    }

    /// Lookup entity by name, only the current scope is searched
    ///
    /// # Safety
    ///
    /// Panics: Ensure that the entity exists before using it.
    /// Use `try_lookup` variant otherwise.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to lookup.
    ///
    /// # Returns
    ///
    /// The entity, if not found, returns an entity with id 0.
    ///
    /// # See also
    ///
    /// * C++ API: `world::lookup`
    #[doc(alias = "world::lookup")]
    #[inline(always)]
    pub fn lookup(&self, name: &str) -> EntityView {
        self.try_lookup(name)
            .expect("Entity not found, when unsure, use try_lookup")
    }

    /// Lookup entity by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to lookup.
    /// * `recursively` - Recursively traverse up the tree until entity is found.
    ///
    /// # Returns
    ///
    /// The entity if found, otherwise None.
    ///
    /// # See also
    ///
    /// * C++ API: `world::lookup`
    #[doc(alias = "world::lookup")]
    fn try_lookup_impl(&self, name: &str, recursive: bool) -> Option<EntityView> {
        let name = compact_str::format_compact!("{}\0", name);

        let entity_id = unsafe {
            sys::ecs_lookup_path_w_sep(
                self.raw_world.as_ptr(),
                0,
                name.as_ptr() as *const _,
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
                recursive,
            )
        };
        if entity_id == 0 {
            None
        } else {
            Some(EntityView::new_from(self, entity_id))
        }
    }

    /// Lookup an entity by name.
    /// The entity is searched recursively recursively traversing
    /// up the tree until found.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to lookup.
    ///
    /// # Returns
    ///
    /// The entity if found, otherwise None.
    ///
    /// # See also
    ///
    /// * C++ API: `world::lookup`
    #[doc(alias = "world::lookup")]
    #[inline(always)]
    pub fn try_lookup_recursive(&self, name: &str) -> Option<EntityView> {
        self.try_lookup_impl(name, true)
    }

    /// Lookup entity by name, only the current scope is searched
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to lookup.
    ///
    /// # Returns
    ///
    /// The entity if found, otherwise None.
    ///
    /// # See also
    ///
    /// * C++ API: `world::lookup`
    #[doc(alias = "world::lookup")]
    #[inline(always)]
    pub fn try_lookup(&self, name: &str) -> Option<EntityView> {
        self.try_lookup_impl(name, false)
    }

    /// Sets a singleton component of type `T` on the world.
    ///
    /// # Arguments
    ///
    /// * `component` - The singleton component to set on the world.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set`
    #[doc(alias = "world::set")]
    pub fn set<T: ComponentId + NotEmptyComponent + ComponentType<Struct>>(&self, component: T) {
        let id = T::get_id(self);
        set_helper(self.raw_world.as_ptr(), id, component, id);
    }

    /// Set a singleton pair using the second element type and a first id.
    ///
    /// # Safety
    ///
    /// Caller must ensure that `First` and `second` pair id data type is the one provided.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set`
    #[doc(alias = "world::set")]
    pub fn set_first<First>(&self, second: impl Into<Entity>, first: First)
    where
        First: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        let entity = EntityView::new_from(self, First::get_id(self));
        entity.set_first::<First>(first, second);
    }

    /// Set a singleton pair using the second element type and a first id.
    ///
    /// # Safety
    ///
    /// Caller must ensure that `first` and `Second` pair id data type is the one provided.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set`
    #[doc(alias = "world::set")]
    pub fn set_second<Second>(&self, first: impl Into<Entity>, second: Second)
    where
        Second: ComponentId + ComponentType<Struct> + NotEmptyComponent,
    {
        let entity = EntityView::new_from(self, Second::get_id(self));
        entity.set_second::<Second>(first, second);
    }

    /// Set singleton pair.
    /// This operation sets the pair value, and uses the first non tag / ZST as type. If the
    /// entity did not yet have the pair, it will be added, otherwise overridden.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set`
    #[doc(alias = "world::set")]
    pub fn set_pair<First, Second>(&self, data: <(First, Second) as FlecsCastType>::CastType)
    where
        First: ComponentId,
        Second: ComponentId,
        (First, Second): FlecsCastType,
    {
        // const {
        //     assert!(!<(First, Second) as IntoComponentId>::IS_TAGS, "setting tag relationships is not possible with `set_pair`. use `add_pair` instead.");
        // };

        let entity = EntityView::new_from(
            self,
            <<(First, Second) as FlecsCastType>::CastType as ComponentId>::get_id(self),
        );
        entity.set_pair::<First, Second>(data);
    }

    /// signal that singleton component was modified.
    ///
    /// # Arguments
    ///
    /// * `id` - The id of the component that was modified.
    ///
    /// # See also
    ///
    /// * C++ API: `world::modified`
    #[doc(alias = "world::modified")]
    #[inline(always)]
    pub fn modified_id(&self, id: impl Into<Entity>) {
        let id = id.into();
        EntityView::new_from(self, id).modified_id(id);
    }

    /// Signal that singleton component was modified.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type of the component that was modified.
    ///
    /// # See also
    ///
    /// * C++ API: `world::modified`
    #[doc(alias = "world::modified")]
    #[inline(always)]
    pub fn modified<T>(&self)
    where
        T: ComponentId,
    {
        self.modified_id(T::get_id(self));
    }

    /// gets a mutable or immutable singleton component and/or relationship(s) from the world.
    /// Only one singleton component at a time is retrievable, but you can call this function multiple times within the callback.
    /// each component type must be marked `&` or `&mut` to indicate if it is mutable or not.
    /// use `Option` wrapper to indicate if the component is optional.
    ///
    /// - `try_get` assumes when not using `Option` wrapper, that the entity has the component. If it does not, it will not run the callback.
    /// If unsure and you still want to have the callback be ran, use `Option` wrapper instead.
    ///
    /// # Note
    ///
    /// - You cannot get single component tags with this function, use `has` functionality instead.
    /// - You can only get relationships with a payload, so where one is not a tag / not a zst.
    /// tag relationships, use `has` functionality instead.
    /// - This causes the table to lock where the entity belongs to to prevent invalided references, see #Panics.
    /// The lock is dropped at the end of the callback.
    ///
    /// # Panics
    ///
    /// - This will panic if within the callback you do any operation that could invalidate the reference.
    /// This happens when the entity is moved to a different table in memory. Such as adding, removing components or
    /// creating/deleting entities where the entity belongs to the same table (which could cause a table grow operation).
    /// In case you need to do such operations, you can either do it after the get operation or defer the world with `world.defer_begin()`.
    ///
    /// # Returns
    ///
    /// - If the callback has ran.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)]
    /// struct Tag;
    ///
    /// #[derive(Component)]
    /// pub struct Position {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// world.set(Position { x: 10.0, y: 20.0 });
    /// world.set_pair::<Tag, Position>(Position { x: 30.0, y: 40.0 });
    ///    
    /// let has_run = world.try_get::<&Position>(|pos| {
    ///    assert_eq!(pos.x, 10.0);
    /// });
    /// assert!(has_run);
    ///
    /// let has_run = world.try_get::<&mut(Tag,Position)>(|pos| {
    ///     assert_eq!(pos.x, 30.0);
    /// });
    /// assert!(has_run);
    ///
    /// ```
    pub fn try_get<T: GetTupleTypeOperation>(
        &self,
        callback: impl for<'e> FnOnce(T::ActualType<'e>),
    ) -> bool
    where
        T::OnlyType: FlecsCastType,
    {
        let entity = EntityView::new_from(
            self,
            <<T::OnlyType as FlecsCastType>::CastType as IntoComponentId>::get_id(self),
        );
        entity.try_get::<T>(callback)
    }

    /// gets a mutable or immutable singleton component and/or relationship(s) from the world.
    /// Only one singleton component at a time is retrievable, but you can call this function multiple times within the callback.
    /// each component type must be marked `&` or `&mut` to indicate if it is mutable or not.
    /// use `Option` wrapper to indicate if the component is optional.
    ///
    /// # Note
    ///
    /// - You cannot get single component tags with this function, use `has` functionality instead.
    /// - You can only get relationships with a payload, so where one is not a tag / not a zst.
    /// tag relationships, use `has` functionality instead.
    /// - This causes the table to lock where the entity belongs to to prevent invalided references, see #Panics.
    /// The lock is dropped at the end of the callback.
    ///
    /// # Panics
    ///
    /// - This will panic if within the callback you do any operation that could invalidate the reference.
    /// This happens when the entity is moved to a different table in memory. Such as adding, removing components or
    /// creating/deleting entities where the entity belongs to the same table (which could cause a table grow operation).
    /// In case you need to do such operations, you can either do it after the get operation or defer the world with `world.defer_begin()`.
    ///
    /// - `get` assumes when not using `Option` wrapper, that the entity has the component.
    /// This will panic if the entity does not have the component. If unsure, use `Option` wrapper or `try_get` function instead.
    /// `try_get` does not run the callback if the entity does not have the component that isn't marked `Option`.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)] struct Tag;
    ///
    /// #[derive(Component)]
    /// pub struct Position {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// world.set(Position { x: 10.0, y: 20.0 });
    /// world.set_pair::<Tag, Position>(Position { x: 30.0, y: 40.0 });
    ///    
    /// world.get::<&Position>(|pos| {
    ///     assert_eq!(pos.x, 10.0);
    /// });
    /// world.get::<&mut(Tag,Position)>(|pos| {
    ///     assert_eq!(pos.x, 30.0);    
    /// });
    /// ```
    pub fn get<T: GetTupleTypeOperation>(&self, callback: impl for<'e> FnOnce(T::ActualType<'e>))
    where
        T::OnlyType: FlecsCastType,
    {
        let entity = EntityView::new_from(
            self,
            <<T::OnlyType as FlecsCastType>::CastType as IntoComponentId>::get_id(self),
        );
        entity.get::<T>(callback);
    }

    /// Clones a singleton component and/or relationship from the world and returns it.
    /// each component type must be marked `&`. This helps Rust type checker to determine if it's a relationship.
    /// use `Option` wrapper to indicate if the component is optional.
    /// use `()` tuple format when getting multiple components.
    ///
    /// # Note
    ///
    /// - You cannot clone component tags with this function.
    /// - You can only clone relationships with a payload, so where one is not a tag / not a zst.
    ///
    /// # Panics
    ///
    /// - This will panic if the world does not have the singleton component that isn't marked `Option`.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)] struct Tag;
    ///
    /// #[derive(Component, Clone)]
    /// pub struct Position {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// #[derive(Component, Clone)]
    /// pub struct Velocity {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// world.set(Position { x: 10.0, y: 20.0 });
    /// world.set_pair::<Tag, Position>(Position { x: 30.0, y: 40.0 });
    ///    
    /// let pos = world.cloned::<&Position>();
    /// assert_eq!(pos.x, 10.0);
    ///
    /// let tag_pos = world.cloned::<&(Tag, Position)>();
    /// assert_eq!(tag_pos.x, 30.0);
    ///
    /// let vel = world.cloned::<Option<&Velocity>>();
    /// assert!(vel.is_none());
    ///
    /// ```
    pub fn cloned<T: ClonedTupleTypeOperation>(&self) -> T::ActualType
    where
        T::OnlyType: FlecsCastType,
    {
        let entity = EntityView::new_from(
            self,
            <<T::OnlyType as FlecsCastType>::CastType as IntoComponentId>::get_id(self),
        );
        entity.cloned::<T>()
    }

    /// gets mutable or immutable component(s) and/or relationship(s) from the world in a callback and return a value.
    /// each component type must be marked `&` or `&mut` to indicate if it is mutable or not.
    /// use `Option` wrapper to indicate if the component is optional.
    ///
    /// - `try_map` assumes when not using `Option` wrapper, that the entity has the component. If it does not, it will not run the callback and return None.
    /// If unsure and you still want to have the callback be ran, use `Option` wrapper instead.
    ///
    /// # Note
    ///
    /// - You cannot get single component tags with this function, use `has` functionality instead.
    /// - You can only get relationships with a payload, so where one is not a tag / not a zst.
    /// tag relationships, use `has` functionality instead.
    /// - This causes the table to lock where the entity belongs to to prevent invalided references, see #Panics.
    /// The lock is dropped at the end of the callback.
    ///
    /// # Panics
    ///
    /// - This will panic if within the callback you do any operation that could invalidate the reference.
    /// This happens when the entity is moved to a different table in memory. Such as adding, removing components or
    /// creating/deleting entities where the entity belongs to the same table (which could cause a table grow operation).
    /// In case you need to do such operations, you can either do it after the get operation or defer the world with `world.defer_begin()`.
    ///
    /// # Returns
    ///
    /// - a Some(value) if the callback has ran. Where the type of value is specified in `Return` generic (can be elided).
    /// None if the callback has not ran.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)] struct Tag;
    ///
    /// #[derive(Component)]
    /// pub struct Velocity {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// #[derive(Component)]
    /// pub struct Position {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// let entity = world.entity()
    ///                   .set(Position { x: 10.0, y: 20.0 })
    ///                   .set_pair::<Tag, Position>(Position { x: 30.0, y: 40.0 });
    ///    
    /// let pos_x = entity.try_map::<&Position, _>(|(pos)| {
    ///     assert_eq!(pos.x, 10.0);
    ///     Some(pos.x)
    /// });
    /// assert!(pos_x.is_some());
    /// assert_eq!(pos_x.unwrap(), 10.0);
    ///
    /// let is_pos_x_10 = entity.try_map::<(Option<&Velocity>, &Position), _>( |(tag, pos)| {
    ///     assert_eq!(pos.x, 10.0);
    ///     assert!(tag.is_none());
    ///     Some(pos.x == 10.0)
    /// });
    /// assert!(is_pos_x_10.is_some());
    /// assert!(is_pos_x_10.unwrap());
    ///
    /// // no return type
    /// let has_run = entity.try_map::<(&mut(Tag,Position), &Position),_>(|(tag_pos_rel, pos)| {
    ///     assert_eq!(pos.x, 10.0);
    ///     assert_eq!(tag_pos_rel.x, 30.0);
    ///     Some(())
    /// });
    /// assert!(has_run.is_some());
    ///
    /// ```
    pub fn try_map<T: GetTupleTypeOperation, Return>(
        &self,
        callback: impl for<'e> FnOnce(T::ActualType<'e>) -> Option<Return>,
    ) -> Option<Return>
    where
        T::OnlyType: FlecsCastType,
    {
        let entity = EntityView::new_from(
            self,
            <<T::OnlyType as FlecsCastType>::CastType as IntoComponentId>::get_id(self),
        );
        entity.try_map::<T, Return>(callback)
    }

    /// gets mutable or immutable singleton component and/or relationship from the world in a callback.
    /// each component type must be marked `&` or `&mut` to indicate if it is mutable or not.
    /// use `Option` wrapper to indicate if the component is optional.
    /// use `()` tuple format when getting multiple components.
    ///
    /// # Note
    ///
    /// - You cannot get single component tags with this function, use `has` functionality instead.
    /// - You can only get relationships with a payload, so where one is not a tag / not a zst.
    /// tag relationships, use `has` functionality instead.
    /// - This causes the table to lock where the entity belongs to to prevent invalided references, see #Panics.
    /// The lock is dropped at the end of the callback.
    ///
    /// # Panics
    ///
    /// - This will panic if within the callback you do any operation that could invalidate the reference.
    /// This happens when the entity is moved to a different table in memory. Such as adding, removing components or
    /// creating/deleting entities where the entity belongs to the same table (which could cause a table grow operation).
    /// In case you need to do such operations, you can either do it after the get operation or defer the world with `world.defer_begin()`.
    ///
    /// - `get` assumes when not using `Option` wrapper, that the entity has the component.
    /// This will panic if the entity does not have the component. If unsure, use `Option` wrapper or `try_get` function instead.
    /// `try_get` does not run the callback if the entity does not have the component that isn't marked `Option`.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component)] struct Tag;
    ///
    /// #[derive(Component)]
    /// pub struct Velocity {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// #[derive(Component)]
    /// pub struct Position {
    ///     pub x: f32,
    ///     pub y: f32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// let entity = world.entity()
    ///                   .set(Position { x: 10.0, y: 20.0 })
    ///                   .set_pair::<Tag, Position>(Position { x: 30.0, y: 40.0 });
    ///
    /// let position_parent = Position { x: 20.0, y: 30.0 };
    ///
    /// let pos_actual = entity.map::<&Position, _>(|pos| {
    ///     assert_eq!(pos.x, 10.0);
    ///     // Calculate actual position
    ///     Position {
    ///         x: pos.x + position_parent.x,
    ///         y: pos.y + position_parent.y,
    ///     }
    /// });
    ///
    /// let pos_x = entity.map::<(Option<&Velocity>, &Position),_>( |(vel, pos)| {
    ///     assert_eq!(pos.x, 10.0);
    ///     assert!(vel.is_none());
    ///     pos.x
    /// });
    /// assert_eq!(pos_x, 10.0);
    ///
    /// let is_x_10 = entity.map::<(&mut(Tag,Position), &Position), _>(|(tag_pos_rel, pos)| {
    ///     assert_eq!(pos.x, 10.0);
    ///     assert_eq!(tag_pos_rel.x, 30.0);
    ///     pos.x == 10.0
    /// });
    /// assert!(is_x_10);
    ///
    /// ```
    pub fn map<T: GetTupleTypeOperation, Return>(
        &self,
        callback: impl for<'e> FnOnce(T::ActualType<'e>) -> Return,
    ) -> Return
    where
        T::OnlyType: FlecsCastType,
    {
        let entity = EntityView::new_from(
            self,
            <<T::OnlyType as FlecsCastType>::CastType as IntoComponentId>::get_id(self),
        );
        entity.map::<T, Return>(callback)
    }

    /// Get a reference to a singleton component.
    ///
    /// A reference allows for quick and safe access to a component value, and is
    /// a faster alternative to repeatedly calling `get` for the same component.
    ///
    /// - `T`: Component for which to get a reference.
    ///
    /// Returns: The reference singleton component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_ref`
    // #[doc(alias = "world::get_ref")]
    // #[inline(always)]
    pub fn get_ref<T>(&self) -> CachedRef<T::UnderlyingType>
    where
        T: ComponentId + NotEmptyComponent,
        T::UnderlyingType: NotEmptyComponent,
    {
        EntityView::new_from(self, T::get_id(self)).get_ref::<T>()
    }

    /// Get singleton entity for type.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to get the singleton entity for.
    ///
    /// # Returns
    ///
    /// The entity representing the component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::singleton`
    #[doc(alias = "world::singleton")]
    #[inline(always)]
    pub fn singleton<T: ComponentId>(&self) -> EntityView {
        EntityView::new_from(self, T::get_id(self))
    }

    /// Gets the target for a given pair from a singleton entity.
    ///
    /// This operation returns the target for a given pair. The optional
    /// `index` can be used to iterate through targets, in case the entity has
    /// multiple instances for the same relationship.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `index` - The index (None for the first instance of the relationship).
    ///
    /// # See also
    ///
    /// * C++ API: `world::target`
    #[doc(alias = "world::target")]
    pub fn target<First>(&self, index: Option<i32>) -> EntityView
    where
        First: ComponentId,
    {
        let id = First::get_id(self);
        EntityView::new_from(self, unsafe {
            sys::ecs_get_target(self.raw_world.as_ptr(), id, id, index.unwrap_or(0))
        })
    }

    /// Retrieves the target for a given pair from a singleton entity.
    ///
    /// This operation fetches the target associated with a specific pair. An optional
    /// `index` parameter allows iterating through multiple targets if the entity
    /// has more than one instance of the same relationship.
    ///
    /// # Arguments
    ///
    /// * `first` - The first element of the pair for which to retrieve the target.
    /// * `index` - The index (0 for the first instance of the relationship).
    ///
    /// # See also
    ///
    /// * C++ API: `world::target`
    #[doc(alias = "world::target")]
    pub fn target_id(&self, relationship: impl Into<Entity>, index: Option<usize>) -> EntityView {
        let relationship = *relationship.into();
        EntityView::new_from(self, unsafe {
            sys::ecs_get_target(
                self.raw_world.as_ptr(),
                relationship,
                relationship,
                index.unwrap_or(0) as i32,
            )
        })
    }

    /// Check if world has the provided id.
    ///
    /// # Arguments
    ///
    /// * `id`: The id to check of a pair, entity or component.
    ///
    /// # Returns
    ///
    /// True if the world has the provided id, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `world::has`
    #[doc(alias = "world::has")]
    #[inline(always)]
    pub fn has_id(&self, id: impl IntoId) -> bool {
        let id = *id.into();
        EntityView::new_from(self, id).has_id(id)
    }

    /// Check if world has the provided type (enum,pair,struct).
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type to check.
    ///
    /// # Returns
    ///
    /// True if the world has the provided type, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `world::has`
    #[doc(alias = "world::has")]
    #[inline(always)]
    pub fn has<T>(&self) -> bool
    where
        T: IntoComponentId,
    {
        EntityView::new_from(self, T::get_id(self)).has::<T>()
    }

    /// Check if world has the provided enum constant.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The enum type.
    ///
    /// # Arguments
    ///
    /// * `constant` - The enum constant to check.
    ///
    /// # Returns
    ///
    /// True if the world has the provided constant, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `world::has`
    #[doc(alias = "world::has")]
    #[inline(always)]
    pub fn has_enum<T>(&self, constant: T) -> bool
    where
        T: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        let id = T::get_id(self);
        EntityView::new_from(self, id).has_enum_id::<T>(id, constant)
    }

    /// Add a singleton component by id.
    /// id can be a component, entity or pair id.
    ///
    /// # Arguments
    ///
    /// * `id`: The id of the component to add.
    ///
    /// # Returns
    ///
    /// `EntityView` handle to the singleton component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::add`
    #[doc(alias = "world::add")]
    #[inline(always)]
    pub fn add_id<T>(&self, id: T) -> EntityView
    where
        T: IntoId,
    {
        let id = *id.into();
        // this branch will compile out in release mode
        if T::IS_PAIR {
            let first_id = id.get_id_first();
            EntityView::new_from(self, first_id).add_id(id)
        } else {
            EntityView::new_from(self, id).add_id(id)
        }
    }

    /// Add a singleton component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component to add.
    ///
    /// # Returns
    ///
    /// `EntityView` handle to the singleton component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::add`
    #[doc(alias = "world::add")]
    #[inline(always)]
    pub fn add<T: IntoComponentId + EmptyComponent>(&self) -> EntityView {
        if T::IS_PAIR {
            let first_id = <T::First as ComponentId>::get_id(self);
            EntityView::new_from(self, first_id).add::<T>()
        } else {
            let id = T::get_id(self);
            EntityView::new_from(self, id).add::<T>()
        }
    }

    /// Add a singleton enum component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The enum component to add.
    ///
    /// # Returns
    ///
    /// `EntityView` handle to the singleton enum component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::add`
    #[doc(alias = "world::add")]
    #[inline(always)]
    pub fn add_enum<T: ComponentId + ComponentType<Enum> + CachedEnumData>(
        &self,
        enum_value: T,
    ) -> EntityView {
        EntityView::new_from(self, T::get_id(self)).add_enum::<T>(enum_value)
    }

    /// Add a singleton pair by first id.
    ///
    /// # Safety
    ///
    /// Caller must ensure the id is a non ZST types. Otherwise it could cause the payload to have uninitialized data.
    ///
    /// # Returns
    ///
    /// `EntityView` handle to the singleton pair.
    #[inline(always)]
    pub fn add_second<Second: ComponentId + EmptyComponent>(
        &self,
        first: impl Into<Entity>,
    ) -> EntityView {
        EntityView::new_from(self, Second::get_id(self)).add_second::<Second>(first)
    }

    /// Add a singleton pair by second id.
    ///
    /// # Safety
    ///
    /// Caller must ensure the id is a non ZST types. Otherwise it could cause the payload to have uninitialized data.
    ///
    /// # Returns
    ///
    /// `EntityView` handle to the singleton pair.
    ///
    /// # See also
    ///
    /// * C++ API: `world::add`
    #[doc(alias = "world::add")]
    #[inline(always)]
    pub fn add_first<First: ComponentId + EmptyComponent>(
        &self,
        second: impl Into<Entity>,
    ) -> EntityView {
        EntityView::new_from(self, First::get_id(self)).add_first::<First>(second)
    }

    /// Add a singleton pair with enum tag.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    /// * `Second` - The second element of the pair of type enum.
    ///
    /// # Arguments
    ///
    /// * `enum_value`: The enum value to add.
    ///
    /// # Returns
    ///
    /// `EntityView` handle to the singleton pair.
    ///
    /// # See also
    ///
    /// * C++ API: `world::add`
    #[doc(alias = "world::add")]
    #[inline(always)]
    pub fn add_pair_enum<First, Second>(&self, enum_value: Second) -> EntityView
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        EntityView::new_from(self, First::get_id(self)).add_pair_enum::<First, Second>(enum_value)
    }

    /// Remove singleton component by id.
    /// id can be a component, entity or pair id.
    ///
    /// # Arguments
    ///
    /// * `id`: The id of the component to remove.
    ///
    /// # See also
    ///
    /// * C++ API: `world::remove`
    #[doc(alias = "world::remove")]
    pub fn remove_id<T>(&self, id: T) -> EntityView
    where
        T: IntoId,
    {
        let id = *id.into();
        if T::IS_PAIR {
            let first_id = id.get_id_first();
            EntityView::new_from(self, first_id).remove_id(id)
        } else {
            EntityView::new_from(self, id).remove_id(id)
        }
    }

    /// Remove singleton component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component to remove.
    ///
    /// # See also
    ///
    /// * C++ API: `world::remove`
    #[doc(alias = "world::remove")]
    #[inline(always)]
    pub fn remove<T: IntoComponentId>(&self) {
        if T::IS_PAIR {
            let first_id = <T::First as ComponentId>::get_id(self);
            EntityView::new_from(self, first_id).remove::<T>();
        } else {
            EntityView::new_from(self, T::get_id(self)).remove::<T>();
        }
    }

    /// Remove singleton pair with enum tag.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    /// * `Second` - The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `enum_value` - The enum value to remove.
    ///
    /// # See also
    ///
    /// * C++ API: `world::remove`
    #[doc(alias = "world::remove")]
    #[inline(always)]
    pub fn remove_enum_tag<First, Second>(&self, enum_value: Second)
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        EntityView::new_from(self, First::get_id(self))
            .remove_enum_tag::<First, Second>(enum_value);
    }

    /// Remove singleton pair by first id.
    ///
    /// # Type Parameters
    ///
    /// * `Second` - The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `world::remove`
    #[doc(alias = "world::remove")]
    #[inline(always)]
    pub fn remove_second<Second: ComponentId>(&self, first: impl Into<Entity>) {
        EntityView::new_from(self, Second::get_id(self)).remove_second::<Second>(first);
    }

    /// Remove singleton pair by second id.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `second`: The second element of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `world::remove`
    #[doc(alias = "world::remove")]
    #[inline(always)]
    pub fn remove_first<First: ComponentId>(&self, second: impl Into<Entity>) {
        EntityView::new_from(self, First::get_id(self)).remove_first::<First>(second);
    }

    /// Iterate entities in root of world
    ///
    /// # Arguments
    ///
    /// * `func` - The function invoked for each child. Must match the signature `FnMut(EntityView)`.
    ///
    /// # See also
    ///
    /// * C++ API: `world::children`
    #[doc(alias = "world::children")]
    #[inline(always)]
    pub fn each_child(&self, callback: impl FnMut(EntityView)) {
        EntityView::new(self).each_child(callback);
    }

    /// create alias for component
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to create an alias for.
    ///
    /// # Arguments
    ///
    /// * `alias` - The alias to create.
    ///
    /// # Returns
    ///
    /// The entity representing the component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::use`
    #[doc(alias = "world::use")]
    #[inline(always)]
    pub fn set_alias_component<T: ComponentId>(&self, alias: &str) -> EntityView {
        let alias = compact_str::format_compact!("{}\0", alias);

        let id = T::get_id(self);
        if alias.is_empty() {
            unsafe {
                sys::ecs_set_alias(
                    self.raw_world.as_ptr(),
                    id,
                    sys::ecs_get_name(self.raw_world.as_ptr(), id),
                );
            };
        } else {
            unsafe { sys::ecs_set_alias(self.raw_world.as_ptr(), id, alias.as_ptr() as *const _) };
        }
        EntityView::new_from(self, id)
    }

    /// create alias for entity by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the entity to create an alias for.
    /// * `alias` - The alias to create.
    ///
    /// # Returns
    ///
    /// The entity found by name.
    ///
    /// # See also
    ///
    /// * C++ API: `world::use`
    #[doc(alias = "world::use")]
    #[inline(always)]
    pub fn set_alias_entity_by_name(&self, name: &str, alias: &str) -> EntityView {
        let name = compact_str::format_compact!("{}\0", name);
        let alias = compact_str::format_compact!("{}\0", alias);

        let id = unsafe {
            sys::ecs_lookup_path_w_sep(
                self.raw_world.as_ptr(),
                0,
                name.as_ptr() as *const _,
                SEPARATOR.as_ptr(),
                SEPARATOR.as_ptr(),
                true,
            )
        };
        ecs_assert!(id != 0, FlecsErrorCode::InvalidParameter);
        unsafe { sys::ecs_set_alias(self.raw_world.as_ptr(), id, alias.as_ptr() as *const _) };
        EntityView::new_from(self, id)
    }

    /// create alias for entity
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to create an alias for.
    /// * `alias` - The alias to create.
    ///
    /// # See also
    ///
    /// * C++ API: `world::use`
    #[doc(alias = "world::use")]
    #[inline(always)]
    pub fn set_alias_entity(&self, entity: impl Into<Entity>, alias: &str) {
        let alias = compact_str::format_compact!("{}\0", alias);

        let entity = *entity.into();
        if alias.is_empty() {
            unsafe {
                sys::ecs_set_alias(
                    self.raw_world.as_ptr(),
                    entity,
                    sys::ecs_get_name(self.raw_world.as_ptr(), entity),
                );
            };
        } else {
            unsafe {
                sys::ecs_set_alias(self.raw_world.as_ptr(), entity, alias.as_ptr() as *const _);
            };
        }
    }

    /// Count entities with the provided id.
    ///
    /// # Arguments
    ///
    /// * `id` - The id to count.
    ///
    /// # Returns
    ///
    /// The number of entities with the provided id.
    ///
    /// # See also
    ///
    /// * C++ API: `world::count`
    #[doc(alias = "world::count")]
    pub fn count_id(&self, id: impl IntoId) -> i32 {
        unsafe { sys::ecs_count_id(self.raw_world.as_ptr(), *id.into()) }
    }

    /// Count entities with the provided component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component to count.
    ///
    /// # Returns
    ///
    /// The number of entities with the provided component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::count`
    #[doc(alias = "world::count")]
    pub fn count<T: IntoComponentId>(&self) -> i32 {
        self.count_id(T::get_id(self))
    }

    /// Count entities with the provided pair.
    ///
    /// # Type Parameters
    ///
    /// * `Second` - The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first` - The ID of the first element of the pair.
    ///
    /// # Returns
    ///
    /// The number of entities with the provided pair.
    ///
    /// # See also
    ///
    /// * C++ API: `world::count`
    #[doc(alias = "world::count")]
    pub fn count_second<Second: ComponentId>(&self, first: impl Into<Entity>) -> i32 {
        self.count_id((first.into(), Second::get_id(self)))
    }

    /// Count entities with the provided pair.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `second` - The ID of the second element of the pair.
    ///
    /// # Returns
    ///
    /// The number of entities with the provided pair.
    ///
    /// # See also
    ///
    /// * C++ API: `world::count`
    #[doc(alias = "world::count")]
    pub fn count_first<First: ComponentId>(&self, second: impl Into<Entity>) -> i32 {
        self.count_id((First::get_id(self), second.into()))
    }

    /// Count entities with the provided enum constant.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The enum type.
    ///
    /// # Arguments
    ///
    /// * `constant` - The enum constant to count.
    ///
    /// # Returns
    ///
    /// The number of entities with the provided enum constant.
    ///
    /// # See also
    ///
    /// * C++ API: `world::count`
    #[doc(alias = "world::count")]
    pub fn count_enum<T: ComponentId + ComponentType<Enum> + CachedEnumData>(
        &self,
        enum_value: T,
    ) -> i32 {
        unsafe {
            sys::ecs_count_id(
                self.raw_world.as_ptr(),
                *(enum_value.get_id_variant(self).id),
            )
        }
    }

    /// Count entities with the provided pair enum tag.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    /// * `Second` - The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `enum_value` - The enum value to count.
    ///
    /// # Returns
    ///
    /// The number of entities with the provided pair enum tag.
    ///
    /// # See also
    ///
    /// * C++ API: `world::count`
    #[doc(alias = "world::count")]
    pub fn count_enum_tag_pair<First, Second>(&self, enum_value: Second) -> i32
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        unsafe {
            sys::ecs_count_id(
                self.raw_world.as_ptr(),
                ecs_pair(First::get_id(self), *(enum_value.get_id_variant(self)).id),
            )
        }
    }

    /// All entities created in function are created in scope. All operations
    /// called in function (such as lookup) are relative to scope.
    ///
    /// # Arguments
    ///
    /// * `parent_id` - The id of the scope to use.
    /// * `func` - The function to run.
    ///
    /// # See also
    ///
    /// * C++ API: `world::scope`
    #[doc(alias = "world::scope")]
    pub fn run_in_scope_with_id(&self, parent_id: impl Into<Entity>, mut func: impl FnMut()) {
        let prev: IdT = unsafe { sys::ecs_set_scope(self.raw_world.as_ptr(), *parent_id.into()) };
        func();
        unsafe {
            sys::ecs_set_scope(self.raw_world.as_ptr(), prev);
        }
    }

    /// All entities created in function are created in scope. All operations
    /// called in function (such as lookup) are relative to scope.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to use as scope / parent.
    ///
    /// # Arguments
    ///
    /// * `func` - The function to run.
    ///
    /// # See also
    ///
    /// * C++ API: `world::scope`
    #[doc(alias = "world::scope")]
    pub fn run_in_scope_with<T: ComponentId>(&self, func: impl FnMut()) {
        self.run_in_scope_with_id(T::get_id(self), func);
    }

    /// Use provided scope for operations ran on returned world.
    /// Operations need to be ran in a single statement
    ///
    /// # Arguments
    ///
    /// * `parent_id` - The id of the scope to use.
    ///
    /// # Returns
    ///
    /// A scoped world.
    ///
    /// # See also
    ///
    /// * C++ API: `world::scope`
    #[doc(alias = "world::scope")]
    pub fn scope_id(&self, parent_id: impl IntoId, mut f: impl FnMut(&World)) {
        let previous_scope = self.set_scope_id(parent_id);
        f(self);
        self.set_scope_id(previous_scope);
    }

    /// Use provided scope for operations ran on returned world.
    /// Operations need to be ran in a single statement
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to use as scope.
    ///
    /// # Returns
    ///
    /// A scoped world.
    ///
    /// # See also
    ///
    /// * C++ API: `world::scope`
    #[doc(alias = "world::scope")]
    pub fn scope<T: ComponentId>(&self, mut f: impl FnMut(&World)) {
        let previous_scope = self.set_scope_id(T::get_id(self));
        f(self);
        self.set_scope_id(previous_scope);
    }

    /// Use provided scope of name for operations ran on returned world.
    /// Operations need to be ran in a single statement
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the scope to use.
    ///
    /// # Returns
    ///
    /// A scoped world.
    ///
    /// # See also
    ///
    /// * C++ API: `world::scope`
    #[doc(alias = "world::scope")]
    pub fn scope_name(&self, name: &str, f: impl FnMut(&World)) {
        self.scope_id(EntityView::new_named(self, name).id, f);
    }

    /// all entities created in function are created with id
    ///
    /// # Arguments
    ///
    /// * `id`: The id to create entities with.
    /// * `func`: The function to run.
    ///
    /// # See also
    ///
    /// * C++ API: `world::with`
    #[doc(alias = "world::with")]
    pub fn with_id(&self, id: impl IntoId, mut func: impl FnMut()) {
        let prev: IdT = unsafe { sys::ecs_set_with(self.raw_world.as_ptr(), *id.into()) };
        func();
        unsafe {
            sys::ecs_set_with(self.raw_world.as_ptr(), prev);
        }
    }

    /// Entities created in function are created with component
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component type.
    ///
    /// # Arguments
    ///
    /// * `func`: The function to run.
    ///
    /// # See also
    ///
    /// * C++ API: `world::with`
    #[doc(alias = "world::with")]
    pub fn with<T: IntoComponentId>(&self, func: impl FnMut()) {
        self.with_id(T::get_id(self), func);
    }

    /// Entities created in function are created with pair
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The first element of the pair.
    /// * `func`: The function to run.
    ///
    /// # See also
    ///
    /// * C++ API: `world::with`
    #[doc(alias = "world::with")]
    pub fn with_second<Second: ComponentId>(&self, first: impl Into<Entity>, func: impl FnMut()) {
        self.with_id(ecs_pair(*first.into(), Second::get_id(self)), func);
    }

    /// Entities created in function are created with pair
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `second`: The second element of the pair.
    /// * `func`: The function to run.
    ///
    /// # See also
    ///
    /// * C++ API: `world::with`
    #[doc(alias = "world::with")]
    pub fn with_first<First: ComponentId>(&self, second: impl Into<Entity>, func: impl FnMut()) {
        self.with_id(ecs_pair(First::get_id(self), *second.into()), func);
    }

    /// Entities created in function are created with enum constant
    ///
    /// # Type Parameters
    ///
    /// * `T`: The enum type.
    ///
    /// # Arguments
    ///
    /// * `enum_value`: The enum value to give the entity.
    /// * `func`: The function to run.
    ///
    /// # See also
    ///
    /// * C++ API: `world::with`
    #[doc(alias = "world::with")]
    pub fn with_enum<T>(&self, enum_value: T, func: impl FnMut())
    where
        T: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        self.with_id(enum_value.get_id_variant(self), func);
    }

    /// Entities created in function are created with enum tag pair
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    /// * `Second`: The enum component type.
    ///
    /// # Arguments
    ///
    /// * `enum_value`: The enum value to give the entity.
    /// * `func`: The function to run.
    ///
    /// # See also
    ///
    /// * C++ API: `world::with`
    #[doc(alias = "world::with")]
    pub fn with_enum_pair<First, Second>(&self, enum_value: Second, func: impl FnMut())
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        self.with_id(
            ecs_pair(First::get_id(self), **(enum_value.get_id_variant(self))),
            func,
        );
    }

    /// Delete all entities with the given id
    ///
    /// # Arguments
    ///
    /// * `id`: The id to delete.
    ///
    /// # See also
    ///
    /// * C++ API: `world::delete_with`
    #[doc(alias = "world::delete_with")]
    pub fn delete_with_id(&self, id: impl IntoId) {
        unsafe {
            sys::ecs_delete_with(self.raw_world.as_ptr(), *id.into());
        }
    }

    /// Delete all entities with the given component
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component type to delete.
    ///
    /// # See also
    ///
    /// * C++ API: `world::delete_with`
    #[doc(alias = "world::delete_with")]
    pub fn delete_entities_with<T: IntoComponentId>(&self) {
        self.delete_with_id(T::get_id(self));
    }

    /// Delete all entities with the given pair
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The first id of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `world::delete_with`
    #[doc(alias = "world::delete_with")]
    pub fn delete_with_second<Second: ComponentId>(&self, first: impl Into<Entity>) {
        self.delete_with_id(ecs_pair(*first.into(), Second::get_id(self)));
    }

    /// Delete all entities with the given pair
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `second`: The second id of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `world::delete_with`
    #[doc(alias = "world::delete_with")]
    pub fn delete_entities_with_second_id<First: ComponentId>(&self, second: impl Into<Entity>) {
        self.delete_with_id(ecs_pair(First::get_id(self), *second.into()));
    }

    /// Delete all entities with the given enum constant
    ///
    /// # Type Parameters
    ///
    /// * `T`: The enum type.
    ///
    /// # Arguments
    ///
    /// * `enum_value`: The enum value to filter against for deletion.
    ///
    /// # See also
    ///
    /// * C++ API: `world::delete_with`
    #[doc(alias = "world::delete_with")]
    pub fn delete_with_enum<T: ComponentId + ComponentType<Enum> + CachedEnumData>(
        &self,
        enum_value: T,
    ) {
        self.delete_with_id(enum_value.get_id_variant(self));
    }

    /// Delete all entities with the given enum tag pair / relationship
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    /// * `Second`: The enum component type.
    ///
    /// # Arguments
    ///
    /// * `enum_value`: The enum value to filter against for deletion.
    ///
    /// # See also
    ///
    /// * `world::delete_with`
    pub fn delete_with_enum_pair<First, Second>(&self, enum_value: Second)
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        self.delete_with_id(ecs_pair(
            First::get_id(self),
            **enum_value.get_id_variant(self),
        ));
    }

    /// Remove all instances of the given id from entities
    ///
    /// # Arguments
    ///
    /// * `id`: The id to remove.
    ///
    /// # See also
    ///
    /// * C++ API: `world::remove_all`
    #[doc(alias = "world::remove_all")]
    pub fn remove_all_id(&self, id: impl IntoId) {
        unsafe {
            sys::ecs_remove_all(self.raw_world.as_ptr(), *id.into());
        }
    }

    /// Remove all instances of the given component from entities
    ///
    /// # Type Parameters
    ///
    /// * `T`: The component type to remove.
    ///
    /// # See also
    ///
    /// * C++ API: `world::remove_all`
    #[doc(alias = "world::remove_all")]
    pub fn remove_all<T: IntoComponentId>(&self) {
        self.remove_all_id(T::get_id(self));
    }

    /// Remove all instances of the given pair from entities
    ///
    /// # Type Parameters
    ///
    /// * `Second`: The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first`: The first id of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `world::remove_all`
    #[doc(alias = "world::remove_all")]
    pub fn remove_all_second<Second: ComponentId>(&self, first: impl Into<Entity>) {
        self.remove_all_id((first.into(), Second::get_id(self)));
    }

    /// Remove all instances of the given pair from entities
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `second`: The second id of the pair.
    ///
    /// # See also
    ///
    /// * C++ API: `world::remove_all`
    #[doc(alias = "world::remove_all")]
    pub fn remove_all_first<First: ComponentId>(&self, second: impl Into<Entity>) {
        self.remove_all_id((First::get_id(self), second.into()));
    }

    /// Remove all instances with the given enum constant from entities
    ///
    /// # Type Parameters
    ///
    /// * `T`: The enum type.
    ///
    /// # Arguments
    ///
    /// * `enum_value`: The enum value to filter against for removal.
    ///
    /// # See also
    ///
    /// * C++ API: `world::remove_all`
    #[doc(alias = "world::remove_all")]
    pub fn remove_all_enum<T: ComponentId + ComponentType<Enum> + CachedEnumData>(
        &self,
        enum_value: T,
    ) {
        self.remove_all_id(enum_value.get_id_variant(self));
    }

    /// Remove all instances with the given enum tag pair / relationship from entities
    ///
    /// # Type Parameters
    ///
    /// * `First`: The first element of the pair.
    /// * `Second`: The enum component type.
    ///
    /// # Arguments
    ///
    /// * `enum_value`: The enum value to filter against for removal.
    ///
    /// # See also
    ///
    /// * C++ API: `world::remove_all`
    #[doc(alias = "world::remove_all")]
    pub fn remove_all_enum_pair<First, Second>(&self, enum_value: Second)
    where
        First: ComponentId,
        Second: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        self.remove_all_id((First::get_id(self), enum_value.get_id_variant(self)));
    }

    /// Defers all operations executed in the passed-in closure. If the world
    /// is already in deferred mode, does nothing.
    ///
    /// # Arguments
    ///
    /// * `func` - The closure to execute.
    ///
    /// # Examples
    #[cfg_attr(doctest, doc = " ````no_test")]
    /// ```
    /// world.defer(|| {
    ///     // deferred operations here
    /// });
    /// ```
    ///
    /// # See also
    ///
    /// * C++ API: `world::defer`
    #[doc(alias = "world::defer")]
    pub fn defer(&self, func: impl FnOnce()) {
        unsafe {
            sys::ecs_defer_begin(self.raw_world.as_ptr());
        }
        func();
        unsafe {
            sys::ecs_defer_end(self.raw_world.as_ptr());
        }
    }

    /// Suspends deferring of operations.
    ///
    /// # See also
    ///
    /// * C++ API: `world::defer_suspend`
    #[doc(alias = "world::defer_suspend")]
    pub fn defer_suspend(&self) {
        unsafe {
            sys::ecs_defer_suspend(self.raw_world.as_ptr());
        }
    }

    /// Resumes deferring of operations.
    ///
    /// # See also
    ///
    /// * C++ API: `world::defer_resume`
    #[doc(alias = "world::defer_resume")]
    pub fn defer_resume(&self) {
        unsafe {
            sys::ecs_defer_resume(self.raw_world.as_ptr());
        }
    }

    /// Checks if the given entity ID exists in the world.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to check.
    ///
    /// # Returns
    ///
    /// True if the entity exists, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `world::exists`
    #[doc(alias = "world::exists")]
    pub fn exists(&self, entity: impl Into<Entity>) -> bool {
        unsafe { sys::ecs_exists(self.raw_world.as_ptr(), *entity.into()) }
    }

    /// Checks if the given entity ID is alive in the world.
    ///
    /// # See also
    ///
    /// * C++ API: `world::is_alive`
    #[doc(alias = "world::is_alive")]
    pub fn is_alive(&self, entity: impl Into<Entity>) -> bool {
        unsafe { sys::ecs_is_alive(self.raw_world.as_ptr(), *entity.into()) }
    }

    /// Checks if the given entity ID is valid.
    /// Invalid entities cannot be used with API functions.
    ///
    /// # See also
    ///
    /// * C++ API: `world::is_valid`
    #[doc(alias = "world::is_valid")]
    pub fn is_valid(&self, entity: impl Into<Entity>) -> bool {
        unsafe { sys::ecs_is_valid(self.raw_world.as_ptr(), *entity.into()) }
    }

    /// Get alive entity for id.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to check
    ///
    /// # Returns
    ///
    /// The entity with the current generation. If the entity is not alive, this
    /// function will return an Entity of 0. Use `try_get_alive` if you want to
    /// return an `Option<EntityView>`.
    ///
    /// # See also
    ///
    /// * C++ API: `world::try_get_alive`
    #[doc(alias = "world::try_get_alive")]
    pub fn get_alive(&self, entity: impl Into<Entity>) -> EntityView {
        let entity = unsafe { sys::ecs_get_alive(self.raw_world.as_ptr(), *entity.into()) };

        EntityView::new_from(self, entity)
    }

    /// Get alive entity for id.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to check
    ///
    /// # Returns
    ///
    /// The entity with the current generation.
    /// If the entity is not alive, this function will return `None`.
    ///
    /// # See also
    ///
    /// * C++ API: `world::try_get_alive`
    #[doc(alias = "world::try_get_alive")]
    pub fn try_get_alive(&self, entity: impl Into<Entity>) -> Option<EntityView> {
        let entity = unsafe { sys::ecs_get_alive(self.raw_world.as_ptr(), *entity.into()) };
        if entity == 0 {
            None
        } else {
            Some(EntityView::new_from(self, entity))
        }
    }

    /// Ensures that entity with provided generation is alive.
    /// This operation will fail if an entity exists with the same id and a
    /// different, non-zero generation.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to ensure is alive.
    ///
    /// # Returns
    ///
    /// The entity with the provided generation.
    ///
    /// # See also
    ///
    /// * C++ API: `world::make_alive`
    #[doc(alias = "world::make_alive")]
    pub fn make_alive(&self, entity: impl Into<Entity>) -> EntityView {
        let entity = *entity.into();
        unsafe { sys::ecs_make_alive(self.raw_world.as_ptr(), entity) };
        EntityView::new_from(self, entity)
    }

    /// Run callback after completing frame
    ///
    /// # Arguments
    ///
    /// * `action` - The action to run.
    /// * `ctx` - The context to pass to the action.
    ///
    /// # See also
    ///
    /// * C++ API: `world::run_post_frame`
    #[doc(alias = "world::run_post_frame")]
    #[allow(clippy::not_unsafe_ptr_arg_deref)] // this doesn't actually deref the pointer
    pub fn run_post_frame(&self, action: sys::ecs_fini_action_t, ctx: *mut c_void) {
        unsafe {
            sys::ecs_run_post_frame(self.raw_world.as_ptr(), action, ctx);
        }
    }
}

/// `EntityView` mixin implementation
impl World {
    /// Convert enum constant to entity
    ///
    /// # Type Parameters
    ///
    /// * `T` - The enum type.
    ///
    /// # Arguments
    ///
    /// * `enum_value` - The enum value to convert.
    ///
    /// # Returns
    ///
    /// `EntityView` wrapping the id of the enum constant.
    ///
    /// # See also
    ///
    /// * C++ API: `world::entity`
    #[doc(alias = "world::entity")]
    #[doc(alias = "world::id")] //enum mixin implementation
    pub fn entity_from_enum<T>(&self, enum_value: T) -> EntityView
    where
        T: ComponentId + ComponentType<Enum> + CachedEnumData,
    {
        EntityView::new_from(self, enum_value.get_id_variant(self))
    }

    /// Create an entity that's associated with a type and name
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to associate with the new entity.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to use for the new entity.
    ///
    /// # See also
    ///
    /// * C++ API: `world::entity`
    #[doc(alias = "world::entity")]
    pub fn entity_from_named<'a, T: ComponentId>(&'a self, name: &str) -> EntityView<'a> {
        EntityView::new_from(self, T::register_explicit_named(self, name))
    }

    /// Create an entity that's associated with a type
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to associate with the new entity.
    ///
    /// # See also
    ///
    /// * C++ API: `world::entity`
    pub fn entity_from<T: ComponentId>(&self) -> EntityView {
        EntityView::new_from(self, T::get_id(self))
    }

    /// Create an entity that's associated with a name.
    /// The name does an extra allocation if it's bigger than 24 bytes. To avoid this, use `entity_named_cstr`.
    /// length of 24 bytes: `"hi this is 24 bytes long"`
    ///
    /// Named entities can be looked up with the lookup functions. Entity names
    /// may be scoped, where each element in the name is separated by "::".
    /// For example: "`Foo::Bar`". If parts of the hierarchy in the scoped name do
    /// not yet exist, they will be automatically created.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let entity = world.entity_named("Foo");
    /// assert_eq!(entity.get_name(), Some("Foo"));
    ///
    /// ```
    ///
    /// # See also
    ///
    /// * C++ API: `world::entity`
    #[doc(alias = "world::entity")]
    pub fn entity_named(&self, name: &str) -> EntityView {
        EntityView::new_named(self, name)
    }

    /// Create an entity that's associated with a name.
    /// The name must be a valid C str. No extra allocation is done.
    ///
    /// Named entities can be looked up with the lookup functions. Entity names
    /// may be scoped, where each element in the name is separated by "::".
    /// For example: "`Foo::Bar`". If parts of the hierarchy in the scoped name do
    /// not yet exist, they will be automatically created.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    ///
    /// let entity = world.entity_named("Foo");
    /// assert_eq!(entity.get_name(), Some("Foo"));
    ///
    /// ```
    ///
    /// # See also
    ///
    /// * C++ API: `world::entity`
    #[doc(alias = "world::entity")]
    pub fn entity_named_cstr(&self, name: &CStr) -> EntityView {
        EntityView::new_named_cstr(self, name)
    }

    /// Create a new entity.
    ///
    /// # See also
    ///
    /// * C++ API: `world::entity`
    #[doc(alias = "world::entity")]
    pub fn entity(&self) -> EntityView {
        EntityView::new(self)
    }

    /// Create entity with id 0.
    /// This function is useful when the API must provide an entity that
    /// belongs to a world, but the entity id is 0.
    ///
    /// # Example
    ///
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// let world = World::new();
    /// let entity = world.entity_null();
    /// assert_eq!(entity.id(), 0);
    /// ```
    ///
    /// # See also
    ///
    /// * C++ API: `world::entity`
    #[doc(alias = "world::entity")]
    pub fn entity_null(&self) -> EntityView {
        EntityView::new_null(self)
    }

    /// Create a new entity with the provided id.
    ///
    /// # Arguments
    ///
    /// * `id` - The id to use for the new entity.
    ///
    /// # See also
    ///
    /// * C++ API: `world::entity`
    #[doc(alias = "world::entity")]
    pub fn entity_from_id(&self, id: impl Into<Entity>) -> EntityView {
        EntityView::new_from(self, id.into())
    }

    /// Creates a prefab
    ///
    /// # Returns
    ///
    /// The prefab entity.
    ///
    /// # See also
    ///
    /// * C++ API: `world::prefab`
    #[doc(alias = "world::prefab")]
    pub fn prefab(&self) -> EntityView {
        let result = EntityView::new(self);
        result.add_id(flecs::Prefab::ID);
        result
    }

    /// Creates a named prefab
    ///
    /// # Arguments
    ///
    /// * `name` - The name to use for the new prefab.
    ///
    /// # Returns
    ///
    /// The prefab entity.
    ///
    /// # See also
    ///
    /// * C++ API: `world::prefab`
    #[doc(alias = "world::prefab")]
    pub fn prefab_named<'a>(&'a self, name: &str) -> EntityView<'a> {
        let result = EntityView::new_named(self, name);
        result.add_id(ECS_PREFAB);
        result
    }

    /// Creates a prefab that's associated with a type
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to associate with the new prefab.
    ///
    /// # Returns
    ///
    /// The prefab entity.
    ///
    /// # See also
    ///
    /// * C++ API: `world::prefab`
    #[doc(alias = "world::prefab")]
    pub fn prefab_type<T: ComponentId + EmptyComponent>(&self) -> EntityView {
        let result = Component::<T>::new(self).entity;
        result.add_id(ECS_PREFAB);
        result.add::<T>();
        result
    }

    /// Creates a named prefab that's associated with a type
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type to associate with the new prefab.
    ///
    /// # Arguments
    ///
    /// * `name` - The name to use for the new prefab.
    ///
    /// # Returns
    ///
    /// The prefab entity.
    ///
    /// # See also
    ///
    /// * C++ API: `world::prefab`
    #[doc(alias = "world::prefab")]
    pub fn prefab_type_named<'a, T: ComponentId + EmptyComponent>(
        &'a self,
        name: &str,
    ) -> EntityView<'a> {
        let result = Component::<T>::new_named(self, name).entity;
        result.add_id(ECS_PREFAB);
        result.add::<T>();
        result
    }
}
/// Id mixin implementation
impl World {
    /// Get  id of component / pair
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type.
    ///
    /// # Returns
    ///
    /// The id of the component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::id`
    /// * C++ API: `world::pair`
    #[doc(alias = "world::pair")]
    pub fn id_from<T: IntoComponentId>(&self) -> IdView {
        IdView::new_from(self, T::get_id(self))
    }

    /// get `IdView` from an id or from a relationship pair
    ///
    /// # Arguments
    ///
    /// * `id` - The id to convert to an `IdView`.
    ///
    /// # Returns
    ///
    /// The `IdView` from the provided id.
    ///
    /// # See also
    ///
    /// * C++ API: `world::pair`
    /// * C++ API: `world::id`
    #[doc(alias = "world::pair")]
    #[doc(alias = "world::id")]
    pub fn id_from_id<Id>(&self, id: Id) -> IdView
    where
        Id: IntoId,
    {
        let id = *id.into();
        if Id::IS_PAIR {
            ecs_assert!(
                {
                    let first = ecs_first(id);
                    let second = ecs_second(id);
                    !ecs_is_pair(first) && !ecs_is_pair(second)
                },
                FlecsErrorCode::InvalidParameter,
                "cannot create nested pairs"
            );
        }

        IdView::new_from(self, id)
    }

    /// get pair id from relationship, object.
    ///
    /// # Type Parameters
    ///
    /// * `First` - The first element of the pair.
    ///
    /// # Arguments
    ///
    /// * `second` - The id of the second element of the pair.
    ///
    /// # Returns
    ///
    /// The pair as Id
    ///
    /// # See also
    ///
    /// * C++ API: `world::pair`
    #[doc(alias = "world::pair")]
    pub fn id_first<First: ComponentId>(&self, second: impl Into<Entity>) -> IdView {
        let id: Entity = second.into();
        ecs_assert!(
            !ecs_is_pair(id),
            FlecsErrorCode::InvalidParameter,
            "cannot create nested pairs"
        );
        IdView::new_from(self, (First::get_id(self), id))
    }

    /// get pair id from relationship, object.
    ///
    /// # Type Parameters
    ///
    /// * `Second` - The second element of the pair.
    ///
    /// # Arguments
    ///
    /// * `first` - The id of the first element of the pair.
    ///
    /// # Returns
    ///
    /// The pair as Id
    ///
    /// # See also
    ///
    /// * C++ API: `world::pair`
    #[doc(alias = "world::pair")]
    pub fn id_second<Second: ComponentId>(&self, first: impl Into<Entity>) -> IdView {
        let id = first.into();
        ecs_assert!(
            !ecs_is_pair(id),
            FlecsErrorCode::InvalidParameter,
            "cannot create nested pairs"
        );
        IdView::new_from(self, (id, Second::get_id(self)))
    }
}

/// Component mixin implementation
impl World {
    /// Find or register component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type.
    ///
    /// # Returns
    ///
    /// The found or registered component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::component`
    #[doc(alias = "world::component")]
    pub fn component<T: ComponentId>(&self) -> Component<T::UnderlyingType> {
        Component::<T::UnderlyingType>::new(self)
    }

    /// Find or register component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the component.
    ///
    /// # Returns
    ///
    /// The found or registered component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::component`
    #[doc(alias = "world::component")]
    pub fn component_named<'a, T: ComponentId>(
        &'a self,
        name: &str,
    ) -> Component<'a, T::UnderlyingType> {
        Component::<T::UnderlyingType>::new_named(self, name)
    }

    /// Find or register untyped component.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The component type.
    ///
    /// # Returns
    ///
    /// The found or registered untyped component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::component`
    #[doc(alias = "world::component")]
    pub fn component_untyped<T: ComponentId>(&self) -> UntypedComponent {
        UntypedComponent::new(self, T::get_id(self))
    }

    /// Find or register untyped component.
    ///
    /// # Arguments
    ///
    /// * `id` - The component id.
    ///
    /// # Returns
    ///
    /// The found or registered untyped component.
    ///
    /// # See also
    ///
    /// * C++ API: `world::component`
    #[doc(alias = "world::component")]
    pub fn component_untyped_id(&self, id: impl Into<Entity>) -> UntypedComponent {
        UntypedComponent::new(self, id)
    }

    /// Convert enum constant to entity
    ///
    /// # Type Parameters
    ///
    /// * `T` - The enum type.
    ///
    /// # Arguments
    ///
    /// * `enum_value` - The enum value to convert.
    ///
    /// # See also
    ///
    /// * C++ API: `world::to_entity`
    #[doc(alias = "world::to_entity")]
    pub fn to_entity<T: ComponentId + ComponentType<Enum> + CachedEnumData>(
        &self,
        enum_value: T,
    ) -> EntityView {
        EntityView::new_from(self, enum_value.get_id_variant(self))
    }
}

/// Term mixin implementation
impl World {
    // /// Creates a term for a (component) type.
    // ///
    // /// # Type Parameters
    // ///
    // /// * `T` - The component type.
    // ///
    // /// # Returns
    // ///
    // /// The term for the component type.
    // ///
    // /// # See also
    // ///
    // /// * C++ API: `world::term`
    // #[doc(alias = "world::term")]
    // pub fn term<T: IntoComponentId>(&self) -> Term {
    //     Term::new_type::<T>(self)
    // }
}

// Event mixin implementation
impl World {
    /// Create a new event builder (untyped) from entity id which represents an event
    ///
    /// # Safety
    /// Caller must ensure that `event` is a ZST or that a pointer to the associated type is set on the builder
    ///
    /// # Arguments
    ///
    /// * `event` - The event id
    ///
    /// # Returns
    ///
    /// A new (untyped) event builder.
    ///
    /// # See also
    ///
    /// * C++ API: `world::event`
    #[doc(alias = "world::event")]
    pub unsafe fn event_id(&self, event: impl Into<Entity>) -> EventBuilder<()> {
        EventBuilder::<()>::new_untyped(self, event)
    }

    /// Create a new event.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The event type.
    ///
    /// # Returns
    ///
    /// A new (typed) event builder.
    ///
    /// # See also
    ///
    /// * C++ API: `world::event`
    #[doc(alias = "world::event")]
    pub fn event<T: ComponentId>(&self) -> EventBuilder<T> {
        EventBuilder::<T>::new(self)
    }
}

// Observer mixin implementation
impl World {
    /// Upcast entity to an observer.
    /// The provided entity must be an observer.
    ///
    /// # Arguments
    ///
    /// * `e` - The entity.
    ///
    /// # Returns
    ///
    /// An observer object.
    ///
    /// # See also
    ///
    /// * C++ API: `world::observer`
    #[doc(alias = "world::observer")]
    pub fn new_observer<'a>(&'a self, e: EntityView<'a>) -> Observer<'a> {
        Observer::new_from_existing(e)
    }

    /// Create a new observer.
    ///
    /// # Type Parameters
    ///
    /// * `Components` - The components to match on.
    ///
    /// # Returns
    ///
    /// Observer builder.
    ///
    /// # See also
    ///
    /// * [`World::observer_id()`]
    /// * [`World::observer_named()`]
    /// * C++ API: `world::observer`
    #[doc(alias = "world::observer")]
    pub fn observer<Event: ComponentId, Components>(&self) -> ObserverBuilder<Event, Components>
    where
        Components: Iterable,
    {
        ObserverBuilder::<Event, Components>::new(self)
    }

    pub fn observer_id<Components>(
        &self,
        event: impl Into<Entity>,
    ) -> ObserverBuilder<(), Components>
    where
        Components: Iterable,
    {
        let mut builder = ObserverBuilder::<(), Components>::new_untyped(self);
        builder.add_event_id(event);
        builder
    }

    /// Create a new named observer.
    ///
    /// # Type Parameters
    ///
    /// * `Components` - The components to match on.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the observer.
    ///
    /// # Returns
    ///
    /// Observer builder.
    ///
    /// # See also
    ///
    /// * [`World::observer()`]
    /// * [`World::observer_id()`]
    /// * C++ API: `world::observer`
    #[doc(alias = "world::observer")]
    pub fn observer_named<'a, Event: ComponentId, Components>(
        &'a self,
        name: &str,
    ) -> ObserverBuilder<'a, Event, Components>
    where
        Components: Iterable,
    {
        ObserverBuilder::<Event, Components>::new_named(self, name)
    }
}

/// Query mixin implementation
impl World {
    /// Create a new uncached query.
    ///
    /// # Type Parameters
    ///
    /// * `Components` - The components to match on.
    ///
    /// # See also
    ///
    /// * C++ API: `world::query`
    #[doc(alias = "world::query")]
    pub fn new_query<Components>(&self) -> Query<Components>
    where
        Components: Iterable,
    {
        QueryBuilder::<Components>::new(self).build()
    }

    /// Create a new named query.
    ///
    /// # Type Parameters
    ///
    /// * `Components` - The components to match on.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the query.
    ///
    /// # Returns
    ///
    /// A new query.
    ///
    /// # See also
    ///
    /// * C++ API: `world::query`
    #[doc(alias = "world::query")]
    pub fn new_query_named<Components>(&self, name: &str) -> Query<Components>
    where
        Components: Iterable,
    {
        QueryBuilder::<Components>::new_named(self, name).build()
    }

    /// Create a new query builder.
    ///
    /// # Type Parameters
    ///
    /// * `Components` - The components to match on.
    ///
    /// # Returns
    ///
    /// A new query builder.
    ///
    /// # See also
    ///
    /// * C++ API: `world::query_builder`
    #[doc(alias = "world::query_builder")]
    pub fn query<Components>(&self) -> QueryBuilder<Components>
    where
        Components: Iterable,
    {
        QueryBuilder::<Components>::new(self)
    }

    /// Create a new named query builder.
    ///
    /// # Type Parameters
    ///
    /// * `Components` - The components to match on.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the query.
    ///
    /// # Returns
    ///
    /// A new query builder.
    ///
    /// # See also
    ///
    /// * C++ API: `world::query_builder`
    #[doc(alias = "world::query_builder")]
    pub fn query_named<'a, Components>(&'a self, name: &str) -> QueryBuilder<'a, Components>
    where
        Components: Iterable,
    {
        QueryBuilder::<Components>::new_named(self, name)
    }

    /// Convert a query entity to a query.
    ///
    /// # Safety
    ///
    /// Proceed with caution. Use `.iter_only` instead.
    ///
    /// # Returns
    ///
    /// returns the untyped query if the entity is alive, otherwise `None`.
    pub fn try_query_from(&self, query_entity: impl Into<Entity>) -> Option<Query<()>> {
        Query::<()>::new_from_entity(self, query_entity)
    }

    /// Convert a query entity to a query.
    /// this method is the same as `try_to_query` but it automatically unwraps the result.
    ///
    /// # Safety
    ///
    /// Proceed with caution. Use `.iter_only` instead.
    ///
    /// # Panics
    ///
    /// Panics if the entity is not alive or a query. Use `try_to_query` if you are unsure.
    pub fn query_from(&self, query_entity: impl Into<Entity>) -> Query<()> {
        self.try_query_from(query_entity)
            .expect("entity / query is not alive or valid")
    }

    /// Create and iterate an uncached query.
    ///
    /// This function creates a query and immediately iterates it.
    ///
    /// # Returns
    ///
    /// The query.
    ///
    /// # Type Parameters
    ///
    /// * `Components`: The components to match on.
    ///
    /// # See also
    ///
    /// * C++ API: `world::each`
    #[doc(alias = "world::each")]
    pub fn each<Components>(&self, func: impl FnMut(Components::TupleType<'_>)) -> Query<Components>
    where
        Components: Iterable,
    {
        let query = QueryBuilder::<Components>::new(self).build();
        query.each(func);
        query
    }

    /// Create and iterate an uncached query.
    ///
    /// This function creates a query and immediately iterates it.
    ///
    /// # Returns
    ///
    /// The query.
    ///
    /// # Type Parameters
    ///
    /// * `Components`: The components to match on.
    ///
    /// # See also
    ///
    /// * C++ API: `world::each`
    #[doc(alias = "world::each")]
    pub fn each_entity<Components>(
        &self,
        func: impl FnMut(EntityView, Components::TupleType<'_>),
    ) -> Query<Components>
    where
        Components: Iterable,
    {
        let query = QueryBuilder::<Components>::new(self).build();
        query.each_entity(func);
        query
    }
}

/// Systems mixin implementation
#[cfg(feature = "flecs_system")]
impl World {
    /// Constructs a `System` from an existing entity.
    ///
    /// This function upcasts the given `entity` to a `System`, assuming the entity represents a system.
    /// The purpose is to facilitate the interaction with entities that are specifically systems within the ECS.
    ///
    /// # Arguments
    /// * `entity` - An `EntityView` that represents a system within the world.
    ///
    /// # See also
    ///
    /// * C++ API: `world::system`
    #[doc(alias = "world::system")]
    pub fn system_from<'a>(&'a self, entity: EntityView<'a>) -> System<'a> {
        System::new_from_existing(entity)
    }

    /// Creates a new `SystemBuilder` instance for constructing systems.
    ///
    /// This function initializes a `SystemBuilder` which is used to create systems that match specific components.
    /// It is a generic method that works with any component types that implement the `Iterable` trait.
    ///
    /// # Type Parameters
    /// - `Components`: The components to match on. Must implement the `Iterable` trait.
    ///
    /// # See also
    ///
    /// * C++ API: `world::system_builder`
    #[doc(alias = "world::system_builder")]
    pub fn system<Components>(&self) -> SystemBuilder<Components>
    where
        Components: Iterable,
    {
        SystemBuilder::<Components>::new(self)
    }

    /// Creates a new named `SystemBuilder` instance.
    ///
    /// Similar to `system_builder`, but allows naming the system for easier identification and debugging.
    /// The name does not affect the system's behavior.
    ///
    /// # Arguments
    /// * `name` - A string slice representing the name of the system.
    ///
    /// # Type Parameters
    /// - `Components`: The components to match on. Must implement the `Iterable` trait.
    ///
    /// # See also
    ///
    /// * C++ API: `world::system_builder`
    #[doc(alias = "world::system_builder")]
    pub fn system_named<'a, Components>(&'a self, name: &str) -> SystemBuilder<'a, Components>
    where
        Components: Iterable,
    {
        SystemBuilder::<Components>::new_named(self, name)
    }

    /// Creates a `SystemBuilder` from a system description.
    ///
    /// This function allows creating a system based on a predefined system description,
    /// facilitating more dynamic or configuration-driven system creation.
    ///
    /// # Arguments
    /// * `desc` - A system description that outlines the parameters for the system builder.
    ///
    /// # Type Parameters
    /// - `Components`: The components to match on. Must implement the `Iterable` trait.
    ///
    /// # See also
    ///
    /// * C++ API: `world::system_builder`
    #[doc(alias = "world::system_builder")]
    pub fn system_builder_from_desc<Components>(
        &self,
        desc: sys::ecs_system_desc_t,
    ) -> SystemBuilder<Components>
    where
        Components: Iterable,
    {
        SystemBuilder::<Components>::new_from_desc(self, desc)
    }
}

/// Pipeline mixin implementation
#[cfg(feature = "flecs_pipeline")]
impl World {
    /// Create a new [`Pipeline`](crate::addons::pipeline::Pipeline).
    ///
    /// # See also
    ///
    /// * [`World::pipeline_named()`]
    /// * [`World::pipeline_type()`]
    /// * C++ API: `world::pipeline`
    #[doc(alias = "world::pipeline")]
    #[inline(always)]
    pub fn pipeline(&self) -> PipelineBuilder<()> {
        PipelineBuilder::<()>::new(self)
    }

    /// Create a new named [`Pipeline`](crate::addons::pipeline::Pipeline).
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the pipeline.
    ///
    /// # See also
    ///
    /// * [`World::pipeline()`]
    /// * [`World::pipeline_type()`]
    /// * C++ API: `world::pipeline`
    #[doc(alias = "world::pipeline")]
    #[inline(always)]
    pub fn pipeline_named<'a>(&'a self, name: &str) -> PipelineBuilder<'a, ()> {
        PipelineBuilder::<()>::new_named(self, name)
    }

    /// Create a new [`Pipeline`](crate::addons::pipeline::Pipeline) with the provided associated type.
    ///
    /// # Type Parameters
    ///
    /// * `Pipeline` - The associated type to use for the pipeline.
    ///
    /// # See also
    ///
    /// * [`World::pipeline()`]
    /// * [`World::pipeline_named()`]
    /// * C++ API: `world::pipeline`
    #[doc(alias = "world::pipeline")]
    #[inline(always)]
    pub fn pipeline_type<Pipeline>(&self) -> PipelineBuilder<()>
    where
        Pipeline: ComponentType<Struct> + ComponentId,
    {
        PipelineBuilder::<()>::new_w_entity(self, Pipeline::get_id(self))
    }

    /// Set a custom pipeline. This operation sets the pipeline to run when [`World::progress()`] is invoked.
    ///
    /// # Arguments
    ///
    /// * `pipeline` - The pipeline to set.
    ///
    /// # See also
    ///
    /// * [`World::get_pipeline()`]
    /// * [`World::set_pipeline()`]
    /// * C++ API: `world::set_pipeline_id`
    #[doc(alias = "world::set_pipeline_id")]
    #[inline(always)]
    pub fn set_pipeline(&self, pipeline: impl Into<Entity>) {
        unsafe {
            sys::ecs_set_pipeline(self.raw_world.as_ptr(), *pipeline.into());
        }
    }

    /// Set a custom pipeline by type. This operation sets the pipeline to run when [`World::progress()`] is invoked.
    ///
    /// # Type Parameters
    ///
    /// * `Pipeline` - The associated type to use for the pipeline.
    ///
    /// # See also
    ///
    /// * [`World::get_pipeline()`]
    /// * [`World::set_pipeline_id()`]
    /// * C++ API: `world::set_pipeline_id`
    #[doc(alias = "world::set_pipeline_id")]
    #[inline(always)]
    pub fn set_pipeline_type<Pipeline>(&self)
    where
        Pipeline: ComponentType<Struct> + ComponentId,
    {
        unsafe {
            sys::ecs_set_pipeline(self.raw_world.as_ptr(), Pipeline::get_id(self));
        }
    }

    /// Get the current pipeline.
    ///
    /// # Returns
    ///
    /// The current pipeline as an entity.
    ///
    /// # See also
    ///
    /// * [`World::set_pipeline()`]
    /// * [`World::set_pipeline_id()`]
    /// * C++ API: `world::get_pipeline`
    #[doc(alias = "world::get_pipeline")]
    #[inline(always)]
    pub fn get_pipeline(&self) -> EntityView {
        EntityView::new_from(self, unsafe {
            sys::ecs_get_pipeline(self.raw_world.as_ptr())
        })
    }

    /// Progress world one tick.
    ///
    /// Progresses the world by running all enabled and periodic systems
    /// on their matching entities.
    ///
    /// This is a wrapper around [`World::progress_time()`]. It passes `0.0` as
    /// the `delta_time` to automatically measure the time passed since the last
    /// frame. This mode is useful for applications that do not manage time
    /// explicitly and want the system to measure the time automatically.
    ///
    /// # Returns
    ///
    /// True if the world has been progressed, false if [`World::quit()`] has been called.
    ///
    /// # See also
    ///
    /// * [`World::progress_time()`]
    /// * C API: `ecs_progress`
    /// * C++ API: `world::progress`
    #[doc(alias = "world::progress")]
    #[inline(always)]
    pub fn progress(&self) -> bool {
        self.progress_time(0.0)
    }

    /// Progress world by delta time.
    ///
    /// Progresses the world by running all enabled and periodic systems
    /// on their matching entities for the specified time since the last frame.
    ///
    /// When `delta_time` is 0, `World::progress_time()` will automatically measure the time passed
    /// since the last frame. For applications not using time management, passing a
    /// non-zero `delta_time` (1.0 recommended) skips automatic time measurement to avoid overhead.
    ///
    /// # Arguments
    ///
    /// * `delta_time` - The time to progress the world by. Pass 0.0 for automatic time measurement.
    ///
    /// # Returns
    ///
    /// True if the world has been progressed, false if [`World::quit()`] has been called.
    ///
    /// # See also
    ///
    /// * [`World::progress()`]
    /// * C API: `ecs_progress`
    /// * C++ API: `world::progress`
    #[doc(alias = "world::progress")]
    #[inline(always)]
    pub fn progress_time(&self, delta_time: f32) -> bool {
        unsafe { sys::ecs_progress(self.raw_world.as_ptr(), delta_time) }
    }

    /// Run pipeline.
    /// Runs all systems in the specified pipeline. Can be invoked from multiple
    /// threads if staging is disabled, managing staging and, if needed, thread
    /// synchronization.
    ///
    /// Providing 0 for pipeline id runs the default pipeline (builtin or set via
    /// `set_pipeline_id()`). Using [`World::progress()`] auto-invokes this for the
    /// default pipeline. Additional pipelines may be run explicitly.
    ///
    /// # Note
    ///
    /// Only supports single-threaded applications with a single stage when called from an application.
    ///
    /// # Arguments
    ///
    /// * `pipeline` - Pipeline to run.
    ///
    /// # See also
    ///
    /// * [`World::run_pipeline()`]
    /// * [`World::run_pipeline_id_time()`]
    /// * [`World::run_pipeline_time()`]
    /// * C++ API: `world::run_pipeline`
    #[doc(alias = "world::run_pipeline")]
    #[inline(always)]
    pub fn run_pipeline_id(&self, pipeline: impl Into<Entity>) {
        Self::run_pipeline_id_time(self, pipeline, 0.0);
    }

    /// Run pipeline.
    /// Runs all systems in the specified pipeline. Can be invoked from multiple
    /// threads if staging is disabled, managing staging and, if needed, thread
    /// synchronization.
    ///
    /// Providing 0 for pipeline id runs the default pipeline (builtin or set via
    /// `set_pipeline_id()`). Using [`World::progress()`] auto-invokes this for the
    /// default pipeline. Additional pipelines may be run explicitly.
    ///
    /// # Note
    ///
    /// Only supports single-threaded applications with a single stage when called from an application.
    ///
    /// # Arguments
    ///
    /// * `pipeline` - Pipeline to run.
    /// * `delta_time` - Time to advance the world.
    ///
    /// # See also
    ///
    /// * [`World::run_pipeline()`]
    /// * [`World::run_pipeline_id()`]
    /// * [`World::run_pipeline_time()`]
    /// * C++ API: `world::run_pipeline`
    #[doc(alias = "world::run_pipeline")]
    #[inline(always)]
    pub fn run_pipeline_id_time(&self, pipeline: impl Into<Entity>, delta_time: super::FTime) {
        unsafe {
            sys::ecs_run_pipeline(self.raw_world.as_ptr(), *pipeline.into(), delta_time);
        }
    }

    /// Run pipeline.
    /// Runs all systems in the specified pipeline. Can be invoked from multiple
    /// threads if staging is disabled, managing staging and, if needed, thread
    /// synchronization.
    ///
    /// Using [`World::progress()`] auto-invokes this for the default pipeline.
    /// Additional pipelines may be run explicitly.
    ///
    /// # Note
    ///
    /// Only supports single-threaded applications with a single stage when called from an application.
    ///
    /// # Type Parameters
    ///
    /// * `Component` - The associated type to use for the pipeline.
    ///
    /// # Arguments
    ///
    /// * `delta_time` - Time to advance the world.
    ///
    /// # See also
    ///
    /// * [`World::run_pipeline()`]
    /// * [`World::run_pipeline_id()`]
    /// * [`World::run_pipeline_id_time()`]
    /// * C++ API: `world::run_pipeline`
    #[doc(alias = "world::run_pipeline")]
    pub fn run_pipeline_time<Component>(&self, delta_time: super::FTime)
    where
        Component: ComponentType<Struct> + ComponentId,
    {
        unsafe {
            sys::ecs_run_pipeline(self.raw_world.as_ptr(), Component::get_id(self), delta_time);
        }
    }

    /// Run pipeline.
    /// Runs all systems in the specified pipeline. Can be invoked from multiple
    /// threads if staging is disabled, managing staging and, if needed, thread
    /// synchronization.
    ///
    /// Using [`World::progress()`] auto-invokes this for the default pipeline.
    /// Additional pipelines may be run explicitly.
    ///
    /// # Note
    ///
    /// Only supports single-threaded applications with a single stage when called from an application.
    ///
    /// # Type Parameters
    ///
    /// * `Component` - The associated type to use for the pipeline.
    ///
    /// # See also
    ///
    /// * [`World::run_pipeline_id()`]
    /// * [`World::run_pipeline_id_time()`]
    /// * [`World::run_pipeline_time()`]
    /// * C++ API: `world::run_pipeline`
    #[doc(alias = "world::run_pipeline")]
    pub fn run_pipeline<Component>(&self)
    where
        Component: ComponentType<Struct> + ComponentId,
    {
        Self::run_pipeline_time::<Component>(self, 0.0);
    }

    /// Set time scale. Increase or decrease simulation speed by the provided multiplier.
    ///
    /// # Arguments
    ///
    /// * `mul` - The multiplier to set the time scale to.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_time_scale`
    #[doc(alias = "world::set_time_scale")]
    #[inline(always)]
    pub fn set_time_scale(&self, mul: super::FTime) {
        unsafe {
            sys::ecs_set_time_scale(self.raw_world.as_ptr(), mul);
        }
    }

    /// Get time scale.
    ///
    /// Retrieves the current time scale of the world, which affects the speed
    /// at which time passes within the simulation. A time scale of 1.0 means
    /// real-time, values greater than 1.0 speed up the simulation, and values
    /// less than 1.0 slow it down.
    ///
    /// # Returns
    ///
    /// The current time scale as a floating point number.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_time_scale`
    #[doc(alias = "world::get_time_scale")]
    #[inline(always)]
    pub fn get_time_scale(&self) -> super::FTime {
        self.get_info().time_scale
    }

    /// Get target frames per second (FPS).
    ///
    /// Retrieves the target FPS for the world. This value is used to calculate
    /// the time step for each simulation tick when the automatic time step is
    /// enabled. Adjusting the target FPS can be used to control simulation
    /// speed.
    ///
    /// # Returns
    ///
    /// The target FPS as a floating point number.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_target_fps`
    #[doc(alias = "world::get_target_fps")]
    #[inline(always)]
    pub fn get_target_fps(&self) -> super::FTime {
        self.get_info().target_fps
    }

    /// Set target frames per second (FPS).
    ///
    /// Configures the world to run at the specified target FPS, ensuring that
    /// [`World::progress()`] is not called more frequently than this rate. This mechanism
    /// enables tracking the elapsed time since the last [`World::progress()`] call and
    /// sleeping for any remaining time in the frame, if applicable.
    ///
    /// Utilizing this feature promotes consistent system execution intervals and
    /// conserves CPU resources by avoiding more frequent system runs than necessary.
    ///
    /// It's important to note that [`World::progress()`] will only introduce sleep periods
    /// when there is surplus time within a frame. This accounts for time consumed both
    /// within Flecs and in external operations.
    ///
    /// # Arguments
    ///
    /// * `world` - The world context.
    /// * `fps` - The desired target FPS as a floating-point number.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_target_fps`
    #[doc(alias = "world::set_target_fps")]
    #[inline(always)]
    pub fn set_target_fps(&self, target_fps: super::FTime) {
        unsafe {
            sys::ecs_set_target_fps(self.raw_world.as_ptr(), target_fps);
        }
    }

    /// Reset world clock. Reset the clock that keeps track of the total time passed in the simulation.
    ///
    /// # See also
    ///
    /// * C++ API: `world::reset_clock`
    #[doc(alias = "world::reset_clock")]
    #[inline(always)]
    pub fn reset_clock(&self) {
        unsafe {
            sys::ecs_reset_clock(self.raw_world.as_ptr());
        }
    }

    /// Set number of worker threads.
    ///
    /// Setting this value to a value higher than 1 will start as many threads and
    /// will cause systems to evenly distribute matched entities across threads.
    /// The operation may be called multiple times to reconfigure the number of threads used,
    /// but never while running a system / pipeline. Calling `sys::ecs_set_threads` will also end the use
    /// of task threads setup with `sys::ecs_set_task_threads` and vice-versa
    ///
    /// # Arguments
    ///
    /// * `threads` - The number of threads to use.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_threads`
    #[doc(alias = "world::set_threads")]
    #[inline(always)]
    pub fn set_threads(&self, threads: i32) {
        unsafe {
            sys::ecs_set_threads(self.raw_world.as_ptr(), threads);
        }
    }

    /// Get number of configured stages. Return number of stages set by `sys::ecs_set_stage_count`.
    ///
    /// # Returns
    ///
    /// The number of stages as an integer.
    ///
    /// # See also
    ///
    /// * C++ API: `world::get_threads`
    #[doc(alias = "world::get_threads")]
    #[inline(always)]
    pub fn get_threads(&self) -> i32 {
        unsafe { sys::ecs_get_stage_count(self.raw_world.as_ptr()) }
    }

    /// Set number of worker task threads.
    ///
    /// Configures the world to use a specified number of short-lived task threads,
    /// distinct from `sys::ecs_set_threads` where threads persist. Here, threads are
    /// created and joined for each world update, leveraging the `os_api_t` tasks
    /// APIs for task management instead of traditional thread APIs. This approach
    /// is advantageous for integrating with external asynchronous job systems,
    /// allowing for the dynamic creation and synchronization of tasks specific to
    /// each world update.
    ///
    /// This function can be invoked multiple times to adjust the count of task threads,
    /// but must not be called concurrently with system or pipeline execution. Switching
    /// to `sys::ecs_set_task_threads` from `sys::ecs_set_threads` (or vice versa) will terminate
    /// the use of the previously configured threading model.
    ///
    /// # Arguments
    ///
    /// * `task_threads` - The number of task threads to use.
    ///
    /// # See also
    ///
    /// * C++ API: `world::set_task_threads`
    #[doc(alias = "world::set_task_threads")]
    #[inline(always)]
    pub fn set_task_threads(&self, task_threads: i32) {
        unsafe {
            sys::ecs_set_task_threads(self.raw_world.as_ptr(), task_threads);
        }
    }

    /// Returns true if task thread use have been requested.
    ///
    /// # Returns
    ///
    /// True if task threads are being used, false otherwise.
    ///
    /// # See also
    ///
    /// * C++ API: `world::using_task_threads`
    #[doc(alias = "world::using_task_threads")]
    #[inline(always)]
    pub fn using_task_threads(&self) -> bool {
        unsafe { sys::ecs_using_task_threads(self.raw_world.as_ptr()) }
    }
}

/// App mixin implementation
#[cfg(feature = "flecs_app")]
impl World {
    /// Create a new app.
    /// The app builder is a convenience wrapper around a loop that runs
    /// [`World::progress()`]. An app allows for writing platform agnostic code,
    /// as it provides hooks to modules for overtaking the main loop which is
    /// required for frameworks like emscripten.
    ///
    /// # See also
    ///
    /// * C++ API: `world::app`
    #[doc(alias = "world::app")]
    #[inline(always)]
    pub fn app(&self) -> App {
        App::new(self)
    }
}

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
