use std::{
    collections::{BTreeMap, HashMap},
    marker::PhantomData,
};

use crate::{
    EntityManager, SystemWorldArgs,
    component::{ComponentId, ComponentManager},
    entity::{ComponentColumns, EntityBitmask, EntityId},
    system::{SafetyInfo, SystemParam},
};

pub(crate) struct QueryInfo {
    pub(crate) query_bitmask: EntityBitmask,
    pub(crate) restrictions_bitmask: EntityBitmask,
}

impl QueryInfo {
    pub(crate) fn new(query_bitmask: EntityBitmask, restrictions_bitmask: EntityBitmask) -> Self {
        Self {
            query_bitmask,
            restrictions_bitmask,
        }
    }

    pub(crate) fn from_query<V: QueryBundle, R: QueryBundle>(
        components_manager: &mut ComponentManager,
    ) -> Self {
        let query_bitmask = V::into_bitmask(components_manager);
        let restrictions_bitmask = R::into_bitmask(components_manager);
        let new_info = Self {
            query_bitmask,
            restrictions_bitmask,
        };
        debug_assert!(
            new_info
                .query_bitmask
                .is_disjoint(&new_info.restrictions_bitmask)
        );
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
        let archetypes = (*args)
            .entity_manager
            .query(&info.query_bitmask, &info.restrictions_bitmask);

        let result = archetypes
            .into_vec()
            .into_iter()
            .flat_map(|(bitmask, archetype)| {
                let archetype_order = Values::into_order((*args).components_manager, bitmask);
                archetype
                    .entities
                    .iter()
                    .enumerate()
                    .map(|(i, &entity)| QueryResult {
                        entity,
                        components: Values::from_columns(
                            i,
                            &archetype_order,
                            &mut archetype.component_columns as *mut ComponentColumns,
                        ),
                    })
                    .collect::<Vec<_>>()
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

type ComponentOrder = Box<[usize]>;
pub trait QueryBundle {
    type ResultType<'a>;
    fn into_bitmask(component_manager: &mut ComponentManager) -> EntityBitmask;
    // NOTE: it is assumed that every component already exists when this function is called
    fn into_order(
        component_manager: &ComponentManager,
        other_bitmask: &EntityBitmask,
    ) -> ComponentOrder;
    /// SAFETY: Cannot have two queries with the same component at the same time or multiple mutable references to the same value is possible.
    unsafe fn from_columns<'a>(
        index: usize,
        archetype_order: &ComponentOrder,
        columns: *mut ComponentColumns,
    ) -> Self::ResultType<'a>;
}

macro_rules! impl_query_bundle {
    ($(($n:tt, $Q:ident)),*) => {
        impl<$($Q: crate::component::Component),*> QueryBundle for ($($Q,)*) {
            type ResultType<'a> = ($(&'a mut $Q,)*);
            #[allow(unused_assignments, unused_variables, unused_mut)]
            fn into_bitmask(component_manager: &mut ComponentManager) -> EntityBitmask {
                let mut bitset = bit_set::BitSet::new();

                $(
                    let id = component_manager.register_component_if_not_exists::<$Q>();
                    let had_inserted = bitset.insert(id);
                    debug_assert!(had_inserted, "duplicate component type in query");
                )*

                bitset.into()
            }

            fn into_order(component_manager: &ComponentManager, other_bitmask: &EntityBitmask) -> ComponentOrder {
                // TODO: use corret size instead of vector
                let mut order = Vec::new();
                let mut iterator = other_bitmask.iter().enumerate();
                $({
                    let current_id = component_manager.get_component_id::<$Q>().unwrap();
                    while let Some((i, id)) = iterator.next() {
                        if current_id == id {
                            order.push(i);
                            break;
                        }
                    }
                })*
                order.into_boxed_slice()
            }

            #[allow(clippy::unused_unit)]
            #[allow(unused_assignments, unused_variables, unused_mut, invalid_value)]
            unsafe fn from_columns<'a>(
                index: usize,
                archetype_order: &ComponentOrder,
                columns: *mut ComponentColumns,
            ) -> Self::ResultType<'a> {
                ($(
                    (*columns).get_mut_from_column::<$Q>(archetype_order[$n], index).unwrap()
                ,)*)
            }
        }
    };
}

variadics_please::all_tuples_enumerated!(impl_query_bundle, 0, 15, B);
