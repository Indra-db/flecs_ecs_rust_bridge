use std::ffi::CStr;

use flecs_ecs::prelude::meta::*;
use flecs_ecs::prelude::*;

fn std_string_support(world: WorldRef) -> Opaque<String> {
    let mut ts = Opaque::<String>::new(world);

    // Let reflection framework know what kind of type this is
    ts.as_type(flecs::meta::String);

    // Forward std::string value to (JSON/...) serializer
    ts.serialize(|s: &Serializer, data: &String| {
        s.value_id(
            flecs::meta::String,
            &data.as_ptr() as *const *const u8 as *const std::ffi::c_void,
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
    #[repr(C)]
    struct Position {
        x: f32,
    }

    let c = world
        .component::<Position>()
        .member::<f32>("x", 1, offset_of!(Position, x));

    assert!(c.id() != 0);

    c.get::<&flecs::Component>(|ptr| {
        assert_eq!(ptr.size, 4);
        assert_eq!(ptr.alignment, 4);
    });

    let xe = c.lookup("x");
    assert!(xe.id() != 0);
    assert!(xe.has::<flecs::meta::Member>());
    xe.get::<&flecs::meta::Member>(|x| {
        assert_eq!(x.type_, flecs::meta::F32);
        assert_eq!(x.offset, 0);
    });
}

#[test]
fn meta_partial_struct_custom_offset() {
    let world = World::new();

    #[derive(Component)]
    #[repr(C)]
    struct Position {
        x: f32,
        y: f32,
    }

    let c = world
        .component::<Position>()
        .member::<f32>("y", 1, offset_of!(Position, y));

    assert!(c.id() != 0);

    c.get::<&flecs::Component>(|ptr| {
        assert_eq!(ptr.size, 8);
        assert_eq!(ptr.alignment, 4);
    });

    let xe = c.lookup("y");
    assert!(xe.id() != 0);
    assert!(xe.has::<flecs::meta::Member>());
    xe.get::<&flecs::meta::Member>(|x| {
        assert_eq!(x.type_, flecs::meta::F32);
        assert_eq!(x.offset, 4);
    });
}

#[test]
fn meta_bitmask() {
    let world = World::new();

    #[derive(Component)]
    struct Toppings {
        value: u32,
    }

    impl Toppings {
        const BACON: u32 = 0x1;
        const LETTUCE: u32 = 0x2;
        const TOMATO: u32 = 0x4;

        fn new() -> Self {
            Toppings { value: 0 }
        }

        fn add(&mut self, topping: u32) {
            self.value |= topping;
        }

        fn remove(&mut self, topping: u32) {
            self.value &= !topping;
        }

        fn has(&self, topping: u32) -> bool {
            self.value & topping != 0
        }
    }

    #[derive(Component)]
    struct Sandwich {
        toppings: Toppings,
    }

    impl Sandwich {
        fn new() -> Self {
            Sandwich {
                toppings: Toppings::new(),
            }
        }

        fn add_topping(&mut self, topping: u32) {
            self.toppings.add(topping);
        }

        fn remove_topping(&mut self, topping: u32) {
            self.toppings.remove(topping);
        }

        fn has_topping(&self, topping: u32) -> bool {
            self.toppings.has(topping)
        }
    }

    world
        .component::<Toppings>()
        .bit("bacon", Toppings::BACON)
        .bit("lettuce", Toppings::LETTUCE)
        .bit("tomato", Toppings::TOMATO);

    world
        .component::<Sandwich>()
        .member::<Toppings>("toppings", 1, offset_of!(Sandwich, toppings));

    // Create entity with Sandwich as usual
    let e = world.entity().set(Sandwich {
        toppings: Toppings {
            value: Toppings::BACON | Toppings::LETTUCE,
        },
    });

    // Convert Sandwidth component to flecs expression string
    e.get::<&Sandwich>(|val| {
        assert_eq!(world.to_expr(val), "{toppings: lettuce|bacon}");
    });
}

#[test]
fn meta_custom_i32_to_json() {
    let world = World::new();

    #[derive(Component)]
    struct Int {
        value: i32,
    }

    world
        .component::<Int>()
        .opaque::<flecs::meta::I32>()
        .serialize(|s: &Serializer, data: &Int| s.value(&data.value));

    let v = Int { value: 10 };
    let json = world.to_json::<Int>(&v);
    assert_eq!(json, "10");
}

#[test]
fn meta_ser_deser_std_string() {
    let world = World::new();

    world.component::<String>().opaque_func(std_string_support);

    let mut v = "Hello World".to_string();

    let json = world.to_json::<String>(&v);
    assert_eq!(json, "\"Hello World\"");

    let json = "\"foo bar\"".to_string();
    world.from_json::<String>(&mut v, &json, None);
    let json = world.to_json::<String>(&v);
    assert_eq!(json, "\"foo bar\"");
}

#[test]
fn meta_ser_deser_flecs_entity() {
    let world = World::new();

    let e1 = world.entity_named("ent1");
    let e2 = world.entity_named("ent2");

    let mut v = e1;
    let json = world.to_json::<Entity>(&e1);
    assert_eq!(json, "\"ent1\"");

    world.from_json::<Entity>(&mut v, "\"ent2\"", None);
    let json = world.to_json::<Entity>(&v);
    assert_eq!(json, "\"ent2\"");
    assert_eq!(v, e2);
}

#[test]
fn meta_world_ser_deser_flecs_entity() {
    #[derive(Component)]
    struct RustEntity {
        entity: Entity,
    }

    let world = World::new();

    world
        .component::<RustEntity>()
        .member::<Entity>("entity", 1, offset_of!(RustEntity, entity));

    let e1 = world.entity_named("ent1");
    let e2 = world
        .entity_named("ent2")
        .set(RustEntity { entity: e1.id() });

    e2.get::<Option<&RustEntity>>(|ptr| {
        assert!(ptr.is_some());
        let ptr = ptr.unwrap();
        assert_eq!(world.to_json::<RustEntity>(ptr), "{\"entity\":\"ent1\"}");
    });

    let json = world.to_json_world();

    let world = World::new();

    world
        .component::<RustEntity>()
        .member::<Entity>("entity", 1, offset_of!(RustEntity, entity));

    world.from_json_world(json.as_str(), None);

    assert!(e1.is_alive());
    assert!(e2.is_alive());

    e2.get::<Option<&RustEntity>>(|ptr| {
        assert!(ptr.is_some());
        let ptr = ptr.unwrap();
        assert_eq!(world.to_json::<RustEntity>(ptr), "{\"entity\":\"ent1\"}");
    });
}

#[test]
fn meta_new_world_ser_deser_flecs_entity() {
    #[derive(Component)]
    struct RustEntity {
        entity: Entity,
    }

    let world = World::new();

    world
        .component::<RustEntity>()
        .member::<Entity>("entity", 1, offset_of!(RustEntity, entity));

    let e1 = world.entity_named("ent1");
    let e2 = world
        .entity_named("ent2")
        .set(RustEntity { entity: e1.id() });

    e2.get::<Option<&RustEntity>>(|ptr| {
        assert!(ptr.is_some());
        let ptr = ptr.unwrap();
        assert_eq!(world.to_json::<RustEntity>(ptr), "{\"entity\":\"ent1\"}");
    });

    let json = world.to_json_world();

    let world = World::new();

    world
        .component::<RustEntity>()
        .member::<Entity>("entity", 1, offset_of!(RustEntity, entity));

    world.from_json_world(json.as_str(), None);

    let e1 = world.lookup("ent1");
    let e2 = world.lookup("ent2");

    assert!(e1.id() != 0);
    assert!(e2.id() != 0);

    assert!(e1.is_alive());
    assert!(e2.is_alive());

    e2.get::<Option<&RustEntity>>(|ptr| {
        assert!(ptr.is_some());
        let ptr = ptr.unwrap();
        assert_eq!(world.to_json::<RustEntity>(ptr), "{\"entity\":\"ent1\"}");
    });
}

#[test]
fn meta_new_world_ser_deser_empty_flecs_entity() {
    #[derive(Component)]
    struct RustEntity {
        entity: Entity,
    }

    let world = World::new();

    world
        .component::<RustEntity>()
        .member::<Entity>("entity", 1, offset_of!(RustEntity, entity));

    let e1 = Entity::null();
    let e2 = world.entity_named("ent2").set(RustEntity { entity: e1 });

    e2.get::<Option<&RustEntity>>(|ptr| {
        assert!(ptr.is_some());
        let ptr = ptr.unwrap();
        assert_eq!(world.to_json::<RustEntity>(ptr), "{\"entity\":\"#0\"}");
    });

    let json = world.to_json_world();

    let world = World::new();

    world
        .component::<RustEntity>()
        .member::<Entity>("entity", 1, offset_of!(RustEntity, entity));

    world.from_json_world(json.as_str(), None);

    let e2 = world.lookup("ent2");

    assert!(e2.id() != 0);

    assert!(e2.is_alive());

    e2.get::<Option<&RustEntity>>(|ptr| {
        assert!(ptr.is_some());
        let ptr = ptr.unwrap();
        assert_eq!(world.to_json::<RustEntity>(ptr), "{\"entity\":\"#0\"}");
    });
}

#[test]
fn meta_opaque_vector_w_builder() {
    let world = World::new();

    #[derive(Component)]
    struct SerVec {
        pub value: Vec<i32>,
    }

    fn ensure_vec_element<'a>(data: &'a mut SerVec, elem: usize) -> &'a mut i32 {
        if data.value.len() <= elem {
            data.value.resize(elem + 1, 0);
        }
        &mut data.value[elem]
    }

    world
        .component::<SerVec>()
        .opaque_collection::<i32>(world.vector::<i32>())
        .serialize(|s: &Serializer, data: &SerVec| {
            for el in data.value.iter() {
                s.value(el);
            }
            0
        })
        .count(|data: &mut SerVec| data.value.len())
        .ensure_element(ensure_vec_element)
        .resize(|data: &mut SerVec, size: usize| {
            data.value.resize(size, 0);
        });

    let mut v = SerVec { value: vec![] };

    world.from_json::<SerVec>(&mut v, "[10, 20, 30]", None);
    assert_eq!(v.value.len(), 3);
    assert_eq!(v.value[0], 10);
    assert_eq!(v.value[1], 20);
    assert_eq!(v.value[2], 30);

    let json = world.to_json::<SerVec>(&v);
    assert_eq!(json, "[10, 20, 30]");
}

#[test]
fn meta_deser_entity_w_path() {
    let world = World::new();

    let ent = world.entity_named("ent");

    let mut e = Entity::null();
    world.from_json::<Entity>(&mut e, "\"ent\"", None);

    assert_eq!(e, ent);
    assert_eq!(world.entity_from_id(e).path().unwrap(), "::ent");
}

#[repr(C)]
#[derive(Component)]
enum EnumWithBits {
    BitA = 0,
    BitB = 1 << 0,
    BitAll = 0xffffffff,
}

#[derive(Component)]
struct EnumWithBitsStruct {
    bits: EnumWithBits,
}

impl Default for EnumWithBitsStruct {
    fn default() -> Self {
        EnumWithBitsStruct {
            bits: EnumWithBits::BitAll,
        }
    }
}

#[test]
fn meta_enum_w_bits() {
    let world = World::new();

    // It is illegal to register an enumeration as bitset, this test makes sure
    // the code doesn't crash.
    world
        .component::<EnumWithBits>()
        .bit("BitA", EnumWithBits::BitA as u32)
        .bit("BitB", EnumWithBits::BitB as u32)
        .bit("BitAll", EnumWithBits::BitAll as u32);

    world
        .component::<EnumWithBitsStruct>()
        .member::<EnumWithBits>("bits", 1, offset_of!(EnumWithBitsStruct, bits));

    for _ in 0..30 {
        world
            .entity()
            .child_of_id(world.entity())
            .add::<EnumWithBitsStruct>();
    }

    let q = world.new_query::<&EnumWithBitsStruct>();
    let s = q.to_json(None);
    assert_eq!(s, None);
}

#[test]
fn meta_value_range() {
    let world = World::new();

    #[derive(Component)]
    struct Position {
        x: f32,
        y: f32,
    }

    let c = world
        .component::<Position>()
        .member::<f32>("x", 1, offset_of!(Position, x))
        .range(-1.0, 1.0)
        .member::<f32>("y", 1, offset_of!(Position, y))
        .range(-2.0, 2.0);

    let x = c.lookup("x");
    assert!(x.id() != 0);
    assert!(x.has::<flecs::meta::MemberRanges>());

    x.get::<&flecs::meta::MemberRanges>(|ranges| {
        assert_eq!(ranges.value.min, -1.0);
        assert_eq!(ranges.value.max, 1.0);
    });

    let y = c.lookup("y");
    assert!(y.id() != 0);
    assert!(y.has::<flecs::meta::MemberRanges>());

    y.get::<&flecs::meta::MemberRanges>(|ranges| {
        assert_eq!(ranges.value.min, -2.0);
        assert_eq!(ranges.value.max, 2.0);
    });
}

#[test]
fn meta_warning_range() {
    let world = World::new();

    #[derive(Component)]
    struct Position {
        x: f32,
        y: f32,
    }

    let c = world
        .component::<Position>()
        .member::<f32>("x", 1, offset_of!(Position, x))
        .warning_range(-1.0, 1.0)
        .member::<f32>("y", 1, offset_of!(Position, y))
        .warning_range(-2.0, 2.0);

    let x = c.lookup("x");
    assert!(x.id() != 0);
    assert!(x.has::<flecs::meta::MemberRanges>());

    x.get::<&flecs::meta::MemberRanges>(|range| {
        assert_eq!(range.warning.min, -1.0);
        assert_eq!(range.warning.max, 1.0);
    });

    let y = c.lookup("y");
    assert!(y.id() != 0);
    assert!(y.has::<flecs::meta::MemberRanges>());

    y.get::<&flecs::meta::MemberRanges>(|range| {
        assert_eq!(range.warning.min, -2.0);
        assert_eq!(range.warning.max, 2.0);
    });
}

#[test]
fn meta_error_range() {
    let world = World::new();

    #[derive(Component)]
    struct Position {
        x: f32,
        y: f32,
    }

    let c = world
        .component::<Position>()
        .member::<f32>("x", 1, offset_of!(Position, x))
        .error_range(-1.0, 1.0)
        .member::<f32>("y", 1, offset_of!(Position, y))
        .error_range(-2.0, 2.0);

    let x = c.lookup("x");
    assert!(x.id() != 0);
    assert!(x.has::<flecs::meta::MemberRanges>());

    x.get::<&flecs::meta::MemberRanges>(|range| {
        assert_eq!(range.error.min, -1.0);
        assert_eq!(range.error.max, 1.0);
    });

    let y = c.lookup("y");
    assert!(y.id() != 0);
    assert!(y.has::<flecs::meta::MemberRanges>());

    y.get::<&flecs::meta::MemberRanges>(|range| {
        assert_eq!(range.error.min, -2.0);
        assert_eq!(range.error.max, 2.0);
    });
}

#[test]
fn meta_struct_member_ptr() {
    let world = World::new();

    #[repr(C)]
    #[derive(Component)]
    struct Test {
        x: i32,
    }

    #[repr(C)]
    #[derive(Component)]
    struct Test2 {
        y: f64,
    }

    #[repr(C)]
    #[derive(Component)]
    struct Nested {
        a: Test, //what offset should this be? Rust returns 16, but CPP gives 0 lol
        pad: i32,
        b: [Test2; 2],
    }

    let t = world
        .component::<Test>()
        .member::<i32>("x", 1, offset_of!(Test, x));

    let t2 = world
        .component::<Test2>()
        .member::<f64>("y", 1, offset_of!(Test2, y));

    let n = world
        .component::<Nested>()
        .member::<Test>("a", 1, offset_of!(Nested, a))
        .member_id(t2, "b", 2, offset_of!(Nested, b));

    //validate Test #1
    assert!(t.id() != 0);

    let x = t.lookup("x");
    assert!(x.id() != 0);
    assert!(x.has::<flecs::meta::Member>());
    x.get::<&flecs::meta::Member>(|xm| {
        assert_eq!(xm.type_, flecs::meta::I32);
        assert_eq!(xm.offset, offset_of!(Test, x));
    });

    //validate Test2 #2
    assert!(t2.id() != 0);

    let y = t2.lookup("y");
    assert!(y.id() != 0);
    assert!(y.has::<flecs::meta::Member>());
    y.get::<&flecs::meta::Member>(|ym| {
        assert_eq!(ym.type_, flecs::meta::F64);
        assert_eq!(ym.offset, offset_of!(Test2, y));
    });

    // Validate Nested
    assert!(n.id() != 0);

    let a = n.lookup("a");
    assert!(a.id() != 0);
    assert!(a.has::<flecs::meta::Member>());
    a.get::<&flecs::meta::Member>(|am| {
        assert_eq!(am.type_, t.id());
        let offset = offset_of!(Nested, a);
        assert_eq!(am.offset, offset);
    });

    let b = n.lookup("b");
    assert!(b.id() != 0);
    assert!(b.has::<flecs::meta::Member>());
    b.get::<&flecs::meta::Member>(|bm| {
        assert_eq!(bm.type_, t2.id());
        assert_eq!(bm.offset, offset_of!(Nested, b));
        assert_eq!(bm.count, 2);
    });
}

#[test]
fn meta_component_as_array() {
    let world = World::new();

    #[derive(Component)]
    struct Position {
        x: f32,
        y: f32,
    }

    let c = world.component::<Position>().array::<f32>(2);

    assert!(c.has::<flecs::meta::Array>());

    c.get::<&flecs::meta::Array>(|ptr| {
        assert_eq!(ptr.type_, world.component_id::<f32>());
        assert_eq!(ptr.count, 2);
    });
}
