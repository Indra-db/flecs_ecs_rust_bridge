#![allow(dead_code)]

use std::ffi::c_void;

use flecs_ecs::core::*;
use flecs_ecs::macros::*;
use timer::TimerAPI;

use crate::common_test::*;

#[derive(Component)]
struct LastVal(i32);

#[test]
fn system_iter() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    world
        .system::<(&mut Position, &mut Velocity)>()
        .run(|mut it| {
            while it.next() {
                let mut p = it.field::<Position>(0).unwrap();
                let v = it.field::<Velocity>(1).unwrap();
                for i in it.iter() {
                    p[i].x += v[i].x;
                    p[i].y += v[i].y;
                }
            }
        });

    world.progress();

    entity.get::<(&Position, &Velocity)>(|(p, v)| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });
}

#[test]
fn system_iter_macro() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    system!(world, &mut Position, &Velocity).run(|mut it| {
        while it.next() {
            let mut p = it.field::<Position>(0).unwrap();
            let v = it.field::<Velocity>(1).unwrap();
            for i in it.iter() {
                p[i].x += v[i].x;
                p[i].y += v[i].y;
            }
        }
    });

    world.progress();

    entity.get::<(&Position, &Velocity)>(|(p, v)| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });
}

#[test]
fn system_iter_const() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    world.system::<(&mut Position, &Velocity)>().run(|mut it| {
        while it.next() {
            let mut p = it.field::<Position>(0).unwrap();
            let v = it.field::<&Velocity>(1).unwrap();
            for i in it.iter() {
                p[i].x += v[i].x;
                p[i].y += v[i].y;
            }
        }
    });

    world.progress();

    entity.get::<(&Position, &Velocity)>(|(p, v)| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });
}

#[test]
fn system_iter_shared() {
    let world = World::new();

    world
        .component::<Position>()
        .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();
    world
        .component::<Velocity>()
        .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

    let base = world.entity().set(Velocity { x: 1, y: 2 });

    let e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .add_id((flecs::IsA::ID, base));

    let e2 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 3, y: 4 });

    world
        .system::<&mut Position>()
        .expr("flecs.common_test.Velocity(self|up IsA)")
        .run(|mut it| {
            while it.next() {
                let mut p = it.field::<Position>(0).unwrap();
                let v = it.field::<&Velocity>(1).unwrap();

                if !it.is_self(1) {
                    for i in it.iter() {
                        p[i].x += v[0].x;
                        p[i].y += v[0].y;
                    }
                } else {
                    for i in it.iter() {
                        p[i].x += v[i].x;
                        p[i].y += v[i].y;
                    }
                }
            }
        });

    world.progress();

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 13);
        assert_eq!(p.y, 24);
    });
}

#[test]
fn system_iter_optional() {
    let world = World::new();
    world.component_named::<Mass>("Mass");

    let e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 })
        .set(Mass { value: 1 });

    let e2 = world
        .entity()
        .set(Position { x: 30, y: 40 })
        .set(Velocity { x: 3, y: 4 })
        .set(Mass { value: 1 });

    let e3 = world.entity().set(Position { x: 50, y: 60 });

    let e4 = world.entity().set(Position { x: 70, y: 80 });

    world
        .system::<(&mut Position, Option<&mut Velocity>, Option<&mut Mass>)>()
        .run(|mut it| {
            while it.next() {
                let mut p = it.field::<Position>(0).unwrap();

                if it.is_set(1) && it.is_set(2) {
                    let v = it.field::<Velocity>(1).unwrap();
                    let m = it.field::<Mass>(2).unwrap();
                    for i in it.iter() {
                        p[i].x += v[i].x * m[i].value;
                        p[i].y += v[i].y * m[i].value;
                    }
                } else {
                    for i in it.iter() {
                        p[i].x += 1;
                        p[i].y += 1;
                    }
                }
            }
        });

    world.progress();

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 33);
        assert_eq!(p.y, 44);
    });

    e3.get::<&Position>(|p| {
        assert_eq!(p.x, 51);
        assert_eq!(p.y, 61);
    });

    e4.get::<&Position>(|p| {
        assert_eq!(p.x, 71);
        assert_eq!(p.y, 81);
    });
}

#[test]
fn system_each() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    world
        .system::<(&mut Position, &mut Velocity)>()
        .each_entity(|_e, (p, v)| {
            p.x += v.x;
            p.y += v.y;
        });

    world.progress();

    entity.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });
}

#[test]
fn system_each_const() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    world
        .system::<(&mut Position, &Velocity)>()
        .each_entity(|_e, (p, v)| {
            p.x += v.x;
            p.y += v.y;
        });

    world.progress();

    entity.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });
}

#[test]
fn system_each_shared() {
    let world = World::new();

    let base = world.entity().set(Velocity { x: 1, y: 2 });

    let e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .add_id((flecs::IsA::ID, base));

    let e2 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 3, y: 4 });

    world
        .system::<(&mut Position, &Velocity)>()
        .each_entity(|_e, (p, v)| {
            p.x += v.x;
            p.y += v.y;
        });

    world.progress();

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 13);
        assert_eq!(p.y, 24);
    });
}

#[test]
fn system_each_optional() {
    let world = World::new();
    world.component_named::<Mass>("Mass");

    let e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 })
        .set(Mass { value: 1 });

    let e2 = world
        .entity()
        .set(Position { x: 30, y: 40 })
        .set(Velocity { x: 3, y: 4 })
        .set(Mass { value: 1 });

    let e3 = world.entity().set(Position { x: 50, y: 60 });

    let e4 = world.entity().set(Position { x: 70, y: 80 });

    world
        .system::<(&mut Position, Option<&mut Velocity>, Option<&mut Mass>)>()
        .each_entity(|_e, (p, v, m)| {
            if v.is_some() && m.is_some() {
                let v = v.unwrap();
                let m = m.unwrap();
                p.x += v.x * m.value;
                p.y += v.y * m.value;
            } else {
                p.x += 1;
                p.y += 1;
            }
        });

    world.progress();

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });
    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 33);
        assert_eq!(p.y, 44);
    });

    e3.get::<&Position>(|p| {
        assert_eq!(p.x, 51);
        assert_eq!(p.y, 61);
    });

    e4.get::<&Position>(|p| {
        assert_eq!(p.x, 71);
        assert_eq!(p.y, 81);
    });
}

