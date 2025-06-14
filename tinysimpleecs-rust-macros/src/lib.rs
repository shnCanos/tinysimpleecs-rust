extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_derive(Component)]
pub fn derive_component(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);
    let ident = &input.ident;
    let component_impl = quote! {
        impl Component for #ident {}
    };

    component_impl.into()
}

#[proc_macro]
pub fn implement_component_bundle(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item with syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated);
    let len = input.len();
    let values: Vec<_> = input.into_iter().collect();
    let add_implementations = values.iter().enumerate().map(|(i, value)| {
        let idx = syn::Index::from(i);
        quote! {
            let (id, component_index) = manager.add_component::<#value>(entity, self.#idx);

            component_indexes[current_index].write(component_index);
            current_index += 1;

            let had_inserted = bitset.insert(id);
            debug_assert!(had_inserted, "duplicate component type in entity");
        }
    });

    quote! {
        impl<#(#values: crate::component::Component),*> crate::component::ComponentBundle for (#(#values,)*) {
            fn add(self, entity: crate::entity::EntityId, manager: &mut ComponentManager) -> crate::entity::EntityInfo {
                let mut bitset = ::bit_set::BitSet::new();

                let mut component_indexes = ::std::boxed::Box::<[usize]>::new_uninit_slice(#len);
                let mut current_index = 0;

                #(#add_implementations)*

                crate::entity::EntityInfo::new(entity, bitset.into(), unsafe {component_indexes.assume_init()})
            }
        }
    }.into()
}

#[proc_macro]
pub fn implement_query_bundle(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item with syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated);
    let len = input.len();
    let values: Vec<_> = input.into_iter().collect();
    let idx: Vec<_> = (0..values.len()).map(syn::Index::from).collect();

    quote! {
        impl<#(#values: crate::component::Component),*> crate::query::QueryBundle for (#(#values,)*) {
            type ResultType<'a> = (#(&'a mut #values,)*);
            fn into_bitmask(component_manager: &mut ComponentManager) -> (EntityBitmask, ComponentOrder) {
                let mut bitset = ::bit_set::BitSet::new();
                let mut order = ::std::collections::HashMap::with_capacity(#len);
                let mut current_index = 0;

                #(
                    if let Some(id) = component_manager.get_component_id::<#values>() {
                        let had_inserted = bitset.insert(id);
                        debug_assert!(had_inserted, "duplicate component type in query");

                        order.insert(id, current_index);
                        current_index += 1;
                    }
                    // else { do nothing, components are added dynamically }
                )*

                (bitset.into(), order)
            }

            #[allow(clippy::unused_unit)]
            unsafe fn from_indexes<'a>(
                order: &ComponentOrder,
                indexes: &[usize],
                component_manager: *mut ComponentManager,
            ) -> Self::ResultType<'a> {
                let mut newtuple: Self::ResultType<'a> = unsafe { ::std::mem::MaybeUninit::uninit().assume_init() };
                #({         
                    let current_id = (*component_manager).get_component_id::<#values>().unwrap();
                    newtuple.#idx = (*component_manager).get_from_index::<#values>(indexes[order[&current_id]]).unwrap();
                })*
                newtuple
            }
        }
    }.into()
}
