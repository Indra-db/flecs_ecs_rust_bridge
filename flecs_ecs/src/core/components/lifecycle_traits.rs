#![doc(hidden)]
// Implementation for lifecycle actions.
//
// Considerations on implementation for user experience vs performance.:
//
// 1. Unlike C++, Rust doesn't support compile-time checks for trivial types.
// 2. Current implementation prioritizes simplicity over performance optimizations.
//    - If trivial type registration incurs a significant performance penalty, reconsider this approach.
//
// Challenges:
// - Rust lacks several features for this scenario:
//   a) Trait specialization.
//   b) Compile-time trivial type checks.
//   c) A direct equivalent of `placement_new` from C++.
//      `ptr::write` still constructs the object on the stack and then moves it, barring optimizations.
//
// Potential Solutions:
// - Bypass the need for `placement_new` with a `placement_ctor` function.
//   - Drawback: Each field needs manual setting, which impacts user experience.
//      - example code:
#![cfg_attr(doctest, doc = " ````no_test")]
//      ```
//           struct MyType {
//               vec: Vec<i32>,
//           }
//
//           trait PlacementNew {
//               unsafe fn placement_new(ptr: *mut Self);
//           }
//
//           impl PlacementNew for MyType {
//               unsafe fn placement_new(ptr: *mut Self) {
//                   (*ptr).vec = Vec::<i32>::default();
//               }
//           }
//      ```
// - For potential type optimizations, consider:
//   a) Utilizing the `Zeroable` trait and rely on user's proper implementation.
//   b) Implement pseudo-trait specialization, as detailed in:
//      - <http://lukaskalbertodt.github.io/2019/12/05/generalized-autoref-based-specialization.html/>
//      - <https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=1e548abff8e35b97b25adcacdddaacda/>
//
// possible helpful crates for trait specialization / type specialization:
// - For type casting: <https://crates.io/crates/castaway/>
//
// Note: C does the same, where the user needs to opt in for non trivial types. We can do the same.
// Note2: zerobit pattern

use std::{ffi::c_void, mem::MaybeUninit, ptr};

use crate::core::*;
use crate::sys;

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn register_lifecycle_actions<T>(type_hooks: &mut TypeHooksT) {
    //type_hooks.ctor = Some(ctor::<T>);
    type_hooks.dtor = Some(dtor::<T>);
    type_hooks.move_dtor = Some(move_dtor::<T>); //same implementation as ctor_move_dtor

    //type_hooks.move_ctor = Some(move_ctor::<T>);
    type_hooks.ctor_move_dtor = Some(ctor_move_dtor::<T>);

    //TODO we could potentially add an autoamtic check if the type is unmoveable to add
    //a sparse component tag
}

pub fn register_ctor_lifecycle_actions<T: Default>(type_hooks: &mut TypeHooksT) {
    type_hooks.ctor = Some(ctor::<T>);
}

pub fn register_ctor_panic_lifecycle_actions<T>(_type_hooks: &mut TypeHooksT) {
    //type_hooks.ctor = Some(panic_ctor::<T>);
}

pub fn register_copy_lifecycle_action<T: Clone>(type_hooks: &mut TypeHooksT) {
    type_hooks.copy = Some(copy::<T>);
    type_hooks.copy_ctor = Some(copy_ctor::<T>); //same implementation as copy
}

pub fn register_copy_panic_lifecycle_action<T>(type_hooks: &mut TypeHooksT) {
    type_hooks.copy = Some(panic_copy::<T>);
    type_hooks.copy_ctor = Some(panic_copy::<T>); //same implementation as copy
}

