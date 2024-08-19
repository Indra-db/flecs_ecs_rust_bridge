use std::marker::PhantomData;

use crate::core::*;
use crate::sys;
use flecs_ecs_derive::tuples;

// Not sure if this has any value, but keeping it here for food for thought.
// pub struct ArrayElement {
//     pub ptr: *mut u8,
//     pub index: i8,
//     pub is_ref: bool,
// }

#[doc(hidden)]
pub struct IsAnyArray {
    pub a_ref: bool, //e.g. singleton
    pub a_row: bool, //e.g. sparse
}

pub struct ComponentsData<T: QueryTuple, const LEN: usize> {
    pub array_components: [*mut u8; LEN],
    pub is_ref_array_components: [bool; LEN],
    pub is_row_array_components: [bool; LEN],
    pub index_array_components: [i8; LEN],
    pub is_any_array: IsAnyArray,
    _marker: PhantomData<T>,
}

pub trait ComponentPointers<T: QueryTuple> {
    fn new(iter: &sys::ecs_iter_t) -> Self;

    fn get_tuple(&mut self, iter: &sys::ecs_iter_t, index: usize) -> T::TupleType<'_>;
}

impl<T: QueryTuple, const LEN: usize> ComponentPointers<T> for ComponentsData<T, LEN> {
    fn new(iter: &sys::ecs_iter_t) -> Self {
        let mut array_components = [std::ptr::null::<u8>() as *mut u8; LEN];
        let mut is_ref_array_components = [false; LEN];
        let mut is_row_array_components = [false; LEN];
        let mut index_array_components = [0; LEN];

        let is_any_array = if (iter.ref_fields | iter.up_fields) != 0 {
            T::populate_array_ptrs(
                iter,
                &mut array_components[..],
                &mut is_ref_array_components[..],
                &mut is_row_array_components[..],
                &mut index_array_components[..],
            )
        } else {
            // TODO since we know there is no is_ref and this always return false, we could mitigate a branch if we
            // split up the functions
            T::populate_self_array_ptrs(iter, &mut array_components[..]);
            IsAnyArray {
                a_ref: false,
                a_row: false,
            }
        };

        Self {
            array_components,
            is_ref_array_components,
            is_row_array_components,
            index_array_components,
            is_any_array,
            _marker: PhantomData::<T>,
        }
    }

    fn get_tuple(&mut self, iter: &sys::ecs_iter_t, index: usize) -> T::TupleType<'_> {
        if self.is_any_array.a_row {
            T::create_tuple_with_row(
                iter,
                &mut self.array_components[..],
                &self.is_ref_array_components[..],
                &self.is_row_array_components[..],
                &self.index_array_components[..],
                index,
            )
        } else if self.is_any_array.a_ref {
            T::create_tuple_with_ref(
                &self.array_components[..],
                &self.is_ref_array_components[..],
                index,
            )
        } else {
            T::create_tuple(&self.array_components[..], index)
        }
    }
}

struct Singleton<T>(T);

pub trait IterableTypeOperation {
    type CastType;
    type ActualType<'w>;
    type SliceType<'w>;
    type OnlyType: ComponentOrPairId;
    type OnlyPairType: ComponentId;
    const ONE: i32 = 1;

    fn populate_term(term: &mut sys::ecs_term_t);

    fn create_tuple_data<'a>(array_components_data: *mut u8, index: usize) -> Self::ActualType<'a>;

    fn create_tuple_with_ref_data<'a>(
        array_components_data: *mut u8,
        is_ref: bool,
        index: usize,
    ) -> Self::ActualType<'a>;
}

impl<T> IterableTypeOperation for &T
where
    T: ComponentOrPairId,
{
    type CastType = *const <T as ComponentOrPairId>::CastType;
    type ActualType<'w> = &'w <T as ComponentOrPairId>::CastType;
    type SliceType<'w> = &'w [<T as ComponentOrPairId>::CastType];
    type OnlyType = T;
    type OnlyPairType = <T as ComponentOrPairId>::CastType;

    fn populate_term(term: &mut sys::ecs_term_t) {
        term.inout = InOutKind::In as i16;
    }

    fn create_tuple_data<'a>(array_components_data: *mut u8, index: usize) -> Self::ActualType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        unsafe { &*data_ptr.add(index) }
    }

    fn create_tuple_with_ref_data<'a>(
        array_components_data: *mut u8,
        is_ref: bool,
        index: usize,
    ) -> Self::ActualType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        unsafe {
            if is_ref {
                &*data_ptr.add(0)
            } else {
                &*data_ptr.add(index)
            }
        }
    }
}

