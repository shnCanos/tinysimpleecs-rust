use std::{
    any::{Any, TypeId},
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use any_vec::{any_value::AnyValueWrapper, AnyVec};
use tinysimpleecs_rust_macros::implement_bundle;

use crate::{entity::EntityBitmask, EntityId};

pub(crate) type ComponentId = usize;
pub(crate) type ComponentIndex = usize;

pub(crate) struct ComponentCollumn {
    /// The id used for the bitmask
    id: ComponentId,
    data: AnyVec,
}

pub(crate) struct ComponentWrapper<C: Component> {
    entity: EntityId,
    component: C,
}

impl<C: Component> ComponentWrapper<C> {
    pub(crate) fn new(entity: EntityId, component: C) -> Self {
        Self { entity, component }
    }
}

impl ComponentCollumn {
    pub(crate) fn new<C: Component>(id: usize) -> Self {
        let data = AnyVec::new::<ComponentWrapper<C>>();
        ComponentCollumn { id, data }
    }
}

impl Deref for ComponentCollumn {
    type Target = AnyVec;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for ComponentCollumn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[derive(Default)]
pub(crate) struct ComponentManager {
    components: HashMap<TypeId, ComponentCollumn>,
    last_used_id: ComponentId,
}

impl ComponentManager {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn register_component_unchecked<C: Component>(&mut self) -> ComponentId {
        let id = self.last_used_id;
        self.last_used_id += 1;

        let result = self
            .components
            .insert(TypeId::of::<C>(), ComponentCollumn::new::<C>(id));
        debug_assert!(result.is_none());
        id
    }

    pub(crate) fn register_component_if_not_exists<C: Component>(&mut self) -> ComponentId {
        if let Some(collumn) = self.components.get(&TypeId::of::<C>()) {
            return collumn.id;
        }
        self.register_component_unchecked::<C>()
    }

    pub(crate) fn add_component<C: Component>(
        &mut self,
        entity: EntityId,
        component: C,
    ) -> (ComponentId, ComponentIndex) {
        let id = self.register_component_if_not_exists::<C>();
        let collumn = self.components.get_mut(&TypeId::of::<C>()).unwrap();
        collumn.push(AnyValueWrapper::new(ComponentWrapper::new(
            entity, component,
        )));
        (id, collumn.len() - 1)
    }
    #[cfg(test)]
    pub(crate) fn component_exists<C: Component>(&self) -> bool {
        self.components.contains_key(&TypeId::of::<C>())
    }
    // #[cfg(test)]
    // pub(crate) fn get_component_id<C: Component>(&self, _component: C) -> Option<ComponentId> {
    //     self.components
    //         .get(&TypeId::of::<C>())
    //         .map(|collumn| collumn.id)
    // }
}

pub trait Component: std::fmt::Debug + 'static {}

pub trait Bundle {
    fn add(self, entity: EntityId, manager: &mut ComponentManager)
        -> (EntityBitmask, Box<[usize]>);
}

variadics_please::all_tuples!(implement_bundle, 1, 15, B);
