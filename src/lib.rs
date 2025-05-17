use component::Component;
use tinysimpleecs_rust_macros::Component;

mod component;
mod entity;

#[derive(Default)]
struct World {
    components_manager: component::ComponentManger,
    entity_manager: entity::EntityManager,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Component)]
pub struct Banana;

pub struct Commands {}

impl Commands {
    pub fn spawn<T: component::Bundle>(tospawn: T) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registering_components() {
        let _ = World::new();
    }
}