/// Initialize the memory with the default constructor.
///
/// # Arguments
///
/// * `ptr` - pointer to the memory to be initialized
/// * `count` - number of elements to be initialized
/// * `_type_info` - type info for the type to be initialized
///
/// # See also
///
/// * C++ API: `ctor_impl`
#[doc(alias = "ctor_impl")]
extern "C" fn ctor<T: Default>(
    ptr: *mut c_void,
    count: i32,
    _type_info: *const sys::ecs_type_info_t,
) {
    ecs_assert!(
        check_type_info::<T>(_type_info),
        FlecsErrorCode::InternalError
    );

    let arr = ptr as *mut MaybeUninit<T>;
    for i in 0..count as usize {
        unsafe {
            // Default construct the value in place
            MaybeUninit::write(&mut *arr.add(i), T::default());
        }
    }
}

/// Runs the destructor for the type.
///
/// # Arguments
///
/// * `ptr` - pointer to the memory to be destructed
/// * `count` - number of elements to be destructed
/// * `_type_info` - type info for the type to be destructed
///
/// # See also
///
/// * C++ API: `dtor_impl`
#[doc(alias = "dtor_impl")]
extern "C" fn dtor<T>(ptr: *mut c_void, count: i32, _type_info: *const sys::ecs_type_info_t) {
    ecs_assert!(
        check_type_info::<T>(_type_info),
        FlecsErrorCode::InternalError
    );
    let arr = ptr as *mut T;
    for i in 0..count as isize {
        unsafe {
            let item = arr.offset(i);
            ptr::drop_in_place(item);
        }
    }
}

/// This is the generic copy for trivial types
/// It will copy the memory
///
/// # See also
///
/// * C++ API: `copy_impl`
#[doc(alias = "copy_impl")]
extern "C" fn copy<T: Clone>(
    dst_ptr: *mut c_void,
    src_ptr: *const c_void,
    count: i32,
    _type_info: *const sys::ecs_type_info_t,
) {
    ecs_assert!(
        check_type_info::<T>(_type_info),
        FlecsErrorCode::InternalError
    );
    let dst_arr = dst_ptr as *mut T;
    let src_arr = src_ptr as *const T;
    for i in 0..count as isize {
        //this is safe because C manages the memory and we're cloning the internal data
        unsafe {
            let src_value = &*(src_arr.offset(i)); //get value of src
            let dst_value = dst_arr.offset(i); // get ptr to dest
            std::ptr::drop_in_place(dst_value); //calls destructor
            std::ptr::write(dst_value, src_value.clone()); //overwrite the memory of dest with new value
        }
    }
}

/// This is the generic copy for trivial types
/// It will copy the memory
///
/// # See also
///
/// * C++ API: `copy_impl`
#[doc(alias = "copy_impl")]
extern "C" fn copy_ctor<T: Clone>(
    dst_ptr: *mut c_void,
    src_ptr: *const c_void,
    count: i32,
    _type_info: *const sys::ecs_type_info_t,
) {
    ecs_assert!(
        check_type_info::<T>(_type_info),
        FlecsErrorCode::InternalError
    );
    let dst_arr = dst_ptr as *mut T;
    let src_arr = src_ptr as *const T;
    for i in 0..count as isize {
        //this is safe because C manages the memory and we're cloning the internal data
        unsafe {
            let src_value = &*(src_arr.offset(i)); //get value of src
            let dst_value = dst_arr.offset(i); // get ptr to dest
            std::ptr::write(dst_value, src_value.clone()); //overwrite the memory of dest with new value
        }
    }
}

extern "C" fn panic_ctor<T>(
    _dst_ptr: *mut c_void,
    _count: i32,
    _type_info: *const sys::ecs_type_info_t,
) {
    panic!("Default is not implemented for type {} which requires drop and it's being used in an operation which calls the constructor", std::any::type_name::<T>());
}

extern "C" fn panic_copy<T>(
    _dst_ptr: *mut c_void,
    _src_ptr: *const c_void,
    _count: i32,
    _type_info: *const sys::ecs_type_info_t,
) {
    panic!("Clone is not implemented for type {} and it's being used in a copy / duplicate operation such as component overriding or duplicating entities / components", std::any::type_name::<T>());
}

