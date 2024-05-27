//! Registering and working with components

use std::{marker::PhantomData, ops::Deref, os::raw::c_void, ptr};

use crate::core::*;
use crate::sys;

#[cfg(feature = "flecs_meta")]
use crate::addons::meta::Opaque;

/// Component class.
/// Class used to register components and component metadata.
pub struct Component<'a, T: ComponentId> {
    pub base: UntypedComponent<'a>,
    _marker: PhantomData<T>,
}

impl<'a, T: ComponentId> Deref for Component<'a, T> {
    type Target = UntypedComponent<'a>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl<'a, T: ComponentId> Component<'a, T> {
    /// Create a new component.
    ///
    /// # Arguments
    ///
    /// * `world`: the world.
    ///
    /// # See also
    ///
    /// * C++ API: `component::component`
    #[doc(alias = "component::component")]
    pub fn new(world: impl IntoWorld<'a>) -> Self {
        if !T::is_registered_with_world(world.world()) {
            T::register_explicit(world.world());
        }

        Self {
            base: UntypedComponent::new(world, unsafe { T::get_id_unchecked() }),
            _marker: PhantomData,
        }
    }

    /// Create a new component with a name.
    ///
    /// # Arguments
    ///
    /// * `world`: the world.
    /// * `name`: the name of the component.
    ///
    /// # See also
    ///
    /// * C++ API: `component::component`
    #[doc(alias = "component::component")]
    pub fn new_named(world: impl IntoWorld<'a>, name: &str) -> Self {
        if !T::is_registered_with_world(world.world()) {
            T::register_explicit_named(world.world(), name);
        }

        Self {
            base: UntypedComponent::new(world, unsafe { T::get_id_unchecked() }),
            _marker: PhantomData,
        }
    }

