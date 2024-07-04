use crate::z_ignore_test_common::*;

use flecs_ecs::prelude::*;
// Transitive relationships make it possible to tell the ECS that if an entity
// has a relationship (R, X) and X has relationship (R, Y), the entity should be
// treated as if it also has (R, Y). In practice this is useful for expressing
// things like:
//
// Bob lives in SanFrancisco
// San Francisco is located in the United States
// Therefore Bob also lives in the United States.
//
// An example of transitivity can be seen in the component_inheritance example.
// This example uses the builtin IsA relationship, which is transitive. This
// example shows how to achieve similar behavior with a user-defined relationship.

#[derive(Component)]
struct LocatedIn;

#[derive(Component)]
struct Planet;

#[derive(Component)]
struct Continent;

#[derive(Component)]
struct Country;

#[derive(Component)]
struct State;

#[derive(Component)]
struct City;

#[derive(Component)]
struct Person;

fn main() {
    let world = World::new();

    // Register the LocatedIn relationship as transitive
    world.component::<LocatedIn>().add::<flecs::Transitive>();

    // Populate the store with locations
    let earth = world.entity_named("Earth").add::<Planet>();

    // Continents
    let north_america = world
        .entity_named("NorthAmerica")
        .add::<Continent>()
        .add_first::<LocatedIn>(earth);

    let europe = world
        .entity_named("Europe")
        .add::<Continent>()
        .add_first::<LocatedIn>(earth);

    // Countries
    let united_states = world
        .entity_named("UnitedStates")
        .add::<Country>()
        .add_first::<LocatedIn>(north_america);

    let netherlands = world
        .entity_named("Netherlands")
        .add::<Country>()
        .add_first::<LocatedIn>(europe);

    // States
    let california = world
        .entity_named("California")
        .add::<State>()
        .add_first::<LocatedIn>(united_states);

    let washington = world
        .entity_named("Washington")
        .add::<State>()
        .add_first::<LocatedIn>(united_states);

    let noord_holland = world
        .entity_named("NoordHolland")
        .add::<State>()
        .add_first::<LocatedIn>(netherlands);

    // Cities
    let san_francisco = world
        .entity_named("SanFrancisco")
        .add::<City>()
        .add_first::<LocatedIn>(california);

    let seattle = world
        .entity_named("Seattle")
        .add::<City>()
        .add_first::<LocatedIn>(washington);

    let amsterdam = world
        .entity_named("Amsterdam")
        .add::<City>()
        .add_first::<LocatedIn>(noord_holland);

    // Inhabitants
    world
        .entity_named("Bob")
        .add::<Person>()
        .add_first::<LocatedIn>(san_francisco);

    world
        .entity_named("Alice")
        .add::<Person>()
        .add_first::<LocatedIn>(seattle);

    world
        .entity_named("Job")
        .add::<Person>()
        .add_first::<LocatedIn>(amsterdam);

    // Create a query that finds the countries persons live in. Note that these
    // have not been explicitly added to the Person entities, but because the
    // LocatedIn is transitive, the query engine will traverse the relationship
    // until it found something that is a country.
    //
    // The equivalent of this query in the DSL is:
    //   Person, (LocatedIn, $Location), Country($Location)

    let query = world
        .query::<()>()
        .with::<&Person>()
        .with_first_name::<&LocatedIn>("$Location")
        .with::<&Country>()
        .set_src_name("$Location")
        .build();

    // Lookup the index of the variable. This will let us quickly lookup its
    // value while we're iterating.
    let location_var = query.find_var("Location").unwrap();

    // Iterate the query
    query.each_iter(|it, index, _| {
        println!("{} lives in {}", it.entity(index), it.get_var(location_var));
    });

    // Output:
    //  Bob lives in UnitedStates
    //  Alice lives in UnitedStates
    //  Job lives in Netherlands
}

#[cfg(feature = "flecs_nightly_tests")]
#[test]
fn test() {
    let output_capture = OutputCapture::capture().unwrap();
    main();
    output_capture.test("query_transitive_queries".to_string());
}