#[test]
fn system_signature() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    world
        .system::<()>()
        .expr("flecs.common_test.Position, flecs.common_test.Velocity")
        .run(|mut it| {
            while it.next() {
                let mut p = it.field::<Position>(0).unwrap();
                let v = it.field::<Velocity>(1).unwrap();

                for i in it.iter() {
                    p[i].x += v[i].x;
                    p[i].y += v[i].y;
                }
            }
        });

    world.progress();

    entity.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    entity.get::<&Velocity>(|v| {
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });
}

#[test]
fn system_signature_const() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    world
        .system::<()>()
        .expr("flecs.common_test.Position, [in] flecs.common_test.Velocity")
        .run(|mut it| {
            while it.next() {
                let mut p = it.field::<Position>(0).unwrap();
                let v = it.field::<Velocity>(1).unwrap();

                for i in it.iter() {
                    p[i].x += v[i].x;
                    p[i].y += v[i].y;
                }
            }
        });

    world.progress();

    entity.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    entity.get::<&Velocity>(|v| {
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });
}

#[test]
fn system_signature_shared() {
    let world = World::new();

    world
        .component::<Position>()
        .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();
    world
        .component::<Velocity>()
        .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

    let base = world.entity().set(Velocity { x: 1, y: 2 });

    let e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .add_id((flecs::IsA::ID, base));

    let e2 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 3, y: 4 });

    world
        .system::<()>()
        .expr("flecs.common_test.Position, [in] flecs.common_test.Velocity(self|up IsA)")
        .run(|mut it| {
            while it.next() {
                let mut p = it.field::<Position>(0).unwrap();
                let v = it.field::<Velocity>(1).unwrap();

                if !it.is_self(1) {
                    for i in it.iter() {
                        p[i].x += v[0].x;
                        p[i].y += v[0].y;
                    }
                } else {
                    for i in it.iter() {
                        p[i].x += v[i].x;
                        p[i].y += v[i].y;
                    }
                }
            }
        });

    world.progress();

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 13);
        assert_eq!(p.y, 24);
    });
}

#[test]
fn system_signature_optional() {
    let world = World::new();
    world.component_named::<Mass>("Mass");

    let e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 })
        .set(Mass { value: 1 });

    let e2 = world
        .entity()
        .set(Position { x: 30, y: 40 })
        .set(Velocity { x: 3, y: 4 })
        .set(Mass { value: 1 });

    let e3 = world.entity().set(Position { x: 50, y: 60 });

    let e4 = world.entity().set(Position { x: 70, y: 80 });

    world
        .system::<()>()
        .expr("flecs.common_test.Position, ?flecs.common_test.Velocity, ?Mass")
        .run(|mut it| {
            while it.next() {
                let mut p = it.field::<Position>(0).unwrap();

                if it.is_set(1) && it.is_set(2) {
                    let v = it.field::<Velocity>(1).unwrap();
                    let m = it.field::<Mass>(2).unwrap();
                    for i in it.iter() {
                        p[i].x += v[i].x * m[i].value;
                        p[i].y += v[i].y * m[i].value;
                    }
                } else {
                    for i in it.iter() {
                        p[i].x += 1;
                        p[i].y += 1;
                    }
                }
            }
        });

    world.progress();

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    e2.get::<&Position>(|p| {
        assert_eq!(p.x, 33);
        assert_eq!(p.y, 44);
    });

    e3.get::<&Position>(|p| {
        assert_eq!(p.x, 51);
        assert_eq!(p.y, 61);
    });

    e4.get::<&Position>(|p| {
        assert_eq!(p.x, 71);
        assert_eq!(p.y, 81);
    });
}

#[test]
fn system_copy_name_on_create() {
    let world = World::new();

    let mut name = "hello";

    let system_1 = world
        .system_named::<&mut Position>(name)
        .run(|mut it| while it.next() {});

    name = "world";
    let system_2 = world
        .system_named::<&mut Position>(name)
        .run(|mut it| while it.next() {});

    assert!(system_1.id() != system_2.id());
}

#[test]
fn system_nested_system() {
    let world = World::new();

    let system_1 = world
        .system_named::<&mut Position>("foo::bar")
        .run(|mut it| while it.next() {});

    assert_eq!(system_1.name(), "bar");

    let e = world.lookup("foo");
    assert!(e.id() != 0);
    assert_eq!(e.name(), "foo");

    let se = e.lookup("bar");
    assert!(se.id() != 0);
    assert_eq!(se.name(), "bar");
}

#[test]
fn system_empty_signature() {
    let world = World::new();

    world.set(Count(0));

    world.system::<()>().run(|mut it| {
        let world = it.world();
        while it.next() {
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        }
    });

    world.progress();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 1);
    });
}

#[test]
fn system_iter_tag() {
    let world = World::new();

    world.set(Count(0));

    world.system::<&TagA>().run(|mut it| {
        let world = it.world();
        while it.next() {
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        }
    });

    world.entity().add::<TagA>();

    world.progress();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 1);
    });
}

#[test]
fn system_each_tag() {
    let world = World::new();

    world.set(Count(0));

    world.system::<&TagA>().run(|mut it| {
        while it.next() {
            for _ in it.iter() {
                let world = it.world();
                world.get::<&mut Count>(|c| {
                    c.0 += 1;
                });
            }
        }
    });

    world.entity().add::<TagA>();

    world.progress();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 1);
    });
}

#[ignore = "implement missing functions"]
#[test]
fn system_set_interval() {
    let world = World::new();

    let _sys = world
        .system::<()>()
        .kind_id(0)
        .set_interval(1.0)
        .run(|_it| {});

    // float i = sys.set_interval();
    // assert_eq!(i, 1.0f);

    // sys.set_interval(2.0f);

    // i = sys.set_interval();
    // assert_eq!(i, 2.0f);
}

