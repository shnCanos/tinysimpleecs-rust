use tinysimpleecs_rust::{Commands, Component, Query, World};

#[allow(dead_code)]
#[derive(Debug, Component)]
struct MyComponent(String);

fn duplicatecommandssystem(_commands: &mut Commands, _othercommands: &mut Commands) {
    println!("Two mutable references to commands but running anyway!");
}

fn mustrestrictsystem(_query: Query<(MyComponent,), ()>, _query2: Query<(MyComponent,), ()>) {
    println!("Two Repeated Queries but running anyway!")
}

fn main() {
    let mut world = World::new();
    println!(
        "This should return a duplicate commands error: {:?}",
        world.add_system(duplicatecommandssystem).unwrap_err()
    );
    println!(
        "This should return a must restrict error: {:?}",
        world.add_system(mustrestrictsystem).unwrap_err()
    );

    // This should work, however
    unsafe {
        world.add_system_unchecked(duplicatecommandssystem);
        world.add_system_unchecked(mustrestrictsystem);
    }
    world.run_all_systems();
}
