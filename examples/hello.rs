use tinysimpleecs_rust::{Commands, Component, Query, World};

#[allow(dead_code)]
#[derive(Debug, Component)]
struct MyComponent(String);

#[derive(Debug, Component)]
struct MyFlag;

fn hello_there(
    commands: &mut Commands,
    query: Query<(MyComponent,), ()>,
    flag_query: Query<(MyFlag,), ()>,
) {
    if flag_query.results.is_empty() {
        commands.spawn((MyComponent("Hello".to_owned()),));
        commands.spawn((MyComponent("Not Hello".to_owned()),));
        commands.spawn((MyFlag,));
    }

    println!("Printing!");
    for result in &query.results {
        dbg!(result);
        commands.despawn(result.entity);
    }
}

fn main() {
    let mut world = World::new();
    world.add_system(hello_there).unwrap();

    println!(
        "- In the first print, nothing should appear since the commands have yet to be applied.\n- However, in the second print, the two entities that we spawned should appear. In the third, nothing should appear again since we despawned the entities!"
    );
    world.run_all_systems();
    world.run_all_systems();
    world.run_all_systems();
}
