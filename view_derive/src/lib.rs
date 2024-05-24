extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr, ItemStruct};

#[proc_macro_attribute]
pub fn view(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(item as ItemStruct);
    let name = item_struct.ident.clone();
    
    let expr: Expr = parse_macro_input!(attr as Expr);

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
