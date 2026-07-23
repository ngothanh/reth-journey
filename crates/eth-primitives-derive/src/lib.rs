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
        impl ::eth_primitives::SimpleEncode for #struct_name {
            fn encode(&self, out: &mut Vec<u8>) {
                #( ::eth_primitives::SimpleEncode::encode(&self.#field_names, out); )*
            }
        }
    );
    expanded.into()
}

/// Shared input validation for the two RLP derives.
///
/// Both accept exactly one shape — a struct with named fields — and every other shape
/// must fail as a `compile_error!` pinned to the `#[derive(…)]` site. Never `panic!`,
/// never `unwrap`: a macro panic surfaces to the user as a compiler-internal-looking
/// mess with no span, instead of a sentence telling them what to change.
fn named_fields<'a>(
    ast: &'a DeriveInput,
    derive: &str,
) -> Result<&'a syn::punctuated::Punctuated<syn::Field, syn::token::Comma>, TokenStream> {
    let shape = match &ast.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(named) => return Ok(&named.named),
            Fields::Unnamed(_) => "tuple structs",
            Fields::Unit => "unit structs",
        },
        Data::Enum(_) => "enums",
        Data::Union(_) => "unions",
    };

    Err(syn::Error::new_spanned(
        ast,
        format!("{derive} cannot be derived on {shape} — RLP structs must be structs with named fields"),
    )
    .to_compile_error()
    .into())
}

/// Adds `T: <bound>` to every type parameter, so a generic struct's fields can be
/// encoded/decoded. Lifetimes and const params are left alone.
fn bound_type_params(generics: &syn::Generics, bound: syn::TypeParamBound) -> syn::Generics {
    let mut generics = generics.clone();
    for param in &mut generics.params {
        if let syn::GenericParam::Type(ty) = param {
            ty.bounds.push(bound.clone());
        }
    }
    generics
}

#[proc_macro_derive(RlpEncodable)]
pub fn derive_rlp_encode(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;
    let fields = match named_fields(&ast, "RlpEncodable") {
        Ok(fields) => fields,
        Err(err) => return err,
    };

    // Bound to a `Vec` so the repetition can be interpolated more than once below;
    // a bare `Map` iterator would be consumed by the first `#(...)*`.
    let field_names: Vec<_> = fields.iter().map(|f| &f.ident).collect();

    let generics = bound_type_params(&ast.generics, syn::parse_quote!(::eth_rlp::Encodable));
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // A struct is an RLP *list*: a list header whose payload is the concatenated
    // field encodings. Summing `length()` first lets us emit the header before the
    // payload without encoding into a scratch buffer to discover the length.
    let expanded = quote!(
        #[automatically_derived]
        #[doc = concat!(
            "RLP encoding for `", stringify!(#struct_name), "`.\n\n",
            "**Wire format.** Field order and field count are part of the encoding. \
             Reordering, adding, or removing a field changes the bytes on the wire and \
             is a consensus-breaking change, not a refactor."
        )]
        impl #impl_generics ::eth_rlp::Encodable for #struct_name #ty_generics #where_clause {
            fn encode(&self, out: &mut dyn ::eth_rlp::BufMut) {
                let __payload_length = 0usize
                    #( + ::eth_rlp::Encodable::length(&self.#field_names) )*;
                ::eth_rlp::Header { list: true, payload_length: __payload_length }.encode(out);
                #( ::eth_rlp::Encodable::encode(&self.#field_names, out); )*
            }

            fn length(&self) -> usize {
                let __payload_length = 0usize
                    #( + ::eth_rlp::Encodable::length(&self.#field_names) )*;
                ::eth_rlp::Header { list: true, payload_length: __payload_length }.length()
                    + __payload_length
            }
        }
    );
    expanded.into()
}

#[proc_macro_derive(RlpDecodable)]
pub fn derive_rlp_decode(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;
    let fields = match named_fields(&ast, "RlpDecodable") {
        Ok(fields) => fields,
        Err(err) => return err,
    };

    let field_names: Vec<_> = fields.iter().map(|f| &f.ident).collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    let generics = bound_type_params(&ast.generics, syn::parse_quote!(::eth_rlp::Decodable));
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Fields are decoded out of a *slice of the payload*, not out of `buf` directly.
    // That bounds every field decoder to this list's own frame: an inner length that
    // overruns the list cannot reach into sibling data, it just runs out of input.
    // It also makes the trailing-bytes check a plain `is_empty`.
    //
    // Struct-literal fields are evaluated in written order, which is declaration
    // order — the same order `RlpEncodable` writes them.
    let expanded = quote!(
        #[automatically_derived]
        #[doc = concat!(
            "RLP decoding for `", stringify!(#struct_name), "`.\n\n",
            "**Wire format.** Field order and field count are part of the encoding. \
             Reordering, adding, or removing a field changes the bytes on the wire and \
             is a consensus-breaking change, not a refactor."
        )]
        impl #impl_generics ::eth_rlp::Decodable for #struct_name #ty_generics #where_clause {
            fn decode(buf: &mut &[u8]) -> ::core::result::Result<Self, ::eth_rlp::Error> {
                let __header = ::eth_rlp::Header::decode(buf)?;
                if !__header.list {
                    return ::core::result::Result::Err(::eth_rlp::Error::UnexpectedString);
                }
                let __payload_length = __header.payload_length;
                if buf.len() < __payload_length {
                    return ::core::result::Result::Err(::eth_rlp::Error::InputTooShort);
                }
                let mut __payload = &buf[..__payload_length];

                let __value = #struct_name {
                    #( #field_names:
                        <#field_types as ::eth_rlp::Decodable>::decode(&mut __payload)?, )*
                };

                // The header promised exactly `__payload_length` bytes of fields. Anything
                // left over means the frame carried data this struct does not model.
                if !__payload.is_empty() {
                    return ::core::result::Result::Err(::eth_rlp::Error::TrailingBytes);
                }

                *buf = &buf[__payload_length..];
                ::core::result::Result::Ok(__value)
            }
        }
    );
    expanded.into()
}
