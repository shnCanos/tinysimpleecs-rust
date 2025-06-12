use std::marker::PhantomData;

use crate::{component::ComponentManager, entity::EntityBitmask, Bundle};

pub struct QueryBitmask<Values: Bundle, Restrictions: Bundle> {
    query_bitmask: EntityBitmask,
    restrictions_bitmaks: EntityBitmask,
    _values: PhantomData<Values>,
    _restrictions: PhantomData<Restrictions>,
}

impl<Values: Bundle, Restrictions: Bundle> QueryBitmask<Values, Restrictions> {
    pub fn new<V: Bundle, R: Bundle>(component_manager: &mut ComponentManager) -> Self {
        Self {
            query_bitmask: V::into_bitmask(component_manager),
            restrictions_bitmaks: R::into_bitmask(component_manager),
            _values: PhantomData,
            _restrictions: PhantomData,
        }
    }
}

pub struct Query<Values: Bundle, Restrictions: Bundle> {
    info: QueryBitmask<Values, Restrictions>,
    result: Values,
}
