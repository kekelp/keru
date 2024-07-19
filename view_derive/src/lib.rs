extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse::{Parse, ParseStream}, parse_macro_input, spanned::Spanned, Block, DeriveInput, Expr, ExprBlock, Fields, ItemConst, ItemStruct, LitStr, Token, Type};
use rand::Rng;

// using an attribute macro instead of a derive macro seems to work better with rust-analyzer, for some reason.
// this way, it fully understands the stuff inside the #[derive_view(...)]
#[proc_macro_attribute]
pub fn node_key(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute and the item
    let default_params_expr = parse_macro_input!(attr as Expr);
    let input = parse_macro_input!(item as ItemConstNoEq);
    
    // Extract the identifier of the original constant
    let ident = &input.ident;

    // Generate a random number
    let random_number: u64 = rand::thread_rng().gen();

    // Generate a unique identifier for the params constant
    let random_number_ident = syn::LitInt::new(&format!("{}", random_number), ident.span());

    // Generate the output tokens
    let expanded = quote! {
        const #ident: NodeKey = NodeKey::new( &#default_params_expr, Id(#random_number_ident));
    };

    TokenStream::from(expanded)
}

// Define a struct to represent the custom parsed constant item
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