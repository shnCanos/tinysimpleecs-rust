use std::{fmt, marker::PhantomData};

use bit_set::BitSet;

use crate::{component::ComponentId, entity::EntityBitmask, query::QueryInfo, World};

pub(crate) trait SystemParam {
    unsafe fn init(world: *mut World) -> Self;
    fn query_info(&self) -> Option<&QueryInfo>;
}

pub(crate) trait IntoSystem<T> {
    fn parse(self) -> Result<Box<dyn System>, SystemParamTupleError>;
}

macro_rules! impl_into_system {
    ($($A:ident),*) => {
        impl<F, $($A: SystemParam,)*> IntoSystem<($($A,)*)> for F
        where
            F: Fn($($A,)*) + 'static
        {
            fn parse(self) -> Result<Box<dyn System>, SystemParamTupleError> {
                // SAFETY:
                //     - No two queries may query the same component
                //     - A component queried by a certain query must be
                //         in the restrictions of the others

                Ok(Box::new(SystemWrapper::new(move |world: &mut World| {
                    let mut consumed_bitmask = BitSet::new();
                    self($({
                        let current = unsafe {$A::init(world)};
                        if let Some(info) = current.query_info() {
                            if let Some(repeated) = info.query_bitmask.intersection(&consumed_bitmask).next() {
                                panic!("Repeated!");
                                // return Err(SystemParamTupleError::new::<$A>(
                                //     repeated,
                                //     SystemParamErrorType::RepeatedComponent
                                // ));
                            }
                            if let Some(difference) = info.restrictions_bitmask.difference(&consumed_bitmask).next() {
                                panic!("Make it different!");
                                // return Err(SystemParamTupleError::new::<$A>(
                                //     difference,
                                //     SystemParamErrorType::MustRestrict
                                // ))
                            }
                            consumed_bitmask.union_with(&info.query_bitmask);
                        }
                        current
                },)*)})))
            }
        }
    };
}

variadics_please::all_tuples!(impl_into_system, 0, 15, A);

enum EcsSystemError {
    SystemParamTupleError(SystemParamTupleError),
}

pub(crate) trait System: 'static {
    fn run(&self, world: &mut World);
}

pub(crate) struct SystemWrapper<F: Fn(&mut World)> {
    fptr: F,
}

impl<F: Fn(&mut World)> SystemWrapper<F> {
    pub(crate) fn new(fptr: F) -> Self {
        Self { fptr }
    }
}

impl<F: Fn(&mut World) + 'static> System for SystemWrapper<F> {
    fn run(&self, world: &mut World) {
        (self.fptr)(world);
    }
}

pub(crate) trait SystemParamTuple: 'static + Sized {
    fn init(world: &mut World) -> Result<Self, SystemParamTupleError>;
}

#[derive(Default)]
pub(crate) struct SystemsManager {
    systems: Vec<Box<dyn System>>,
}

impl SystemsManager {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn add_system<T>(&mut self, system: impl IntoSystem<T>) {
        self.systems.push(system.parse().unwrap());
    }

    pub(crate) fn run_all(&self, world: &mut World) {
        for system in &self.systems {
            system.run(world);
        }
    }
}

pub(crate) struct SystemParamTupleError {
    query_string: String,
    component: ComponentId,
    err: SystemParamErrorType,
}

impl fmt::Debug for SystemParamTupleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error {:?} in query {} for component with ID {}",
            self.err, self.query_string, self.component
        )
    }
}

impl SystemParamTupleError {
    fn new<Query>(component: ComponentId, err: SystemParamErrorType) -> Self {
        Self {
            query_string: std::any::type_name::<Query>().into(),
            component,
            err,
        }
    }
}

#[derive(Debug)]
enum SystemParamErrorType {
    RepeatedComponent,
    MustRestrict,
}