#[test]
fn system_order_by_type() {
    let world = World::new();

    world.entity().set(Position { x: 3, y: 0 });
    world.entity().set(Position { x: 1, y: 0 });
    world.entity().set(Position { x: 5, y: 0 });
    world.entity().set(Position { x: 2, y: 0 });
    world.entity().set(Position { x: 4, y: 0 });

    world.set(LastVal(0));
    world.set(Count(0));

    let sys = world
        .system::<&Position>()
        .order_by::<Position>(|_e1, p1: &Position, _e2, p2: &Position| -> i32 {
            (p1.x > p2.x) as i32 - (p1.x < p2.x) as i32
        })
        .each_entity(|e, p| {
            let world = e.world();
            world.get::<&mut LastVal>(|last_val| {
                assert!(p.x > last_val.0);
                last_val.0 = p.x;
            });
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        });

    sys.run();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 5);
    });
}

#[test]
fn system_order_by_id() {
    let world = World::new();

    let pos = world.component::<Position>();

    world.entity().set(Position { x: 3, y: 0 });
    world.entity().set(Position { x: 1, y: 0 });
    world.entity().set(Position { x: 5, y: 0 });
    world.entity().set(Position { x: 2, y: 0 });
    world.entity().set(Position { x: 4, y: 0 });

    world.set(LastVal(0));
    world.set(Count(0));

    let sys = world
        .system::<&Position>()
        .order_by_id(
            pos,
            |_e1, p1: *const c_void, _e2, p2: *const c_void| -> i32 {
                let p1 = unsafe { &*(p1 as *const Position) };
                let p2 = unsafe { &*(p2 as *const Position) };
                (p1.x > p2.x) as i32 - (p1.x < p2.x) as i32
            },
        )
        .each_entity(|e, p| {
            let world = e.world();
            world.get::<&mut LastVal>(|last_val| {
                assert!(p.x > last_val.0);
                last_val.0 = p.x;
            });
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        });

    sys.run();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 5);
    });
}

#[test]
fn system_order_by_type_after_create() {
    let world = World::new();

    world.entity().set(Position { x: 3, y: 0 });
    world.entity().set(Position { x: 1, y: 0 });
    world.entity().set(Position { x: 5, y: 0 });
    world.entity().set(Position { x: 2, y: 0 });
    world.entity().set(Position { x: 4, y: 0 });

    world.set(LastVal(0));
    world.set(Count(0));

    let sys = world
        .system::<&Position>()
        .order_by::<Position>(|_e1, p1: &Position, _e2, p2: &Position| -> i32 {
            (p1.x > p2.x) as i32 - (p1.x < p2.x) as i32
        })
        .each_entity(|e, p| {
            let world = e.world();
            world.get::<&mut LastVal>(|last_val| {
                assert!(p.x > last_val.0);
                last_val.0 = p.x;
            });
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        });

    sys.run();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 5);
    });
}

#[test]
fn system_order_by_id_after_create() {
    let world = World::new();

    let pos = world.component::<Position>();

    world.entity().set(Position { x: 3, y: 0 });
    world.entity().set(Position { x: 1, y: 0 });
    world.entity().set(Position { x: 5, y: 0 });
    world.entity().set(Position { x: 2, y: 0 });
    world.entity().set(Position { x: 4, y: 0 });

    world.set(LastVal(0));
    world.set(Count(0));

    let sys = world
        .system::<&Position>()
        .order_by_id(
            pos,
            |_e1, p1: *const c_void, _e2, p2: *const c_void| -> i32 {
                let p1 = unsafe { &*(p1 as *const Position) };
                let p2 = unsafe { &*(p2 as *const Position) };
                (p1.x > p2.x) as i32 - (p1.x < p2.x) as i32
            },
        )
        .each_entity(|e, p| {
            let world = e.world();
            world.get::<&mut LastVal>(|last_val| {
                assert!(p.x > last_val.0);
                last_val.0 = p.x;
            });
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        });

    sys.run();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 5);
    });
}

#[test]
fn system_get_query() {
    let world = World::new();

    world.entity().set(Position { x: 0, y: 0 });
    world.entity().set(Position { x: 1, y: 0 });
    world.entity().set(Position { x: 2, y: 0 });

    world.set(Count(0));

    let sys = world.system::<&Position>().each_entity(|_e, _p| {
        // Not used
    });

    let q = sys.query();

    q.run(|mut it| {
        let world = it.world();
        while it.next() {
            let pos = it.field::<&Position>(0).unwrap();
            for i in it.iter() {
                assert_eq!(i as i32, pos[i].x);
                world.get::<&mut Count>(|c| {
                    c.0 += 1;
                });
            }
        }
    });

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 3);
    });
}

#[test]
fn system_add_from_each() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 0, y: 0 });
    let e2 = world.entity().set(Position { x: 1, y: 0 });
    let e3 = world.entity().set(Position { x: 2, y: 0 });

    world.system::<&Position>().each_entity(|e, _p| {
        e.add::<Velocity>();
        // Add is deferred
        assert!(!e.has::<Velocity>());
    });

    world.progress();

    assert!(e1.has::<Velocity>());
    assert!(e2.has::<Velocity>());
    assert!(e3.has::<Velocity>());
}

#[test]
fn system_delete_from_each() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 0, y: 0 });
    let e2 = world.entity().set(Position { x: 1, y: 0 });
    let e3 = world.entity().set(Position { x: 2, y: 0 });

    world.system::<&Position>().each_entity(|e, _p| {
        e.destruct();
        // Delete is deferred
        assert!(e.is_alive());
    });

    world.progress();

    assert!(!e1.is_alive());
    assert!(!e2.is_alive());
    assert!(!e3.is_alive());
}

#[test]
fn system_add_from_each_world_handle() {
    let world = World::new();

    let e1 = world.entity().set(EntityRef {
        value: world.entity().id(),
    });
    let e2 = world.entity().set(EntityRef {
        value: world.entity().id(),
    });
    let e3 = world.entity().set(EntityRef {
        value: world.entity().id(),
    });

    world.system::<&EntityRef>().each_entity(|e, c| {
        let world = e.world();
        let e = world.entity_from_id(c.value);
        e.mut_stage_of(e).add::<Position>();
    });

    world.progress();

    e1.get::<&EntityRef>(|c| {
        let e = world.entity_from_id(c.value);
        assert!(e.has::<Position>());
    });

    e2.get::<&EntityRef>(|c| {
        let e = world.entity_from_id(c.value);
        assert!(e.has::<Position>());
    });

    e3.get::<&EntityRef>(|c| {
        let e = world.entity_from_id(c.value);
        assert!(e.has::<Position>());
    });
}

