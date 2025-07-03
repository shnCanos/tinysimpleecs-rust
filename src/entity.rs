use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

use any_vec::AnyVec;
use bit_set::BitSet;

use crate::Component;
use crate::ComponentBundle;
use crate::component;

#[derive(Hash, Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct EntityId(usize);

impl EntityId {
    pub(crate) fn new(id: usize) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct EntityBitmask(pub(crate) BitSet);

impl DerefMut for EntityBitmask {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl EntityBitmask {
    pub(crate) fn new(bitset: BitSet) -> Self {
        Self(bitset)
    }

    pub(crate) fn matches_query(&self, query: &Self, restrictions: &Self) -> bool {
        query.is_subset(self) && restrictions.is_disjoint(self)
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
pub struct ComponentColumns(Box<[AnyVec]>);

impl ComponentColumns {
    fn new(columns: Box<[AnyVec]>) -> Self {
        Self(columns)
    }

    pub(crate) fn get_mut_from_column<C: Component>(
        &mut self,
        column: usize,
        index: usize,
    ) -> Option<&mut C> {
        self[column]
            .get_mut(index)
            .and_then(|mut val| val.downcast_mut::<C>())
    }
}

impl Deref for ComponentColumns {
    type Target = Box<[AnyVec]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ComponentColumns {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub(crate) struct Archetype {
    pub(crate) entities: Vec<EntityId>,
    pub(crate) component_columns: ComponentColumns,
}

impl Archetype {
    pub(crate) fn new(component_columns: Box<[AnyVec]>) -> Self {
        Self {
            entities: Vec::default(),
            component_columns: ComponentColumns::new(component_columns),
        }
    }
}

#[derive(Default, Debug)]
pub struct EntityManager {
    pub(crate) archetypes: HashMap<EntityBitmask, Archetype>,
}

impl EntityManager {
    pub(crate) fn spawn(
        &mut self,
        new_entity_id: EntityId,
        components: impl ComponentBundle,
        components_manager: &mut component::ComponentManager,
    ) {
        components.spawn(new_entity_id, self, components_manager);
    }

    pub(crate) fn add_entity(
        &mut self,
        id: EntityId,
        bitmask: EntityBitmask,
        default_columns: Box<[fn() -> AnyVec]>,
        inserters: Box<[Box<dyn FnOnce(&mut AnyVec)>]>,
    ) {
        let archetype = self
            .archetypes
            .entry(bitmask)
            .or_insert_with(|| Archetype::new(default_columns.iter().map(|f| f()).collect()));

        archetype.entities.push(id);

        for (column, inserter) in archetype
            .component_columns
            .iter_mut()
            .zip(inserters.into_iter())
        {
            inserter(column);
        }
    }

    // TODO: Change this to something more sane
    #[cfg(test)]
    pub(crate) fn entity_exists(&self, entity_id: &EntityId) -> bool {
        self.archetypes
            .values()
            .any(|a| a.entities.iter().any(|e| e == entity_id))
    }

    // TODO: Change this to something more sane
    pub(crate) fn despawn(&mut self, entity_id: &EntityId) {
        let (archetype, entity_index) = self
            .archetypes
            .values_mut()
            .find_map(|archetype| {
                archetype
                    .entities
                    .iter()
                    .position(|current_entity| current_entity == entity_id)
                    .map(|entity_index| (archetype, entity_index))
            })
            .expect("Attempted to despawn non-existent entity!");

        archetype.entities.swap_remove(entity_index);
        for component_list in archetype.component_columns.iter_mut() {
            component_list.swap_remove(entity_index);
        }
    }

    pub(crate) fn query(
        &mut self,
        query_bitmask: &EntityBitmask,
        restrictions_bitmask: &EntityBitmask,
    ) -> Box<[(&EntityBitmask, &mut Archetype)]> {
        self.archetypes
            .iter_mut()
            .filter_map(|(bitmask, archetype)| {
                if bitmask.matches_query(query_bitmask, restrictions_bitmask) {
                    Some((bitmask, archetype))
                } else {
                    None
                }
            })
            .collect()
    }
}
