extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse::{Parse, ParseStream}, parse_macro_input, Expr, Token, Type};
use rand::Rng;

// using an attribute macro instead of a derive macro seems to work better with rust-analyzer, for some reason.
// this way, it fully understands the stuff inside the #[derive_view(...)]
#[proc_macro_attribute]
pub fn node_key(attr: TokenStream, item: TokenStream) -> TokenStream {
    let default_params_expr = parse_macro_input!(attr as Expr);
    let input = parse_macro_input!(item as ItemConstNoEq);
    
    let key_ident = &input.ident;

    let random_number: u64 = rand::thread_rng().gen();
    let random_number_ident = syn::LitInt::new(&format!("{}", random_number), key_ident.span());

    let expanded = quote! {
        const #key_ident: NodeKey = NodeKey::new( &#default_params_expr, Id(#random_number_ident));
    };

    TokenStream::from(expanded)
}

struct ItemConstNoEq {
    _attrs: Vec<syn::Attribute>,
    _vis: syn::Visibility,
    _const_token: Token![const],
    ident: Ident,
    _colon_token: Token![:],
    _ty: Box<Type>,
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
            _ty: input.parse()?,
            // todo, check that the type is right?
            // but it could be renamed (use NodeKey as SomethingElse)
            _semi_token: input.parse()?,
        })
    }
}