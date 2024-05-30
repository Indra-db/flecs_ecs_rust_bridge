//! contains traits that define what a component is and also the API's for [`Query`][super::Query], [`Observer`][super::Observer] and [`System`][crate::addons::system::System].
//! Also contains lower level utility functions on ECS IDs. This is mostly used internally by the library.

mod errors;
mod functions;
mod log;
pub mod traits;
pub mod types;

pub use errors::*;
pub use functions::*;
pub use log::*;

#[doc(hidden)]
pub use traits::*;
#[doc(hidden)]
pub use types::*;

#[macro_export]
macro_rules! offset_of {
    ($type:ty, $field:ident) => {
        std::mem::offset_of!($type, $field) as i32
    };
}

pub use offset_of;
