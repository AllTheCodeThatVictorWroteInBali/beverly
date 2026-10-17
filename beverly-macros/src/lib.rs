use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Attribute, Data, DeriveInput, Fields, Meta, parse_macro_input, parse_quote};
use syn::{Ident, Path, Token, braced, bracketed};

#[proc_macro]
pub fn controller(input: TokenStream) -> TokenStream {
    let controller = parse_macro_input!(input as Controller);
    controller.expand().into()
}

struct Controller {
    name: Ident,
    routes: Punctuated<ControllerRoute, Token![,]>,
}

struct ControllerRoute {
    message: Path,
    action: ControllerAction,
}

enum ControllerAction {
    Handler(Path),
    Series(Vec<ControllerAction>),
    Parallel(Vec<ControllerAction>),
}

impl Parse for Controller {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let routes;
        braced!(routes in input);
        Ok(Self {
            name,
            routes: routes.parse_terminated(ControllerRoute::parse, Token![,])?,
        })
    }
}

impl Parse for ControllerRoute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let message = input.parse()?;
        input.parse::<Token![=>]>()?;
        Ok(Self {
            message,
            action: input.parse()?,
        })
    }
}

impl Parse for ControllerAction {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Bracket) || input.peek(syn::token::Brace) {
            let contents;
            let parallel = input.peek(syn::token::Brace);
            if parallel {
                braced!(contents in input);
            } else {
                bracketed!(contents in input);
            }
            let actions = contents.parse_terminated(Self::parse, Token![,])?;
            if actions.is_empty() {
                return Err(contents.error("controller composition requires at least one action"));
            }
            let actions = actions.into_iter().collect();
            Ok(if parallel {
                Self::Parallel(actions)
            } else {
                Self::Series(actions)
            })
        } else {
            input.parse().map(Self::Handler)
        }
    }
}

impl Controller {
    fn expand(&self) -> proc_macro2::TokenStream {
        let name = &self.name;
        let systems = self.routes.iter().map(|route| {
            let message = &route.message;
            let execute = if let ControllerAction::Handler(handler) = &route.action {
                quote! { let _ = #handler(message); }
            } else {
                let pipeline = route.action.expand(quote! { message });
                quote! {
                    if let Err(error) = #pipeline {
                        ::bevy::log::warn!(
                            controller = stringify!(#name),
                            message = stringify!(#message),
                            error = %error,
                            "controller composition failed",
                        );
                    }
                }
            };
            quote! {
                app.add_systems(::bevy::prelude::Update,
                    |mut messages: ::bevy::prelude::MessageReader<#message>| {
                        for message in messages.read() {
                            #execute
                        }
                    },
                );
            }
        });
        quote! {
            pub struct #name;
            impl ::bevy::prelude::Plugin for #name {
                fn build(&self, app: &mut ::bevy::prelude::App) {
                    #(#systems)*
                }
            }
        }
    }
}

