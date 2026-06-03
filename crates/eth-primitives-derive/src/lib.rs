use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

#[proc_macro_derive(FieldNames)]
pub fn derive_field_names(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;
    let fields = match ast.data {
        Data::Struct(s) => match s.fields {
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

#[proc_macro_derive(SimpleEncode)]
pub fn derive_simple_encode(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;
    let fields = match &ast.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(s) => &s.named,
            Fields::Unnamed(_) => {
                return syn::Error::new_spanned(
                    &ast,
                    "SimpleEncode does not support Unnamed fields",
                )
                .to_compile_error()
                .into()
            }

            Fields::Unit => {
                return syn::Error::new_spanned(&ast, "SimpleEncode does not support Unit fields")
                    .to_compile_error()
                    .into()
            }
        },
        Data::Enum(_) => {
            return syn::Error::new_spanned(&ast, "SimpleEncode is not supported for enums")
                .to_compile_error()
                .into()
        }
        Data::Union(_) => {
            return syn::Error::new_spanned(&ast, "SimpleEncode is not supported for Unions")
                .to_compile_error()
                .into()
        }
    };
    let field_names = fields.iter().map(|f| &f.ident);
    let expanded = quote!(
        impl SimpleEncode for #struct_name {
            fn encode(&self, out: &mut Vec<u8>) {
                #( self.#field_names.encode(out); )*
            }
        }
    );
    expanded.into()
}
