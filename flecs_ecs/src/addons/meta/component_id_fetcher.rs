use flecs_ecs::prelude::*;

pub struct ComponentIdFetcher<T> {
    pub phantom: std::marker::PhantomData<T>,
}

pub trait FlecsComponent {
    fn deref_id<'a>(&self, world: impl IntoWorld<'a>) -> u64;
}

pub trait ExternalComponent {
    fn deref_id<'a>(&self, world: impl IntoWorld<'a>) -> u64;
}

impl<T: ComponentId> FlecsComponent for &&ComponentIdFetcher<T> {
    fn deref_id<'a>(&self, world: impl IntoWorld<'a>) -> u64 {
        T::id(world)
    }
}

impl<T: 'static> ExternalComponent for &ComponentIdFetcher<T> {
    fn deref_id<'a>(&self, world: impl IntoWorld<'a>) -> u64 {
        let world = world.world();
        let map = world.components_map();
        *(map
            .entry(std::any::TypeId::of::<T>())
            .or_insert_with(|| external_register_component::<T>(world, std::ptr::null())))
    }
}

// The reason this macro exists is while we could use lookup by name, it's not as efficient as using the typeid map.
// a simple benchmark of looking up 100'000 components by name vs typeid map:
// typeid map:
// Elapsed: 236.083µs
// Elapsed per id: 2ns
//
// lookup by name through `external_register_component`:
// Elapsed: 28.224417ms
// Elapsed per id: 282ns
#[macro_export]
macro_rules! Id {
    ($world:expr, $type:ty) => {
        (&&flecs_ecs::addons::meta::ComponentIdFetcher::<$type> {
            phantom: std::marker::PhantomData,
        })
            .deref_id($world)
    };
}

#[cfg(test)]
mod test {

    #[test]
    fn meta_id_macro_test() {
        use flecs_ecs::prelude::*;

        #[derive(Component)]
        struct Position {
            x: f32,
            y: f32,
        }

        struct ExtermalPosition {
            x: f32,
            y: f32,
        }

        let world = World::new();

        let id = Id!(&world, Position);
        assert_eq!(id, world.component_id::<Position>());
        let id_ext = Id!(&world, ExtermalPosition);
        assert_ne!(id_ext, id);

        //compile test
        let world_ref = world.get_world();

        let id_world_ref = Id!(world_ref, Position);
        assert_eq!(id_world_ref, id);

        let id_ext_world_ref = Id!(world_ref, ExtermalPosition);
        assert_eq!(id_ext_world_ref, id_ext);
    }
}
