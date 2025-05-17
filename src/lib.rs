use component::Component;
use entity::EntityId;
use tinysimpleecs_rust_macros::Component;

mod component;
mod entity;

#[derive(Default)]
struct World {
    components_manager: component::ComponentManger,
    entity_manager: entity::EntityManager,
    commands: Commands,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn<T: component::Bundle>(&mut self, components: T) -> EntityId {
        self.entity_manager
            .spawn(components, &mut self.components_manager)
    }

    pub fn despawn(&mut self, entity: entity::EntityId) {
        self.entity_manager.despawn(entity);
    }
}

#[derive(Default)]
pub struct Commands {}

impl Commands {
    pub fn spawn<T: component::Bundle>(tospawn: T) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Component, Debug)]
    pub struct Banana;

    #[derive(Component, Debug)]
    pub struct Banana2(usize);

    #[test]
    fn manual_spawn_entity() {
        let mut world = World::new();
        let id = world.spawn((Banana {}, Banana2(23)));
        assert!(world.entity_manager.entity_exists(id));
        assert!(world.components_manager.component_exists::<Banana>());
        assert!(world.components_manager.component_exists::<Banana2>());
    }
}
