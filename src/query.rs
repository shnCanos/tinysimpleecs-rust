use std::{collections::HashMap, marker::PhantomData};

use crate::{
    component::{ComponentId, ComponentManager},
    entity::{EntityBitmask, EntityId},
    system::{SafetyInfo, SystemParam},
    SystemWorldArgs,
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

#[derive(Debug)]
pub struct QueryResult<ResultType> {
    pub entity: EntityId,
    pub components: ResultType,
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
    pub results: Box<[QueryResult<Values::ResultType<'a>>]>,
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
    unsafe fn init(args: *mut crate::SystemWorldArgs) -> Self {
        let info: QueryInfo =
            QueryInfo::from_query::<Values, Restrictions>((*args).components_manager);
        // NOTE: The results are ordered by component_id
        let indexes_slice = (*args)
            .entity_manager
            .query(&info.query_bitmask, &info.restrictions_bitmask);

        let result = indexes_slice
            .into_iter()
            .map(|(entity, indexes)| QueryResult {
                entity: *entity,
                components: Values::from_indexes(
                    &info.query_order,
                    indexes,
                    (*args).components_manager,
                ),
            })
            .collect();

        Self::new(result, info)
    }

    fn safety_info(args: &mut SystemWorldArgs) -> Option<SafetyInfo> {
        Some(SafetyInfo::Query(QueryInfo::from_query::<
            Values,
            Restrictions,
        >(args.components_manager)))
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

macro_rules! impl_query_bundle {
    ($(($n:tt, $Q:ident)),*) => {
        impl<$($Q: crate::component::Component),*> QueryBundle for ($($Q,)*) {
            type ResultType<'a> = ($(&'a mut $Q,)*);
            #[allow(unused_assignments, unused_variables, unused_mut)]
            fn into_bitmask(component_manager: &mut ComponentManager) -> (EntityBitmask, ComponentOrder) {
                let mut bitset = ::bit_set::BitSet::new();
                let mut order = ::std::collections::HashMap::new();
                let mut current_index = 0;

                $(
                    let id = component_manager.register_component_if_not_exists::<$Q>();
                    let had_inserted = bitset.insert(id);
                    debug_assert!(had_inserted, "duplicate component type in query");

                    order.insert(id, current_index);
                    current_index += 1;
                )*

                (bitset.into(), order)
            }

            #[allow(clippy::unused_unit)]
            #[allow(unused_assignments, unused_variables, unused_mut, invalid_value)]
            unsafe fn from_indexes<'a>(
                order: &ComponentOrder,
                indexes: &[usize],
                component_manager: *mut ComponentManager,
            ) -> Self::ResultType<'a> {
                let mut newtuple: Self::ResultType<'a> = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
                $({
                    let current_id = (*component_manager).get_component_id::<$Q>().unwrap();
                    newtuple.$n = (*component_manager).get_from_index::<$Q>(indexes[order[&current_id]]).unwrap();
                })*
                newtuple
            }
        }
    };
}

variadics_please::all_tuples_enumerated!(impl_query_bundle, 0, 15, B);