impl<T> IterableTypeOperation for &mut T
where
    T: ComponentOrPairId,
{
    type CastType = *mut <T as ComponentOrPairId>::CastType;
    type ActualType<'w> = &'w mut <T as ComponentOrPairId>::CastType;
    type SliceType<'w> = &'w mut [<T as ComponentOrPairId>::CastType];
    type OnlyType = T;
    type OnlyPairType = <T as ComponentOrPairId>::CastType;

    fn populate_term(term: &mut sys::ecs_term_t) {
        term.inout = InOutKind::InOut as i16;
    }

    fn create_tuple_data<'a>(array_components_data: *mut u8, index: usize) -> Self::ActualType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        unsafe { &mut *data_ptr.add(index) }
    }

    fn create_tuple_with_ref_data<'a>(
        array_components_data: *mut u8,
        is_ref: bool,
        index: usize,
    ) -> Self::ActualType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        unsafe {
            if is_ref {
                &mut *data_ptr.add(0)
            } else {
                &mut *data_ptr.add(index)
            }
        }
    }
}

impl<T> IterableTypeOperation for Option<&T>
where
    T: ComponentOrPairId,
{
    type CastType = *const <T as ComponentOrPairId>::CastType;
    type ActualType<'w> = Option<&'w <T as ComponentOrPairId>::CastType>;
    type SliceType<'w> = Option<&'w [<T as ComponentOrPairId>::CastType]>;
    type OnlyType = T;
    type OnlyPairType = <T as ComponentOrPairId>::CastType;

    fn populate_term(term: &mut sys::ecs_term_t) {
        term.inout = InOutKind::In as i16;
        term.oper = OperKind::Optional as i16;
    }

    fn create_tuple_data<'a>(array_components_data: *mut u8, index: usize) -> Self::ActualType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        if data_ptr.is_null() {
            None
        } else {
            Some(unsafe { &*data_ptr.add(index) })
        }
    }

    fn create_tuple_with_ref_data<'a>(
        array_components_data: *mut u8,
        is_ref: bool,
        index: usize,
    ) -> Self::ActualType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        if data_ptr.is_null() {
            None
        } else if is_ref {
            Some(unsafe { &*data_ptr.add(0) })
        } else {
            Some(unsafe { &*data_ptr.add(index) })
        }
    }
}

impl<T> IterableTypeOperation for Option<&mut T>
where
    T: ComponentOrPairId,
{
    type CastType = *mut <T as ComponentOrPairId>::CastType;
    type ActualType<'w> = Option<&'w mut <T as ComponentOrPairId>::CastType>;
    type SliceType<'w> = Option<&'w mut [<T as ComponentOrPairId>::CastType]>;
    type OnlyType = T;
    type OnlyPairType = <T as ComponentOrPairId>::CastType;

    fn populate_term(term: &mut sys::ecs_term_t) {
        term.inout = InOutKind::InOut as i16;
        term.oper = OperKind::Optional as i16;
    }

    fn create_tuple_data<'a>(array_components_data: *mut u8, index: usize) -> Self::ActualType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        if data_ptr.is_null() {
            None
        } else {
            Some(unsafe { &mut *data_ptr.add(index) })
        }
    }

    fn create_tuple_with_ref_data<'a>(
        array_components_data: *mut u8,
        is_ref: bool,
        index: usize,
    ) -> Self::ActualType<'a> {
        let data_ptr = array_components_data as Self::CastType;
        if data_ptr.is_null() {
            None
        } else if is_ref {
            Some(unsafe { &mut *data_ptr.add(0) })
        } else {
            Some(unsafe { &mut *data_ptr.add(index) })
        }
    }
}

pub trait QueryTuple: Sized {
    type Pointers: ComponentPointers<Self>;
    type TupleType<'a>;
    const CONTAINS_ANY_TAG_TERM: bool;
    const COUNT: i32;

    fn create_ptrs(iter: &sys::ecs_iter_t) -> Self::Pointers {
        Self::Pointers::new(iter)
    }

