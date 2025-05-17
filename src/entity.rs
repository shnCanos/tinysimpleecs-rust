use crate::component;

#[derive(Hash, Default, Debug, PartialEq, Eq, Clone, Copy)]
struct EntityId(usize);

impl EntityId {
    fn new(id: usize) -> Self {
        Self(id)
    }
}

#[derive(Default)]
struct EntityComponents(Box<[Box<dyn component::Component>]>);

impl<T: component::Bundle> From<T> for EntityComponents {
    fn from(value: T) -> Self {
        Self(value.into_array())
    }
}

struct EntityInfo {
    id: EntityId,
    components: EntityComponents,
}

impl EntityInfo {
    fn new(id: EntityId, components: EntityComponents) -> Self {
        Self { id, components }
    }
}

#[derive(Default)]
pub(crate) struct EntityManager {
    entities: Vec<EntityInfo>,
    next_id: usize,
}

impl EntityManager {
    fn new_entity_id(&mut self) -> EntityId {
        let new_entity = EntityId::new(self.next_id);
        self.next_id += 1;
        return new_entity;
    }

    pub(crate) fn spawn<T: component::Bundle>(&mut self, components: T) {
        let new_entity_id = self.new_entity_id();
        let new_entity = EntityInfo::new(new_entity_id, components.into());
        self.entities.push(new_entity);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_empty_entity() {}
}
