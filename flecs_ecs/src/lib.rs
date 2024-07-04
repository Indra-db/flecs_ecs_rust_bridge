#![allow(dead_code)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#[cfg(all(
    feature = "flecs_force_build_release_c",
    feature = "flecs_force_build_debug_c"
))]
compile_error!("Features 'flecs_force_build_release_c' and 'flecs_force_build_debug_c' cannot be enabled at the same time.");

pub use flecs_ecs_derive as macros;
pub use flecs_ecs_sys as sys;

pub mod core;

pub mod addons;

/// this is to allow using the proc macro's inside lib itself that implements its own traits.
extern crate self as flecs_ecs;

pub mod prelude;
