use std::any::TypeId;

use tinysimpleecs_rust_macros::implement_bundle;

use crate::entity;

#[derive(Default)]
pub(crate) struct ComponentManger {
    components: Vec<TypeId>,
}

impl ComponentManger {}

struct ComponentType(TypeId);

pub trait Component {}

/// Why create a "Bundle" and an "into_slice" when what they do is equivalent to the code below?
/// ```rust
/// impl Into<Box<[Box<dyn Component>]>> for (/* --- */) {
///     /* defining into */
/// }
/// ```
///
/// Why, simply because it looks better
pub trait Bundle {
    fn into_array(self) -> Box<[Box<dyn Component>]>;
}

variadics_please::all_tuples!(implement_bundle, 1, 15, B);
