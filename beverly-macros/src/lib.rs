use proc_macro::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{Attribute, Data, DeriveInput, Fields, Meta, parse_macro_input, parse_quote};

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