    /// Return the component as an entity
    ///
    /// # See also
    ///
    /// * C++ API: `id::entity`
    #[doc(alias = "id::entity")]
    #[inline(always)]
    pub fn entity(self) -> EntityView<'a> {
        self.base.entity
    }

    /// Get the binding context for the component.
    ///
    /// # Arguments
    ///
    /// * `type_hooks`: the type hooks.
    ///
    /// # See also
    ///
    /// * C++ API: `component::get_binding_context`
    #[doc(alias = "component::get_binding_context")]
    fn get_binding_context(type_hooks: &mut TypeHooksT) -> &mut ComponentBindingCtx {
        let mut binding_ctx: *mut ComponentBindingCtx = type_hooks.binding_ctx as *mut _;

        if binding_ctx.is_null() {
            let new_binding_ctx = Box::<ComponentBindingCtx>::default();
            let static_ref = Box::leak(new_binding_ctx);
            binding_ctx = static_ref;
            type_hooks.binding_ctx = binding_ctx as *mut c_void;
            type_hooks.binding_ctx_free = Some(Self::binding_ctx_drop);
        }
        unsafe { &mut *binding_ctx }
    }

    /// Get the type hooks for the component.
    ///
    /// # See also
    ///
    /// * C++ API: `component::get_hooks`
    #[doc(alias = "component::get_hooks")]
    pub fn get_hooks(&self) -> TypeHooksT {
        let type_hooks: *const TypeHooksT =
            unsafe { sys::ecs_get_hooks_id(self.world.world_ptr_mut(), *self.id) };
        if type_hooks.is_null() {
            TypeHooksT::default()
        } else {
            unsafe { *type_hooks }
        }
    }

    /// Function to free the binding context.
    ///
    /// # See also
    ///
    /// * C++ API: `component::binding_ctx_free`
    #[doc(alias = "component::binding_ctx_free")]
    unsafe extern "C" fn binding_ctx_drop(ptr: *mut c_void) {
        let ptr_struct: *mut ComponentBindingCtx = ptr as *mut ComponentBindingCtx;
        unsafe {
            ptr::drop_in_place(ptr_struct);
        }
    }

    /// Register on add hook.
    ///
    /// # See also
    ///
    /// * C++ API: `component::on_add`
    #[doc(alias = "component::on_add")]
    pub fn on_add<Func>(&mut self, func: Func) -> &mut Self
    where
        Func: FnMut(EntityView, &mut T) + 'static,
    {
        let mut type_hooks: TypeHooksT = self.get_hooks();

        ecs_assert!(
            type_hooks.on_add.is_none(),
            FlecsErrorCode::InvalidOperation,
            "on_add hook already set for component {}",
            std::any::type_name::<T>()
        );

        let binding_ctx = Self::get_binding_context(&mut type_hooks);
        let boxed_func = Box::new(func);
        let static_ref = Box::leak(boxed_func);
        binding_ctx.on_add = Some(static_ref as *mut _ as *mut c_void);
        binding_ctx.free_on_add = Some(Self::on_add_drop::<Func>);
        type_hooks.on_add = Some(Self::run_add::<Func>);
        unsafe { sys::ecs_set_hooks_id(self.world.world_ptr_mut(), *self.id, &type_hooks) };
        self
    }

    /// Register on remove hook.
    ///
    /// # See also
    ///
    /// * C++ API: `component::on_remove`
    #[doc(alias = "component::on_remove")]
    pub fn on_remove<Func>(&mut self, func: Func) -> &mut Self
    where
        Func: FnMut(EntityView, &mut T) + 'static,
    {
        let mut type_hooks: TypeHooksT = self.get_hooks();

        ecs_assert!(
            type_hooks.on_remove.is_none(),
            FlecsErrorCode::InvalidOperation,
            "on_remove hook already set for component {}",
            std::any::type_name::<T>()
        );

        let binding_ctx = Self::get_binding_context(&mut type_hooks);
        let boxed_func = Box::new(func);
        let static_ref = Box::leak(boxed_func);
        binding_ctx.on_remove = Some(static_ref as *mut _ as *mut c_void);
        binding_ctx.free_on_remove = Some(Self::on_remove_drop::<Func>);
        type_hooks.on_remove = Some(Self::run_remove::<Func>);
        unsafe { sys::ecs_set_hooks_id(self.world.world_ptr_mut(), *self.id, &type_hooks) };
        self
    }

    /// Register on set hook.
    ///
    /// # See also
    ///
    /// * C++ API: `component::on_set`
    #[doc(alias = "component::on_set")]
    pub fn on_set<Func>(&mut self, func: Func) -> &mut Self
    where
        Func: FnMut(EntityView, &mut T) + 'static,
    {
        let mut type_hooks: TypeHooksT = self.get_hooks();

        ecs_assert!(
            type_hooks.on_set.is_none(),
            FlecsErrorCode::InvalidOperation,
            "on_set hook already set for component {}",
            std::any::type_name::<T>()
        );

        let binding_ctx = Self::get_binding_context(&mut type_hooks);
        let boxed_func = Box::new(func);
        let static_ref = Box::leak(boxed_func);
        binding_ctx.on_set = Some(static_ref as *mut _ as *mut c_void);
        binding_ctx.free_on_set = Some(Self::on_set_drop::<Func>);
        type_hooks.on_set = Some(Self::run_set::<Func>);
        unsafe { sys::ecs_set_hooks_id(self.world.world_ptr_mut(), *self.id, &type_hooks) };
        self
    }

    /// Function to free the on add hook.
    unsafe extern "C" fn on_add_drop<Func>(func: *mut c_void)
    where
        Func: FnMut(EntityView, &mut T) + 'static,
    {
        let ptr_func: *mut Func = func as *mut Func;
        unsafe {
            ptr::drop_in_place(ptr_func);
        }
    }

    /// Function to free the on remove hook.
    unsafe extern "C" fn on_remove_drop<Func>(func: *mut c_void)
    where
        Func: FnMut(EntityView, &mut T) + 'static,
    {
        let ptr_func: *mut Func = func as *mut Func;
        unsafe {
            ptr::drop_in_place(ptr_func);
        }
    }

    /// Function to free the on set hook.
    unsafe extern "C" fn on_set_drop<Func>(func: *mut c_void)
    where
        Func: FnMut(EntityView, &mut T) + 'static,
    {
        let ptr_func: *mut Func = func as *mut Func;
        unsafe {
            ptr::drop_in_place(ptr_func);
        }
    }

    /// Function to run the on add hook.
    unsafe extern "C" fn run_add<Func>(iter: *mut IterT)
    where
        Func: FnMut(EntityView, &mut T) + 'static,
    {
        let ctx: *mut ComponentBindingCtx = (*iter).callback_ctx as *mut _;
        let on_add = (*ctx).on_add.unwrap();
        let on_add = on_add as *mut Func;
        let on_add = &mut *on_add;
        let world = WorldRef::from_ptr((*iter).world);
        let entity = EntityView::new_from(world, *(*iter).entities);
        let component: *mut T = ecs_field::<T>(iter, 0);
        on_add(entity, &mut *component);
    }

    /// Function to run the on set hook.
    unsafe extern "C" fn run_set<Func>(iter: *mut IterT)
    where
        Func: FnMut(EntityView, &mut T) + 'static,
    {
        let ctx: *mut ComponentBindingCtx = unsafe { (*iter).callback_ctx as *mut _ };
        let on_set = unsafe { (*ctx).on_set.unwrap() };
        let on_set = on_set as *mut Func;
        let on_set = unsafe { &mut *on_set };
        let world = unsafe { WorldRef::from_ptr((*iter).world) };
        let entity = EntityView::new_from(world, *(*iter).entities);
        let component: *mut T = unsafe { ecs_field::<T>(iter, 0) };
        on_set(entity, unsafe { &mut *component });
    }

    /// Function to run the on remove hook.
    unsafe extern "C" fn run_remove<Func>(iter: *mut IterT)
    where
        Func: FnMut(EntityView, &mut T) + 'static,
    {
        let ctx: *mut ComponentBindingCtx = unsafe { (*iter).callback_ctx as *mut _ };
        let on_remove = unsafe { (*ctx).on_remove.unwrap() };
        let on_remove = on_remove as *mut Func;
        let on_remove = unsafe { &mut *on_remove };
        let world = unsafe { WorldRef::from_ptr((*iter).world) };
        let entity = EntityView::new_from(world, *(*iter).entities);
        let component: *mut T = unsafe { ecs_field::<T>(iter, 0) };
        on_remove(entity, unsafe { &mut *component });
    }
}

