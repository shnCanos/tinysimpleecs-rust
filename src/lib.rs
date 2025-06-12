#![feature(alloc_layout_extra)]
#![feature(allocator_api)]

use std::collections::HashMap;

use component::ComponentManager;
use entity::{EntityBitmask, EntityId, EntityInfo};
use tinysimpleecs_rust_macros::implement_bundle;

mod component;
mod entity;
mod query;

#[derive(Default)]
pub struct World {
    components_manager: component::ComponentManager,
    entity_manager: entity::EntityManager,
    commands: Commands,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn spawn(&mut self, components: impl Bundle) -> EntityId {
        self.entity_manager
            .spawn(components, &mut self.components_manager)
    }

    pub(crate) fn despawn(&mut self, entity: &entity::EntityId) {
        self.entity_manager.despawn(entity);
    }
}

type CommandAction = Vec<Box<dyn FnOnce(&mut World)>>;

#[derive(Default)]
pub struct Commands {
    actions_queue: CommandAction,
}

impl Commands {
    pub fn spawn(&mut self, tospawn: impl Bundle + 'static) {
        self.actions_queue.push(Box::new(move |world: &mut World| {
            world.spawn(tospawn);
        }));
    }
}

type ComponentOrder = HashMap<usize, usize>;
pub trait Bundle {
    fn add(self, entity: EntityId, manager: &mut ComponentManager) -> EntityInfo;
    fn into_bitmask(component_manager: &mut ComponentManager) -> (EntityBitmask, ComponentOrder);
    fn from_indexes(
        bitmask: &EntityBitmask,
        order: &ComponentOrder,
        indexes: &[usize],
        component_manager: &mut ComponentManager,
    ) -> Self;
}

variadics_please::all_tuples!(implement_bundle, 0, 15, B);

#[cfg(test)]
mod tests {
    use super::component::*;
    use super::*;
    use bit_set::BitSet;
    use tinysimpleecs_rust_macros::Component;

    #[derive(Component, Debug)]
    pub struct Banana;

    #[derive(Component, Debug)]
    pub struct Banana2(usize);

    #[test]
    fn manual_spawn_entity() {
        let mut world = World::new();
        let id = world.spawn((Banana {}, Banana2(23)));
        assert!(world.entity_manager.entity_exists(&id));
        assert!(world.components_manager.component_exists::<Banana>());
        assert!(world.components_manager.component_exists::<Banana2>());
    }

    #[test]
    fn entityinfo() {
        let mut world = World::new();
        let id1 = world.spawn(((Banana {}),));
        let id2 = world.spawn((Banana {}, Banana2(23)));
        let id3 = world.spawn(((Banana2(23)),));

        let info1 = world.entity_manager.get_entity_info(&id1).unwrap();
        let info2 = world.entity_manager.get_entity_info(&id2).unwrap();
        let info3 = world.entity_manager.get_entity_info(&id3).unwrap();

        assert_eq!(*info1.component_indexes, [0]);
        assert_eq!(*info2.component_indexes, [1, 0]);
        assert_eq!(*info3.component_indexes, [1]);

        assert_eq!(info1.id, EntityId::new(0));
        assert_eq!(info2.id, EntityId::new(1));
        assert_eq!(info3.id, EntityId::new(2));

        assert_eq!(info1.bitmask.0, BitSet::from_bytes(&[0b10000000]));
        assert_eq!(info2.bitmask.0, BitSet::from_bytes(&[0b11000000]));
        assert_eq!(info3.bitmask.0, BitSet::from_bytes(&[0b01000000]));
    }

    #[test]
    fn query_entities() {
        let mut world = World::new();
        const BANANA_STARTING: usize = 0;
    }
}
