//! Class that extends the capabilities of a [`Query`] by providing additional operations on the query's iterator.
use std::ffi::c_void;

use crate::core::*;
use crate::sys;

pub struct QueryIter<'a, P, T>
where
    T: QueryTuple,
{
    iter: IterT,
    iter_next: unsafe extern "C" fn(*mut IterT) -> bool,
    _phantom: std::marker::PhantomData<&'a (P, T)>,
}

impl<'a, P, T> QueryIter<'a, P, T>
where
    T: QueryTuple,
{
    pub fn new(iter: IterT, iter_next: unsafe extern "C" fn(*mut IterT) -> bool) -> Self {
        Self {
            iter,
            iter_next,
            _phantom: std::marker::PhantomData,
        }
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
    pub fn set_group_id(&mut self, group_id: impl Into<Entity>) -> &mut Self {
        unsafe { sys::ecs_iter_set_group(&mut self.iter, *group_id.into()) }
        self
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
    pub fn set_group<Group: ComponentId>(&mut self) -> &mut Self {
        let world = unsafe { WorldRef::from_ptr(self.iter.real_world) };
        unsafe { sys::ecs_iter_set_group(&mut self.iter, Group::id(world)) }
        self
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
    /// * C++ API: `iter_iterable::set_var`
    #[doc(alias = "iter_iterable::set_var")]
    pub fn set_var(&mut self, var_id: i32, value: impl Into<Entity>) -> &mut Self {
        ecs_assert!(var_id != -1, FlecsErrorCode::InvalidParameter, 0);
        unsafe { sys::ecs_iter_set_var(&mut self.iter, var_id, *value.into()) };
        self
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
    pub fn set_var_table(&mut self, var_id: i32, table: impl IntoTableRange) -> &mut Self {
        ecs_assert!(var_id != -1, FlecsErrorCode::InvalidParameter, 0);
        unsafe { sys::ecs_iter_set_var_as_range(&mut self.iter, var_id, &table.table_range_raw()) };
        self
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
    pub fn set_var_expr(&mut self, name: &str, value: impl Into<Entity>) -> &mut Self {
        let name = compact_str::format_compact!("{}\0", name);

        let qit = unsafe { &mut self.iter.priv_.iter.query };
        let var_id = unsafe { sys::ecs_query_find_var(qit.query, name.as_ptr() as *const _) };
        ecs_assert!(
            var_id != -1,
            FlecsErrorCode::InvalidParameter,
            name.as_str()
        );
        unsafe { sys::ecs_iter_set_var(&mut self.iter, var_id, *value.into()) };
        self
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
    pub fn set_var_table_expr(&mut self, name: &str, table: impl IntoTableRange) -> &mut Self {
        let name = compact_str::format_compact!("{}\0", name);

        let qit = unsafe { &mut self.iter.priv_.iter.query };
        let var_id = unsafe { sys::ecs_query_find_var(qit.query, name.as_ptr() as *const _) };
        ecs_assert!(
            var_id != -1,
            FlecsErrorCode::InvalidParameter,
            name.as_str()
        );
        unsafe { sys::ecs_iter_set_var_as_range(&mut self.iter, var_id, &table.table_range_raw()) };
        self
    }
}

#[doc(hidden)]
impl<'a, P, T> IterOperations for QueryIter<'a, P, T>
where
    T: QueryTuple,
{
    fn retrieve_iter(&self) -> IterT {
        self.iter
    }

    fn retrieve_iter_stage<'w>(&self, _stage: impl IntoWorld<'w>) -> IterT {
        panic!("Cannot change the stage of an iterator that already exists. Use retrieve_iter_stage on the underlying query instead.");
    }

    fn iter_next(&self, iter: &mut IterT) -> bool {
        unsafe { (self.iter_next)(iter) }
    }

    fn query_ptr(&self) -> *const QueryT {
        self.iter.query
    }

    fn iter_next_func(&self) -> unsafe extern "C" fn(*mut IterT) -> bool {
        self.iter_next
    }
}

impl<'a, P, T> QueryAPI<'a, P, T> for QueryIter<'a, P, T>
where
    T: QueryTuple,
    Self: IntoWorld<'a>,
{
    fn entity(&self) -> EntityView {
        let world = unsafe { WorldRef::from_ptr(self.iter.real_world) };
        EntityView::new_from(world, unsafe {
            sys::ecs_get_entity(self.iter.query as *const c_void)
        })
    }
}

impl<'a, P, T> IntoWorld<'a> for QueryIter<'a, P, T>
where
    T: QueryTuple,
{
    fn world(&self) -> WorldRef<'a> {
        unsafe { WorldRef::from_ptr(self.iter.world) }
    }
}

// TODO : worker_iterable and page_iterable not implemented yet