#[test]
fn system_new_from_each() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 0, y: 0 });
    let e2 = world.entity().set(Position { x: 0, y: 0 });
    let e3 = world.entity().set(Position { x: 0, y: 0 });

    world.system::<&Position>().each_entity(|e, _p| {
        e.set(EntityRef {
            value: e.world().entity().add::<Velocity>().id(),
        });
    });

    world.progress();

    assert!(e1.has::<EntityRef>());
    assert!(e2.has::<EntityRef>());
    assert!(e3.has::<EntityRef>());

    e1.get::<&EntityRef>(|c| {
        assert!(world.entity_from_id(c.value).has::<Velocity>());
    });

    e2.get::<&EntityRef>(|c| {
        assert!(world.entity_from_id(c.value).has::<Velocity>());
    });

    e3.get::<&EntityRef>(|c| {
        assert!(world.entity_from_id(c.value).has::<Velocity>());
    });
}

#[test]
fn system_add_from_iter() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 0, y: 0 });
    let e2 = world.entity().set(Position { x: 1, y: 0 });
    let e3 = world.entity().set(Position { x: 2, y: 0 });

    world.system::<&Position>().run(|mut it| {
        while it.next() {
            for i in it.iter() {
                it.entity(i).add::<Velocity>();
                assert!(!it.entity(i).has::<Velocity>());
            }
        }
    });

    world.progress();

    assert!(e1.has::<Velocity>());
    assert!(e2.has::<Velocity>());
    assert!(e3.has::<Velocity>());
}

#[test]
fn system_delete_from_iter() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 0, y: 0 });
    let e2 = world.entity().set(Position { x: 1, y: 0 });
    let e3 = world.entity().set(Position { x: 2, y: 0 });

    world.system::<&Position>().run(|mut it| {
        while it.next() {
            for i in it.iter() {
                it.entity(i).destruct();
                // Delete is deferred
                assert!(it.entity(i).is_alive());
            }
        }
    });

    world.progress();

    assert!(!e1.is_alive());
    assert!(!e2.is_alive());
    assert!(!e3.is_alive());
}

#[test]
fn system_add_from_iter_world_handle() {
    let world = World::new();

    let e1 = world.entity().set(EntityRef {
        value: world.entity().id(),
    });
    let e2 = world.entity().set(EntityRef {
        value: world.entity().id(),
    });
    let e3 = world.entity().set(EntityRef {
        value: world.entity().id(),
    });

    world.system::<&EntityRef>().run(|mut it| {
        let world = it.world();
        while it.next() {
            let c = it.field::<EntityRef>(0).unwrap();
            for i in it.iter() {
                world
                    .entity_from_id(c[i].value)
                    .mut_current_stage(it.world())
                    .add::<Position>();
            }
        }
    });

    world.progress();

    e1.get::<&EntityRef>(|c| {
        let e = world.entity_from_id(c.value);
        assert!(e.has::<Position>());
    });

    e2.get::<&EntityRef>(|c| {
        let e = world.entity_from_id(c.value);
        assert!(e.has::<Position>());
    });

    e3.get::<&EntityRef>(|c| {
        let e = world.entity_from_id(c.value);
        assert!(e.has::<Position>());
    });
}

#[test]
fn system_new_from_iter() {
    let world = World::new();

    let e1 = world.entity().set(Position { x: 0, y: 0 });
    let e2 = world.entity().set(Position { x: 0, y: 0 });
    let e3 = world.entity().set(Position { x: 0, y: 0 });

    world.system::<&Position>().run(|mut it| {
        while it.next() {
            for i in it.iter() {
                it.entity(i).set(EntityRef {
                    value: it.world().entity().add::<Velocity>().id(),
                });
            }
        }
    });

    world.progress();

    assert!(e1.has::<EntityRef>());
    assert!(e2.has::<EntityRef>());
    assert!(e3.has::<EntityRef>());

    e1.get::<&EntityRef>(|c| {
        assert!(world.entity_from_id(c.value).has::<Velocity>());
    });

    e2.get::<&EntityRef>(|c| {
        assert!(world.entity_from_id(c.value).has::<Velocity>());
    });

    e3.get::<&EntityRef>(|c| {
        assert!(world.entity_from_id(c.value).has::<Velocity>());
    });
}

#[test]
fn system_each_w_mut_children_it() {
    let world = World::new();

    let parent = world.entity().set(Position { x: 0, y: 0 });
    let e1 = world
        .entity()
        .set(Position { x: 0, y: 0 })
        .child_of_id(parent);
    let e2 = world
        .entity()
        .set(Position { x: 0, y: 0 })
        .child_of_id(parent);
    let e3 = world
        .entity()
        .set(Position { x: 0, y: 0 })
        .child_of_id(parent);

    world.set(Count(0));

    world.system::<&Position>().run(|mut it| {
        let world = it.world();
        while it.next() {
            for i in it.iter() {
                it.entity(i).each_child(|child| {
                    child.add::<Velocity>();
                    world.get::<&mut Count>(|c| {
                        c.0 += 1;
                    });
                });
            }
        }
    });

    world.progress();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 3);
    });

    assert!(e1.has::<Velocity>());
    assert!(e2.has::<Velocity>());
    assert!(e3.has::<Velocity>());
}

#[test]
fn system_readonly_children_iter() {
    let world = World::new();

    let parent = world.entity();
    world.entity().set(EntityRef { value: parent.id() });
    world
        .entity()
        .set(Position { x: 1, y: 0 })
        .child_of_id(parent);
    world
        .entity()
        .set(Position { x: 1, y: 0 })
        .child_of_id(parent);
    world
        .entity()
        .set(Position { x: 1, y: 0 })
        .child_of_id(parent);

    world.set(Count(0));

    world.system::<&EntityRef>().run(|mut it| {
        let world = it.world();
        while it.next() {
            let c = it.field::<EntityRef>(0).unwrap();
            for i in it.iter() {
                world.entity_from_id(c[i].value).each_child(|child| {
                    // Dummy code to ensure we can access the entity
                    child.get::<&Position>(|p| {
                        assert_eq!(p.x, 1);
                        assert_eq!(p.y, 0);
                    });

                    world.get::<&mut Count>(|c| {
                        c.0 += 1;
                    });
                });
            }
        }
    });

    world.progress();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 3);
    });
}

