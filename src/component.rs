use std::{
    any::TypeId,
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use any_vec::AnyVec;
use tinysimpleecs_rust_macros::implement_bundle;

pub(crate) struct ComponentCollumn {
    /// The id used for the bitmask
    id: usize,
    data: AnyVec,
}

impl ComponentCollumn {
    pub(crate) fn new<C: Component + 'static>(id: usize) -> Self {
        let data = AnyVec::new::<ComponentInserter<C>>();
        return ComponentCollumn { id, data };
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
    last_used_id: usize,
}

impl ComponentManager {
    pub(crate) fn new() -> Self {
        return Self::default();
    }

    pub(crate) fn register_component_unchecked<C: Component>(&mut self) {
        let type_id = TypeId::of::<C>();
        let result = self.components.insert(type_id, AnyVec::new::<C>());
        debug_assert!(result.is_none());
    }

    pub(crate) fn uncheked_component_vec<C: Component>(&mut self) -> &mut AnyVec {
        let result = self.components.get_mut(&TypeId::of::<C>());
        debug_assert!(result.is_some());
        return result.unwrap();
    }

    pub(crate) fn register_component_if_not_exists<C: Component>(&mut self) {
        let type_id = TypeId::of::<C>();
        if self.components.contains_key(&type_id) {
            return;
        }
        self.register_component_unchecked::<C>();
    }

    pub(crate) fn add_component_as<C: Component>(&mut self, component: C) {
        self.register_component_if_not_exists::<C>();
        let type_id = TypeId::of::<C>();
        let anyvec = self.components.get_mut(&type_id).unwrap();
        anyvec.push(component);
    }
}

pub struct ComponentInserter<C: Component>(C);

pub trait Component: std::fmt::Debug {}

pub trait Bundle {
    fn add(self, manager: &mut ComponentManager);
}

variadics_please::all_tuples!(implement_bundle, 1, 15, B);