    fn populate<'a>(query: &mut impl QueryBuilderImpl<'a>);

    fn register_ids_descriptor(world: *mut sys::ecs_world_t, desc: &mut sys::ecs_query_desc_t) {
        Self::register_ids_descriptor_at(world, &mut desc.terms[..], &mut 0);
    }

    fn register_ids_descriptor_at(
        world: *mut sys::ecs_world_t,
        terms: &mut [sys::ecs_term_t],
        index: &mut usize,
    );

    fn populate_array_ptrs(
        it: &sys::ecs_iter_t,
        components: &mut [*mut u8],
        is_ref: &mut [bool],
        is_row: &mut [bool],
        indexes: &mut [i8],
    ) -> IsAnyArray;

    fn populate_self_array_ptrs(it: &sys::ecs_iter_t, components: &mut [*mut u8]);

    fn create_tuple(array_components: &[*mut u8], index: usize) -> Self::TupleType<'_>;

    fn create_tuple_with_ref<'a>(
        array_components: &'a [*mut u8],
        is_ref_array_components: &[bool],
        index: usize,
    ) -> Self::TupleType<'a>;

    fn create_tuple_with_row<'a>(
        iter: *const sys::ecs_iter_t,
        array_components: &'a mut [*mut u8],
        is_ref_array_components: &[bool],
        is_row_array_components: &[bool],
        indexes_array_components: &[i8],
        index_row_entity: usize,
    ) -> Self::TupleType<'a>;
}

/////////////////////
// The higher sized tuples are done by a macro towards the bottom of this file.
/////////////////////

#[rustfmt::skip]
impl<A> QueryTuple for A
where
    A: IterableTypeOperation,
{ 
    type Pointers = ComponentsData<A, 1>;
    type TupleType<'w> = A::ActualType<'w>;
    const CONTAINS_ANY_TAG_TERM: bool = <<A::OnlyPairType as ComponentId>::UnderlyingType as ComponentInfo>::IS_TAG;
    const COUNT : i32 = 1;

    fn populate<'a>(query: &mut impl QueryBuilderImpl<'a>) {
        let id = <A::OnlyType as ComponentOrPairId>::get_id(query.world());

        ecs_assert!(
        {
            if (id & (sys::ECS_ID_FLAGS_MASK as u64)) == 0 {
                let ti =  unsafe { sys::ecs_get_type_info(query.world_ptr(), id) };
                if !ti.is_null() {
                    // Union relationships always return a value of type
                    // flecs::entity_t which holds the target id of the 
                    // union relationship.
                    // If a union component with a non-zero size (like an  
                    // enum) is added to the query signature, the each/iter
                    // functions would accept a parameter of the component
                    // type instead of flecs::entity_t, which would cause
                    // an assert.
                    (unsafe { (*ti).size == 0 } || !unsafe { sys::ecs_has_id(query.world_ptr(), id, *flecs::Union)})
                } else { true }
            } else { true }
        }, FlecsErrorCode::InvalidParameter, "use `with` method to add union relationship");
        
        query.with_id(id);
        let term = query.current_term_mut();
        A::populate_term(term);

    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn register_ids_descriptor_at(
        world: *mut sys::ecs_world_t,
        terms: &mut [sys::ecs_term_t],
        index: &mut usize,
    ) {
        let world = unsafe { WorldRef::from_ptr(world) };
        terms[*index].id = <A::OnlyType as ComponentOrPairId>::get_id(world);
        A::populate_term(&mut terms[*index]);
        *index += 1;
    }

    fn populate_array_ptrs(
        it: &sys::ecs_iter_t,
        components: &mut [*mut u8],
        is_ref: &mut [bool],
        is_row: &mut [bool],
        indexes: &mut [i8],
    ) -> IsAnyArray {
        if it.row_fields & (1u32 << 0) != 0 {
            is_ref[0] = unsafe { *it.sources.add(0) != 0 };
            is_row[0] = true;
            indexes[0] = 0;
        } else {
            components[0] = unsafe { ecs_field::<A::OnlyPairType>(it, 0) as *mut u8 };
            is_ref[0] = unsafe { *it.sources.add(0) != 0 };
        };
        IsAnyArray {
            a_ref: is_ref[0],
            a_row: is_row[0],
        }
    }

    fn populate_self_array_ptrs(
        it: &sys::ecs_iter_t,
        components: &mut [*mut u8],
    ) {
        components[0] = unsafe { ecs_field::<A::OnlyPairType>(it, 0) as *mut u8 };
    }

    fn create_tuple(array_components: &[*mut u8], index: usize) -> Self::TupleType<'_> {
        A::create_tuple_data(array_components[0], index)

    }

    // TODO since it's only one component, we don't need to check if it's a ref array or not, we can just return the first element of the array
    // I think this is the case for all tuples of size 1
    fn create_tuple_with_ref<'a>(
        array_components: &'a [*mut u8],
        is_ref_array_components: &[bool],
        index: usize
    ) -> Self::TupleType<'a> {
        A::create_tuple_with_ref_data(array_components[0], is_ref_array_components[0], index)
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn create_tuple_with_row<'a>(
            iter: *const sys::ecs_iter_t,
            array_components: &'a mut [*mut u8],
            is_ref_array_components: &[bool],
            is_row_array_components: &[bool],
            indexes_array_components: &[i8],
            index_row_entity: usize
        ) -> Self::TupleType<'a> {

        if is_row_array_components[0] {
            let ptr_to_first_index_array = &mut array_components[0];
            *ptr_to_first_index_array = unsafe { ecs_field_at::<A::OnlyPairType>(iter, indexes_array_components[0], index_row_entity as i32) } as *mut u8;
        }

        A::create_tuple_with_ref_data(
            array_components[0],
            is_ref_array_components[0],
            index_row_entity,
        )
    }
}

