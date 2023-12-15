use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, Data, DataStruct, DeriveInput, Fields, GenericParam,
    Ident, Token, Type,
};

#[proc_macro_derive(WithFields, attributes(with_fields))]
pub fn with_fields(tokens: TokenStream) -> TokenStream {
    inline_impl(
        tokens,
        "WithFields",
        "with_fields",
        "with_",
        |func_name, ident, ty| {
            quote! {
                pub fn #func_name<WithFieldsInto: Into<#ty>>(mut self, #ident: WithFieldsInto) -> Self {
                    self.#ident = #ident.into();
                    self
                }
            }
        },
    )
}

#[proc_macro_derive(SetFields, attributes(set_fields))]
pub fn set_fields(tokens: TokenStream) -> TokenStream {
    inline_impl(
        tokens,
        "SetFields",
        "set_fields",
        "set_",
        |func_name, ident, ty| {
            quote! {
                pub fn #func_name<SetFieldsInto: Into<#ty>>(&mut self, #ident: SetFieldsInto) -> &mut Self {
                    self.#ident = #ident.into();
                    self
                }
            }
        },
    )
}

#[proc_macro_derive(MapFields, attributes(map_fields))]
pub fn map_fields(tokens: TokenStream) -> TokenStream {
    inline_impl(
        tokens,
        "MapFields",
        "map_fields",
        "map_",
        |func_name, ident, ty| {
            quote! {
                pub fn #func_name (mut self, f: impl FnOnce(#ty) -> #ty) -> Self {
                    self.#ident = f(self.#ident);
                    self
                }
            }
        },
    )
}

fn inline_impl(
    tokens: TokenStream,
    derive: &str,
    attribute: &str,
    method_prefix: &str,
    token_fn: impl Fn(&Ident, &Ident, &Type) -> TokenStream2,
) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);

    let ident = input.ident;
    let generics = input.generics;

    let fields_punct = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields.named,
        _ => panic!(
            "{} may only be derived for structs with named fields",
            derive
        ),
    };

    let functions = fields_punct.iter().flat_map(|field| {
        if field.attrs.iter().any(|attr| {
            if attr.path.segments[0].ident == attribute {
                if attr.tokens.to_string() == "(ignore)" {
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }) {
            return None;
        }
        let ident = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        let func_name = method_prefix.to_owned() + &ident.to_string();
        let func_name = Ident::new(&func_name, ident.span());

        let tokens = token_fn(&func_name, ident, ty);

        Some(tokens)
    });

    let generic_params = generics.params;

    let generic_types = generic_params
        .iter()
        .flat_map(|param| {
            if let GenericParam::Type(ty) = param {
                Some(ty.ident.clone())
            } else {
                None
            }
        })
        .collect::<Punctuated<Ident, Token![,]>>();

    let where_clause = generics.where_clause.clone();

    let modified = quote! {
        impl < #generic_params > #ident < #generic_types > #where_clause {
            #(#functions)*
        }
    };

    TokenStream::from(modified)
}
