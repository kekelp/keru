extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse::{Parse, ParseStream}, parse_macro_input, spanned::Spanned, Expr, Token, Type};
use rand::Rng;


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
pub fn node_key(attr: TokenStream, item: TokenStream) -> TokenStream {
    // todo: make sure that attr is empty now
    // let params_expr = parse_macro_input!(attr as Expr);
    let input = parse_macro_input!(item as ItemConstNoEq);
    
    let key_ident = &input.ident;
    let key_type = &input.ty;
    let debug_name = format!("{}", key_ident);

    // todo, use a hash of ident instead of a random number?
    let random_number: u64 = rand::thread_rng().gen();
    let random_number_lit = syn::LitInt::new(&format!("{}", random_number), key_ident.span());


    let expanded = quote! {
        pub const #key_ident: #key_type = <#key_type>::new(
            #debug_name,
            Id(#random_number_lit)
        );
    };

    TokenStream::from(expanded)
}



/// Struct to represent the two input parameters: an expression and a type.
struct Input {
    expr: Expr,
    ty: Type,
}

impl Parse for Input {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let expr = input.parse::<Expr>()?;
        input.parse::<Token![,]>()?;
        let ty = input.parse::<Type>()?;
        Ok(Input { expr, ty })
    }
}

#[proc_macro]
pub fn anon_node_key(input: TokenStream) -> TokenStream {
    // Parse the input token stream into the `Input` struct.
    let Input {
        expr: default_params_expr,
        ty,
    } = parse_macro_input!(input as Input);

    // Generate a random number for the ID.
    let random_number: u64 = rand::thread_rng().gen();
    let random_number_lit =
        syn::LitInt::new(&format!("{}", random_number), default_params_expr.span());

    // Generate the expanded code.
    let expanded = quote! {
        {
            {
                // const DEBUG_NAME: &str = &const_format::formatcp!(
                //     "Anon {} ({}:{}:{})",
                //     #default_params_expr.debug_name,
                //     std::file!(),
                //     std::line!(),
                //     std::column!()
                // );
                const DEBUG_NAME: &str = "Nobody cares";

                const PARAMS: NodeParams = #default_params_expr;
                <#ty>::new(
                    &DEBUG_NAME,
                    Id(#random_number_lit),
                )
            }

        }
    };

    // Return the generated code as a TokenStream.
    TokenStream::from(expanded)
}