pub struct Wrapper<T>(T);

pub trait TupleForm<'a, T, U> {
    type Tuple;
    type TupleSlice;
    const IS_OPTION: bool;

    fn return_type_for_tuple(array: *mut U, index: usize) -> Self::Tuple;
    fn return_type_for_tuple_with_ref(array: *mut U, is_ref: bool, index: usize) -> Self::Tuple;
}

impl<'a, T: 'a> TupleForm<'a, T, T> for Wrapper<T> {
    type Tuple = &'a mut T;
    type TupleSlice = &'a mut [T];
    const IS_OPTION: bool = false;

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn return_type_for_tuple(array: *mut T, index: usize) -> Self::Tuple {
        unsafe { &mut (*array.add(index)) }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn return_type_for_tuple_with_ref(array: *mut T, is_ref: bool, index: usize) -> Self::Tuple {
        unsafe {
            if is_ref {
                &mut (*array.add(0))
            } else {
                &mut (*array.add(index))
            }
        }
    }
}

impl<'a, T: 'a> TupleForm<'a, Option<T>, T> for Wrapper<T> {
    type Tuple = Option<&'a mut T>;
    type TupleSlice = Option<&'a mut [T]>;
    const IS_OPTION: bool = true;

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn return_type_for_tuple(array: *mut T, index: usize) -> Self::Tuple {
        unsafe {
            if array.is_null() {
                None
            } else {
                Some(&mut (*array.add(index)))
            }
        }
    }

    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    #[inline(always)]
    fn return_type_for_tuple_with_ref(array: *mut T, is_ref: bool, index: usize) -> Self::Tuple {
        unsafe {
            if array.is_null() {
                None
            } else if is_ref {
                Some(&mut (*array.add(0)))
            } else {
                Some(&mut (*array.add(index)))
            }
        }
    }
}

macro_rules! tuple_count {
    () => { 0 };
    ($head:ident) => { 1 };
    ($head:ident, $($tail:ident),*) => { 1 + tuple_count!($($tail),*) };
}

