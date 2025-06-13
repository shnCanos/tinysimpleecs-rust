#![feature(alloc_layout_extra)]
#![feature(allocator_api)]

use component::{ComponentBundle, ComponentManager};
use entity::{EntityBitmask, EntityId, EntityInfo};

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

    pub(crate) fn spawn(&mut self, components: impl ComponentBundle) -> EntityId {
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
    pub fn spawn(&mut self, tospawn: impl ComponentBundle + 'static) {
        self.actions_queue.push(Box::new(move |world: &mut World| {
            world.spawn(tospawn);
        }));
    }
}

#[cfg(test)]
mod tests {
    use crate::query::Query;

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
    #[should_panic(expected = "duplicate component type in query")]
    fn test_query_with_duplicate_component_panics() {
        let mut world = World::new();
        let _ = world.spawn((Banana {},));

        // This should panic due to repeated component type `Banana`
        let _query: Query<(Banana, Banana), ()> =
            unsafe { Query::apply(&world.entity_manager, &mut world.components_manager) };
    }

    #[test]
    #[should_panic(expected = "duplicate component type in entity")]
    fn test_entity_with_duplicate_component_panics() {
        let mut world = World::new();

        // This should panic because `Banana` appears twice
        let _ = world.spawn((Banana {}, Banana {}));
    }

    #[test]
    fn query_entities() {
        let mut world = World::new();
        let _ = world.spawn(((Banana {}),));
        let _ = world.spawn((Banana {}, Banana2(23)));
        let _ = world.spawn(((Banana2(24)),));

        // SAFETY: no two queries are alive at the same time, therefore it's safe

        {
            let query: Query<(Banana,), ()> =
                unsafe { Query::apply(&world.entity_manager, &mut world.components_manager) };
            assert_eq!(query.results[0].entity, EntityId::new(0));
            assert_eq!(query.results[1].entity, EntityId::new(1));
            assert_eq!(query.results.len(), 2);
        }

        {
            let query: Query<(Banana2,), ()> =
                unsafe { Query::apply(&world.entity_manager, &mut world.components_manager) };
            assert_eq!(query.results.len(), 2);
            assert_eq!(query.results[0].entity, EntityId::new(1));
            assert_eq!(query.results[1].entity, EntityId::new(2));
            assert_eq!(query.results[1].components.0 .0, 24);
            assert_eq!(query.results[0].components.0 .0, 23);
            assert_eq!(query.results[1].components.0 .0, 24);
            query.results[1].components.0 .0 += 1;
        }

        {
            let query: Query<(Banana, Banana2), ()> =
                unsafe { Query::apply(&world.entity_manager, &mut world.components_manager) };
            assert_eq!(query.results.len(), 1);
            assert_eq!(query.results[0].entity, EntityId::new(1));
        }

        {
            let query: Query<(Banana,), (Banana2,)> =
                unsafe { Query::apply(&world.entity_manager, &mut world.components_manager) };
            assert_eq!(query.results.len(), 1);
            assert_eq!(query.results[0].entity, EntityId::new(0));
        }

        {
            // Re-run the query to check new state
            let query: Query<(Banana2,), ()> =
                unsafe { Query::apply(&world.entity_manager, &mut world.components_manager) };
            assert_eq!(query.results[0].entity, EntityId::new(1));
            assert_eq!(query.results[1].entity, EntityId::new(2));
            assert_eq!(query.results[1].components.0 .0, 25);
        }
    }
}
