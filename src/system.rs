use std::{
    collections::HashMap,
    fmt::{self, Debug},
};

use crate::{SystemWorldArgs, component::ComponentId, entity::EntityBitmask, query::QueryInfo};

pub(crate) enum SafetyInfo {
    Commands,
    Query(QueryInfo),
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
                    return Err(SystemParamError::new_query_error::<P>(component));
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
        if self.has_commands {
            return Err(SystemParamError::DuplicateCommands);
        }
        self.has_commands = true;
        Ok(())
    }

    pub(crate) fn check<P: SystemParam>(
        &mut self,
        info: SafetyInfo,
    ) -> Result<(), SystemParamError> {
        match info {
            SafetyInfo::Commands => self.check_commands(),
            SafetyInfo::Query(query_info) => self.check_query::<P>(&query_info),
        }
    }
}

pub(crate) trait SystemParam {
    unsafe fn init(args: *mut SystemWorldArgs) -> Self;
    fn safety_info(args: &mut SystemWorldArgs) -> Option<SafetyInfo>;
}

pub trait IntoSystem<T> {
    fn parse(self, args: &mut SystemWorldArgs) -> Result<Box<dyn System>, SystemParamError>;
    /// SAFETY: Calling this function from outside `IntoSystem::parse` might lead to multiple
    /// mutable references to the same value.
    unsafe fn parse_unchecked(self) -> Box<dyn System>;
}

macro_rules! impl_into_system {
    ($($A:ident),*) => {
        impl<F, $($A: SystemParam,)*> IntoSystem<($($A,)*)> for F
        where
            F: Fn($($A,)*) + 'static
        {
            #[allow(unused_variables, unused_mut)]
            fn parse(self, args: &mut SystemWorldArgs) -> Result<Box<dyn System>, SystemParamError> {
                let mut safety_check = SafetyCheck::new();
                $(
                    if let Some(info) = $A::safety_info(args) {
                        safety_check.check::<$A>(info)?;
                    }
                )*
                // SAFETY:
                //     - No two queries may query the same component
                //     - A component queried by a certain query must be
                //         in the restrictions of the others
                //     - No two mutable references to Commands may coexist
                Ok(unsafe {self.parse_unchecked()})
            }

            #[allow(unused_variables)]
            unsafe fn parse_unchecked(self) -> Box<dyn System> {
                Box::new(SystemWrapper::new(move |args: &mut SystemWorldArgs| self($(unsafe {$A::init(args)},)*)))
            }
        }
    };
}

variadics_please::all_tuples!(impl_into_system, 0, 15, A);

pub trait System: 'static {
    fn run(&self, args: &mut SystemWorldArgs);
}

pub(crate) struct SystemWrapper<F: Fn(&mut SystemWorldArgs)> {
    fptr: F,
}

impl<F: Fn(&mut SystemWorldArgs)> SystemWrapper<F> {
    pub(crate) fn new(fptr: F) -> Self {
        Self { fptr }
    }
}

impl<F: Fn(&mut SystemWorldArgs) + 'static> System for SystemWrapper<F> {
    fn run(&self, args: &mut SystemWorldArgs) {
        (self.fptr)(args)
    }
}

#[derive(Default)]
pub(crate) struct SystemsManager {
    systems: Vec<Box<dyn System>>,
}

impl SystemsManager {
    pub(crate) fn add_system<T>(
        &mut self,
        mut args: SystemWorldArgs,
        system: impl IntoSystem<T>,
    ) -> Result<(), SystemParamError> {
        self.systems.push(system.parse(&mut args)?);
        Ok(())
    }

    pub(crate) unsafe fn add_system_unchecked<T>(&mut self, system: impl IntoSystem<T>) {
        unsafe { self.systems.push(system.parse_unchecked()) };
    }

    pub(crate) fn run_all(&self, mut args: SystemWorldArgs) {
        for system in &self.systems {
            system.run(&mut args);
        }

        args.commands
            .apply(args.entity_manager, args.components_manager);
    }
}

pub enum SystemParamError {
    DuplicateCommands,
    MustRestrictQuery {
        query_string: String,
        component_id: ComponentId,
    },
}

impl Debug for SystemParamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateCommands => write!(f, "DuplicateCommands"),
            Self::MustRestrictQuery {
                query_string,
                component_id,
            } => f.write_fmt(format_args!(
                "MustRestrict Error for query {query_string} in component with ID {component_id}",
            )),
        }
    }
}

impl SystemParamError {
    fn new_query_error<Query>(component_id: ComponentId) -> Self {
        Self::MustRestrictQuery {
            query_string: std::any::type_name::<Query>().into(),
            component_id,
        }
    }
}
