use std::ffi::c_void;

use crate::core::*;

pub trait SystemAPI<'a, P, T>: Builder<'a> + private::internal_SystemAPI<'a, P, T>
where
    T: QueryTuple,
    P: ComponentId,
{
    /// Set context
    ///
    /// # See also
    ///
    /// * C++ API: `observer_builder_i::ctx`
    /// * C++ API: `system_builder_i::ctx`
    #[doc(alias = "observer_builder_i::ctx")]
    #[doc(alias = "system_builder_i::ctx")]
    fn set_context(&mut self, context: *mut c_void) -> &mut Self;

    fn each<Func>(&mut self, func: Func) -> <Self as builder::Builder<'a>>::BuiltType
    where
        Func: FnMut(T::TupleType<'_>) + 'static,
    {
        const {
            assert!(
                !T::CONTAINS_ANY_TAG_TERM,
                "a type provided in the query signature is a Tag and cannot be used with `.each`. use `.run` instead or provide the tag with `.with()`"
            );
        }

        let each_func = Box::new(func);
        let each_static_ref = Box::leak(each_func);

        self.set_callback_binding_context(each_static_ref as *mut _ as *mut c_void);
        self.set_callback_binding_context_free(Some(Self::free_callback::<Func>));
        self.set_desc_callback(Some(
            Self::execute_each::<false, Func> as unsafe extern "C" fn(_),
        ));

        self.build()
    }

    fn each_entity<Func>(&mut self, func: Func) -> <Self as builder::Builder<'a>>::BuiltType
    where
        Func: FnMut(EntityView, T::TupleType<'_>) + 'static,
    {
        const {
            assert!(
                !T::CONTAINS_ANY_TAG_TERM,
                "a type provided in the query signature is a Tag and cannot be used with `.each`. use `.run` instead or provide the tag with `.with()`"
            );
        }

        let each_entity_func = Box::new(func);
        let each_entity_static_ref = Box::leak(each_entity_func);

        self.set_callback_binding_context(each_entity_static_ref as *mut _ as *mut c_void);
        self.set_callback_binding_context_free(Some(Self::free_callback::<Func>));
        self.set_desc_callback(Some(
            Self::execute_each_entity::<false, Func> as unsafe extern "C" fn(_),
        ));

        self.build()
    }

    /// Each iterator. This variant of `each` provides access to the [`TableIter`] object,
    /// which contains more information about the object being iterated.
    /// The `usize` argument contains the index of the entity being iterated,
    /// which can be used to obtain entity-specific data from the `TableIter` object.
    ///
    /// # Example
    /// ```
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component, Debug)]
    /// struct Position {
    ///     x: i32,
    ///     y: i32,
    /// }
    ///
    /// #[derive(Component, Debug)]
    /// struct Likes;
    ///
    /// let world = World::new();
    ///
    /// let eva = world.entity_named("eva");
    ///
    /// world
    ///     .entity_named("adam")
    ///     .set(Position { x: 10, y: 20 })
    ///     .add_first::<Likes>(eva);
    ///
    /// world
    /// .system::<&Position>()
    /// .with::<(Likes, flecs::Wildcard)>()
    /// .each_iter(|it, index, p| {
    ///     let e = it.entity(index);
    ///     println!("{:?}: {:?} - {:?}", e.name(), p, it.id(1).to_str());
    /// })
    /// .run();
    ///
    /// // Output:
    /// //  "adam": Position2 { x: 10, y: 20 } - "(flecs_ecs.main.Likes,eva)"
    /// ```
    fn each_iter<Func>(&mut self, func: Func) -> <Self as builder::Builder<'a>>::BuiltType
    where
        Func: FnMut(TableIter<false, P>, usize, T::TupleType<'_>) + 'static,
    {
        const {
            assert!(
                !T::CONTAINS_ANY_TAG_TERM,
                "a type provided in the query signature is a Tag and cannot be used with `.each`. use `.run` instead or provide the tag with `.with()`"
            );
        }

        let each_iter_func = Box::new(func);
        let each_iter_static_ref = Box::leak(each_iter_func);

        self.set_callback_binding_context(each_iter_static_ref as *mut _ as *mut c_void);
        self.set_callback_binding_context_free(Some(Self::free_callback::<Func>));

        self.set_desc_callback(Some(
            Self::execute_each_iter::<Func> as unsafe extern "C" fn(_),
        ));

        self.build()
    }

    /// Run iterator. This operation expects manual iteration over the tables with `iter.next()` and `iter.iter()`.
    ///
    /// The "run" iterator accepts a function that is invoked for each matching
    /// table. The following function signature is valid:
    ///  - func(it: &mut `TableIter`)
    ///
    /// allows for more control over how entities
    /// are iterated as it provides multiple entities in the same callback
    /// and allows to determine what should happen before and past iteration.
    ///
    /// # Example
    ///
    /// ```
    /// use std::{rc::Rc, cell::RefCell};
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component, Debug)]
    /// struct Tag;
    ///
    /// #[derive(Debug, Component, Default)]
    /// pub struct Position {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// #[derive(Debug, Component, Default)]
    /// pub struct Velocity {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// world.entity().add::<Tag>().add::<Position>();
    /// world.entity().add::<Tag>().add::<Position>();
    /// world
    ///     .entity()
    ///     .add::<Tag>()
    ///     .add::<Position>()
    ///     .add::<Velocity>();
    ///
    /// let count_entities = Rc::new(RefCell::new(0));
    /// let count_tables = Rc::new(RefCell::new(0));
    /// // Clone the `Rc` to retain shared ownership and move it into the closure to satisfy the 'static lifetime requirement.
    /// let count_entities_ref = count_entities.clone();
    /// let count_tables_ref = count_tables.clone();
    ///
    /// let system = world.system::<(&Tag, &Position)>().run(move |mut it| {
    ///     println!("start operations");
    ///     while it.next() {
    ///         *count_tables_ref.borrow_mut() += 1;
    ///         let pos = it.field::<&Position>(1).unwrap(); //at index 1 in (&Tag, &Position)
    ///         for i in it.iter() {
    ///             *count_entities_ref.borrow_mut() += 1;
    ///             let entity = it.entity(i);
    ///             println!("{:?}: {:?}", entity, pos[i]);
    ///         }
    ///     }
    ///     println!("end operations");
    /// });
    ///
    /// system.run();
    ///
    /// assert_eq!(*count_tables.borrow(), 2);
    /// assert_eq!(*count_entities.borrow(), 3);
    ///
    /// // Output:
    /// //  start operations
    /// //  Entity name:  -- id: 508 -- archetype: flecs_ecs.main.Tag, flecs_ecs.main.Position: Position { x: 0, y: 0 }
    /// //  Entity name:  -- id: 511 -- archetype: flecs_ecs.main.Tag, flecs_ecs.main.Position: Position { x: 0, y: 0 }
    /// //  Entity name:  -- id: 512 -- archetype: flecs_ecs.main.Tag, flecs_ecs.main.Position, flecs_ecs.main.Velocity: Position { x: 0, y: 0 }
    /// //  end operations
    /// ```
    ///
    /// # See also
    ///
    /// * C++ API: `iterable::run`
    #[doc(alias = "iterable::run")]
    fn run<Func>(&mut self, func: Func) -> <Self as builder::Builder<'a>>::BuiltType
    where
        Func: FnMut(TableIter<true, P>) + 'static,
    {
        let run = Box::new(func);
        let run_static_ref = Box::leak(run);

        self.set_run_binding_context(run_static_ref as *mut _ as *mut c_void);
        self.set_run_binding_context_free(Some(Self::free_callback::<Func>));

        self.set_desc_run(Some(Self::execute_run::<Func> as unsafe extern "C" fn(_)));
        self.build()
    }

    /// Run iterator with each forwarding. This operation expects manual
    /// iteration over the tables with `iter.next()` and `iter.each()`
    ///
    /// The "iter" iterator accepts a function that is invoked for each matching
    /// table. The following function signature is valid:
    ///  - `func`: (it: &mut `TableIter`) + `func_each`: (comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// allows for more control over how entities
    /// are iterated as it provides multiple entities in the same callback
    /// and allows to determine what should happen before and past iteration.
    ///
    /// # Example
    ///
    /// ```
    /// use std::{rc::Rc, cell::RefCell};
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component, Debug)]
    /// struct Tag;
    ///
    /// #[derive(Debug, Component, Default)]
    /// pub struct Position {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// #[derive(Debug, Component, Default)]
    /// pub struct Velocity {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// world.entity().add::<Tag>().add::<Position>();
    /// world.entity().add::<Tag>().add::<Position>();
    /// world
    ///     .entity()
    ///     .add::<Tag>()
    ///     .add::<Position>()
    ///     .add::<Velocity>();
    ///
    ///
    ///
    /// let count_entities = Rc::new(RefCell::new(0));
    /// let count_tables = Rc::new(RefCell::new(0));
    /// // Clone the `Rc` to retain shared ownership and move it into the closure to satisfy the 'static lifetime requirement.
    /// let count_entities_ref = count_entities.clone();
    /// let count_tables_ref = count_tables.clone();
    ///
    /// let system = world.system::<(&Position)>().with::<Tag>()
    /// .run_each(
    ///     move |mut it| {
    ///         println!("start operations");
    ///         while it.next() {
    ///             *count_tables_ref.borrow_mut() += 1;
    ///             it.each();
    ///         }
    ///         println!("end operations");
    ///     },
    ///     move |pos| {
    ///         *count_entities_ref.borrow_mut() += 1;
    ///         println!("{:?}", pos);
    ///     },
    /// );
    ///
    /// system.run();
    ///
    /// assert_eq!(*count_tables.borrow(), 2);
    /// assert_eq!(*count_entities.borrow(), 3);
    ///
    /// // Output:
    /// //  start operations
    /// //  Position { x: 0, y: 0 }
    /// //  Position { x: 0, y: 0 }
    /// //  Position { x: 0, y: 0 }
    /// //  end operations
    /// ```
    ///
    /// # See also
    ///
    /// * C++ API: `iterable::run`
    #[doc(alias = "iterable::run")]
    fn run_each<Func, FuncEach>(
        &mut self,
        func: Func,
        func_each: FuncEach,
    ) -> <Self as builder::Builder<'a>>::BuiltType
    where
        Func: FnMut(TableIter<true, P>) + 'static,
        FuncEach: FnMut(T::TupleType<'_>) + 'static,
    {
        let run_func = Box::new(func);
        let run_static_ref = Box::leak(run_func);

        self.set_run_binding_context(run_static_ref as *mut _ as *mut c_void);
        self.set_run_binding_context_free(Some(Self::free_callback::<Func>));

        self.set_desc_run(Some(Self::execute_run::<Func> as unsafe extern "C" fn(_)));

        let each_func = Box::new(func_each);
        let each_static_ref = Box::leak(each_func);

        self.set_callback_binding_context(each_static_ref as *mut _ as *mut c_void);
        self.set_callback_binding_context_free(Some(Self::free_callback::<FuncEach>));

        self.set_desc_callback(Some(
            Self::execute_each::<true, FuncEach> as unsafe extern "C" fn(_),
        ));

        self.build()
    }

    /// Run iterator with each entity forwarding. This operation expects manual
    /// iteration over the tables with `iter.next()` and `iter.each()`  
    ///
    /// The "iter" iterator accepts a function that is invoked for each matching
    /// table. The following function signature is valid:
    /// - `func`: (it: &mut `TableIter`) + `func_each_entity`: (entity: `EntityView`, comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// allows for more control over how entities
    /// are iterated as it provides multiple entities in the same callback
    /// and allows to determine what should happen before and past iteration.
    ///
    /// # Example
    ///
    /// ```
    /// use std::{rc::Rc, cell::RefCell};
    /// use flecs_ecs::prelude::*;
    ///
    /// #[derive(Component, Debug)]
    /// struct Tag;
    ///
    /// #[derive(Debug, Component, Default)]
    /// pub struct Position {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// #[derive(Debug, Component, Default)]
    /// pub struct Velocity {
    ///     pub x: i32,
    ///     pub y: i32,
    /// }
    ///
    /// let world = World::new();
    ///
    /// world.entity().add::<Tag>().add::<Position>();
    /// world.entity().add::<Tag>().add::<Position>();
    /// world
    ///     .entity()
    ///     .add::<Tag>()
    ///     .add::<Position>()
    ///     .add::<Velocity>();
    ///
    ///
    ///
    /// let count_entities = Rc::new(RefCell::new(0));
    /// let count_tables = Rc::new(RefCell::new(0));
    /// // Clone the `Rc` to retain shared ownership and move it into the closure to satisfy the 'static lifetime requirement.
    /// let count_entities_ref = count_entities.clone();
    /// let count_tables_ref = count_tables.clone();
    ///
    /// let system = world.system::<(&Position)>().with::<Tag>()
    /// .run_each_entity(
    ///     move |mut it| {
    ///         println!("start operations");
    ///         while it.next() {
    ///             *count_tables_ref.borrow_mut() += 1;
    ///             it.each();
    ///         }
    ///         println!("end operations");
    ///     },
    ///     move |e, pos| {
    ///         *count_entities_ref.borrow_mut() += 1;
    ///         println!("{:?}: {:?}", e, pos);
    ///     },
    /// );
    ///
    /// system.run();
    ///
    /// assert_eq!(*count_tables.borrow(), 2);
    /// assert_eq!(*count_entities.borrow(), 3);
    ///
    /// // Output:
    /// //  start operations
    /// //  Entity name:  -- id: 508 -- archetype: flecs_ecs.main.Position: Position { x: 0, y: 0 }
    /// //  Entity name:  -- id: 511 -- archetype: flecs_ecs.main.Position: Position { x: 0, y: 0 }
    /// //  Entity name:  -- id: 512 -- archetype: flecs_ecs.main.Position, flecs_ecs.main.Velocity: Position { x: 0, y: 0 }
    /// //  end operations
    /// ```
    ///
    /// # See also
    ///
    /// * C++ API: `iterable::run`
    #[doc(alias = "iterable::run")]
    fn run_each_entity<Func, FuncEachEntity>(
        &mut self,
        func: Func,
        func_each_entity: FuncEachEntity,
    ) -> <Self as builder::Builder<'a>>::BuiltType
    where
        Func: FnMut(TableIter<true, P>) + 'static,
        FuncEachEntity: FnMut(EntityView, T::TupleType<'_>) + 'static,
    {
        let run_func = Box::new(func);
        let run_static_ref = Box::leak(run_func);

        self.set_run_binding_context(run_static_ref as *mut _ as *mut c_void);
        self.set_run_binding_context_free(Some(Self::free_callback::<Func>));

        self.set_desc_run(Some(Self::execute_run::<Func> as unsafe extern "C" fn(_)));

        let each_entity_func = Box::new(func_each_entity);
        let each_entity_static_ref = Box::leak(each_entity_func);

        self.set_callback_binding_context(each_entity_static_ref as *mut _ as *mut c_void);
        self.set_callback_binding_context_free(Some(Self::free_callback::<FuncEachEntity>));

        self.set_desc_callback(Some(
            Self::execute_each_entity::<true, FuncEachEntity> as unsafe extern "C" fn(_),
        ));

        self.build()
    }
}

macro_rules! implement_reactor_api {
    ($param:ty, $type:ty) => {
        impl<'a, T> internal_SystemAPI<'a, $param, T> for $type
        where
            T: QueryTuple,
        {
            fn set_callback_binding_context(&mut self, binding_ctx: *mut c_void) -> &mut Self {
                self.desc.callback_ctx = binding_ctx;
                self
            }

            fn set_callback_binding_context_free(
                &mut self,
                binding_ctx_free: flecs_ecs_sys::ecs_ctx_free_t,
            ) -> &mut Self {
                self.desc.callback_ctx_free = binding_ctx_free;
                self
            }

            fn set_run_binding_context(&mut self, binding_ctx: *mut c_void) -> &mut Self {
                self.desc.run_ctx = binding_ctx;
                self
            }

            fn set_run_binding_context_free(
                &mut self,
                run_ctx_free: flecs_ecs_sys::ecs_ctx_free_t,
            ) -> &mut Self {
                self.desc.run_ctx_free = run_ctx_free;
                self
            }

            fn desc_binding_context(&self) -> *mut c_void {
                self.desc.callback_ctx
            }

            fn set_desc_callback(
                &mut self,
                callback: Option<unsafe extern "C" fn(*mut flecs_ecs_sys::ecs_iter_t)>,
            ) {
                self.desc.callback = callback;
            }

            fn set_desc_run(
                &mut self,
                callback: Option<unsafe extern "C" fn(*mut sys::ecs_iter_t)>,
            ) {
                self.desc.run = callback;
            }
        }

        impl<'a, T> SystemAPI<'a, $param, T> for $type
        where
            T: QueryTuple,
        {
            fn set_context(&mut self, context: *mut c_void) -> &mut Self {
                self.desc.ctx = context;
                self
            }
        }
    };
    ($type:ty) => {
        impl<'a, P, T> internal_SystemAPI<'a, P, T> for $type
        where
            T: QueryTuple,
            P: ComponentId,
        {
            fn set_callback_binding_context(&mut self, binding_ctx: *mut c_void) -> &mut Self {
                self.desc.callback_ctx = binding_ctx;
                self
            }

            fn set_callback_binding_context_free(
                &mut self,
                binding_ctx_free: flecs_ecs_sys::ecs_ctx_free_t,
            ) -> &mut Self {
                self.desc.callback_ctx_free = binding_ctx_free;
                self
            }

            fn set_run_binding_context(&mut self, binding_ctx: *mut c_void) -> &mut Self {
                self.desc.run_ctx = binding_ctx;
                self
            }

            fn set_run_binding_context_free(
                &mut self,
                run_ctx_free: flecs_ecs_sys::ecs_ctx_free_t,
            ) -> &mut Self {
                self.desc.run_ctx_free = run_ctx_free;
                self
            }

            fn desc_binding_context(&self) -> *mut c_void {
                self.desc.callback_ctx
            }

            fn set_desc_callback(
                &mut self,
                callback: Option<unsafe extern "C" fn(*mut flecs_ecs_sys::ecs_iter_t)>,
            ) {
                self.desc.callback = callback;
            }

            fn set_desc_run(
                &mut self,
                callback: Option<unsafe extern "C" fn(*mut sys::ecs_iter_t)>,
            ) {
                self.desc.run = callback;
            }
        }

        impl<'a, P, T> SystemAPI<'a, P, T> for $type
        where
            T: QueryTuple,
            P: ComponentId,
        {
            fn set_context(&mut self, context: *mut c_void) -> &mut Self {
                self.desc.ctx = context;
                self
            }
        }
    };
}

pub(crate) use implement_reactor_api;
