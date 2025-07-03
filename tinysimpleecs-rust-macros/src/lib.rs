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

// #[proc_macro]
// pub fn implement_component_bundle(item: TokenStream) -> TokenStream {
//     let input = syn::parse_macro_input!(item with syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated);
//     let len = input.len();
//     let values: Vec<_> = input.into_iter().collect();
//     let n: Vec<_> = values
//         .iter()
//         .enumerate()
//         .map(|(i, _)| syn::Index::from(i))
//         .collect();
//
//     quote! {
//         impl<#(#values: Component),*> ComponentBundle for (#(#values,)*) {
//             fn apply(self, id: EntityId, entity_manager: &mut EntityManager, component_manager: &mut ComponentManager) {
//                 let bitmask = EntityBitmask::default();
//                 let components_btree = BTreeMap::<usize, usize>::new();
//                 #({
//                     let id = component_manager.register_component_if_not_exists::<#values>();
//                     let previous = components_btree.insert(id, #n);
//                     debug_assert!(previous.is_none());
//
//                     bitmask.insert(id);
//                 })*
//
//                 let components_order: HashMap<usize, usize> = components_btree.into_iter().enumerate().map(|(i, (k, v))| (v, i)).collect();
//
//                 let default_columns: [fn() -> AnyVec; #len];
//                 let inserters: [Box<dyn Fn(&mut AnyVec)>; #len];
//                 #({
//                     let current_index = components_order[#n];
//                     default_columns[current_index] = || AnyVec::new::<#values>();
//                     inserters[current_index] = Box::new(|v: &mut AnyVec| v.push(AnyValueWrapper::new(self.#n)));
//                 })*
//
//                 entity_manager.add_entity(id, bitmask, &default_columns, &inserters);
//             }
//         }
//     }
//     .into()
// }
