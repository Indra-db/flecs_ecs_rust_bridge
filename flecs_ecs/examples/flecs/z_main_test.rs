//#![feature(internal_output_capture)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused)]

// to initialize the OS api for flecs before tests run.
#[cfg(test)]
#[ctor::ctor]
fn init() {
    unsafe {
        flecs_ecs::sys::ecs_os_init();
    }
}

// pub mod z_ignore_test_common;

// mod a_hello_world;

// mod entity_basics;
// mod entity_hierarchy;
// mod entity_hooks;
// mod entity_iterate_components;
// mod entity_prefab;

// mod observer_basics;
// mod observer_custom_event;
// mod observer_entity_event;
// mod observer_monitor;
// mod observer_propagate;
// mod observer_two_components;
// mod observer_yield_existing;

// mod prefab_basics;
// mod prefab_hierarchy;
// mod prefab_nested;
// mod prefab_override;
// mod prefab_slots;
// mod prefab_typed;
// mod prefab_variant;

// mod query_basics;
// mod query_chaining_queries;
// mod query_change_tracking;
// mod query_find_entity;
// mod query_group_by;
// mod query_group_by_callbacks;
// mod query_group_by_custom;
// mod query_group_iter;
// mod query_hierarchy;
// mod query_instancing;
// mod query_iter;
// mod query_singleton;
// mod query_sorting;
// mod query_wildcard;
// mod query_with;
// mod query_without;
// mod query_world_query;

// mod relationships_basics;
// mod relationships_component_data;
// mod relationships_enum;
// mod relationships_exclusive;
// mod relationships_symmetric;
// mod relationships_union;

// mod rules_basics;
// mod rules_component_inheritance;
// mod rules_cyclic_variables;
// mod rules_facts;
// mod rules_setting_variables;
// mod rules_transitive_queries;

// mod system_basics;
// mod system_ctx;
// mod system_custom_phases;
// mod system_custom_phases_no_builtin;
// mod system_custom_pipeline;
// mod system_custom_runner;
// mod system_delta_time;
// mod system_mutate_entity;
// mod system_mutate_entity_handle;
// mod system_no_readonly;
// mod system_pipeline;
// mod system_startup_system;
// mod system_sync_point;
// mod system_sync_point_delete;
// mod system_target_fps;
// mod system_time_interval;

fn main() {}