#[test]
fn system_rate_filter() {
    let world = World::new();

    #[derive(Default, Component)]
    struct Counter {
        root: i32,
        l1_a: i32,
        l1_b: i32,
        l1_c: i32,
        l2_a: i32,
        l2_b: i32,
    }

    world.set(Counter::default());

    let root_mult: i32 = 1;
    let l1_a_mult: i32 = 1;
    let l1_b_mult: i32 = 2;
    let l1_c_mult: i32 = 3;
    let l2_a_mult: i32 = 2;
    let l2_b_mult: i32 = 4;
    let mut frame_count: i32 = 0;

    let root = world.system_named::<()>("root").run(|mut it| {
        let world = it.world();
        while it.next() {
            world.get::<&mut Counter>(|c| {
                c.root += 1;
            });
        }
    });

    let l1_a = world
        .system_named::<()>("l1_a")
        .set_tick_source_rate(root.id(), 1)
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                world.get::<&mut Counter>(|c| {
                    c.l1_a += 1;
                });
            }
        });

    let l1_b = world
        .system_named::<()>("l1_b")
        .set_tick_source_rate(root.id(), 2)
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                world.get::<&mut Counter>(|c| {
                    c.l1_b += 1;
                });
            }
        });

    world
        .system_named::<()>("l1_c")
        .set_tick_source_rate(root.id(), 3)
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                world.get::<&mut Counter>(|c| {
                    c.l1_c += 1;
                });
            }
        });

    world
        .system_named::<()>("l2_a")
        .set_tick_source_rate(l1_a.id(), 2)
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                world.get::<&mut Counter>(|c| {
                    c.l2_a += 1;
                });
            }
        });

    world
        .system_named::<()>("l2_b")
        .set_tick_source_rate(l1_b.id(), 2)
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                world.get::<&mut Counter>(|c| {
                    c.l2_b += 1;
                });
            }
        });

    for _i in 0..30 {
        world.progress();
        frame_count += 1;
        world.get::<&Counter>(|c| {
            assert_eq!(c.root, frame_count / root_mult);
            assert_eq!(c.l1_a, frame_count / l1_a_mult);
            assert_eq!(c.l1_b, frame_count / l1_b_mult);
            assert_eq!(c.l1_c, frame_count / l1_c_mult);
            assert_eq!(c.l2_a, frame_count / l2_a_mult);
            assert_eq!(c.l2_b, frame_count / l2_b_mult);
        });
    }
}

#[test]
fn system_self_rate_filter() {
    let world = World::new();

    world.set(Count(0));

    world
        .system::<&Position>()
        .set_rate(2)
        .each_entity(|e, _p| {
            let world = e.world();
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        });

    world.entity().set(Position { x: 1, y: 2 });

    for _i in 0..10 {
        world.progress();
    }

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 5);
    });
}

#[ignore = "implement missing functions"]
#[test]
fn system_update_rate_filter() {
    let world = World::new();

    #[derive(Default, Component)]
    struct Counter {
        root: i32,
        l1: i32,
        l2: i32,
    }

    world.set(Counter::default());

    let root_mult: i32 = 1;
    let mut l1_mult: i32 = 2;
    let mut l2_mult: i32 = 6;
    let mut frame_count: i32 = 0;

    let root = world.system_named::<()>("root").run(|mut it| {
        let world = it.world();
        while it.next() {
            world.get::<&mut Counter>(|c| {
                c.root += 1;
            });
        }
    });

    let l1 = world
        .system_named::<()>("l1")
        .set_tick_source_rate(root.id(), 2)
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                world.get::<&mut Counter>(|c| {
                    c.l1 += 1;
                });
            }
        });

    world
        .system_named::<()>("l2")
        .set_tick_source_rate(l1.id(), 3)
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                world.get::<&mut Counter>(|c| {
                    c.l2 += 1;
                });
            }
        });

    for _i in 0..12 {
        world.progress();
        frame_count += 1;
        world.get::<&Counter>(|c| {
            assert_eq!(c.root, frame_count / root_mult);
            assert_eq!(c.l1, frame_count / l1_mult);
            assert_eq!(c.l2, frame_count / l2_mult);
        });
    }

    //FIXME
    //l1.set_rate(4); // Run twice as slow
    l1_mult *= 2;
    l2_mult *= 2;

    frame_count = 0;
    world.set(Counter::default());

    for _i in 0..32 {
        world.progress();
        frame_count += 1;
        world.get::<&Counter>(|c| {
            assert_eq!(c.root, frame_count / root_mult);
            assert_eq!(c.l1, frame_count / l1_mult);
            assert_eq!(c.l2, frame_count / l2_mult);
        });
    }
}

#[test]
fn system_test_let_defer_each() {
    let world = World::new();

    let e1 = world.entity().add::<Tag>().set(Value { value: 10 });
    let e2 = world.entity().add::<Tag>().set(Value { value: 20 });
    let e3 = world.entity().add::<Tag>().set(Value { value: 30 });

    let s = world
        .system::<&mut Value>()
        .with::<Tag>()
        .each_entity(|e, v| {
            v.value += 1;
            e.remove::<Tag>();
        });

    s.run();

    assert!(!e1.has::<Tag>());
    assert!(!e2.has::<Tag>());
    assert!(!e3.has::<Tag>());

    assert!(e1.has::<Value>());
    assert!(e2.has::<Value>());
    assert!(e3.has::<Value>());

    e1.get::<&Value>(|v| {
        assert_eq!(v.value, 11);
    });

    e2.get::<&Value>(|v| {
        assert_eq!(v.value, 21);
    });

    e3.get::<&Value>(|v| {
        assert_eq!(v.value, 31);
    });
}