/// This is the generic move for non-trivial types
/// It will move the memory
///
/// # See also
///
/// * C++ API: `move_impl`
#[doc(alias = "move_impl")]
extern "C" fn move_dtor<T>(
    dst_ptr: *mut c_void,
    src_ptr: *mut c_void,
    count: i32,
    _type_info: *const sys::ecs_type_info_t,
) {
    ecs_assert!(
        check_type_info::<T>(_type_info),
        FlecsErrorCode::InternalError
    );
    let dst_arr = dst_ptr as *mut T;
    let src_arr = src_ptr as *mut T;
    for i in 0..count as isize {
        //this is safe because C manages the memory and we are just moving the internal data around
        unsafe {
            let src_value = src_arr.offset(i); //get value of src
            let dst_value = dst_arr.offset(i); // get ptr to dest

            std::ptr::drop_in_place(dst_value); //calls destructor on dest

            //memcpy the bytes of src to dest
            //src value and dest value point to the same thing
            std::ptr::copy_nonoverlapping(src_value, dst_value, 1);
        }
    }
}

/// a move to from src to dest where src will not be used anymore and dest is in control of the drop.
///
/// # See also
///
/// * C++ API: `move_impl`
#[doc(alias = "move_impl")]
extern "C" fn move_ctor<T>(
    dst_ptr: *mut c_void,
    src_ptr: *mut c_void,
    count: i32,
    _type_info: *const sys::ecs_type_info_t,
) {
    ecs_assert!(
        check_type_info::<T>(_type_info),
        FlecsErrorCode::InternalError
    );
    let dst_arr = dst_ptr as *mut T;
    let src_arr = src_ptr as *mut T;
    for i in 0..count as isize {
        //this is safe because src will not get dropped and dst will get dropped
        unsafe {
            // memcpy the bytes from src to dst
            std::ptr::copy_nonoverlapping(src_arr.offset(i), dst_arr.offset(i), 1);
        }
    }
}

extern "C" fn ctor_move_dtor<T>(
    dst_ptr: *mut c_void,
    src_ptr: *mut c_void,
    count: i32,
    _type_info: *const sys::ecs_type_info_t,
) {
    ecs_assert!(
        check_type_info::<T>(_type_info),
        FlecsErrorCode::InternalError
    );
    let dst_arr = dst_ptr as *mut T;
    let src_arr = src_ptr as *mut T;
    for i in 0..count as isize {
        //this is safe because src will not get dropped and dst will get dropped
        unsafe {
            // memcpy the bytes from src to dst
            std::ptr::copy_nonoverlapping(src_arr.offset(i), dst_arr.offset(i), 1);
        }
    }
}

fn check_type_info<T>(_type_info: *const sys::ecs_type_info_t) -> bool {
    if !_type_info.is_null() {
        unsafe { (*_type_info).size == std::mem::size_of::<T>() as i32 }
    } else {
        true
    }
}

mod tests {
    #![allow(unused_imports)]
    use std::{mem::MaybeUninit, os::raw::c_void, vec};

    use crate::core::lifecycle_traits::move_dtor;

    #[derive(Default, Debug, Clone)]
    struct MyType {
        vec: Vec<i32>,
        text: String,
        value: i32,
    }

