extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parse;

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
pub fn implement_bundle(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item with syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated);
    let len = input.len();
    let values: Vec<_> = input.into_iter().collect();
    let add_implementations = (0..len).map(|i| {
        let idx = syn::Index::from(i);
        let value = &values[i];
        quote! {
            let (id, component_index) = manager.add_component::<#value>(entity, self.#idx);

            component_indexes[current_index].write(component_index);
            current_index += 1;

            let had_inserted = bitset.insert(id);
            debug_assert!(had_inserted, "Only one of each component type per entity allowed");
        }
    });

    let into_bitmask_implementations = values.iter().map(|type_name| {
        quote! {
            if let Some(id) = component_manager.get_component_id::<#type_name>() {
                let had_inserted = bitset.insert(id);
                debug_assert!(had_inserted, "Only one of each component type per entity allowed");
            }
            // else { do nothing, components are added dynamically }
        }
    });

    let full = quote! {
        impl<#(#values: Component),*> crate::Bundle for (#(#values,)*) {
            fn add(self, entity: crate::entity::EntityId, manager: &mut ComponentManager) -> crate::entity::EntityInfo {
                let mut bitset = ::bit_set::BitSet::new();

                let mut component_indexes = ::std::boxed::Box::<[usize]>::new_uninit_slice(#len);
                let mut current_index = 0;

                #(#add_implementations)*

                crate::entity::EntityInfo::new(entity, bitset.into(), unsafe {component_indexes.assume_init()})
            }

            fn into_bitmask(component_manager: &mut ComponentManager) -> EntityBitmask {
                let mut bitset = ::bit_set::BitSet::new();

                #(#into_bitmask_implementations)*

                bitset.into()
            }
        }
    };
    full.into()
}
