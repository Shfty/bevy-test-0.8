use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Parse, parse_macro_input, Token, Type, TypeTuple};

struct Input {
    tys: Vec<Type>,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut tys = vec![];

        while let Ok(ty) = input.parse::<Type>() {
            tys.push(ty);
            input.parse::<Token![,]>().ok();
        }

        Ok(Input { tys })
    }
}

#[proc_macro]
pub fn vertex(input: TokenStream) -> TokenStream {
    let tys = parse_macro_input!(input as Input);

    let generics = tys
        .tys
        .iter()
        .flat_map(|ty| {
            match ty {
                Type::Tuple(TypeTuple {
                    paren_token: syn::token::Paren { .. },
                    elems,
                }) if elems.len() == 0 => return None,
                _ => (),
            }

            Some(ty)
        })
        .collect::<Vec<_>>();

    let idx = generics.len();
    let indices = (0..idx).collect::<Vec<_>>();

    let mut tokens = vec![];

    tokens.push(quote! {
        impl<F, R, #(#generics),*> crate::prelude::Edges for Function<#idx, F, (#(#generics),*), R>
        where
            F: 'static + Send + Sync + Fn(#(#generics),*) -> R,
            #(
            #generics: 'static + Send + Sync,
            )*
            R: 'static + Send + Sync,
        {
            type Inputs: = crate::Cons![
                #(crate::prelude::Input<#indices, #generics>),*
            ];
            type Outputs = crate::Cons![(Self, crate::prelude::Out<R>)];
        }
    });

    for i in 0..idx {
        let generic = generics[i];
        tokens.push(quote! {
            impl<'a, T, R, #(#generics),*> crate::prelude::VertexInput<crate::prelude::Input<#i, #generic>> for Function<#idx, T, (#(#generics),*), R>
            where
                T: 'static + Send + Sync + Fn(#(#generics),*) -> R,
                #(
                #generics: 'static + Send + Sync,
                )*
                R: 'static + Send + Sync,
            {
                type Type = R;
            }
        });
    }

    let input_calls = generics
        .iter()
        .enumerate()
        .map(|(i, generic)| {
            quote! {
                let #generic =  <crate::prelude::Input::<#i, #generic> as crate::prelude::EvaluateInEdge>::evaluate_in(world, entity);
            }
        })
        .collect::<Vec<_>>();

    tokens.push(quote! {
        #[allow(non_snake_case)]
        impl<T, R, #(#generics),*> crate::prelude::VertexOutput<crate::prelude::Out<R>> for Function<#idx, T, (#(#generics),*), R> where
            T: 'static + Send + Sync + Fn(#(#generics),*) -> R,
            #(
            #generics: 'static + Send + Sync,
            )*
            R: 'static + Send + Sync,
        {
            type Context = bevy::prelude::World;
            type Key = bevy::prelude::Entity;
            type Type = R;

            fn evaluate(world: &bevy::prelude::World, entity: bevy::prelude::Entity) -> R {
                bevy::prelude::debug!(
                    "Evaluate {} for {:?}",
                    std::any::type_name::<Self>(),
                    entity
                );

                #(
                    #input_calls
                )*

                let f = world
                    .get::<Function<#idx, T, (#(#generics),*), R>>(entity)
                    .expect("Invalid Function Vertex");

                f(#(#generics),*)
            }
        }
    });

    let output = quote! {
        #(#tokens)*
    };

    output.into()
}