macro_rules! impl_iterable {
    ($($t:ident),*) => {
        impl<$($t: IterableTypeOperation),*> QueryTuple for ($($t,)*) {
            type TupleType<'w> = ($(
                $t::ActualType<'w>,
            )*);

            const CONTAINS_ANY_TAG_TERM: bool = $(<<$t::OnlyPairType as ComponentId>::UnderlyingType as ComponentInfo>::IS_TAG ||)* false;

            type Pointers = ComponentsData<Self, { tuple_count!($($t),*) }>;
            const COUNT : i32 = tuple_count!($($t),*);

            fn populate<'a>(query: &mut impl QueryBuilderImpl<'a>) {
                let _world = query.world();

                $(
                    let id = <$t::OnlyType as ComponentOrPairId>::get_id(_world);

                    ecs_assert!(
                    {
                        if (id & (sys::ECS_ID_FLAGS_MASK as u64)) == 0 {
                            let ti =  unsafe { sys::ecs_get_type_info(query.world_ptr(), id) };
                            if !ti.is_null() {
                                // Union relationships always return a value of type
                                // flecs::entity_t which holds the target id of the
                                // union relationship.
                                // If a union component with a non-zero size (like an
                                // enum) is added to the query signature, the each/iter
                                // functions would accept a parameter of the component
                                // type instead of flecs::entity_t, which would cause
                                // an assert.
                                (unsafe { (*ti).size == 0 } || !unsafe { sys::ecs_has_id(query.world_ptr(), id, *flecs::Union)})
                            } else { true }
                        } else { true }
                    }, FlecsErrorCode::InvalidParameter, "use `with` method to add union relationship");

                    query.with_id(id);
                    let term = query.current_term_mut();
                    $t::populate_term(term);

                )*
            }

            #[allow(unused)]
            fn register_ids_descriptor_at(world: *mut sys::ecs_world_t, terms: &mut [sys::ecs_term_t], index: &mut usize) {
                $( $t::register_ids_descriptor_at(world, terms, index); )*
            }

            #[allow(unused)]
            fn populate_array_ptrs(
                it: &sys::ecs_iter_t,
                components: &mut [*mut u8],
                is_ref: &mut [bool],
                is_row: &mut [bool],
                indexes: &mut [i8],
            ) -> IsAnyArray {
                let mut index = 0;
                let mut any_ref = false;
                let mut any_row = false;
                $(
                    if it.row_fields & (1u32 << index) != 0 {
                        //components[index] = unsafe { ecs_field_at::<$t::OnlyPairType>(it, index as i8, 0) as *mut u8 };
                        is_ref[index as usize] =  unsafe { *it.sources.add(index as usize) != 0 };
                        is_row[index as usize] = true;
                        indexes[index as usize] = index as i8;
                    } else {
                        components[index as usize] =
                            unsafe { ecs_field::<$t::OnlyPairType>(it, index as i8) as *mut u8 };
                        is_ref[index as usize] = unsafe { *it.sources.add(index as usize) != 0 };
                    }
                    any_ref |= is_ref[index as usize];
                    any_row |= is_row[index as usize];
                    index += 1;
                )*
                IsAnyArray {
                    a_ref: any_ref,
                    a_row: any_row,
                }
            }

            #[allow(unused)]
            fn populate_self_array_ptrs(
                it: &sys::ecs_iter_t,
                components: &mut [*mut u8],
            ) {
                let mut index = 0;
                $(
                    components[index as usize] =
                        unsafe { ecs_field::<$t::OnlyPairType>(it, index) as *mut u8 };
                    index += 1;
                )*

            }

            #[allow(unused, clippy::unused_unit)]
            fn create_tuple(array_components: &[*mut u8], index: usize) -> Self::TupleType<'_> {
                let mut column: isize = -1;
                ($({
                    column += 1;
                    $t::create_tuple_data(array_components[column as usize], index)
                },)*)
            }

            #[allow(unused, clippy::unused_unit)]
            fn create_tuple_with_ref<'a>(array_components: &'a [*mut u8], is_ref_array_components: &[bool], index: usize) -> Self::TupleType<'a> {
                let mut column: isize = -1;
                ($({
                    column += 1;
                    $t::create_tuple_with_ref_data(array_components[column as usize], is_ref_array_components[column as usize], index)
                },)*)
            }

            #[allow(unused, clippy::unused_unit)]
            #[allow(clippy::not_unsafe_ptr_arg_deref)]
            fn create_tuple_with_row<'a>(
                iter: *const sys::ecs_iter_t,
                array_components: &'a mut [*mut u8],
                is_ref_array_components: &[bool],
                is_row_array_components: &[bool],
                indexes_array_components: &[i8],
                index_row_entity: usize
            ) -> Self::TupleType<'a> {
                let mut column: isize = -1;
                ($({
                    column += 1;
                    if is_row_array_components[column as usize] {
                        let ptr_to_first_index_array = &mut array_components[column as usize];
                        *ptr_to_first_index_array = unsafe { ecs_field_at::<$t::OnlyPairType>(iter, indexes_array_components[column as usize], index_row_entity as i32) } as *mut $t::OnlyPairType as *mut u8;
                    }

                    $t::create_tuple_with_ref_data(array_components[column as usize], is_ref_array_components[column as usize], index_row_entity)
                },)*)
            }
        }
    }
}

tuples!(impl_iterable, 0, 16);
