use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields};

#[proc_macro_derive(FieldNames)]
pub fn derive_field_names(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;
    let fields = match ast.data {
        syn::Data::Struct(s) => match s.fields {
            Fields::Named(named) => named.named,
            _ => panic!("Only named-field structs"),
        },
        _ => panic!("only structs"),
    };

    let names = fields.iter().map(|f| &f.ident);
    let expanded = quote!(
        impl #struct_name {
            fn field_names() -> Vec<&'static str> {
                vec![ #( stringify!(#names) ),* ]
            }
        }
    );
    expanded.into()
}
