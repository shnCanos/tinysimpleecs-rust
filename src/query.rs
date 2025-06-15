use std::{collections::HashMap, marker::PhantomData};

use tinysimpleecs_rust_macros::implement_query_bundle;

use crate::{
    component::{ComponentId, ComponentManager},
    entity::{EntityBitmask, EntityId},
    system::SystemParam,
};

pub(crate) struct QueryInfo {
    pub(crate) query_bitmask: EntityBitmask,
    pub(crate) restrictions_bitmask: EntityBitmask,
    query_order: ComponentOrder,
}

impl QueryInfo {
    pub(crate) fn new(
        query_bitmask: EntityBitmask,
        restrictions_bitmask: EntityBitmask,
        query_order: ComponentOrder,
    ) -> Self {
        Self {
            query_bitmask,
            restrictions_bitmask,
            query_order,
        }
    }

    pub(crate) fn from_query<V: QueryBundle, R: QueryBundle>(
        components_manager: &mut ComponentManager,
    ) -> Self {
        let (query_bitmask, query_order) = V::into_bitmask(components_manager);
        let (restrictions_bitmask, _) = R::into_bitmask(components_manager);
        let new_info = Self {
            query_bitmask,
            restrictions_bitmask,
            query_order,
        };
        debug_assert!(new_info
            .query_bitmask
            .is_disjoint(&new_info.restrictions_bitmask));
        new_info
    }
}

pub(crate) struct QueryResult<ResultType> {
    pub(crate) entity: EntityId,
    pub(crate) components: ResultType,
}

impl<ResultType> From<(EntityId, ResultType)> for QueryResult<ResultType> {
    fn from(value: (EntityId, ResultType)) -> Self {
        Self {
            entity: value.0,
            components: value.1,
        }
    }
}

pub struct Query<'a, Values: QueryBundle, Restrictions: QueryBundle> {
    pub(crate) results: Box<[QueryResult<Values::ResultType<'a>>]>,
    pub(crate) info: QueryInfo,
    _restrictions: PhantomData<Restrictions>,
}

impl<'a, Values: QueryBundle, Restrictions: QueryBundle> Query<'a, Values, Restrictions> {
    fn new(results: Box<[QueryResult<Values::ResultType<'a>>]>, info: QueryInfo) -> Self {
        Self {
            results,
            info,
            _restrictions: PhantomData,
        }
    }
}

impl<'a, Values: QueryBundle, Restrictions: QueryBundle> SystemParam
    for Query<'a, Values, Restrictions>
{
    /// SAFETY: Cannot have two queries with the same component at the same time or multiple mutable references to the same value is possible.
    unsafe fn init(world: *mut crate::World) -> Self {
        let info: QueryInfo =
            QueryInfo::from_query::<Values, Restrictions>(&mut (*world).components_manager);
        // NOTE: The results are ordered by component_id
        let indexes_slice = (*world)
            .entity_manager
            .query(&info.query_bitmask, &info.restrictions_bitmask);

        let result = indexes_slice
            .into_iter()
            .map(|(entity, indexes)| QueryResult {
                entity: *entity,
                components: Values::from_indexes(
                    &info.query_order,
                    indexes,
                    &mut (*world).components_manager,
                ),
            })
            .collect();

        Self::new(result, info)
    }

    fn query_info(&self) -> Option<&QueryInfo> {
        Some(&self.info)
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
