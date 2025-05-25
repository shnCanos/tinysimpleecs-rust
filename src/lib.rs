use std::{any::TypeId, cell::RefCell, rc::Rc};

use component::{Component, ComponentManger};
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

    pub fn with_components(components: &[TypeId]) -> Self {
        Self {
            components_manager: components.into(),
            ..Default::default()
        }
    }

    fn spawn(&mut self, components: impl component::Bundle) -> EntityId {
        self.entity_manager
            .spawn(components, &mut self.components_manager)
    }

    fn despawn(&mut self, entity: entity::EntityId) {
        self.entity_manager.despawn(entity);
    }

    fn query(
        &mut self,
        query_bitmask: entity::QueryBitmask,
    ) -> Box<[Box<[Rc<RefCell<dyn component::Component>>]>]> {
        self.entity_manager
            .query(query_bitmask, &self.components_manager)
    }
}

#[derive(Default)]
pub struct Commands {}

impl Commands {
    pub fn spawn(tospawn: impl component::Bundle) {}
}

#[macro_export]
macro_rules! mkcomponents {
    ($($entity:ident),*) => {
        &[$(::std::any::TypeId::of::<$entity>()),*]
    };
}

#[cfg(test)]
mod tests {
    use super::component::*;
    use super::entity::*;
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
        assert!(world
            .components_manager
            .component_exists(&TypeId::of::<Banana>()));
        assert!(world
            .components_manager
            .component_exists(&TypeId::of::<Banana2>()));
    }

    #[test]
    fn add_entities_macro() {
        let world = World::with_components(mkcomponents!(Banana, Banana2));
        assert!(world
            .components_manager
            .component_exists(&TypeId::of::<Banana>()));
        assert!(world
            .components_manager
            .component_exists(&TypeId::of::<Banana2>()));
    }

    #[test]
    fn query_entities() {
        let mut world = World::with_components(mkcomponents!(Banana, Banana2));
        const BANANA_STARTING: usize = 0;
    }
}
