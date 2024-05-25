extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr, Fields, ItemStruct};
use rand::Rng;

// using an attribute macro instead of a derive macro seems to work better with rust-analyzer, for some reason.
// this way, it fully understands the stuff inside the #[derive_view(...)]
// it also uses one line less, so I guess it's staying this way for now.
#[proc_macro_attribute]
pub fn derive_view(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(item as ItemStruct);
    let name = item_struct.ident.clone();
    
    let expr: Expr = parse_macro_input!(attr as Expr);

    if ! is_zero_sized(&item_struct) {
        return TokenStream::from(quote! {
            compile_error!("Only zero-sized structs should be Views.");
        });
    }

    let random_id: u64 = rand::thread_rng().gen();

    let expanded = quote! {
        #item_struct
        
        impl View for #name {
            fn defaults(&self) -> NodeParams {
                return #expr;
            }

            fn id(&self) -> Id {
                return Id(#random_id);
            }
        }
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