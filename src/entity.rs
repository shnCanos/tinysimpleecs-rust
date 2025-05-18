use std::{any::TypeId, collections::HashMap, fmt::Debug};

use bit_set::BitSet;
use tinysimpleecs_rust_macros::create_query_type;

use crate::component::{self, Bundle};

#[derive(Hash, Default, Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) struct EntityId(usize);

impl EntityId {
    fn new(id: usize) -> Self {
        Self(id)
    }
}

#[derive(Default, Debug)]
pub(crate) struct EntityComponents(Box<[Box<dyn component::Component>]>);

impl<T: Bundle> From<T> for EntityComponents {
    fn from(value: T) -> Self {
        Self(value.into_array())
    }
}
#[derive(Debug)]
pub(crate) struct EntityBitmask(BitSet);

impl EntityBitmask {
    pub(crate) fn new(bitset: BitSet) -> Self {
        Self(bitset)
    }
}

impl std::ops::Deref for EntityBitmask {
    type Target = BitSet;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl EntityBitmask {
    pub(crate) fn from_components(
        components: &EntityComponents,
        components_manager: &mut component::ComponentManger,
    ) -> Self {
        let component_types: Vec<_> = components.0.iter().map(|comp| (**comp).type_id()).collect();
        let mut bit_indexes = BitSet::new();
        for comptype in component_types {
            let id = components_manager.register_component_if_not_exists(comptype);
            assert!(
                bit_indexes.insert(id),
                "Only one of each component type per entity allowed"
            );
        }
        Self::new(bit_indexes)
    }
}

#[derive(Debug)]
struct EntityInfo {
    id: EntityId,
    bitmask: EntityBitmask,
    components: EntityComponents,
}

impl EntityInfo {
    fn new<T: Bundle>(
        id: EntityId,
        components: T,
        components_manager: &mut component::ComponentManger,
    ) -> Self {
        let entity_components = components.into();
        Self {
            id,
            bitmask: EntityBitmask::from_components(&entity_components, components_manager),
            components: entity_components,
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct EntityManager {
    entities: Vec<EntityInfo>,
    next_id: usize,
}

trait ComponentsQuery {
    fn into_bitmask(self, components_manager: &component::ComponentManger) -> EntityBitmask;
}

create_query_type!(1, 15, ComponentsQuery);

struct Query<Q, R = ()>
where
    Q: ComponentsQuery,
{
    components: Q,
    restrictions: R,
}

struct QueryBitmask {
    components: EntityBitmask,
    restrictions: EntityBitmask,
}

impl QueryBitmask {
    fn new(components: EntityBitmask, restrictions: EntityBitmask) -> Self {
        Self {
            components,
            restrictions,
        }
    }
}

impl<Q, R> Query<Q, R>
where
    Q: ComponentsQuery,
{
    pub(crate) fn into_bitmask(
        self,
        components_manager: &component::ComponentManger,
    ) -> QueryBitmask {
        // 1. Add restrictions too
        // 2. Make sure that there's no overlap between restrictions and queries
        let components_bitmask = self.components.into_bitmask(components_manager);

        QueryBitmask::new(components_bitmask, EntityBitmask::new(BitSet::default()))
    }
}

impl EntityManager {
    fn new_entity_id(&mut self) -> EntityId {
        let new_entity = EntityId::new(self.next_id);
        self.next_id += 1;
        return new_entity;
    }

    pub(crate) fn spawn<T: Bundle>(
        &mut self,
        components: T,
        components_manager: &mut component::ComponentManger,
    ) -> EntityId {
        let new_entity_id = self.new_entity_id();
        let new_entity = EntityInfo::new(new_entity_id, components, components_manager);
        self.entities.push(new_entity);
        new_entity_id
    }

    /// Not to be confused with entity_id
    pub(crate) fn get_entity_index(&self, entity_id: EntityId) -> Option<usize> {
        self.entities
            .iter()
            .position(|entity_info| entity_info.id == entity_id)
    }

    pub(crate) fn get_entity_info(&self, entity_id: EntityId) -> Option<&EntityInfo> {
        self.entities
            .iter()
            .find(|entity_info| entity_info.id == entity_id)
    }

    pub(crate) fn entity_exists(&self, entity_id: EntityId) -> bool {
        self.get_entity_index(entity_id).is_some()
    }

    pub(crate) fn despawn(&mut self, entity_id: EntityId) {
        self.entities
            .swap_remove(match self.get_entity_index(entity_id) {
                Some(index) => index,
                None => panic!("Unable to find entity!"),
            });
    }

    pub(crate) fn query<Q: ComponentsQuery, R>(
        &mut self,
        query: Query<Q, R>,
        components_manager: &component::ComponentManger,
    ) -> Vec<&mut EntityComponents> {
        let query_bitmask = query.into_bitmask(components_manager);

        let mut matches = Vec::new();
        for entity_info in &mut self.entities {
            let within_query = entity_info.bitmask.is_subset(&query_bitmask.components);
            let within_restrictions = entity_info
                .bitmask
                .intersection(&query_bitmask.restrictions)
                .next()
                .is_some();

            if !within_query || within_restrictions {
                continue;
            }

            matches.push(&mut entity_info.components);
        }

        matches
    }
}
