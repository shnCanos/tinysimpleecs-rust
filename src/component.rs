use std::{
    any::TypeId,
    collections::{BTreeMap, HashMap},
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
};

use any_vec::{AnyVec, any_value::AnyValueWrapper};

use crate::{EntityId, EntityManager, entity::EntityBitmask};

pub trait Component: std::fmt::Debug + 'static {}

pub(crate) type ComponentId = usize;
#[derive(Default)]
pub struct ComponentManager {
    components: HashMap<TypeId, ComponentId>,
    last_used_id: ComponentId,
}

impl ComponentManager {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    fn get_new_id(&mut self) -> usize {
        let id = self.last_used_id;
        self.last_used_id += 1;
        id
    }

    pub(crate) fn register_component_unchecked<C: Component>(&mut self) -> ComponentId {
        let id = self.get_new_id();
        let result = self.components.insert(TypeId::of::<C>(), id);
        debug_assert!(result.is_none());
        id
    }

    pub(crate) fn get_component_id<C: Component>(&self) -> Option<ComponentId> {
        self.components.get(&TypeId::of::<C>()).map(|&id| id)
    }

    pub(crate) fn register_component_if_not_exists<C: Component>(&mut self) -> ComponentId {
        self.get_component_id::<C>()
            .unwrap_or_else(|| self.register_component_unchecked::<C>())
    }

    pub(crate) fn component_exists<C: Component>(&self) -> bool {
        self.components.contains_key(&TypeId::of::<C>())
    }
}

pub trait ComponentBundle {
    fn spawn(
        self,
        id: EntityId,
        entity_manager: &mut EntityManager,
        component_manager: &mut ComponentManager,
    );
}

macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}
macro_rules! impl_component_bundle {
    ($(($n:tt, $B:ident)),*) => {
        impl<$($B: Component),*> ComponentBundle for ($($B,)*) {
            fn spawn(
                self,
                id: EntityId,
                entity_manager: &mut EntityManager,
                component_manager: &mut ComponentManager,
            ) {
                let len = <[()]>::len(&[$(replace_expr!($n ())),*]);
                let mut bitmask = EntityBitmask::default();
                let mut components_btree = BTreeMap::<usize, usize>::new();
                $({
                    let id = component_manager.register_component_if_not_exists::<$B>();
                    let previous = components_btree.insert(id, $n);
                    debug_assert!(previous.is_none(), "duplicate component type in entity");

                    bitmask.insert(id);
                })*

                let components_order: HashMap<usize, usize> = components_btree.into_iter().enumerate().map(|(i, (_, v))| (v, i)).collect();

                let mut default_columns = Box::<[fn() -> AnyVec]>::new_uninit_slice(len);
                let mut inserters = Box::<[Box<dyn FnOnce(&mut AnyVec)>]>::new_uninit_slice(len);
                $({
                    let current_index = components_order[&$n];
                    default_columns[current_index].write(|| AnyVec::new::<$B>());
                    inserters[current_index].write(Box::new(|v: &mut AnyVec| v.push(AnyValueWrapper::new(self.$n))));
                })*

                entity_manager.add_entity(id, bitmask, unsafe {default_columns.assume_init()}, unsafe {inserters.assume_init()});
            }
        }
    };
}

variadics_please::all_tuples_enumerated!(impl_component_bundle, 0, 15, B);
