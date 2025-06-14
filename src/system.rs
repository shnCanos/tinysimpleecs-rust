use std::fmt;

use bit_set::BitSet;

use crate::{component::ComponentId, entity::EntityBitmask, query::QueryInfo, World};

pub(crate) trait SystemArg {
    unsafe fn init(world: *mut World) -> Self;
    fn query_info(&self) -> Option<&QueryInfo>;
}

pub(crate) trait System: 'static {
    fn run(&self, world: &mut World) -> Result<(), SystemRunError>;
}

#[derive(Default)]
pub(crate) struct SystemsManager {
    systems: Vec<Box<dyn System>>,
}

impl SystemsManager {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn add_system(&mut self, system: impl System) {
        self.systems.push(Box::new(system));
    }

    pub(crate) fn add_systems(&mut self, systems: impl SystemBundle) {
        systems.add_to(self);
    }

    pub(crate) fn run_all(&self, world: &mut World) {
        for system in &self.systems {
            system.run(world).unwrap();
        }
    }
}

pub(crate) struct SystemRunError {
    query_string: String,
    component: ComponentId,
    err: SystemRunErrorType,
}

impl fmt::Debug for SystemRunError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error {:?} in query {} for component with ID {}",
            self.err, self.query_string, self.component
        )
    }
}

impl SystemRunError {
    fn new<Query>(component: ComponentId, err: SystemRunErrorType) -> Self {
        Self {
            query_string: std::any::type_name::<Query>().into(),
            component,
            err,
        }
    }
}

#[derive(Debug)]
enum SystemRunErrorType {
    RepeatedComponent,
    MustRestrict,
}

macro_rules! impl_system_fnptr {
    ($(($n:tt, $S:ident)),*) => {
        impl<$($S: SystemArg + 'static),*> System for fn($($S,)*) {
            #[allow(unused_variables, unused_mut)]
            fn run(&self, world: &mut World) -> Result<(), SystemRunError> {
                // SAFETY:
                //     - No two queries may query the same component
                //     - A component queried by a certain query must be
                //         in the restrictions of the others
                let mut consumed_bitmask = BitSet::new();
                self($({
                    let current = unsafe {$S::init(world)};
                    if let Some(info) = current.query_info() {
                        if let Some(repeated) = info.query_bitmask.union(&consumed_bitmask).next() {
                            return Err(SystemRunError::new::<$S>(
                                repeated,
                                SystemRunErrorType::RepeatedComponent
                            ));
                        }
                        if let Some(difference) = info.query_bitmask.difference(&consumed_bitmask).next() {
                            return Err(SystemRunError::new::<$S>(
                                difference,
                                SystemRunErrorType::MustRestrict
                            ))
                        }
                        consumed_bitmask.union_with(&info.query_bitmask);
                    }
                    current
                },)*);
                Ok(())
            }
        }

        // If the function is unsafe, ignore the safety checks
        impl<$($S: SystemArg + 'static),*> System for unsafe fn($($S,)*) {
            #[allow(unused_variables)]
            fn run(&self, world: &mut World) -> Result<(), SystemRunError> {
                unsafe {self($($S::init(world),)*)};
                Ok(())
            }
        }
    };
}

variadics_please::all_tuples_enumerated!(impl_system_fnptr, 0, 15, S);

pub(crate) trait SystemBundle {
    fn add_to(self, systems_manager: &mut SystemsManager);
}

macro_rules! impl_system_bundle {
    ($(($n:tt, $S:ident)),*) => {
        impl<$($S: System),*> SystemBundle for ($($S,)*) {
            fn add_to(self, systems_manager: &mut SystemsManager) {
                $(systems_manager.add_system(self.$n);)*
            }
        }
    };
}

variadics_please::all_tuples_enumerated!(impl_system_bundle, 1, 15, S);
