use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use tinysimpleecs_rust_macros::implement_bundle;

use crate::entity;

#[derive(Default)]
pub(crate) struct ComponentManger {
    components: HashMap<TypeId, usize>,
    next_id: usize,
}

impl ComponentManger {
    pub(crate) fn register_component_unchecked(&mut self, component: TypeId) -> usize {
        let id = self.next_id;
        self.components.insert(component, id);
        self.next_id += 1;
        id
    }
    pub(crate) fn register_component_if_not_exists(&mut self, component: TypeId) -> usize {
        match self.get_component_id(component) {
            Some(id) => *id,
            None => self.register_component_unchecked(component),
        }
    }
    pub(crate) fn get_component_id(&self, component: TypeId) -> Option<&usize> {
        self.components.get(&component)
    }

    pub(crate) fn component_exists<T: Component>(&self) -> bool {
        let comp = TypeId::of::<T>();
        self.components.contains_key(&comp)
    }
}

pub trait Component: Any + std::fmt::Debug {}

/// Why create a "Bundle" and an "into_slice" when what they do is equivalent to the code below?
/// impl Into<Box<[Box<dyn Component>]>> for (/* --- */) {
///     /* defining into */
/// }
///
/// Why, simply because it looks better
pub trait Bundle {
    fn into_array(self) -> Box<[Box<dyn Component>]>;
}

variadics_please::all_tuples!(implement_bundle, 1, 15, B);