mod eq_operations {
    use super::*;

    impl<'a, T: ComponentId> PartialEq<Component<'a, T>> for u64 {
        #[inline]
        fn eq(&self, other: &Component<'a, T>) -> bool {
            *self == other.base.entity.id
        }
    }

    impl<'a, T: ComponentId> PartialEq<u64> for Component<'a, T> {
        #[inline]
        fn eq(&self, other: &u64) -> bool {
            self.base.entity.id == *other
        }
    }

    impl<'a, T: ComponentId> PartialEq<Entity> for Component<'a, T> {
        #[inline]
        fn eq(&self, other: &Entity) -> bool {
            self.base.entity.id == *other
        }
    }

    impl<'a, T: ComponentId> PartialEq<Id> for Component<'a, T> {
        #[inline]
        fn eq(&self, other: &Id) -> bool {
            self.base.entity.id == *other
        }
    }

    impl<'a, T: ComponentId> PartialEq<EntityView<'a>> for Component<'a, T> {
        #[inline]
        fn eq(&self, other: &EntityView<'a>) -> bool {
            self.base.entity == *other
        }
    }

    impl<'a, T: ComponentId> PartialEq<IdView<'a>> for Component<'a, T> {
        #[inline]
        fn eq(&self, other: &IdView<'a>) -> bool {
            self.base.entity == other.id
        }
    }

    impl<'a, T: ComponentId> PartialEq<UntypedComponent<'a>> for Component<'a, T> {
        #[inline]
        fn eq(&self, other: &UntypedComponent<'a>) -> bool {
            self.base.entity == other.entity
        }
    }

    impl<'a, T: ComponentId> PartialEq for Component<'a, T> {
        #[inline]
        fn eq(&self, other: &Self) -> bool {
            self.base.entity == other.base.entity
        }
    }

    impl<'a, T: ComponentId> Eq for Component<'a, T> {}

    impl<'a, T: ComponentId> PartialOrd<Component<'a, T>> for u64 {
        #[inline]
        fn partial_cmp(&self, other: &Component<'a, T>) -> Option<std::cmp::Ordering> {
            self.partial_cmp(&other.base.entity.id)
        }
    }
}

mod ord_operations {
    use super::*;
    impl<'a, T: ComponentId> PartialOrd<u64> for Component<'a, T> {
        #[inline]
        fn partial_cmp(&self, other: &u64) -> Option<std::cmp::Ordering> {
            self.base.entity.id.partial_cmp(other)
        }
    }

    impl<'a, T: ComponentId> PartialOrd<Entity> for Component<'a, T> {
        #[inline]
        fn partial_cmp(&self, other: &Entity) -> Option<std::cmp::Ordering> {
            self.base.entity.id.partial_cmp(other)
        }
    }

    impl<'a, T: ComponentId> PartialOrd<Id> for Component<'a, T> {
        #[inline]
        fn partial_cmp(&self, other: &Id) -> Option<std::cmp::Ordering> {
            self.base.entity.id.partial_cmp(other)
        }
    }

    impl<'a, T: ComponentId> PartialOrd<EntityView<'a>> for Component<'a, T> {
        #[inline]
        fn partial_cmp(&self, other: &EntityView<'a>) -> Option<std::cmp::Ordering> {
            self.base.entity.partial_cmp(other)
        }
    }

    impl<'a, T: ComponentId> PartialOrd<IdView<'a>> for Component<'a, T> {
        #[inline]
        fn partial_cmp(&self, other: &IdView<'a>) -> Option<std::cmp::Ordering> {
            self.base.entity.partial_cmp(&other.id)
        }
    }

    impl<'a, T: ComponentId> PartialOrd<UntypedComponent<'a>> for Component<'a, T> {
        #[inline]
        fn partial_cmp(&self, other: &UntypedComponent<'a>) -> Option<std::cmp::Ordering> {
            self.base.entity.partial_cmp(&other.entity)
        }
    }

    impl<'a, T: ComponentId> PartialOrd for Component<'a, T> {
        #[inline]
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl<'a, T: ComponentId> Ord for Component<'a, T> {
        #[inline]
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.base.entity.cmp(&other.base.entity)
        }
    }
}