#[test]
fn system_test_let_defer_iter() {
    let world = World::new();

    let e1 = world.entity().add::<Tag>().set(Value { value: 10 });
    let e2 = world.entity().add::<Tag>().set(Value { value: 20 });
    let e3 = world.entity().add::<Tag>().set(Value { value: 30 });

    let s = world.system::<&mut Value>().with::<Tag>().run(|mut it| {
        while it.next() {
            let mut v = it.field::<Value>(0).unwrap();
            for i in it.iter() {
                v[i].value += 1;
                it.entity(i).remove::<Tag>();
            }
        }
    });

    s.run();

    assert!(!e1.has::<Tag>());
    assert!(!e2.has::<Tag>());
    assert!(!e3.has::<Tag>());

    assert!(e1.has::<Value>());
    assert!(e2.has::<Value>());
    assert!(e3.has::<Value>());

    e1.get::<&Value>(|v| {
        assert_eq!(v.value, 11);
    });

    e2.get::<&Value>(|v| {
        assert_eq!(v.value, 21);
    });

    e3.get::<&Value>(|v| {
        assert_eq!(v.value, 31);
    });
}

#[ignore = "pre,on,post frame is internal only, replace or delete this test"]
#[test]
fn system_custom_pipeline() {
    // let world = World::new();

    // let mut preFrame = world.entity().add::<flecs::pipeline::Phase>();
    // let OnFrame = world.entity().add::<flecs::pipeline::Phase>().depends_on(PreFrame);
    // let mut postFrame = world.entity().add::<flecs::pipeline::Phase>().depends_on(OnFrame);
    // let tag = world.entity();

    // let pip = world.pipeline()
    //     .with::<flecs::system::System>()
    //     .with(flecs::pipeline::Phase).cascade(flecs::DependsOn)
    //     .with(Tag)
    //     .build();

    // int count = 0;

    // world.system::<()>()
    //     .kind(PostFrame)
    //     .run(|mut it| {
    //         while it.next() {
    //                 world.get::<&Count>(|c| {
    //                 assert_eq!(c.0,2);
    //             });
    //             world.get::<&mut Count>(|c| {
    //                 c.0 += 1;
    //             });
    //         }
    //     })
    //     .add(Tag);

    // world.system::<()>()
    //     .kind(OnFrame)
    //     .run(|mut it| {
    //         while it.next() {
    //             world.get::<&Count>(|c| {
    //                 assert_eq!(c.0,1);
    //             });
    //             world.get::<&mut Count>(|c| {
    //                 c.0 += 1;
    //             });
    //         }
    //     })
    //     .add(Tag);

    // world.system::<()>()
    //     .kind(PreFrame)
    //     .run(|mut it| {
    //         while it.next() {
    //             assert_eq!(count, 0);
    //             world.get::<&mut Count>(|c| {
    //                 c.0 += 1;
    //             });
    //         }
    //     })
    //     .add(Tag);

    // assert_eq!(count, 0);

    // world.set_pipeline_id(pip);

    // world.progress();

    //     world.get::<&Count>(|c| {
    //                 assert_eq!(c.0,3);
    //             });
}

#[test]
fn system_custom_pipeline_w_kind() {
    let world = World::new();

    let tag = world.entity();

    let pip = world
        .pipeline()
        .with::<flecs::system::System>()
        .with_id(tag)
        .build();

    world.set(Count(0));

    world.system::<()>().kind_id(tag).run(|mut it| {
        while it.next() {
            let world = it.world();
            world.get::<&mut Count>(|c| {
                assert_eq!(c, 0);
                c.0 += 1;
            });
        }
    });

    world.system::<()>().kind_id(tag).run(|mut it| {
        let world = it.world();
        while it.next() {
            world.get::<&mut Count>(|c| {
                assert_eq!(c.0, 1);
                c.0 += 1;
            });
        }
    });

    world.system::<()>().kind_id(tag).run(|mut it| {
        let world = it.world();
        while it.next() {
            world.get::<&mut Count>(|c| {
                assert_eq!(c.0, 2);
                c.0 += 1;
            });
        }
    });

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 0);
    });

    world.set_pipeline_id(pip.id());

    world.progress();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 3);
    });
}

#[test]
fn system_kind_while_deferred() {
    let world = World::new();

    let sys = world.defer(|| {
        world
            .system::<()>()
            .kind_id(flecs::pipeline::OnValidate::ID)
            .run(|_| {})
    });

    world.progress();

    let sys = sys.entity_view(&world);
    assert!(sys.has_id(flecs::pipeline::OnValidate::ID));
    assert!(sys.has_first::<flecs::DependsOn>(flecs::pipeline::OnValidate::ID));
    assert!(!sys.has_id(flecs::pipeline::OnUpdate::ID));
    assert!(!sys.has_first::<flecs::DependsOn>(flecs::pipeline::OnUpdate::ID));
}

#[test]
fn system_create_w_no_template_args() {
    let world = World::new();

    let entity = world.entity().set(Position { x: 10, y: 20 });

    let entity_id = entity.id();
    world.set(Count(0));

    let s = world
        .system::<()>()
        .with::<Position>()
        .each_entity(move |e, _| {
            let world = e.world();
            assert!(e == entity_id);
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        });

    s.run();

    world.get::<&Count>(|c| {
        assert_eq!(c.0, 1);
    });
}

#[derive(Component)]
struct PipelineType;
#[derive(Component)]
struct First;
#[derive(Component)]
struct Second;

#[test]
fn system_system_w_type_kind_type_pipeline() {
    let world = World::new();

    world
        .component::<Second>()
        .add::<flecs::pipeline::Phase>()
        .depends_on_id(world.component::<First>().add::<flecs::pipeline::Phase>());

    world
        .pipeline_type::<PipelineType>()
        .with::<flecs::system::System>()
        .with::<flecs::pipeline::Phase>()
        .cascade_type::<flecs::DependsOn>()
        .build();

    world.set_pipeline::<PipelineType>();

    let entity = world.entity().add::<Tag>();
    let entity_id = entity.id();

    world.set(Count2 { a: 0, b: 0 });

    world.system::<&Tag>().kind::<Second>().run(move |mut it| {
        while it.next() {
            for i in it.iter() {
                let e = it.entity(i);
                let world = e.world();
                assert!(e == entity_id);
                world.get::<&mut Count2>(|c| {
                    assert_eq!(c.a, 0);
                    assert_eq!(c.b, 1);
                    c.a += 1;
                });
            }
        }
    });

    world.system::<&Tag>().kind::<First>().run(move |mut it| {
        while it.next() {
            for i in it.iter() {
                let world = it.world();
                let e = it.entity(i);
                assert!(e == entity_id);
                world.get::<&mut Count2>(|c| {
                    assert_eq!(c.b, 0);
                    c.b += 1;
                });
            }
        }
    });

    world.progress();

    world.get::<&Count2>(|c| {
        assert_eq!(c.a, 1);
        assert_eq!(c.b, 1);
    });
}

