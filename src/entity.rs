use bit_set::BitSet;

use crate::component;
use crate::Bundle;

#[derive(Hash, Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct EntityId(usize);

impl EntityId {
    pub(crate) fn new(id: usize) -> Self {
        Self(id)
    }
}

#[derive(Debug)]
pub struct EntityBitmask(pub(crate) BitSet);

impl EntityBitmask {
    pub(crate) fn new(bitset: BitSet) -> Self {
        Self(bitset)
    }
}

impl From<BitSet> for EntityBitmask {
    fn from(value: BitSet) -> Self {
        Self::new(value)
    }
}

impl std::ops::Deref for EntityBitmask {
    type Target = BitSet;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct EntityInfo {
    pub(crate) id: EntityId,
    pub(crate) bitmask: EntityBitmask,
    pub(crate) component_indexes: Box<[usize]>,
}

impl EntityInfo {
    pub(crate) fn new(
        id: EntityId,
        bitmask: EntityBitmask,
        component_indexes: Box<[usize]>,
    ) -> Self {
        Self {
            id,
            bitmask,
            component_indexes,
        }
    }

    pub(crate) fn from_bundle(
        id: EntityId,
        components: impl Bundle,
        components_manager: &mut component::ComponentManager,
    ) -> Self {
        components.add(id, components_manager)
    }
}

#[derive(Default, Debug)]
pub(crate) struct EntityManager {
    entities: Vec<EntityInfo>,
    next_id: usize,
}

impl EntityManager {
    fn new_entity_id(&mut self) -> EntityId {
        let new_entity = EntityId::new(self.next_id);
        self.next_id += 1;
        new_entity
    }

    pub(crate) fn spawn(
        &mut self,
        components: impl Bundle,
        components_manager: &mut component::ComponentManager,
    ) -> EntityId {
        let new_entity_id = self.new_entity_id();
        let new_entity = EntityInfo::from_bundle(new_entity_id, components, components_manager);
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
}
