extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse::{Parse, ParseStream}, parse_macro_input, spanned::Spanned, Block, DeriveInput, Expr, ExprBlock, Fields, ItemConst, ItemStruct, LitStr, Token, Type};
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



#[proc_macro]
pub fn addproc(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as syn::Expr);

    // Extract the components
    let (ui_expr, node_key_expr, code_block) = match input {
        Expr::Tuple(ref tuple) if tuple.elems.len() == 3 => {
            let ui_expr = &tuple.elems[0];
            let node_key_expr = &tuple.elems[1];
            let code_block = match &tuple.elems[2] {
                Expr::Block(ExprBlock { block, .. }) => block,
                _ => panic!("Expected a block of code"),
            };
            (ui_expr, node_key_expr, code_block)
        }
        _ => panic!("Expected three arguments: ui expression, node key expression, and a block of code"),
    };

    // Generate the new code
    let expanded = quote! {
        {
            #ui_expr.add(#node_key_expr);
            #ui_expr.start_layer(#node_key_expr.id());
            #code_block
            #ui_expr.end_layer();
        }
    };

    // Convert the generated code back into a TokenStream and return it
    TokenStream::from(expanded)
}



struct AddMacroInput {
    ui_expr: Expr,
    node_key_expr: Expr,
    code_block: Block,
}

impl Parse for AddMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ui_expr: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let node_key_expr: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let code_block: Block = input.parse()?;

        Ok(AddMacroInput {
            ui_expr,
            node_key_expr,
            code_block,
        })
    }
}

#[proc_macro]
pub fn add_anon(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let AddMacroInput { ui_expr, node_key_expr, code_block } = parse_macro_input!(input as AddMacroInput);

    // Generate a random number for the struct name
    let random_number: u32 = rand::thread_rng().gen_range(1000000..9999999);

    // Create the struct name with the random number
    let struct_name = syn::Ident::new(&format!("Anon{}", random_number), proc_macro2::Span::call_site());

    // Generate the new code
    let expanded = quote! {
        // define an ad-hoc view thing
        #[derive_view(#node_key_expr)]
        pub struct #struct_name;

        #ui_expr.add(#struct_name);
        #ui_expr.start_layer(#struct_name.id());
        #code_block
        #ui_expr.end_layer();
    };

    // Convert the generated code back into a TokenStream and return it
    TokenStream::from(expanded)
}


#[proc_macro_attribute]
pub fn derive_key2(attr: TokenStream, item: TokenStream) -> TokenStream {
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
        const #ident: NodeKey = NodeKey {
            params: &#default_params_expr,
            id: Id(#random_number_ident),
        };
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