#[test]
fn system_entity_ctor() {
    let world = World::new();

    world.set(Count(0));

    let sys = world.system::<()>().run(|mut it| {
        let world = it.world();
        while it.next() {
            world.get::<&mut Count>(|c| {
                c.0 += 1;
            });
        }
    });

    let sys_from_id = world.system_from(sys.entity_view(&world));

    sys_from_id.run();
    world.get::<&Count>(|c| {
        assert_eq!(c.0, 1);
    });
}

#[test]
fn system_multithread_system_w_query_each() {
    let world = World::new();

    world.set_threads(2);

    let e1 = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world.new_query::<&Velocity>();

    world
        .system::<&mut Position>()
        .multi_threaded()
        .each_entity(move |e, p| {
            let world = e.world();
            q.iter_stage(world).each(|v| {
                p.x += v.x;
                p.y += v.y;
            });
        });

    world.progress();

    e1.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });
}

#[test]
fn system_multithread_system_w_query_each_w_iter() {
    let world = World::new();

    world.set_threads(2);

    let e = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world.new_query::<&Velocity>();

    world
        .system::<&mut Position>()
        .multi_threaded()
        .each_iter(move |it, _i, p| {
            q.iter_stage(it.world()).each(|v| {
                p.x += v.x;
                p.y += v.y;
            });
        });

    world.progress();

    e.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });
}

#[test]
fn system_multithread_system_w_query_each_w_world() {
    let world = World::new();

    world.set_threads(2);

    let e = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world.new_query::<&Velocity>();
    world
        .system::<&mut Position>()
        .multi_threaded()
        .each_iter(move |it, _i, p| {
            let world = it.world();
            q.iter_stage(world).each(|v| {
                p.x += v.x;
                p.y += v.y;
            });
        });

    world.progress();

    e.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });
}

#[test]
fn system_multithread_system_w_query_iter() {
    let world = World::new();

    world.set_threads(2);

    let e = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world.new_query::<&Velocity>();

    world
        .system::<&mut Position>()
        .multi_threaded()
        .each_entity(move |e, p| {
            q.iter_stage(e).run(|mut it| {
                while it.next() {
                    let v = it.field::<Velocity>(0).unwrap();

                    for i in it.iter() {
                        p.x += v[i].x;
                        p.y += v[i].y;
                    }
                }
            });
        });

    world.progress();

    e.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });
}

#[test]
fn system_multithread_system_w_query_iter_w_iter() {
    let world = World::new();

    world.set_threads(2);

    let e = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world.new_query::<&Velocity>();

    world
        .system::<&mut Position>()
        .multi_threaded()
        .each_iter(move |it, _i, p| {
            q.iter_stage(it.world()).run(|mut it| {
                while it.next() {
                    let v = it.field::<Velocity>(0).unwrap();
                    for i in it.iter() {
                        p.x += v[i].x;
                        p.y += v[i].y;
                    }
                }
            });
        });

    world.progress();

    e.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });
}

#[test]
fn system_multithread_system_w_query_iter_w_world() {
    let world = World::new();

    world.set_threads(2);

    let e = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    let q = world.new_query::<&Velocity>();

    world
        .system::<&mut Position>()
        .multi_threaded()
        .each_iter(move |it, _i, p| {
            q.iter_stage(it.world()).run(|mut it| {
                while it.next() {
                    let v = it.field::<Velocity>(0).unwrap();
                    for i in it.iter() {
                        p.x += v[i].x;
                        p.y += v[i].y;
                    }
                }
            });
        });

    world.progress();

    e.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });
}

#[test]
fn system_run_callback() {
    let world = World::new();

    let entity = world
        .entity()
        .set(Position { x: 10, y: 20 })
        .set(Velocity { x: 1, y: 2 });

    world.system::<(&mut Position, &Velocity)>().run_each(
        |mut it| {
            while it.next() {
                it.each();
            }
        },
        |(p, v)| {
            p.x += v.x;
            p.y += v.y;
        },
    );

    world.progress();

    entity.get::<&Position>(|p| {
        assert_eq!(p.x, 11);
        assert_eq!(p.y, 22);
    });

    entity.get::<&Velocity>(|v| {
        assert_eq!(v.x, 1);
        assert_eq!(v.y, 2);
    });
}

#[test]
fn system_startup_system() {
    let world = World::new();

    world.set(Count2 { a: 0, b: 0 });

    world
        .system::<()>()
        .kind_id(flecs::pipeline::OnStart::ID)
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                assert!(it.delta_time() == 0.0);
                world.get::<&mut Count2>(|c| {
                    c.a += 1;
                });
            }
        });

    world
        .system::<()>()
        .kind_id(flecs::pipeline::OnUpdate::ID)
        .run(|mut it| {
            let world = it.world();
            while it.next() {
                assert!(it.delta_time() != 0.0);
                world.get::<&mut Count2>(|c| {
                    c.b += 1;
                });
            }
        });

    world.progress();
    world.get::<&Count2>(|c| {
        assert_eq!(c.a, 1);
        assert_eq!(c.b, 1);
    });
    world.progress();
    world.get::<&Count2>(|c| {
        assert_eq!(c.a, 1);
        assert_eq!(c.b, 2);
    });
}

