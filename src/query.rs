use std::marker::PhantomData;

use crate::{
    component::ComponentManager,
    entity::{EntityBitmask, EntityManager},
    Bundle, ComponentOrder,
};

pub struct QueryInfo<Values: Bundle, Restrictions: Bundle> {
    query_bitmask: EntityBitmask,
    restrictions_bitmask: EntityBitmask,
    query_order: ComponentOrder,
    _values: PhantomData<Values>,
    _restrictions: PhantomData<Restrictions>,
}

impl<Values: Bundle, Restrictions: Bundle> QueryInfo<Values, Restrictions> {
    pub fn new<V: Bundle, R: Bundle>(component_manager: &mut ComponentManager) -> Self {
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

    pub fn from_query<V: Bundle, R: Bundle>(
        component_manager: &mut ComponentManager,
        _query: &Query<V, R>,
    ) -> Self {
        Self::new::<V, R>(component_manager)
    }
}

pub struct Query<Values: Bundle, Restrictions: Bundle> {
    result: Box<[Values]>,
    _values: PhantomData<Values>,
    _restrictions: PhantomData<Restrictions>,
}

impl<Values: Bundle, Restrictions: Bundle> Query<Values, Restrictions> {
    fn new(result: Box<[Values]>) -> Self {
        Self {
            result,
            _values: PhantomData,
            _restrictions: PhantomData,
        }
    }

    pub fn apply(entity_manager: &EntityManager, component_manager: &mut ComponentManager) -> Self {
        let info: QueryInfo<Values, Restrictions> =
            QueryInfo::new::<Values, Restrictions>(component_manager);
        // NOTE: The results are ordered by component_id
        let indexes_slice = entity_manager.query(&info.query_bitmask, &info.restrictions_bitmask);

        let result = indexes_slice
            .into_iter()
            .map(|indexes| {
                Values::from_indexes(
                    &info.query_bitmask,
                    &info.query_order,
                    indexes,
                    component_manager,
                )
            })
            .collect();

        Self::new(result)
    }
}
