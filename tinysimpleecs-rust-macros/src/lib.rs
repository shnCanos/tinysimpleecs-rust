extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parse;

#[proc_macro_derive(Component)]
pub fn derive_into_hash_map(item: TokenStream) -> TokenStream {
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
    let values: Vec<_> = input.into_iter().collect();
    let implementation = (0..values.len()).map(|i| {
        let idx = syn::Index::from(i);
        quote! {
            ::std::rc::Rc::new(::std::cell::RefCell::new(self.#idx)),
        }
    });
    let full = quote! {
        impl<#(#values: Component + 'static),*> Bundle for (#(#values,)*) {
            fn into_array(self) -> ::std::boxed::Box<[::std::rc::Rc<::std::cell::RefCell<dyn Component>>]> {
                ::std::boxed::Box::new([
                    #(#implementation)*
                ])
            }
        }
    };
    full.into()
}

struct QueryTypeMaker {
    min: usize,
    max: usize,
    query_trait: syn::Type,
}

impl Parse for QueryTypeMaker {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let min = input.parse::<syn::LitInt>()?;
        input.parse::<syn::Token![,]>().unwrap();
        let max = input.parse::<syn::LitInt>()?;
        input.parse::<syn::Token![,]>().unwrap();
        let query_type = input.parse::<syn::Type>()?;

        Ok(Self {
            min: min.base10_parse().unwrap(),
            max: max.base10_parse().unwrap(),
            query_trait: query_type,
        })
    }
}

#[proc_macro]
pub fn create_query_type(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as QueryTypeMaker);
    let query_trait = input.query_trait;

    let mut joint = Vec::with_capacity(input.max - input.min);
    for current in input.min..=input.max {
        let queries: Vec<_> = (0..current).map(|_| quote! {TypeId}).collect();
        let queries_n: Vec<_> = (0..current).map(syn::Index::from).collect();

        joint.push(quote! {
            impl #query_trait for (#(#queries,)*) {
                fn into_bitmask(self, components_manager: &component::ComponentManger) -> EntityBitmask {
                    let mut bitset = BitSet::new();

                    #(
                    if let Some(id) = components_manager.get_component_id(self.#queries_n) {
                        bitset.insert(*id);
                    } 
                    // else {
                    // Do nothing. The components are added dynamically
                    // }
                    )*

                    EntityBitmask::new(bitset)
                }
            }
        });
    }

    quote! {
        #(#joint)*
    }
    .into()
}