    //#[test]
    fn test_move_dtor() {
        let vec_check = vec![0, 1, 2, 3];
        let str_check = "original";
        let val_check = 42;

        let mut moved_to = MyType {
            vec: Vec::new(),
            text: String::new(),
            value: 0,
        };
        {
            let mut original = MyType {
                vec: vec![0, 1, 2, 3],
                text: "original".to_string(),
                value: 42,
            };

            move_dtor::<MyType>(
                &mut moved_to as *mut _ as *mut c_void,
                &mut original as *mut _ as *mut c_void,
                1,
                std::ptr::null(),
            );

            assert_eq!(original.vec, vec_check); // Original should have remained unchanged
            assert_eq!(original.text, str_check);
            assert_eq!(original.value, val_check);

            assert_eq!(moved_to.vec, vec_check); // Moved_to should have original's values
            assert_eq!(moved_to.text, str_check);
            assert_eq!(moved_to.value, val_check);

            // forget original as that's what happens in C
            std::mem::forget(original);
        }

        // Moved to should have not been dropped despite original being out of scope

        assert_eq!(moved_to.vec, vec_check); // Moved_to should have original's values
        assert_eq!(moved_to.text, str_check);
        assert_eq!(moved_to.value, val_check);
    }

    //#[test]
    //#[ignore]
    fn test_modify_moved_to() {
        // let mut original = MyType {
        //     vec: vec![0, 1, 2, 3],
        //     text: "original".to_string(),
        //     value: 42,
        // };
        // let mut moved_to: MyType = Default::default();

        // move_::<MyType>(
        //     &mut moved_to as *mut _ as *mut c_void,
        //     &mut original as *mut _ as *mut c_void,
        //     1,
        //     std::ptr::null(),
        // );

        // moved_to.vec.push(4);
        // moved_to.text.push_str("_modified");
        // moved_to.value += 10;

        // assert_eq!(original.vec, Vec::<i32>::new()); // Original should be default
        // assert_eq!(moved_to.vec, vec![0, 1, 2, 3, 4]); // Moved_to should have new value
        // assert_eq!(original.text, String::new());
        // assert_eq!(moved_to.text, "original_modified");
        // assert_eq!(original.value, 0);
        // assert_eq!(moved_to.value, 52);
    }

    //#[test]
    fn test_generic_copy() {
        // let original = MyType {
        //     vec: vec![0, 1, 2, 3],
        //     text: "original".to_string(),
        //     value: 42,
        // };
        // let mut copied_to: MyType = Default::default();

        // let original_vec_ptr = original.vec.as_ptr();

        // generic_copy::<MyType>(
        //     &mut copied_to as *mut _ as *mut c_void,
        //     &original as *const _ as *const c_void,
        //     1,
        //     std::ptr::null(),
        // );

        // assert_eq!(original.vec, vec![0, 1, 2, 3]); // Original should remain unchanged
        // assert_eq!(copied_to.vec, vec![0, 1, 2, 3]); // copied_to should have original's values
        // assert_eq!(original.text, "original");
        // assert_eq!(copied_to.text, "original");
        // assert_eq!(original.value, 42);
        // assert_eq!(copied_to.value, 42);

        // // The pointers should be different
        // assert_ne!(original.vec.as_ptr(), copied_to.vec.as_ptr());
        // assert_eq!(original.vec.as_ptr(), original_vec_ptr);
        // assert_ne!(original.text.as_ptr(), copied_to.text.as_ptr());
    }

    //#[test]
    fn test_modify_copied_to() {
        // let original = MyType {
        //     vec: vec![0, 1, 2, 3],
        //     text: "original".to_string(),
        //     value: 42,
        // };
        // let mut copied_to: MyType = Default::default();

        // generic_copy::<MyType>(
        //     &mut copied_to as *mut _ as *mut c_void,
        //     &original as *const _ as *const c_void,
        //     1,
        //     std::ptr::null(),
        // );

        // copied_to.vec.push(4);
        // copied_to.text.push_str("_modified");
        // copied_to.value += 10;

        // assert_eq!(original.vec, vec![0, 1, 2, 3]); // Original should remain unchanged
        // assert_eq!(copied_to.vec, vec![0, 1, 2, 3, 4]); // copied_to should have the new value
        // assert_eq!(original.text, "original");
        // assert_eq!(copied_to.text, "original_modified");
        // assert_eq!(original.value, 42);
        // assert_eq!(copied_to.value, 52);
    }
}
