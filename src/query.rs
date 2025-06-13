use std::{
    collections::{BinaryHeap, HashMap},
    marker::PhantomData,
    mem::MaybeUninit,
};

use tinysimpleecs_rust_macros::implement_query_bundle;

use crate::{
    component::{ComponentBundle, ComponentId, ComponentManager},
    entity::{EntityBitmask, EntityManager},
};

pub struct QueryInfo<Values: QueryBundle, Restrictions: QueryBundle> {
    query_bitmask: EntityBitmask,
    restrictions_bitmask: EntityBitmask,
    query_order: ComponentOrder,
    _values: PhantomData<Values>,
    _restrictions: PhantomData<Restrictions>,
}

impl<Values: QueryBundle, Restrictions: QueryBundle> QueryInfo<Values, Restrictions> {
    pub fn new<V: QueryBundle, R: QueryBundle>(component_manager: &mut ComponentManager) -> Self {
        let (query_bitmask, query_order) = V::into_bitmask(component_manager);
        let (restrictions_bitmask, _) = R::into_bitmask(component_manager);

        let query_bitmask = Self {
            query_bitmask,
            restrictions_bitmask,
            query_order,
            _values: PhantomData,
            _restrictions: PhantomData,
        };

        debug_assert!(query_bitmask
            .query_bitmask
            .is_disjoint(&query_bitmask.restrictions_bitmask));

        query_bitmask
    }

    pub fn from_query<V: QueryBundle, R: QueryBundle>(
        component_manager: &mut ComponentManager,
        _query: &Query<V, R>,
    ) -> Self {
        Self::new::<V, R>(component_manager)
    }
}

pub struct Query<'a, Values: QueryBundle, Restrictions: QueryBundle> {
    pub(crate) result: Box<[Values::ResultType<'a>]>,
    _values: PhantomData<Values>,
    _restrictions: PhantomData<Restrictions>,
}

impl<'a, Values: QueryBundle, Restrictions: QueryBundle> Query<'a, Values, Restrictions> {
    fn new(result: Box<[Values::ResultType<'a>]>) -> Self {
        Self {
            result,
            _values: PhantomData,
            _restrictions: PhantomData,
        }
    }

    /// SAFETY: Cannot have two queries with the same component at the same time or multiple mutable references to the same value is possible.
    pub unsafe fn apply(
        entity_manager: &EntityManager,
        component_manager: &mut ComponentManager,
    ) -> Self {
        let info: QueryInfo<Values, Restrictions> =
            QueryInfo::new::<Values, Restrictions>(component_manager);
        // NOTE: The results are ordered by component_id
        let indexes_slice = entity_manager.query(&info.query_bitmask, &info.restrictions_bitmask);

        let result = indexes_slice
            .into_iter()
            .map(|indexes| {
                Values::from_indexes(
                    &info.query_order,
                    indexes,
                    component_manager as *mut ComponentManager,
                )
            })
            .collect();

        Self::new(result)
    }
}

type ComponentOrder = HashMap<ComponentId, usize>;
pub trait QueryBundle {
    type ResultType<'a>;
    fn into_bitmask(component_manager: &mut ComponentManager) -> (EntityBitmask, ComponentOrder);
    /// SAFETY: Cannot have two queries with the same component at the same time or multiple mutable references to the same value is possible.
    unsafe fn from_indexes<'a>(
        order: &ComponentOrder,
        indexes: &[usize],
        component_manager: *mut ComponentManager,
    ) -> Self::ResultType<'a>;
}

variadics_please::all_tuples!(implement_query_bundle, 0, 15, B);
