use std::ffi::CStr;

use flecs_ecs::prelude::meta::*;
use flecs_ecs::prelude::*;

fn std_string_support(world: &World) -> Opaque<String> {
    let mut ts = Opaque::<String>::new(world);

    // Let reflection framework know what kind of type this is
    ts.as_type(flecs::meta::String);

    // Forward std::string value to (JSON/...) serializer
    ts.serialize(|s: &Serializer, data: &String| {
        let value = data.as_str();
        s.value_id(
            flecs::meta::String,
            value as *const str as *const std::ffi::c_void,
        )
    });

    // Serialize string into std::string
    ts.assign_string(|data: &mut String, value: *const i8| {
        *data = unsafe { CStr::from_ptr(value).to_string_lossy().into_owned() }
    });

    ts
}

#[test]
fn meta_struct() {
    let world = World::new();

    #[derive(Component)]
    struct Test {
        a: i32,
        b: f32,
    }

    let c = world
        .component::<Test>()
        .member::<i32>("a", 1, offset_of!(Test, a))
        .member::<f32>("b", 1, offset_of!(Test, b));

    assert!(c.id() != 0);

    let a = c.lookup("a");
    assert!(a.id() != 0);
    assert!(a.has::<flecs::meta::Member>());

    a.get::<&flecs::meta::Member>(|mem| {
        assert_eq!(mem.type_, flecs::meta::I32);
    });

    let b = c.lookup("b");
    assert!(b.id() != 0);
    assert!(b.has::<flecs::meta::Member>());

    b.get::<&flecs::meta::Member>(|mem| {
        assert_eq!(mem.type_, flecs::meta::F32);
    });
}

#[test]
fn meta_nested_struct() {
    let world = World::new();

    #[derive(Component)]
    struct Test {
        x: i32,
    }

    #[derive(Component)]
    struct Nested {
        a: Test,
    }

    let t = world
        .component::<Test>()
        .member::<i32>("x", 1, offset_of!(Test, x));

    let n = world
        .component::<Nested>()
        .member_id(t, "a", 1, offset_of!(Nested, a));

    assert!(n.id() != 0);

    let a = n.lookup("a");
    assert!(a.id() != 0);
    assert!(a.has::<flecs::meta::Member>());

    a.get::<&flecs::meta::Member>(|mem| {
        assert_eq!(mem.type_, t.id());
    });
}

//TODO meta_units -- units addon is not yet implemented in Rust
//TODO Meta_unit_w_quantity -- units addon is not yet implemented in Rust
//TODO Meta_unit_w_prefix -- units addon is not yet implemented in Rust
//TODO Meta_unit_w_over -- units addon is not yet implemented in Rust

#[test]
fn meta_partial_struct() {
    let world = World::new();

    #[derive(Component)]
    struct Position {
        x: f32,
    }

    let c = world
        .component::<Position>()
        .member::<f32>("x", 1, offset_of!(Position, x));

    assert!(c.id() != 0);

    c.get::<&flecs::Component>(|ptr| {
        assert_eq!(ptr.size, 8);
        assert_eq!(ptr.alignment, 4);
    });
}

/*

void Meta_partial_struct(void) {
    flecs::world ecs;

    auto c = ecs.component<Position>()
        .member<float>("x");
    test_assert(c != 0);

    const flecs::Component *ptr = c.get<flecs::Component>();
    test_int(ptr->size, 8);
    test_int(ptr->alignment, 4);

    auto xe = c.lookup("x");
    test_assert(xe != 0);
    test_assert( xe.has<flecs::Member>() );
    const flecs::Member *x = xe.get<flecs::Member>();
    test_uint(x->type, flecs::F32);
    test_uint(x->offset, 0);
}

void Meta_partial_struct_custom_offset(void) {
    flecs::world ecs;

    auto c = ecs.component<Position>()
        .member<float>("y", 1, offsetof(Position, y));
    test_assert(c != 0);

    const flecs::Component *ptr = c.get<flecs::Component>();
    test_int(ptr->size, 8);
    test_int(ptr->alignment, 4);

    auto xe = c.lookup("y");
    test_assert(xe != 0);
    test_assert( xe.has<flecs::Member>() );
    const flecs::Member *x = xe.get<flecs::Member>();
    test_uint(x->type, flecs::F32);
    test_uint(x->offset, 4);
}
*/
