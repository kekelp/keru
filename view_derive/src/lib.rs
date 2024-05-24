extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr, Fields, ItemStruct};

#[proc_macro_attribute]
pub fn view(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(item as ItemStruct);
    let name = item_struct.ident.clone();
    
    let expr: Expr = parse_macro_input!(attr as Expr);

    if ! is_zero_sized(&item_struct) {
        return TokenStream::from(quote! {
            compile_error!("Only zero-sized structs should be Views.");
        });
    }

    let expanded = quote! {
        impl View for #name {
            fn defaults(&self) -> NodeParams {
                #expr
            }
        }

        #[derive(Default, Debug)]
        #item_struct
    };

    TokenStream::from(expanded)
}

fn is_zero_sized(item_struct: &ItemStruct) -> bool {
    match &item_struct.fields {
        Fields::Named(fields) => fields.named.is_empty(),
        Fields::Unnamed(fields) => fields.unnamed.is_empty(),
        Fields::Unit => true,
    }
}