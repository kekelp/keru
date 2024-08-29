extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use rand::Rng;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    LitStr, Token, Type,
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

// using an attribute macro instead of a derive macro seems to work better with rust-analyzer, for some reason.
// this way, it fully understands the stuff inside the #[derive_view(...)]
// also, a plain proc macro (const KEY: NodeKey = node_key!(PARAMS);) wouldn't have access to the "KEY" ident.
// currently we're putting that into debug_name
#[proc_macro_attribute]
pub fn node_key(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // todo: make sure that attr is empty now
    // let params_expr = parse_macro_input!(attr as Expr);
    let input = parse_macro_input!(item as ItemConstNoEq);

    let key_ident = &input.ident;
    let key_type = &input.ty;

    let debug_name = format!(
        "{} ({}:{}:{})",
        key_ident,
        std::file!(),
        std::line!(),
        std::column!()
    );

    // todo, use a hash of ident instead of a random number?
    let random_number: u64 = rand::thread_rng().gen();
    let random_number_lit = syn::LitInt::new(&format!("{}", random_number), key_ident.span());

    let expanded = quote! {
        pub const #key_ident: #key_type = <#key_type>::new(
            Id(#random_number_lit),
            #debug_name,
        );
    };

    return TokenStream::from(expanded);
}

struct AnonNodeKeyInput {
    key_type: Type,
    base_debug_name: LitStr,
}

impl Parse for AnonNodeKeyInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty = input.parse::<Type>()?;
        input.parse::<Token![,]>()?;
        let debug_name = input.parse::<LitStr>()?;
        return Ok(AnonNodeKeyInput {
            key_type: ty,
            base_debug_name: debug_name,
        });
    }
}

#[proc_macro]
pub fn anon_node_key(input: TokenStream) -> TokenStream {
    // Parse the input token stream into the `Input` struct.
    let input = parse_macro_input!(input as AnonNodeKeyInput);
    let key_type = input.key_type;
    let base_debug_name = input.base_debug_name;

    // Generate a random number for the ID.
    let random_number: u64 = rand::thread_rng().gen();
    let random_number_lit =
        syn::LitInt::new(&format!("{}", random_number), Span::call_site());

    let debug_name = format!(
        "Anon {} ({}:{}:{})",
        base_debug_name.value(),
        std::file!(),
        std::line!(),
        std::column!()
    );
    let debug_name_lit = syn::LitStr::new(&debug_name, Span::call_site());

    let expanded = quote! {
        <#key_type>::new(
            Id(#random_number_lit),
            &#debug_name_lit,
        )
    };

    return TokenStream::from(expanded);
}



#[proc_macro]
pub fn node_key_2(input: TokenStream) -> TokenStream {
    // Parse the input (should be an identifier like SLIDER_FILL)
    let input_ident = parse_macro_input!(input as Ident);
    let ident_str = input_ident.to_string();

    // Generate a random u64 ID
    let id = rand::thread_rng().gen::<u64>();

    // Generate the output token stream
    let expanded = quote! {
        pub const #input_ident: NodeKey = <NodeKey>::new(
            Id(#id),
            #ident_str,
        );
    };

    // Convert into a TokenStream and return
    TokenStream::from(expanded)
}