extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

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
            ::std::boxed::Box::new(self.#idx),
        }
    });
    let full = quote! {
        impl<#(#values: Component + 'static),*> Bundle for (#(#values,)*) {
            fn into_array(self) -> Box<[Box<dyn Component>]> {
                Box::new([
                    #(#implementation)*
                ])
            }
        }
    };
    full.into()
}