impl ControllerAction {
    fn expand(&self, input: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        match self {
            Self::Handler(handler) => quote! { #handler(#input) },
            Self::Series(actions) => {
                let mut current = input;
                let mut stages = Vec::new();
                for (index, action) in actions.iter().enumerate() {
                    let output = action.expand(current);
                    if index + 1 == actions.len() {
                        return quote! { (|| { #(#stages)* #output })() };
                    }
                    let value = format_ident!("__beverly_stage_{}", index);
                    stages.push(quote! { let #value = #output?; });
                    current = quote! { #value };
                }
                unreachable!("empty compositions are rejected by the parser")
            }
            Self::Parallel(actions) => {
                let inputs: Vec<_> = (0..actions.len())
                    .map(|index| format_ident!("__beverly_input_{}", index))
                    .collect();
                let slots: Vec<_> = (0..actions.len())
                    .map(|index| format_ident!("__beverly_output_{}", index))
                    .collect();
                let values: Vec<_> = (0..actions.len())
                    .map(|index| format_ident!("__beverly_value_{}", index))
                    .collect();
                let copies = inputs.iter().enumerate().map(|(index, name)| {
                    if index + 1 == inputs.len() {
                        quote! { let #name = __beverly_input; }
                    } else {
                        quote! { let #name = ::core::clone::Clone::clone(&__beverly_input); }
                    }
                });
                let tasks =
                    actions
                        .iter()
                        .zip(&inputs)
                        .zip(&slots)
                        .map(|((action, input), slot)| {
                            let execute = action.expand(quote! { #input });
                            quote! {
                                let output = &mut #slot;
                                scope.spawn(async move { *output = Some(#execute); });
                            }
                        });
                let mut joined = quote! { Ok((#(#values,)*)) };
                for (slot, value) in slots.iter().zip(&values).rev() {
                    joined = quote! {
                        #slot.expect("controller branch must finish before joining")
                            .and_then(|#value| #joined)
                    };
                }
                quote! {{
                    let __beverly_input = #input;
                    #(#copies)*
                    #(let mut #slots = None;)*
                    ::bevy::tasks::ComputeTaskPool::get_or_init(::bevy::tasks::TaskPool::new)
                        .scope(|scope| { #(#tasks)* });
                    #joined
                }}
            }
        }
    }
}

#[cfg(test)]
mod controller_tests {
    use super::*;

    #[test]
    fn parses_single_sequential_parallel_and_nested_routes() {
        let controller: Controller = syn::parse_quote! {
            Example {
                Single => handle,
                Sequence => [load, transform, save,],
                FanOut => {notify, index,},
                Combined => [load, { [parse, summarize], index }, save],
            }
        };
        assert_eq!(controller.routes.len(), 4);
        assert!(matches!(
            controller.routes[0].action,
            ControllerAction::Handler(_)
        ));
        assert!(matches!(
            controller.routes[1].action,
            ControllerAction::Series(_)
        ));
        assert!(matches!(
            controller.routes[2].action,
            ControllerAction::Parallel(_)
        ));
        assert!(matches!(
            controller.routes[3].action,
            ControllerAction::Series(_)
        ));
        assert!(!controller.expand().is_empty());
    }

    #[test]
    fn rejects_empty_and_malformed_compositions() {
        for source in [
            "Example { Message => [] }",
            "Example { Message => {} }",
            "Example { Message => [load, {}] }",
            "Example { Message => [load save] }",
        ] {
            assert!(syn::parse_str::<Controller>(source).is_err(), "{source}");
        }
    }
}

#[proc_macro_attribute]
pub fn model(_attributes: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);
    let fields = match &mut input.data {
        Data::Struct(data) => match &mut data.fields {
            Fields::Named(fields) => &mut fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input.ident,
                    "#[model] requires a struct with named fields",
                )
                .to_compile_error()
                .into();
            }
        },
        _ => {
            return syn::Error::new_spanned(
                &input.ident,
                "#[model] can only be applied to a struct",
            )
            .to_compile_error()
            .into();
        }
    };

    let mut descriptors = Vec::new();
    let mut accessors = Vec::new();

    for field in fields {
        let Some(name) = field.ident.clone() else {
            continue;
        };
        let field_type = field.ty.clone();
        let setter = match take_setter(&mut field.attrs) {
            Ok(setter) => setter,
            Err(error) => return error.to_compile_error().into(),
        };
        let getter_fn = format_ident!("__beverly_read_{}", name);
        let setter_fn = format_ident!("__beverly_set_{}", name);
        let accessor_fn = format_ident!("get_{}", name);
        let value_ident = format_ident!("value");

        let setter_body = if let Some(custom_setter) = setter {
            quote! {
                model.#custom_setter(#value_ident)
                    .map_err(|error| ::beverly::primitives::binding::BindingErrors::new(error.to_string()))
            }
        } else {
            quote! {
                model.#name = #value_ident;
                Ok(())
            }
        };

        accessors.push(quote! {
            pub fn #accessor_fn(&self) -> &#field_type {
                &self.#name
            }
        });

        descriptors.push(quote! {
            #[allow(non_upper_case_globals)]
            pub const #name: ::beverly::primitives::binding::FieldBinding<Self, #field_type> =
                ::beverly::primitives::binding::FieldBinding::new(
                    Self::#getter_fn,
                    Self::#setter_fn,
                );

            fn #getter_fn(model: &Self) -> #field_type
            where
                #field_type: Clone,
            {
                model.#name.clone()
            }

            fn #setter_fn(
                model: &mut Self,
                #value_ident: #field_type,
            ) -> Result<(), ::beverly::primitives::binding::BindingErrors> {
                #setter_body
            }
        });
    }

    if !has_resource_derive(&input.attrs) {
        input
            .attrs
            .push(parse_quote!(#[derive(::bevy::prelude::Resource)]));
    }

    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();
    let name = &input.ident;
    quote! {
        #input

        impl #impl_generics #name #type_generics #where_clause {
            #(#accessors)*
            #(#descriptors)*
        }
    }
    .into()
}

fn has_resource_derive(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        if !attribute.path().is_ident("derive") {
            return false;
        }
        attribute
            .parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            )
            .is_ok_and(|paths| {
                paths.iter().any(|path| {
                    path.segments
                        .last()
                        .is_some_and(|segment| segment.ident == "Resource")
                })
            })
    })
}

fn take_setter(attributes: &mut Vec<Attribute>) -> syn::Result<Option<syn::Path>> {
    let mut setter = None;
    let mut error = None;
    attributes.retain(|attribute| {
        if !attribute.path().is_ident("setter") {
            return true;
        }

        match &attribute.meta {
            Meta::NameValue(value) => match syn::parse2(value.value.to_token_stream()) {
                Ok(path) => setter = Some(path),
                Err(parse_error) => error = Some(parse_error),
            },
            _ => {
                error = Some(syn::Error::new_spanned(
                    attribute,
                    "expected #[setter = function_name]",
                ));
            }
        }
        false
    });
    if let Some(error) = error {
        return Err(error);
    }
    Ok(setter)
}
