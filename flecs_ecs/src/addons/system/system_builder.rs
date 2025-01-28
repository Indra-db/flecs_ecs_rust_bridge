//! `SystemBuilder` is a builder pattern for creating systems.

use crate::addons::system::*;
use crate::core::internals::*;
use crate::core::private::internal_SystemAPI;
use crate::core::*;

/// `SystemBuilder` is a builder pattern for creating systems.
pub struct SystemBuilder<'a, T>
where
    T: QueryTuple,
{
    pub(crate) desc: sys::ecs_system_desc_t,
    term_builder: TermBuilder,
    world: WorldRef<'a>,
    /// Skip setting default phase (`OnUpdate`) if `kind` was set,
    /// or an existing entity was passed in (via [`Self::new_from_desc`])
    kind_set: bool,
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T> SystemBuilder<'a, T>
where
    T: QueryTuple,
{
    /// Create a new system builder
    pub(crate) fn new(world: &'a World) -> Self {
        let mut obj = Self {
            desc: Default::default(),
            term_builder: TermBuilder::default(),
            world: world.into(),
            kind_set: false,
            _phantom: std::marker::PhantomData,
        };

        obj.desc.entity = unsafe { sys::ecs_entity_init(obj.world_ptr_mut(), &Default::default()) };

        T::populate(&mut obj);

        obj
    }

    pub(crate) fn new_from_desc(world: &'a World, desc: sys::ecs_system_desc_t) -> Self {
        let mut obj = Self {
            desc,
            term_builder: TermBuilder::default(),
            world: world.into(),
            kind_set: false,
            _phantom: std::marker::PhantomData,
        };

        if obj.desc.entity == 0 {
            obj.desc.entity =
                unsafe { sys::ecs_entity_init(obj.world_ptr_mut(), &Default::default()) };
        } else {
            // Can't make assumptions about the kind on an existing entity.
            obj.kind_set = true;
        }

        T::populate(&mut obj);

        obj
    }

    /// Create a new system builder with a name
    pub(crate) fn new_named(world: &'a World, name: &str) -> Self {
        let name = compact_str::format_compact!("{}\0", name);

        let mut obj = Self {
            desc: Default::default(),
            term_builder: TermBuilder::default(),
            world: world.into(),
            kind_set: false,
            _phantom: std::marker::PhantomData,
        };

        let entity_desc: sys::ecs_entity_desc_t = sys::ecs_entity_desc_t {
            name: name.as_ptr() as *const _,
            sep: SEPARATOR.as_ptr(),
            root_sep: SEPARATOR.as_ptr(),
            ..Default::default()
        };
        obj.desc.entity = unsafe { sys::ecs_entity_init(obj.world_ptr_mut(), &entity_desc) };

        T::populate(&mut obj);

        obj
    }

    /// Specify in which phase the system should run
    ///
    /// # Arguments
    ///
    /// * `phase` - the phase
    ///
    /// # See also
    ///
    /// * C++ API: `system_builder_i::kind`
    #[doc(alias = "system_builder_i::kind")]
    pub fn kind_id(&mut self, phase: impl Into<Entity>) -> &mut Self {
        let phase = *phase.into();
        let current_phase: sys::ecs_entity_t = unsafe {
            sys::ecs_get_target(self.world_ptr_mut(), self.desc.entity, ECS_DEPENDS_ON, 0)
        };
        unsafe {
            if current_phase != 0 {
                sys::ecs_remove_id(
                    self.world_ptr_mut(),
                    self.desc.entity,
                    ecs_dependson(current_phase),
                );
                sys::ecs_remove_id(self.world_ptr_mut(), self.desc.entity, current_phase);
            }
            if phase != 0 {
                sys::ecs_add_id(self.world_ptr_mut(), self.desc.entity, ecs_dependson(phase));
                sys::ecs_add_id(self.world_ptr_mut(), self.desc.entity, phase);
                self.kind_set = true;
            }
        };
        self
    }

    /// Specify in which phase the system should run
    ///
    /// # Type Parameters
    ///
    /// * `Phase` - the phase
    ///
    /// # See also
    ///
    /// * C++ API: `system_builder_i::kind`
    #[doc(alias = "system_builder_i::kind")]
    pub fn kind<Phase>(&mut self) -> &mut Self
    where
        Phase: ComponentId + ComponentType<Struct>,
    {
        self.kind_id(Phase::id(self.world()))
    }

    /// Specify in which enum phase the system should run
    ///
    /// # Arguments
    ///
    /// * `phase` - the phase
    ///
    /// # See also
    ///
    /// * C++ API: `system_builder_i::kind`
    #[doc(alias = "system_builder_i::kind")]
    pub fn kind_enum<Phase>(&mut self, phase: Phase) -> &mut Self
    where
        Phase: ComponentId + ComponentType<Enum> + EnumComponentInfo,
    {
        let enum_id = phase.id_variant(self.world());
        self.kind_id(enum_id)
    }

    /// Specify whether system can run on multiple threads.
    ///
    /// # Arguments
    ///
    /// * `value` - if false, the system will always run on a single thread.
    ///
    /// # See also
    ///
    /// * C++ API: `system_builder_i::multi_threaded`
    #[doc(alias = "system_builder_i::multi_threaded")]
    pub fn multi_threaded(&mut self) -> &mut Self {
        self.desc.multi_threaded = true;
        self
    }

    /// Specify whether system should be ran in staged context.
    ///
    /// # Arguments
    ///
    /// * `value` - If false,  system will always run staged.
    ///
    /// # See also
    ///
    /// * C++ API: `system_builder_i::immediate`
    #[doc(alias = "system_builder_i::immediate")]
    pub fn immediate(&mut self, value: bool) -> &mut Self {
        self.desc.immediate = value;
        self
    }
}

#[doc(hidden)]
impl<'a, T: QueryTuple> internals::QueryConfig<'a> for SystemBuilder<'a, T> {
    #[inline(always)]
    fn term_builder(&self) -> &TermBuilder {
        &self.term_builder
    }

    #[inline(always)]
    fn term_builder_mut(&mut self) -> &mut TermBuilder {
        &mut self.term_builder
    }

    #[inline(always)]
    fn query_desc(&self) -> &sys::ecs_query_desc_t {
        &self.desc.query
    }

    #[inline(always)]
    fn query_desc_mut(&mut self) -> &mut sys::ecs_query_desc_t {
        &mut self.desc.query
    }
    #[inline(always)]
    fn count_generic_terms(&self) -> i32 {
        T::COUNT
    }
}

