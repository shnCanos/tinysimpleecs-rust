use std::{collections::HashMap, fmt, marker::PhantomData};

use bit_set::BitSet;

use crate::{
    component::ComponentId,
    entity::{EntityBitmask, EntityManager},
    query::QueryInfo,
    SystemWorldArgs, World,
};

pub(crate) enum SafetyInfo<'a> {
    Commands,
    Query(&'a QueryInfo),
}

#[derive(Default)]
pub(crate) struct SafetyCheck {
    /// This is a hashmap with a queried component as key and restrictions to its query as value
    /// The usage is pretty simple:
    /// If there's a component being queried two times, then it must have one of its components in
    /// the restrictions
    consumed_bitmasks: HashMap<ComponentId, EntityBitmask>,
    /// There can be only one commands in each query
    has_commands: bool,
}

impl SafetyCheck {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn check_query<P: SystemParam>(
        &mut self,
        info: &QueryInfo,
    ) -> Result<(), SystemParamError> {
        for component in info.query_bitmask.iter() {
            if let Some(restriction) = self.consumed_bitmasks.get_mut(&component) {
                if info
                    .query_bitmask
                    .intersection(restriction)
                    .next()
                    .is_none()
                {
                    return Err(SystemParamError::new::<P>(
                        component,
                        SystemParamErrorType::MustRestrict,
                    ));
                }
                restriction.difference_with(&info.query_bitmask);
            } else {
                self.consumed_bitmasks
                    .insert(component, info.restrictions_bitmask.clone());
            }
        }
        Ok(())
    }

    pub(crate) fn check_commands(&mut self) -> Result<(), SystemParamError> {
        assert!(!self.has_commands);
        self.has_commands = true;
        Ok(())
    }

    pub(crate) fn check<P: SystemParam>(
        &mut self,
        info: SafetyInfo,
    ) -> Result<(), SystemParamError> {
        match info {
            SafetyInfo::Commands => self.check_commands(),
            SafetyInfo::Query(query_info) => self.check_query::<P>(query_info),
        }
    }
}

pub(crate) trait SystemParam {
    unsafe fn init(args: *mut SystemWorldArgs) -> Self;
    fn query_info<'a>(&'a self) -> Option<SafetyInfo<'a>>;
}

pub(crate) trait IntoSystem<T> {
    fn parse(self) -> Result<Box<dyn System>, SystemParamError>;
}

macro_rules! impl_into_system {
    ($($A:ident),*) => {
        impl<'a, F, $($A: SystemParam,)*> IntoSystem<($($A,)*)> for F
        where
            F: Fn($($A,)*) + 'static
        {
            fn parse(self) -> Result<Box<dyn System>, SystemParamError> {
                // SAFETY:
                //     - No two queries may query the same component
                //     - A component queried by a certain query must be
                //         in the restrictions of the others

                Ok(Box::new(SystemWrapper::new(move |args: &mut SystemWorldArgs| -> Result<(), SystemParamError> {
                    let mut safety_check = SafetyCheck::new();
                    self($({
                        let current = unsafe {$A::init(args)};
                        if let Some(info) = current.query_info() {
                            safety_check.check::<$A>(info)?;
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

    pub(crate) fn run_all(&self, mut args: SystemWorldArgs) -> Result<(), SystemParamError> {
        for system in &self.systems {
            system.run(&mut args)?;
        }

        args.commands
            .apply(args.entity_manager, args.components_manager);
        Ok(())
    }
}

pub struct SystemParamError {
    query_string: String,
    component: ComponentId,
    err: SystemParamErrorType,
}

impl fmt::Debug for SystemParamError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:?} in query {} for component with ID {}",
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
    MustRestrict,
}
