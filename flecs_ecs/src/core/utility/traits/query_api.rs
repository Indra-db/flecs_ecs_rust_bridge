use std::ffi::c_char;

use flecs_ecs::core::*;
use flecs_ecs::sys;

pub trait IterOperations {
    #[doc(hidden)]
    fn retrieve_iter(&self) -> IterT;

    #[doc(hidden)]
    fn retrieve_iter_stage<'a>(&self, stage: impl IntoWorld<'a>) -> IterT;

    #[doc(hidden)]
    fn iter_next(&self, iter: &mut IterT) -> bool;

    #[doc(hidden)]
    fn iter_next_func(&self) -> unsafe extern "C" fn(*mut IterT) -> bool;

    #[doc(hidden)]
    fn query_ptr(&self) -> *const QueryT;
}

pub trait QueryAPI<'a, P, T>: IterOperations + IntoWorld<'a>
where
    T: QueryTuple,
{
    // TODO once we have tests in place, I will split this functionality up into multiple functions, which should give a small performance boost
    // by caching if the query has used a "is_ref" operation.
    // is_ref is true for any query that contains fields that are not matched on the entity itself
    // so parents, prefabs but also singletons, or fields that are matched on a fixed entity (.with<Foo>().src_id(my_entity))
    /// Each iterator.
    /// The "each" iterator accepts a function that is invoked for each matching entity.
    /// The following function signatures is valid:
    ///  - func(comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// Each iterators are automatically instanced.
    ///
    /// # See also
    ///
    /// * C++ API: `iterable::each`
    #[doc(alias = "iterable::each")]
    fn each(&self, mut func: impl FnMut(T::TupleType<'_>)) {
        unsafe {
            let mut iter = self.retrieve_iter();
            iter.flags |= sys::EcsIterIsInstanced;
            iter.flags |= sys::EcsIterCppEach;

            while self.iter_next(&mut iter) {
                let mut components_data = T::create_ptrs(&iter);
                let iter_count = {
                    if iter.count == 0 && iter.table.is_null() {
                        1_usize
                    } else {
                        iter.count as usize
                    }
                };

                sys::ecs_table_lock(self.world_ptr_mut(), iter.table);

                for i in 0..iter_count {
                    let tuple = components_data.get_tuple(i);
                    func(tuple);
                }

                sys::ecs_table_unlock(self.world_ptr_mut(), iter.table);
            }
        }
    }

    /// Each iterator.
    /// The "each" iterator accepts a function that is invoked for each matching entity.
    /// The following function signatures is valid:
    ///  - func(e : Entity , comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// Each iterators are automatically instanced.
    ///
    /// # See also
    ///
    /// * C++ API: `iterable::each`
    #[doc(alias = "iterable::each")]
    fn each_entity(&self, mut func: impl FnMut(EntityView, T::TupleType<'_>)) {
        unsafe {
            let world = self.world_ptr_mut();
            let mut iter = self.retrieve_iter();
            iter.flags |= sys::EcsIterIsInstanced;
            iter.flags |= sys::EcsIterCppEach;

            while self.iter_next(&mut iter) {
                let mut components_data = T::create_ptrs(&iter);
                let iter_count = {
                    if iter.count == 0 && iter.table.is_null() {
                        1_usize
                    } else {
                        iter.count as usize
                    }
                };

                ecs_assert!(
                    iter.count > 0,
                    FlecsErrorCode::InvalidOperation,
                    "no entities returned, use each() without flecs::entity argument",
                );

                sys::ecs_table_lock(world, iter.table);

                // TODO random thought, I think I can determine the elements is a ref or not before the for loop and then pass two arrays with the indices of the ref and non ref elements
                // I will come back to this in the future, my thoughts are somewhere else right now. If my assumption is correct, this will get rid of the branch in the for loop
                // and potentially allow for more conditions for vectorization to happen. This could potentially offer a (small) performance boost since the branch predictor avoids probably
                // most of the cost since the branch is almost always the same.
                // update: I believe it's not possible due to not knowing the order of the components in the tuple. I will leave this here for now, maybe I will come back to it in the future.
                for i in 0..iter_count {
                    let world = self.world();
                    let tuple = components_data.get_tuple(i);

                    func(EntityView::new_from(world, *iter.entities.add(i)), tuple);
                }

                sys::ecs_table_unlock(world, iter.table);
            }
        }
    }

    fn each_iter(&self, mut func: impl FnMut(TableIter<false, P>, usize, T::TupleType<'_>))
    where
        P: ComponentId,
    {
        unsafe {
            let world = self.world_ptr_mut();
            let mut iter = self.retrieve_iter();
            iter.flags |= sys::EcsIterIsInstanced;
            iter.flags |= sys::EcsIterCppEach;

            while self.iter_next(&mut iter) {
                let mut components_data = T::create_ptrs(&iter);
                let iter_count = {
                    if iter.count == 0 && iter.table.is_null() {
                        1_usize
                    } else {
                        iter.count as usize
                    }
                };

                sys::ecs_table_lock(world, iter.table);

                for i in 0..iter_count {
                    let iter_t = TableIter::new(&mut iter);
                    let tuple = components_data.get_tuple(i);

                    func(iter_t, i, tuple);
                }

                sys::ecs_table_unlock(world, iter.table);
            }
        }
    }

    /// find iterator to find an entity
    /// The "find" iterator accepts a function that is invoked for each matching entity and checks if the condition is true.
    /// if it is, it returns that entity.
    /// The following function signatures is valid:
    ///  - func(comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// Each iterators are automatically instanced.
    ///
    /// # Returns
    ///
    /// * `Some(EntityView<'_>)` if the entity was found, `None` if no entity was found.
    ///
    /// # See also
    ///
    /// * C++ API: `find_delegate::invoke_callback`
    #[doc(alias = "find_delegate::invoke_callback")]
    fn find(&self, mut func: impl FnMut(T::TupleType<'_>) -> bool) -> Option<EntityView<'a>> {
        unsafe {
            let mut iter = self.retrieve_iter();
            let mut entity: Option<EntityView> = None;
            let world = self.world_ptr_mut();

            while self.iter_next(&mut iter) {
                let mut components_data = T::create_ptrs(&iter);
                let iter_count = iter.count as usize;

                sys::ecs_table_lock(world, iter.table);

                for i in 0..iter_count {
                    let world = self.world();
                    let tuple = components_data.get_tuple(i);
                    if func(tuple) {
                        entity = Some(EntityView::new_from(world, *iter.entities.add(i)));
                        break;
                    }
                }

                sys::ecs_table_unlock(world, iter.table);
            }
            entity
        }
    }

    /// find iterator to find an entity
    /// The "find" iterator accepts a function that is invoked for each matching entity and checks if the condition is true.
    /// if it is, it returns that entity.
    /// The following function signatures is valid:
    ///  - func(entity : Entity, comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// Each iterators are automatically instanced.
    ///
    /// # Returns
    ///
    /// * `Some(EntityView<'_>)` if the entity was found, `None` if no entity was found.
    ///
    /// # See also
    ///
    /// * C++ API: `find_delegate::invoke_callback`
    #[doc(alias = "find_delegate::invoke_callback")]
    fn find_entity(
        &self,
        mut func: impl FnMut(EntityView, T::TupleType<'_>) -> bool,
    ) -> Option<EntityView<'a>> {
        unsafe {
            let mut iter = self.retrieve_iter();
            let mut entity_result: Option<EntityView> = None;
            let world = self.world_ptr_mut();

            while self.iter_next(&mut iter) {
                let mut components_data = T::create_ptrs(&iter);
                let iter_count = iter.count as usize;

                sys::ecs_table_lock(world, iter.table);

                for i in 0..iter_count {
                    let world = self.world();
                    let entity = EntityView::new_from(world, *iter.entities.add(i));

                    let tuple = components_data.get_tuple(i);
                    if func(entity, tuple) {
                        entity_result = Some(entity);
                        break;
                    }
                }

                sys::ecs_table_unlock(world, iter.table);
            }
            entity_result
        }
    }

    /// find iterator to find an entity.
    /// The "find" iterator accepts a function that is invoked for each matching entity and checks if the condition is true.
    /// if it is, it returns that entity.
    /// The following function signatures is valid:
    ///  - func(iter : `TableIter`, index : usize, comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// Each iterators are automatically instanced.
    ///
    /// # Returns
    ///
    /// * `Some(EntityView<'_>)` if the entity was found, `None` if no entity was found.
    ///
    /// # See also
    ///
    /// * C++ API: `find_delegate::invoke_callback`
    #[doc(alias = "find_delegate::invoke_callback")]
    fn find_iter(
        &self,
        mut func: impl FnMut(TableIter<false, P>, usize, T::TupleType<'_>) -> bool,
    ) -> Option<EntityView<'a>>
    where
        P: ComponentId,
    {
        unsafe {
            let mut iter = self.retrieve_iter();
            let mut entity_result: Option<EntityView> = None;
            let world = self.world_ptr_mut();

            while self.iter_next(&mut iter) {
                let mut components_data = T::create_ptrs(&iter);
                let iter_count = {
                    if iter.count == 0 {
                        1_usize
                    } else {
                        iter.count as usize
                    }
                };

                sys::ecs_table_lock(world, iter.table);

                for i in 0..iter_count {
                    let iter_t = TableIter::new(&mut iter);
                    let world = self.world();
                    let tuple = components_data.get_tuple(i);
                    if func(iter_t, i, tuple) {
                        entity_result = Some(EntityView::new_from(world, *iter.entities.add(i)));
                        break;
                    }
                }

                sys::ecs_table_unlock(world, iter.table);
            }
            entity_result
        }
    }

    /// run iter iterator. loops the iterator automatically compared to `run()`
    ///
    /// The "iter" iterator accepts a function that is invoked for each matching
    /// table. The following function signature is valid:
    ///  - func(it: &mut `TableIter`, comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// [`TableIter`] iterators are not automatically instanced. When a result contains
    /// shared components, entities of the result will be iterated one by one.
    /// This ensures that applications can't accidentally read out of bounds by
    /// accessing a shared component as an array.
    ///
    /// # Example
    /// ```
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
    /// world
    ///     .entity()
    ///     .add::<Tag>()
    ///     .add::<Position>()
    ///     .set(Velocity { x: 3, y: 4 });
    ///
    /// world
    ///     .entity()
    ///     .add::<Tag>()
    ///     .add::<Position>()
    ///     .set(Velocity { x: 1, y: 2 });
    ///
    /// world
    ///     .entity()
    ///     .add::<Position>()
    ///     .set(Velocity { x: 3, y: 4 });
    ///
    /// let mut count_entities = 0;
    /// let mut count_tables = 0;
    ///
    /// world
    ///     .new_query::<(&mut Position, &Velocity)>()
    ///     .run_iter(|it, (pos, vel)| {
    ///         count_tables += 1;
    ///         for i in it.iter() {
    ///             count_entities += 1;
    ///             let entity = it.entity(i);
    ///             pos[i].x += vel[i].x;
    ///             pos[i].y += vel[i].y;
    ///             println!("{:?}: {:?}", entity, pos[i]);
    ///         }
    ///     });
    ///
    /// assert_eq!(count_tables, 2);
    /// assert_eq!(count_entities, 3);
    ///
    /// // Output:
    /// // Entity name:  -- id: 508 -- archetype: flecs_ecs.Tag, flecs_ecs.Position, flecs_ecs.Velocity: Position { x: 3, y: 4 }
    /// // Entity name:  -- id: 510 -- archetype: flecs_ecs.Tag, flecs_ecs.Position, flecs_ecs.Velocity: Position { x: 1, y: 2 }
    /// // Entity name:  -- id: 511 -- archetype: flecs_ecs.Position, flecs_ecs.Velocity: Position { x: 3, y: 4 }
    /// ```
    fn run_iter(&self, mut func: impl FnMut(TableIter<false, P>, T::TupleSliceType<'_>))
    where
        P: ComponentId,
    {
        unsafe {
            let mut iter = self.retrieve_iter();
            let world = self.world_ptr_mut();

            while self.iter_next(&mut iter) {
                let mut components_data = T::create_ptrs(&iter);
                let iter_count = iter.count as usize;

                sys::ecs_table_lock(world, iter.table);

                let tuple = components_data.get_slice(iter_count);
                let iter_t = TableIter::new(&mut iter);
                func(iter_t, tuple);

                sys::ecs_table_unlock(world, iter.table);
            }
        }
    }

    /// Run iterator.
    ///
    /// The "run" iterator accepts a function that is invoked for each matching
    /// table. The following function signature is valid:
    ///  - func(it: &mut `TableIter`)
    ///
    /// allows for more control over how entities
    /// are iterated as it provides multiple entities in the same callback
    /// and allows to determine what should happen before and past iteration.
    ///
    /// [`TableIter`] iterators are not automatically instanced. When a result contains
    /// shared components, entities of the result will be iterated one by one.
    /// This ensures that applications can't accidentally read out of bounds by
    /// accessing a shared component as an array.
    ///
    /// # Example
    ///
    /// ```
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
    /// let query = world.new_query::<(&Tag, &Position)>();
    ///
    /// let mut count_tables = 0;
    /// let mut count_entities = 0;
    ///
    /// query.run(|mut it| {
    ///     println!("start operations");
    ///     while it.next() {
    ///         count_tables += 1;
    ///         let pos = it.field::<&Position>(1).unwrap(); //at index 1 in (&Tag, &Position)
    ///         for i in it.iter() {
    ///             count_entities += 1;
    ///             let entity = it.entity(i);
    ///             println!("{:?}: {:?}", entity, pos[i]);
    ///         }
    ///     }
    ///     println!("end operations");
    /// });
    ///
    /// assert_eq!(count_tables, 2);
    /// assert_eq!(count_entities, 3);
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
    fn run(&self, mut func: impl FnMut(TableIter<true, P>))
    where
        P: ComponentId,
    {
        let mut iter = self.retrieve_iter();
        let mut iter_t = unsafe { TableIter::new(&mut iter) };
        iter_t.iter_mut().flags &= !sys::EcsIterIsValid;
        func(iter_t);

        //TODO flecs will integrate this before V4 release.
        // // if the while loop with `iter_next()` exited early, the table should be unlocked
        // // and iter should be cleaned up
        // if (iter.flags & sys::EcsIterIsValid) != 0 && !iter.table.is_null() {
        //     unsafe {
        //         sys::ecs_table_unlock(iter.world, iter.table);
        //         sys::ecs_iter_fini(&mut iter)
        //     };
        // }

        // ecs_assert!(
        //     iter.flags & sys::EcsIterIsValid != 0,
        //     FlecsErrorCode::InvalidOperation,
        //     "when using `run()`, you must call `while it.next()` in the callback"
        // );
    }

    /// Run iterator with each forwarding.
    /// The "iter" iterator accepts a function that is invoked for each matching
    /// table. The following function signature is valid:
    ///  - `func`: (it: &mut `TableIter`) + `func_each`: (comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// allows for more control over how entities
    /// are iterated as it provides multiple entities in the same callback
    /// and allows to determine what should happen before and past iteration.
    ///
    /// [`TableIter`] iterators are not automatically instanced. When a result contains
    /// shared components, entities of the result will be iterated one by one.
    /// This ensures that applications can't accidentally read out of bounds by
    /// accessing a shared component as an array.
    ///
    /// # Example
    ///
    /// ```
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
    /// let query = world.new_query::<(&Tag, &Position)>();
    ///
    /// let mut count_tables = 0;
    /// let mut count_entities = 0;
    ///
    /// query.run_each(
    ///     |mut it| {
    ///         println!("start operations");
    ///         while it.next() {
    ///             count_tables += 1;
    ///             it.each();
    ///         }
    ///         println!("end operations");
    ///     },
    ///     |(tag, pos)| {
    ///         count_entities += 1;
    ///         println!("{:?}, {:?}", tag, pos);
    ///     },
    /// );
    ///
    /// assert_eq!(count_tables, 2);
    /// assert_eq!(count_entities, 3);
    ///
    /// // Output:
    /// //  start operations
    /// //  Tag, Position { x: 0, y: 0 }
    /// //  Tag, Position { x: 0, y: 0 }
    /// //  Tag, Position { x: 0, y: 0 }
    /// //  end operations
    /// ```
    ///
    /// # See also
    ///
    /// * C++ API: `iterable::run`
    #[doc(alias = "iterable::run")]
    fn run_each<FuncEach>(&self, mut func: impl FnMut(TableIter<true, P>), mut func_each: FuncEach)
    where
        P: ComponentId,
        FuncEach: FnMut(T::TupleType<'_>),
    {
        let mut iter = self.retrieve_iter();
        iter.callback_ctx = &mut func_each as *mut _ as *mut std::ffi::c_void;
        iter.callback = Some(
            __internal_query_execute_each::<T, FuncEach>
                as unsafe extern "C" fn(*mut sys::ecs_iter_t),
        );
        let mut iter_t = unsafe { TableIter::new(&mut iter) };
        iter_t.iter_mut().flags &= !sys::EcsIterIsValid;
        func(iter_t);
        iter.callback = None;
        iter.callback_ctx = std::ptr::null_mut();
    }

    /// Run iterator with each entity forwarding.
    ///
    /// The "iter" iterator accepts a function that is invoked for each matching
    /// table. The following function signature is valid:
    /// - `func`: (it: &mut `TableIter`) + `func_each`: (entity: Entity, comp1 : &mut T1, comp2 : &mut T2, ...)
    ///
    /// allows for more control over how entities
    /// are iterated as it provides multiple entities in the same callback
    /// and allows to determine what should happen before and past iteration.
    ///
    /// [`TableIter`] iterators are not automatically instanced. When a result contains
    /// shared components, entities of the result will be iterated one by one.
    /// This ensures that applications can't accidentally read out of bounds by
    ///
    /// # Example
    ///
    /// ```
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
    /// let query = world.new_query::<(&Tag, &Position)>();
    ///
    /// let mut count_tables = 0;
    /// let mut count_entities = 0;
    ///
    /// query.run_each_entity(
    ///     |mut it| {
    ///         println!("start operations");
    ///         while it.next() {
    ///             count_tables += 1;
    ///             it.each();
    ///         }
    ///         println!("end operations");
    ///     },
    ///     |e, (tag, pos)| {
    ///         count_entities += 1;
    ///         println!("{:?} : {:?}, {:?}", e, tag, pos);
    ///     },
    /// );
    ///
    /// assert_eq!(count_tables, 2);
    /// assert_eq!(count_entities, 3);
    ///
    /// // Output:
    /// //  start operations
    /// //  Entity name:  -- id: 508 -- archetype: flecs_ecs.main.Tag, flecs_ecs.main.Position : Tag, Position { x: 0, y: 0 }
    /// //  Entity name:  -- id: 511 -- archetype: flecs_ecs.main.Tag, flecs_ecs.main.Position : Tag, Position { x: 0, y: 0 }
    /// //  Entity name:  -- id: 512 -- archetype: flecs_ecs.main.Tag, flecs_ecs.main.Position, flecs_ecs.main.Velocity : Tag, Position { x: 0, y: 0 }
    /// //  end operations
    /// ```
    ///
    /// # See also
    ///
    /// * C++ API: `iterable::run`
    #[doc(alias = "iterable::run")]
    fn run_each_entity<FuncEachEntity>(
        &self,
        mut func: impl FnMut(TableIter<true, P>),
        mut func_each: FuncEachEntity,
    ) where
        P: ComponentId,
        FuncEachEntity: FnMut(EntityView, T::TupleType<'_>),
    {
        let mut iter = self.retrieve_iter();
        iter.callback_ctx = &mut func_each as *mut _ as *mut std::ffi::c_void;
        iter.callback = Some(
            __internal_query_execute_each_entity::<T, FuncEachEntity>
                as unsafe extern "C" fn(*mut sys::ecs_iter_t),
        );
        let mut iter_t = unsafe { TableIter::new(&mut iter) };
        iter_t.iter_mut().flags &= !sys::EcsIterIsValid;
        func(iter_t);
        iter.callback = None;
        iter.callback_ctx = std::ptr::null_mut();
    }

    /// Get the entity of the current filter
    ///
    /// # Arguments
    ///
    /// * `filter`: the filter to get the entity from
    ///
    /// # Returns
    ///
    /// The entity of the current filter
    ///
    /// # See also
    ///
    /// * C++ API: `query_base::entity`
    #[doc(alias = "query_base::entity")]
    fn entity(&self) -> EntityView;

    /// Each term iterator.
    /// The `each_term` iterator accepts a function that is invoked for each term
    /// in the filter. The following function signature is valid:
    ///  - func(term: &mut Term)
    ///
    /// # See also
    ///
    /// * C++ API: `query_base::term`
    #[doc(alias = "query_base::each_term")]
    fn each_term(&self, mut func: impl FnMut(&TermRef)) {
        let query = self.query_ptr();
        ecs_assert!(
            !query.is_null(),
            FlecsErrorCode::InvalidParameter,
            "query filter is null"
        );
        let query = unsafe { &*query };
        for i in 0..query.term_count {
            let term = TermRef::new(&query.terms[i as usize]);
            func(&term);
        }
    }

    /// Get a immutable reference of the term of the current query at the given index
    /// This is mostly used for debugging purposes.
    ///
    /// # Arguments
    ///
    /// * `index`: the index of the term to get
    /// * `filter`: the filter to get the term from
    ///
    /// # Returns
    ///
    /// The term requested
    ///
    /// # See also
    ///
    /// * C++ API: `query_base::term`
    #[doc(alias = "query_base::term")]
    fn term(&self, index: usize) -> TermRef<'_> {
        let query = self.query_ptr();
        ecs_assert!(
            !query.is_null(),
            FlecsErrorCode::InvalidParameter,
            "query filter is null"
        );
        let query = unsafe { &*query };
        TermRef::new(&query.terms[index])
    }

    /// Get the field count of the current filter
    ///
    /// # Arguments
    ///
    /// * `filter`: the filter to get the field count from
    ///
    /// # Returns
    ///
    /// The field count of the current filter
    ///
    /// # See also
    ///
    /// * C++ API: `query_base::field_count`
    #[doc(alias = "query_base::field_count")]
    fn field_count(&self) -> i8 {
        let query = self.query_ptr();
        unsafe { (*query).field_count }
    }

    /// Get the count of terms set of the current query
    fn term_count(&self) -> u32 {
        let query = self.query_ptr();
        unsafe { (*query).term_count as u32 }
    }

    /// Convert filter to string expression. Convert filter terms to a string expression.
    /// The resulting expression can be parsed to create the same filter.
    ///
    /// # Arguments
    ///
    /// * `filter`: the filter to convert to a string
    ///
    /// # Returns
    ///
    /// The string representation of the filter
    ///
    /// # See also
    ///
    /// * C++ API: `query_base::str`
    #[doc(alias = "query_base::str")]
    #[allow(clippy::inherent_to_string)] // this is a wrapper around a c function
    fn to_string(&self) -> String {
        let query = self.query_ptr();
        let result: *mut c_char = unsafe { sys::ecs_query_str(query as *const _) };
        let rust_string =
            String::from(unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() });
        unsafe {
            if let Some(free_func) = sys::ecs_os_api.free_ {
                free_func(result as *mut _);
            }
        }
        rust_string
    }

    fn find_var(&self, name: &str) -> Option<i32> {
        let name = compact_str::format_compact!("{}\0", name);

        let var_index =
            unsafe { sys::ecs_query_find_var(self.query_ptr(), name.as_ptr() as *const _) };
        if var_index == -1 {
            None
        } else {
            Some(var_index)
        }
    }

    fn plan(&self) -> String {
        let query = self.query_ptr();
        let result: *mut c_char = unsafe { sys::ecs_query_plan(query as *const _) };
        let rust_string =
            String::from(unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() });
        unsafe {
            if let Some(free_func) = sys::ecs_os_api.free_ {
                free_func(result as *mut _);
            }
        }
        rust_string
    }

    fn iterable(&self) -> QueryIter<P, T> {
        QueryIter::new(self.retrieve_iter(), self.iter_next_func())
    }

    fn iter_stage(&'a self, stage: impl IntoWorld<'a>) -> QueryIter<'a, P, T> {
        QueryIter::new(self.retrieve_iter_stage(stage), self.iter_next_func())
    }

    /// Return first matching entity.
    ///
    /// # See also
    ///
    /// * C++ API: `iterable::first`
    /// * C++ API: `iter_iterable::first`
    #[doc(alias = "iterable::first")]
    #[doc(alias = "iter_iterable::first")]
    fn first_entity(&mut self) -> Option<EntityView<'a>> {
        let mut entity = None;

        let it = &mut self.retrieve_iter();

        if self.iter_next(it) && it.count > 0 {
            entity = Some(EntityView::new_from(self.world(), unsafe {
                *it.entities.add(0)
            }));
            unsafe { sys::ecs_iter_fini(it) };
        }
        entity
    }

    /// Returns true if iterator yields at least once result.
    fn is_true(&mut self) -> bool {
        let mut it = self.retrieve_iter();

        let result = self.iter_next(&mut it);
        if result {
            unsafe { sys::ecs_iter_fini(&mut it) };
        }
        result
    }

    /// Return total number of entities in result.
    ///
    /// # Returns
    ///
    /// The total number of entities in the result
    ///
    /// # See also
    ///
    /// * C++ API: `iter_iterable::count`
    #[doc(alias = "iter_iterable::count")]
    fn count(&mut self) -> i32 {
        let mut it = self.retrieve_iter();
        let mut result = 0;
        while self.iter_next(&mut it) {
            result += it.count;
        }
        result
    }

    /// Limit results to tables with specified group id (grouped queries only)
    ///
    /// # Arguments
    ///
    /// * `group_id`: the group id to set
    ///
    /// # See also
    ///
    /// * C++ API: `iter_iterable::set_group`
    #[doc(alias = "iter_iterable::set_group")]
    fn set_group_id(&mut self, group_id: impl Into<Entity>) -> QueryIter<P, T> {
        let mut iter = self.iterable();
        iter.set_group_id(group_id);
        iter
    }

    /// Limit results to tables with specified group id (grouped queries only)
    ///
    /// # Type parameters
    ///
    /// * `Group`: the group to set
    ///
    /// # See also
    ///
    /// * C++ API: `iter_iterable::set_group`
    #[doc(alias = "iter_iterable::set_group")]
    fn set_group<Group: ComponentId>(&mut self) -> QueryIter<P, T> {
        let mut iter = self.iterable();
        iter.set_group::<Group>();
        iter
    }

    /// set variable of iter
    ///
    /// # Arguments
    ///
    /// * `var_id`: the variable id to set
    ///
    /// * `value`: the value to set
    ///
    /// # See also
    ///
    /// * C++ API: `iterable::set_var`
    #[doc(alias = "iterable::set_var")]
    fn set_var(&mut self, var_id: i32, value: impl Into<Entity>) -> QueryIter<P, T> {
        let mut iter = self.iterable();
        iter.set_var(var_id, value);
        iter
    }

    /// set variable of iter as table
    ///
    /// # Arguments
    ///
    /// * `var_id`: the variable id to set
    ///
    /// * `range`: the range to set
    ///
    /// # See also
    ///
    /// * C++ API: `iter_iterable::set_var`
    #[doc(alias = "iter_iterable::set_var")]
    fn set_var_table(&mut self, var_id: i32, table: impl IntoTableRange) -> QueryIter<P, T> {
        let mut iter = self.iterable();
        iter.set_var_table(var_id, table);
        iter
    }

    /// set variable for rule iter
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the variable to set
    /// * `value`: the value to set
    ///
    /// # See also
    ///
    /// * C++ API: `iter_iterable::set_var`
    #[doc(alias = "iter_iterable::set_var")]
    fn set_var_expr(&mut self, name: &str, value: impl Into<Entity>) -> QueryIter<P, T> {
        let mut iter = self.iterable();
        iter.set_var_expr(name, value);
        iter
    }

    /// set variable for rule iter as table
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the variable to set
    /// * `range`: the range to set
    ///
    /// # See also
    ///
    /// * C++ API: `iter_iterable::set_var`
    #[doc(alias = "iter_iterable::set_var")]
    fn set_var_table_expr(&mut self, name: &str, table: impl IntoTableRange) -> QueryIter<P, T> {
        let mut iter = self.iterable();
        iter.set_var_table_expr(name, table);
        iter
    }

    /// Serialize iterator result to JSON.
    ///
    /// # See also
    ///
    /// * C++ API: `iter::to_json`
    #[doc(alias = "iter::to_json")]
    #[cfg(feature = "flecs_json")]
    fn to_json(&self, desc: Option<&crate::prelude::json::IterToJsonDesc>) -> Option<String> {
        let desc_ptr = desc
            .map(|d| d as *const crate::prelude::json::IterToJsonDesc)
            .unwrap_or(std::ptr::null());

        let mut iter = self.retrieve_iter();

        unsafe {
            let json_ptr = sys::ecs_iter_to_json(&mut iter, desc_ptr);
            if json_ptr.is_null() {
                return None;
            }
            let json = std::ffi::CStr::from_ptr(json_ptr)
                .to_str()
                .unwrap()
                .to_string();
            sys::ecs_os_api.free_.expect("os api is missing")(json_ptr as *mut std::ffi::c_void);
            Some(json)
        }
    }
}

unsafe extern "C" fn __internal_query_execute_each<T, Func>(iter: *mut IterT)
where
    T: QueryTuple,
    Func: FnMut(T::TupleType<'_>),
{
    let func = &mut *((*iter).callback_ctx as *mut Func);

    let mut components_data = T::create_ptrs(&*iter);
    let iter_count = (*iter).count as usize;

    for i in 0..iter_count {
        let tuple = components_data.get_tuple(i);
        func(tuple);
    }
}

unsafe extern "C" fn __internal_query_execute_each_entity<T, Func>(iter: *mut IterT)
where
    T: QueryTuple,
    Func: FnMut(EntityView, T::TupleType<'_>),
{
    let func = &mut *((*iter).callback_ctx as *mut Func);

    let mut components_data = T::create_ptrs(&*iter);
    let iter_count = (*iter).count as usize;
    let world = WorldRef::from_ptr((*iter).world);

    for i in 0..iter_count {
        let tuple = components_data.get_tuple(i);

        func(EntityView::new_from(world, *(*iter).entities.add(i)), tuple);
    }
}
