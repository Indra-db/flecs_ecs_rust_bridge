use std::{ops::Deref, os::raw::c_void, ptr};

use super::{
    c_binding::bindings::{
        ecs_filter_desc_t, ecs_filter_next, ecs_iter_action_t, ecs_iter_t, ecs_observer_desc_t,
        ecs_table_lock, ecs_table_unlock,
    },
    c_types::{EntityT, IterT, TermT, WorldT, SEPARATOR},
    component_registration::CachedComponentData,
    filter_builder::{FilterBuilder, FilterBuilderImpl},
    iterable::{Filterable, Iterable},
    term::TermBuilder,
    world::World,
};

type EcsCtxFreeT = extern "C" fn(*mut std::ffi::c_void);

struct ObserverBindingCtx {
    each: Option<*mut c_void>,
    entity_each: Option<*mut c_void>,
    free_each: Option<EcsCtxFreeT>,
    free_entity_each: Option<EcsCtxFreeT>,
}

impl Drop for ObserverBindingCtx {
    fn drop(&mut self) {
        if let Some(each) = self.each {
            if let Some(free_each) = self.free_each {
                free_each(each);
            }
        }
        if let Some(entity_each) = self.entity_each {
            if let Some(free_entity_each) = self.free_entity_each {
                free_entity_each(entity_each);
            }
        }
    }
}

#[allow(clippy::derivable_impls)]
impl Default for ObserverBindingCtx {
    fn default() -> Self {
        Self {
            each: None,
            entity_each: None,
            free_each: None,
            free_entity_each: None,
        }
    }
}
impl ObserverBindingCtx {
    pub fn new(
        each: Option<*mut std::ffi::c_void>,
        entity_each: Option<*mut std::ffi::c_void>,
        free_each: Option<EcsCtxFreeT>,
        free_entity_each: Option<EcsCtxFreeT>,
    ) -> Self {
        Self {
            each,
            entity_each,
            free_each,
            free_entity_each,
        }
    }
}

pub struct ObserverBuilder<'a, 'w, T>
where
    T: Iterable<'a>,
{
    filter_builder: FilterBuilder<'a, 'w, T>,
    desc: ecs_observer_desc_t,
    event_count: i32,
    /// non-owning world reference
    world: World,
}

/// Deref to FilterBuilder to allow access to FilterBuilder methods without having to access FilterBuilder through ObserverBuilder
impl<'a, 'w, T> Deref for ObserverBuilder<'a, 'w, T>
where
    T: Iterable<'a>,
{
    type Target = FilterBuilder<'a, 'w, T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.filter_builder
    }
}

impl<'a, 'w, T> ObserverBuilder<'a, 'w, T>
where
    T: Iterable<'a>,
{
    pub fn new(world: &'w World) -> Self {
        let mut desc = Default::default();
        let mut obj = Self {
            desc,
            filter_builder: FilterBuilder::new_with_desc(world, &mut desc.filter, 0),
            world: world.clone(),
            event_count: 0,
        };
        T::populate(&mut obj);
        obj
    }

    pub fn new_named(world: &'w World, name: &str) -> Self {
        let mut obj = Self {
            desc: Default::default(),
            filter_builder: FilterBuilder::new(world),
            world: world.clone(),
            event_count: 0,
        };
        T::populate(&mut obj);
        obj
    }

    fn get_binding_ctx(&mut self) -> &mut ObserverBindingCtx {
        let mut binding_ctx: *mut ObserverBindingCtx = self.desc.binding_ctx as *mut _;

        if binding_ctx.is_null() {
            let new_binding_ctx = Box::<ObserverBindingCtx>::default();
            let static_ref = Box::leak(new_binding_ctx);
            binding_ctx = static_ref;
            self.desc.binding_ctx = binding_ctx as *mut c_void;
            self.desc.binding_ctx_free = Some(Self::binding_ctx_drop);
        }
        unsafe { &mut *binding_ctx }
    }

    extern "C" fn binding_ctx_drop(ptr: *mut c_void) {
        let ptr_struct: *mut ObserverBindingCtx = ptr as *mut ObserverBindingCtx;
        unsafe {
            ptr::drop_in_place(ptr_struct);
        }
    }

    pub fn on_each(&mut self, func: impl FnMut(T::TupleType)) -> &mut Self {
        let binding_ctx = self.get_binding_ctx();
        let each_func = Box::new(func);
        let each_static_ref = Box::leak(each_func);
        binding_ctx.each = Some(each_static_ref as *mut _ as *mut c_void);
        binding_ctx.free_each = Some(Self::on_free_each);

        self
    }

    extern "C" fn on_free_each(ptr: *mut c_void) {
        let ptr_func: *mut fn(T::TupleType) = ptr as *mut fn(T::TupleType);
        unsafe {
            ptr::drop_in_place(ptr_func);
        }
    }

    unsafe extern "C" fn run_each<Func>(iter: *mut IterT)
    where
        Func: FnMut(T::TupleType),
    {
        let ctx: *mut ObserverBindingCtx = (*iter).binding_ctx as *mut _;
        let each = (*ctx).each.unwrap();
        let each = &mut *(each as *mut Func);

        while ecs_filter_next(iter) {
            let components_data = T::get_array_ptrs_of_components(&*iter);
            let iter_count = (*iter).count as usize;
            let array_components = &components_data.array_components;

            ecs_table_lock((*iter).world, (*iter).table);

            for i in 0..iter_count {
                let tuple = if components_data.is_any_array_a_ref {
                    let is_ref_array_components = &components_data.is_ref_array_components;
                    T::get_tuple_with_ref(array_components, is_ref_array_components, i)
                } else {
                    T::get_tuple(array_components, i)
                };
                each(tuple);
            }

            ecs_table_unlock((*iter).world, (*iter).table);
        }
    }
}