impl<'a, T: QueryTuple> TermBuilderImpl<'a> for SystemBuilder<'a, T> {}

impl<'a, T: QueryTuple> QueryBuilderImpl<'a> for SystemBuilder<'a, T> {}

impl<'a, T> Builder<'a> for SystemBuilder<'a, T>
where
    T: QueryTuple,
{
    type BuiltType = System<'a>;

    /// Build the `system_builder` into an system
    ///
    /// See also
    ///
    /// * C++ API: `node_builder::build`
    #[doc(alias = "node_builder::build")]
    fn build(&mut self) -> Self::BuiltType {
        #[cfg(feature = "flecs_pipeline")]
        if !self.kind_set {
            unsafe {
                sys::ecs_add_id(
                    self.world().world_ptr_mut(),
                    self.desc.entity,
                    ecs_dependson(ECS_ON_UPDATE),
                );
                sys::ecs_add_id(
                    self.world().world_ptr_mut(),
                    self.desc.entity,
                    ECS_ON_UPDATE,
                );
            }
        }

        let system = System::new(self.world(), self.desc);
        for s in self.term_builder.str_ptrs_to_free.iter_mut() {
            unsafe { std::mem::ManuallyDrop::drop(s) };
        }
        self.term_builder.str_ptrs_to_free.clear();
        system
    }
}

impl<'a, T: QueryTuple> WorldProvider<'a> for SystemBuilder<'a, T> {
    fn world(&self) -> WorldRef<'a> {
        self.world
    }
}

implement_reactor_api!((), SystemBuilder<'a, T>);
