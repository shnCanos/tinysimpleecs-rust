use std::{fmt, marker::PhantomData};

use bit_set::BitSet;

use crate::{
    component::ComponentId,
    entity::{EntityBitmask, EntityManager},
    query::QueryInfo,
    SystemWorldArgs, World,
};

pub(crate) trait SystemParam {
    unsafe fn init(args: *mut SystemWorldArgs) -> Self;
    fn query_info(&self) -> Option<&QueryInfo>;
}

pub(crate) trait IntoSystem<T> {
    fn parse(self) -> Result<Box<dyn System>, SystemParamError>;
}

macro_rules! impl_into_system {
    ($($A:ident),*) => {
        impl<F, $($A: SystemParam,)*> IntoSystem<($($A,)*)> for F
        where
            F: Fn($($A,)*) + 'static
        {
            fn parse(self) -> Result<Box<dyn System>, SystemParamError> {
                // SAFETY:
                //     - No two queries may query the same component
                //     - A component queried by a certain query must be
                //         in the restrictions of the others

                Ok(Box::new(SystemWrapper::new(move |args: &mut SystemWorldArgs| -> Result<(), SystemParamError> {
                    let mut consumed_bitmask = BitSet::new();
                    self($({
                        let current = unsafe {$A::init(args)};
                        if let Some(info) = current.query_info() {
                            if let Some(repeated) = info.query_bitmask.intersection(&consumed_bitmask).next() {
                                return Err(SystemParamError::new::<$A>(
                                    repeated,
                                    SystemParamErrorType::RepeatedComponent
                                ));
                            }
                            if let Some(difference) = info.restrictions_bitmask.difference(&consumed_bitmask).next() {
                                return Err(SystemParamError::new::<$A>(
                                    difference,
                                    SystemParamErrorType::MustRestrict
                                ))
                            }
                            consumed_bitmask.union_with(&info.query_bitmask);
                        }
                        current
                },)*); Ok(())})))
            }
        }
    };
}

variadics_please::all_tuples!(impl_into_system, 0, 15, A);

enum EcsSystemError {
    Param(SystemParamError),
}

pub(crate) trait System: 'static {
    fn run(&self, args: &mut SystemWorldArgs) -> Result<(), SystemParamError>;
}

pub(crate) struct SystemWrapper<F: Fn(&mut SystemWorldArgs) -> Result<(), SystemParamError>> {
    fptr: F,
}

impl<F: Fn(&mut SystemWorldArgs) -> Result<(), SystemParamError>> SystemWrapper<F> {
    pub(crate) fn new(fptr: F) -> Self {
        Self { fptr }
    }
}

impl<F: Fn(&mut SystemWorldArgs) -> Result<(), SystemParamError> + 'static> System
    for SystemWrapper<F>
{
    fn run(&self, args: &mut SystemWorldArgs) -> Result<(), SystemParamError> {
        (self.fptr)(args)
    }
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

    pub(crate) fn run_all(&self, mut args: SystemWorldArgs) {
        for system in &self.systems {
            system.run(&mut args).unwrap();
        }

        args.commands
            .apply(args.entity_manager, args.components_manager);
    }
}

pub(crate) struct SystemParamError {
    query_string: String,
    component: ComponentId,
    err: SystemParamErrorType,
}

impl fmt::Debug for SystemParamError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error {:?} in query {} for component with ID {}",
            self.err, self.query_string, self.component
        )
    }
}

impl SystemParamError {
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