impl<'a, 'w, T> Filterable for ObserverBuilder<'a, 'w, T>
where
    T: Iterable<'a>,
{
    fn get_world(&self) -> *mut WorldT {
        self.filter_builder.world.raw_world
    }

    fn current_term(&mut self) -> &mut TermT {
        self.filter_builder.current_term()
    }

    fn next_term(&mut self) {
        self.filter_builder.next_term()
    }
}

impl<'a, 'w, T> FilterBuilderImpl for ObserverBuilder<'a, 'w, T>
where
    T: Iterable<'a>,
{
    #[inline]
    fn get_desc_filter(&mut self) -> &mut ecs_filter_desc_t {
        self.filter_builder.get_desc_filter()
    }

    #[inline]
    fn get_expr_count(&mut self) -> &mut i32 {
        self.filter_builder.get_expr_count()
    }

    #[inline]
    fn get_term_index(&mut self) -> &mut i32 {
        self.filter_builder.get_term_index()
    }
}

impl<'a, 'w, T> TermBuilder for ObserverBuilder<'a, 'w, T>
where
    T: Iterable<'a>,
{
    #[inline]
    fn get_world(&self) -> *mut WorldT {
        self.filter_builder.world.raw_world
    }

    #[inline]
    fn get_term(&mut self) -> &mut super::term::Term {
        self.filter_builder.get_term()
    }

    #[inline]
    fn get_raw_term(&mut self) -> *mut TermT {
        self.filter_builder.get_raw_term()
    }

    #[inline]
    fn get_term_id(&mut self) -> *mut super::c_types::TermIdT {
        self.filter_builder.get_term_id()
    }
}

pub trait ObserverBuilderImpl: FilterBuilderImpl {
    fn get_desc_observer(&self) -> &mut ecs_observer_desc_t;

    fn get_event_count(&self) -> i32;

    fn increment_event_count(&mut self);

    fn add_event(&mut self, event: EntityT) -> &mut Self {
        let desc = self.get_desc_observer();
        let event_count = self.get_event_count() as usize;
        desc.events[event_count] = event;
        self.increment_event_count();
        self
    }

    //todo!()
    fn add_event_of_type<T>(&mut self) -> &mut Self
    where
        T: CachedComponentData,
    {
        let desc = self.get_desc_observer();
        let event_count = self.get_event_count() as usize;
        let id = T::get_id(self.get_world());
        desc.events[event_count] = id;
        self.increment_event_count();
        self
    }

    //todo!() better function name
    fn yield_existing(&mut self, should_yield: bool) -> &mut Self {
        self.get_desc_observer().yield_existing = should_yield;
        self
    }

    fn set_context(&mut self, context: *mut c_void) -> &mut Self {
        self.get_desc_observer().ctx = context;
        self
    }

    fn set_run_callback(&mut self, callback: ecs_iter_action_t) -> &mut Self {
        self.get_desc_observer().run = callback;
        self
    }
}
