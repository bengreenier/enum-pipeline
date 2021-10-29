use std::fmt::Debug;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{Arm, Attribute, Data, DeriveInput};

const HANDLER_ATTR_NAME: &str = "handler";
const ARG_ATTR_NAME: &str = "execute_with";

#[derive(Debug)]
enum Opts<'a> {
    None,
    RefParam(&'a str),
    RefMutParam(&'a str),
}

#[derive(Debug)]
struct IncrementalId {
    from: i32,
}

fn base_derive_macro(input: DeriveInput, opts: Opts) -> TokenStream {
    let enum_ident = input.ident;

    let variants = match input.data {
        Data::Enum(e) => e.variants,
        _ => panic!("Only `enum` types are supported"),
    };

    let arms = variants
        .into_iter()
        .map(|variant| {
            let handler_attrs: Vec<Attribute> = variant
                .attrs
                .into_iter()
                .filter(|attr| matches!(attr.path.get_ident(), Some(ident) if ident == HANDLER_ATTR_NAME))
                .collect();

            if handler_attrs.len() != 1 {
                panic!(
                    "Variant `{}` is missing attribute #[handler(your_handler_function)]",
                    variant.ident
                );
            }

            let handler_attr = &handler_attrs[0];
            let handler_token = handler_attr.tokens.to_string();
            let handler_name = match handler_token[1..handler_token.len() - 1].to_string() {
                s if s.contains("::") => s,
                u => format!("{}::{}", enum_ident.to_string(), u),
            };

            let field_placeholders: Vec<String> = variant
                .fields
                .into_iter()
                .enumerate()
                .map(|(index, field)| match field.ident {
                    Some(ident) => ident.to_string(),
                    None => format!("__{}", index + 1),
                })
                .collect();

            // TODO(bengreenier): This could be cleaned up now that deeper inspection of ident is no longer needed
            let handler_pipeline_arg = match &opts {
                Opts::None => "".to_string(),
                Opts::RefParam(ident) => ident.to_string(),
                Opts::RefMutParam(ident) => ident.to_string(),
            };

            let arm_text = match field_placeholders.len() {
                0 => format!(
                    "{}::{} => {}({})",
                    enum_ident, variant.ident, handler_name, handler_pipeline_arg
                ),
                _ => {
                    let pl = field_placeholders.join(",");
                    let mut pl_with_arg = field_placeholders;
                    pl_with_arg.extend_from_slice(&[handler_pipeline_arg]);

                    format!(
                        "{}::{}({}) => {}({})",
                        enum_ident,
                        variant.ident,
                        pl,
                        handler_name,
                        pl_with_arg.join(",")
                    )
                }
            };

            syn::parse_str::<Arm>(&arm_text).expect("Failed to generate a variant arm")
        })
        .collect::<Vec<Arm>>();

    quote! {
        match self {
            #(#arms),*
        }
    }
}

fn parse_argtype(attrs: &[Attribute], ident: &Ident) -> Ident {
    let arg_type_attrs: Vec<&Attribute> = attrs
        .iter()
        .filter(|attr| matches!(attr.path.get_ident(), Some(ident) if ident == ARG_ATTR_NAME))
        .collect();

    if arg_type_attrs.len() != 1 {
        panic!(
            "Enum `{}` is missing attribute #[argtype(your_arg_type)]",
            ident
        );
    }

    let arg_type_attr = &arg_type_attrs[0];
    let arg_type_token = arg_type_attr.tokens.to_string();
    let arg_type_name = arg_type_token[1..arg_type_token.len() - 1].to_string();

    syn::parse_str::<Ident>(&arg_type_name)
        .unwrap_or_else(|_| panic!("Failed to parse argtype attribute on Enum `{}`", ident))
}

pub fn execute_derive_macro(input: DeriveInput) -> TokenStream {
    let enum_ident = input.ident.clone();
    let matcher = base_derive_macro(input, Opts::None);

    quote! {
        #[automatically_derived]
        impl Execute for #enum_ident {
            fn execute(self) {
                #matcher
            }
        }
    }
}

pub fn execute_with_derive_macro(input: DeriveInput) -> TokenStream {
    let enum_ident = input.ident.clone();
    let arg_type = parse_argtype(&input.attrs, &input.ident);
    let matcher = base_derive_macro(input, Opts::RefParam("args"));

    let arg_type_ts = arg_type.into_token_stream();

    quote! {
        #[automatically_derived]
        impl ExecuteWith<#arg_type_ts> for #enum_ident {
            fn execute_with(self, args: &#arg_type_ts) {
                #matcher
            }
        }
    }
}

pub fn execute_with_mut_derive_macro(input: DeriveInput) -> TokenStream {
    let enum_ident = input.ident.clone();
    let arg_type = parse_argtype(&input.attrs, &input.ident);
    let matcher = base_derive_macro(input, Opts::RefMutParam("args"));

    let arg_type_ts = arg_type.into_token_stream();

    quote! {
        #[automatically_derived]
        impl ExecuteWithMut<#arg_type_ts> for #enum_ident {
            fn execute_with_mut(self, args: &mut #arg_type_ts) {
                #matcher
            }
        }
    }
}
