extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use rand::Rng;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Token, Type,
};

struct ItemConstNoEq {
    _attrs: Vec<syn::Attribute>,
    _vis: syn::Visibility,
    _const_token: Token![const],
    ident: Ident,
    _colon_token: Token![:],
    ty: Type,
    _semi_token: Token![;],
}

impl Parse for ItemConstNoEq {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(ItemConstNoEq {
            _attrs: input.call(syn::Attribute::parse_outer)?,
            _vis: input.parse()?,
            _const_token: input.parse()?,
            ident: input.parse()?,
            _colon_token: input.parse()?,
            ty: input.parse()?,
            // todo, check that the type is right?
            // but it could be renamed (use NodeKey as SomethingElse)
            _semi_token: input.parse()?,
        })
    }
}

/// A macro that creates an unique [`NodeKey`](../keru/node_key/struct.NodeKey.html).
/// 
/// The `id` field is obtained by rolling a random `u64` at compile time, while `debug_name` is obtained from the identifier.
/// 
/// ### Example
/// 
/// ```rust
/// # use keru::*;
/// #[node_key] const UNIQUE_KEY: NodeKey;
/// ```
/// 
/// ### Expands To
/// 
/// ```rust
/// # use keru::*;
/// const INCREASE: UNIQUE_KEY = <NodeKey>::new(keru::Id(13624446487038443998u64), "UNIQUE_KEY");
/// ```
/// 
#[proc_macro_attribute]
pub fn node_key(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // todo: make sure that attr is empty now
    let input = parse_macro_input!(item as ItemConstNoEq);

    let key_ident = &input.ident;
    let key_type = &input.ty;

    let debug_name = format!("{}", key_ident);

    // todo, use a hash of ident instead of a random number?
    let random_id: u64 = rand::thread_rng().gen();

    let expanded = quote! {
        pub const #key_ident: #key_type = <#key_type>::new(
            keru::Id(#random_id),
            #debug_name,
        );
    };

    return TokenStream::from(expanded);
}
