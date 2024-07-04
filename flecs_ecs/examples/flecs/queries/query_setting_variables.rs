use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;
// This example extends the component_inheritance example, and shows how
// we can use a single query to match units from different players and platoons
// by setting query variables before we iterate.
//
// The units in this example belong to a platoon, with the platoons belonging
// to a player.

// unit datamodel
#[derive(Component)]
struct Unit;

#[derive(Component)]
struct CombatUnit;

#[derive(Component)]
struct MeleeUnit;

#[derive(Component)]
struct RangedUnit;

#[derive(Component)]
struct Warrior;

#[derive(Component)]
struct Wizard;

#[derive(Component)]
struct Marksman;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Platoon;

const PLAYER_COUNT: usize = 100;
const PLATOONS_PER_PLAYER: usize = 3;

fn main() {
    let world = World::new();

    // Make the ECS aware of the inheritance relationships. Note that IsA
    // relationship used here is the same as in the prefab example.
    world.component::<CombatUnit>().is_a::<Unit>();
    world.component::<MeleeUnit>().is_a::<CombatUnit>();
    world.component::<RangedUnit>().is_a::<CombatUnit>();

    world.component::<Warrior>().is_a::<MeleeUnit>();
    world.component::<Wizard>().is_a::<RangedUnit>();
    world.component::<Marksman>().is_a::<RangedUnit>();

    // Populate store with players and platoons
    for p in 0..PLAYER_COUNT {
        let player = if p == 0 {
            // Give first player a name so we can look it up later
            world.entity_named("MyPlayer")
        } else {
            world.entity()
        };

        // Add player tag so we can query for all players if we want to
        player.add::<Player>();

        for _ in 0..PLATOONS_PER_PLAYER {
            let platoon = world
                .entity()
                .add_first::<Player>(player)
                // Add platoon tag so we can query for all platoons if we want to
                .add::<Platoon>();

            // Add warriors, wizards and marksmen to the platoon
            world
                .entity()
                .add::<Warrior>()
                .add_first::<Platoon>(platoon);
            world
                .entity()
                .add::<Marksman>()
                .add_first::<Platoon>(platoon);
            world.entity().add::<Wizard>().add_first::<Platoon>(platoon);
        }
    }

    // Create a query to find all RangedUnits for a platoon/player. The
    // equivalent query in the query DSL would look like this:
    //   (Platoon, $Platoon), Player($Platoon, $Player)
    //
    // The way to read how this query is evaluated is:
    // - find all entities with (Platoon, *), store * in _Platoon
    // - check if _Platoon has (Player, *), store * in _Player
    let mut query = world
        .query::<&RangedUnit>()
        .with::<&Platoon>()
        .set_second_name("$platoon")
        .with_first_name::<&Player>("$player")
        .set_src_name("$platoon")
        .build();

    // If we would iterate this query it would return all ranged units for all
    // platoons & for all players. We can limit the results to just a single
    // platoon or a single player setting a variable beforehand. In this example
    // we'll just find all platoons & ranged units for a single player.

    let player_var = query.find_var("player").unwrap();
    let platoon_var = query.find_var("platoon").unwrap();

    // Iterate query, limit the results to units of MyPlayer
    query
        .set_var(player_var, world.lookup_recursively("MyPlayer"))
        .each_iter(|it, index, _| {
            let unit = it.entity(index);
            println!(
                "Unit id: {} of class {} in platoon id: {} for player {}",
                unit,
                it.id(0).to_str(),
                it.get_var(platoon_var),
                it.get_var(player_var)
            );
        });

    // Output:
    //  Unit id: 529 of class Wizard in platoon id: 526 for player MyPlayer
    //  Unit id: 533 of class Wizard in platoon id: 530 for player MyPlayer
    //  Unit id: 537 of class Wizard in platoon id: 534 for player MyPlayer
    //  Unit id: 528 of class Marksman in platoon id: 526 for player MyPlayer
    //  Unit id: 532 of class Marksman in platoon id: 530 for player MyPlayer
    //  Unit id: 536 of class Marksman in platoon id: 534 for player MyPlayer

    // Try removing the set_var call, this will cause the iterator to return
    // all units in all platoons for all players.
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("query_setting_variables".to_string());
}