#[test]
fn system_interval_tick_source() {
    let world = World::new();

    let t = world.timer().set_interval(2.1);

    t.get::<&mut flecs::timer::Timer>(|timer| {
        timer.time = 0.0;
    });

    world.set(Count2 { a: 0, b: 0 });

    world.system::<()>().set_tick_source_id(t).run(|mut it| {
        let world = it.world();
        while it.next() {
            world.get::<&mut Count2>(|c| {
                c.a += 1;
            });
        }
    });

    world.system::<()>().set_tick_source_id(t).run(|mut it| {
        let world = it.world();
        while it.next() {
            world.get::<&mut Count2>(|c| {
                c.b += 1;
            });
        }
    });

    world.progress_time(1.0);
    let c = world.cloned::<&Count2>();
    assert_eq!(c.a, 0);
    assert_eq!(c.b, 0);

    world.progress_time(1.0);
    let c = world.cloned::<&Count2>();
    assert_eq!(c.a, 0);
    assert_eq!(c.b, 0);

    world.progress_time(1.0);
    let c = world.cloned::<&Count2>();
    assert_eq!(c.a, 1);
    assert_eq!(c.b, 1);
}

#[test]
fn system_rate_tick_source() {
    let world = World::new();

    let t = world.timer().set_rate(3);

    world.set(Count2 { a: 0, b: 0 });

    world.system::<()>().set_tick_source_id(t).run(|mut it| {
        let world = it.world();
        while it.next() {
            world.get::<&mut Count2>(|c| {
                c.a += 1;
            });
        }
    });

    world.system::<()>().set_tick_source_id(t).run(|mut it| {
        let world = it.world();
        while it.next() {
            world.get::<&mut Count2>(|c| {
                c.b += 1;
            });
        }
    });

    world.progress_time(1.0);
    let c = world.cloned::<&Count2>();
    assert_eq!(0, c.a);
    assert_eq!(0, c.b);

    world.progress_time(1.0);
    let c = world.cloned::<&Count2>();
    assert_eq!(0, c.a);
    assert_eq!(0, c.b);

    world.progress_time(1.0);
    let c = world.cloned::<&Count2>();
    assert_eq!(1, c.a);
    assert_eq!(1, c.b);
}

#[ignore = "timer addon not implemented"]
#[test]
fn system_nested_rate_tick_source() {
    // let world = World::new();

    // let t_3 = world.timer().set_rate(3);
    // let t_6 = world.timer().set_rate(2, t_3);

    // int32_t sys_a_invoked = 0, sys_b_invoked = 0;

    // world.system::<()>()
    //     .tick_source(t_3)
    //     .run(|mut it| {
    //         while it.next() {
    //             sys_a_world.get::<&mut Count>(|c| {
    //                 c.0 += 1;
    //             });
    //         }
    //     });

    // world.system::<()>()
    //     .tick_source(t_6)
    //     .run(|mut it| {
    //         while it.next() {
    //             sys_b_world.get::<&mut Count>(|c| {
    //                 c.0 += 1;
    //             });
    //         }
    //     });

    // world.progress(1.0);
    // assert_eq!(0, sys_a_invoked);
    // assert_eq!(0, sys_b_invoked);

    // world.progress(1.0);
    // assert_eq!(0, sys_a_invoked);
    // assert_eq!(0, sys_b_invoked);

    // world.progress(1.0);
    // assert_eq!(1, sys_a_invoked);
    // assert_eq!(0, sys_b_invoked);

    // world.progress(1.0);
    // assert_eq!(1, sys_a_invoked);
    // assert_eq!(0, sys_b_invoked);

    // world.progress(1.0);
    // assert_eq!(1, sys_a_invoked);
    // assert_eq!(0, sys_b_invoked);

    // world.progress(1.0);
    // assert_eq!(2, sys_a_invoked);
    // assert_eq!(1, sys_b_invoked);
}

// #[test] fn system_table_get() {
//     let world = World::new();

//     let e1 = world.entity().set(Position{x: 10, y: 20});
//     flecs::entity e2 = world.entity().set(Position{x: 20, y: 30});

//     let s = world.system::<()>()
//         .with::<Position>()
//         .each([&](flecs::iter& iter, size_t index) {
//             let e = iter.entity(index);
//             &Position *p = &iter.table().get<Position>()[index];
//             assert!(p != nullptr);
//             assert!(e == e1 || e == e2);
//             if (e == e1) {
//                 assert_eq!(p.x, 10);
//                 assert_eq!(p.y, 20);
//             } else if (e == e2) {
//                 assert_eq!(p.x, 20);
//                 assert_eq!(p.y, 30);
//             }
//         });

//     s.run();
// }

// #[test] fn system_range_get() {
//     let world = World::new();

//     let e1 = world.entity().set(Position{x: 10, y: 20});
//     flecs::entity e2 = world.entity().set(Position{x: 20, y: 30});

//     let s = world.system::<()>()
//         .with::<Position>()
//         .each([&](flecs::iter& iter, size_t index) {
//             let e = iter.entity(index);
//             &Position *p = &iter.range().get<Position>()[index];
//             assert!(p != nullptr);
//             assert!(e == e1 || e == e2);
//             if (e == e1) {
//                 assert_eq!(p.x, 10);
//                 assert_eq!(p.y, 20);
//             } else if (e == e2) {
//                 assert_eq!(p.x, 20);
//                 assert_eq!(p.y, 30);
//             }
//         });

//     s.run();
// }

#[test]
fn system_randomize_timers() {
    let world = World::new();

    let s1 = world
        .system::<()>()
        .set_interval(1.0)
        .run(|mut it| while it.next() {});

    {
        let t = s1.try_cloned::<&flecs::timer::Timer>();
        assert!(t.is_some());
        assert!(t.unwrap().time == 0.0);
    }

    world.randomize_timers();

    let s2 = world
        .system::<()>()
        .set_interval(1.0)
        .run(|mut it| while it.next() {});

    {
        let t = s1.try_cloned::<&flecs::timer::Timer>();
        assert!(t.is_some());
        assert!(t.unwrap().time != 0.0);
    }

    {
        let t = s2.try_cloned::<&flecs::timer::Timer>();
        assert!(t.is_some());
        assert!(t.unwrap().time != 0.0);
    }
}

#[test]
fn system_run_w_0_src_query() {
    let world = World::new();

    world.set(Count(0));

    world.system::<()>().write::<Position>().run(|it| {
        let world = it.world();
        world.get::<&mut Count>(|c| {
            c.0 += 1;
        });
    });

    world.progress();
    world.get::<&Count>(|c| {
        assert_eq!(c.0, 1);
    });
}
