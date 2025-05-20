use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
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

    pub(crate) fn component_exists(&self, comp: &TypeId) -> bool {
        self.components.contains_key(comp)
    }
}

impl From<&[TypeId]> for ComponentManger {
    fn from(value: &[TypeId]) -> Self {
        let mut components_manager = Self::default();
        for comp in value {
            assert!(!components_manager.component_exists(comp));
            components_manager.register_component_unchecked(*comp);
        }
        components_manager
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
    fn into_array(self) -> Box<[Rc<RefCell<dyn Component>>]>;
}

variadics_please::all_tuples!(implement_bundle, 1, 15, B);
