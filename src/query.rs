use std::marker::PhantomData;

use crate::{
    component::ComponentManager,
    entity::{EntityBitmask, EntityManager},
    Bundle,
};

pub struct QueryBitmask<Values: Bundle, Restrictions: Bundle> {
    query_bitmask: EntityBitmask,
    restrictions_bitmaks: EntityBitmask,
    _values: PhantomData<Values>,
    _restrictions: PhantomData<Restrictions>,
}

impl<Values: Bundle, Restrictions: Bundle> QueryBitmask<Values, Restrictions> {
    pub fn new<V: Bundle, R: Bundle>(component_manager: &mut ComponentManager) -> Self {
        let query_bitmask = Self {
            query_bitmask: V::into_bitmask(component_manager),
            restrictions_bitmaks: R::into_bitmask(component_manager),
            _values: PhantomData,
            _restrictions: PhantomData,
        };

        debug_assert!(query_bitmask
            .query_bitmask
            .is_disjoint(&query_bitmask.restrictions_bitmaks));

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
    result: Values,
    _values: PhantomData<Values>,
    _restrictions: PhantomData<Restrictions>,
}

impl<Values: Bundle, Restrictions: Bundle> Query<Values, Restrictions> {
    pub fn apply(entity_manager: &EntityManager, component_manager: &mut ComponentManager) -> Self {
        let bitmask = QueryBitmask::new::<Values, Restrictions>(component_manager);
        // NOTE: The results are ordered by component_id
        let results = entity_manager.query(&bitmask.query_bitmask, &bitmask.restrictions_bitmaks);
    }
}
