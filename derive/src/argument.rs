use std::ops::RangeInclusive;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Fields, FieldsUnnamed, Ident, Lit, Meta, Variant};

use crate::{
    attributes::{parse_argument_attribute, ArgAttr, ArgumentsAttr},
    flags::{Flags, Value},
};

pub(crate) struct Argument {
    pub(crate) ident: Ident,
    pub(crate) name: String,
    pub(crate) arg_type: ArgType,
    pub(crate) help: String,
}

pub(crate) enum ArgType {
    Option {
        flags: Flags,
        hidden: bool,
        takes_value: bool,
        default: TokenStream,
    },
    Positional {
        num_args: RangeInclusive<usize>,
        last: bool,
    },
}

pub(crate) fn parse_arguments_attr(attrs: &[Attribute]) -> ArgumentsAttr {
    for attr in attrs {
        if attr.path.is_ident("arguments") {
            return ArgumentsAttr::parse(attr);
        }
    }
    ArgumentsAttr::default()
}

pub(crate) fn parse_argument(v: Variant) -> Option<Argument> {
    let ident = v.ident;
    let name = ident.to_string();
    let attribute = get_arg_attribute(&v.attrs)?;
    let help = collect_help(&v.attrs);

    let field = match v.fields {
        Fields::Unit => None,
        Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
            let v: Vec<_> = unnamed.iter().collect();
            assert!(
                v.len() == 1,
                "Variants in an Arguments enum can have at most 1 field."
            );
            Some(v[0].ty.clone())
        }
        Fields::Named(_) => {
            panic!("Named fields are not supported in Arguments");
        }
    };

    let arg_type = match attribute {
        ArgAttr::Option(opt) => {
            let default_expr = match opt.default {
                Some(expr) => quote!(#expr),
                None => quote!(Default::default()),
            };
            ArgType::Option {
                flags: opt.flags,
                takes_value: field.is_some(),
                default: default_expr,
                hidden: opt.hidden,
            }
        }
        ArgAttr::Positional(pos) => {
            assert!(field.is_some(), "Positional arguments must have a field");
            ArgType::Positional {
                num_args: pos.num_args,
                last: pos.last,
            }
        }
    };

    Some(Argument {
        ident,
        name,
        arg_type,
        help,
    })
}

fn collect_help(attrs: &[Attribute]) -> String {
    let mut help = Vec::new();
    for attr in attrs {
        let Ok(meta) = attr.parse_meta() else { continue; };
        let Meta::NameValue(name_value) = meta else { continue; };
        if !name_value.path.is_ident("doc") {
            continue;
        }
        let Lit::Str(litstr) = name_value.lit else { continue; };
        help.push(litstr.value().trim().to_string())
    }
    help.join("\n")
}

fn get_arg_attribute(attrs: &[Attribute]) -> Option<ArgAttr> {
    let attrs: Vec<_> = attrs
        .iter()
        .filter(|a| a.path.is_ident("option") || a.path.is_ident("positional"))
        .collect();
    match attrs[..] {
        [] => None,
        [attr] => Some(parse_argument_attribute(attr)),
        _ => panic!("Can only specify one #[option] or #[positional] per argument variant"),
    }
}

pub(crate) fn short_handling(args: &[Argument]) -> TokenStream {
    let mut match_arms = Vec::new();

    for arg in args {
        let (flags, takes_value, default) = match arg.arg_type {
            ArgType::Option {
                ref flags,
                takes_value,
                ref default,
                hidden: _,
            } => (flags, takes_value, default),
            ArgType::Positional { .. } => continue,
        };

        if flags.short.is_empty() {
            continue;
        }

        for flag in &flags.short {
            let pat = flag.flag;
            let expr = match (&flag.value, takes_value) {
                (Value::No, false) => no_value_expression(&arg.ident),
                (_, false) => {
                    panic!("Option cannot take a value if the variant doesn't have a field")
                }
                (Value::No, true) => default_value_expression(&arg.ident, default),
                (Value::Optional(_), true) => optional_value_expression(&arg.ident, default),
                (Value::Required(_), true) => required_value_expression(&arg.ident),
            };
            match_arms.push(quote!(#pat => { #expr }))
        }
    }

    quote!(
        let option = format!("-{}", short);
        match short {
            #(#match_arms)*
            _ => return Err(arg.unexpected().into()),
        }
    )
}

pub(crate) fn long_handling(args: &[Argument], help_flags: &Flags) -> TokenStream {
    let mut match_arms = Vec::new();
    let mut options = Vec::new();

    options.extend(help_flags.long.iter().map(|f| f.flag.clone()));

    for arg in args {
        let (flags, takes_value, default) = match &arg.arg_type {
            ArgType::Option {
                flags,
                takes_value,
                ref default,
                hidden: _,
            } => (flags, takes_value, default),
            ArgType::Positional { .. } => continue,
        };

        if flags.long.is_empty() {
            continue;
        }

        for flag in &flags.long {
            let pat = &flag.flag;
            let expr = match (&flag.value, takes_value) {
                (Value::No, false) => no_value_expression(&arg.ident),
                (_, false) => {
                    panic!("Option cannot take a value if the variant doesn't have a field")
                }
                (Value::No, true) => default_value_expression(&arg.ident, default),
                (Value::Optional(_), true) => optional_value_expression(&arg.ident, default),
                (Value::Required(_), true) => required_value_expression(&arg.ident),
            };
            match_arms.push(quote!(#pat => { #expr }));
            options.push(flag.flag.clone());
        }
    }

    if options.is_empty() {
        return quote!(return Err(arg.unexpected().into()));
    }

    // TODO: Add version check
    let help_check = if !help_flags.long.is_empty() {
        let long_help_flags = help_flags.long.iter().map(|f| &f.flag);
        quote!(if let #(#long_help_flags)|* = long {
            return Ok(Some(Argument::Help));
        })
    } else {
        quote!()
    };

    let num_opts = options.len();

    quote!(
        let long_options: [&str; #num_opts] = [#(#options),*];
        let mut candidates = Vec::new();
        let mut exact_match = None;
        for opt in long_options {
            if opt == long {
                exact_match = Some(opt);
                break;
            } else if opt.starts_with(long) {
                candidates.push(opt);
            }
        }

        let long = match (exact_match, &candidates[..]) {
            (Some(opt), _) => opt,
            (None, [opt]) => opt,
            (None, []) => return Err(arg.unexpected().into()),
            (None, opts) => return Err(Error::AmbiguousOption {
                option: long.to_string(),
                candidates: candidates.iter().map(|s| s.to_string()).collect(),
            })
        };

        #help_check

        let option = format!("--{}", long);
        match long {
            #(#match_arms)*
            _ => unreachable!("Should be caught by (None, []) case above.")
        }
    )
}

pub(crate) fn positional_handling(args: &[Argument]) -> (TokenStream, TokenStream) {
    let mut match_arms = Vec::new();
    // The largest index of the previous argument, so the the argument after this should
    // belong to the next argument.
    let mut last_index = 0;

    // The minimum number of arguments needed to not return a missing argument error.
    let mut minimum_needed = 0;
    let mut missing_argument_checks = vec![];

    for arg @ Argument { name, arg_type, .. } in args {
        let (num_args, last) = match arg_type {
            ArgType::Positional { num_args, last } => (num_args, last),
            ArgType::Option { .. } => continue,
        };

        if *num_args.start() > 0 {
            minimum_needed = last_index + num_args.start();
            missing_argument_checks.push(quote!(if positional_idx < #minimum_needed {
                missing.push(#name);
            }));
        }

        last_index += num_args.end();

        let expr = if *last {
            last_positional_expression(&arg.ident)
        } else {
            positional_expression(&arg.ident)
        };
        match_arms.push(quote!(0..=#last_index => { #expr }));
    }

    let value_handling = quote!(
        *positional_idx += 1;
        match positional_idx {
            #(#match_arms)*
            _ => return Err(lexopt::Arg::Value(value).unexpected().into()),
        }
    );

    let missing_argument_checks = quote!(
        // We have the minimum number of required arguments overall.
        // So we don't need to check the others.
        if positional_idx >= #minimum_needed {
            return Ok(());
        }

        let mut missing: Vec<&str> = vec![];
        #(#missing_argument_checks)*
        if !missing.is_empty() {
            Err(uutils_args::Error::MissingPositionalArguments(
                missing.iter().map(ToString::to_string).collect::<Vec<String>>()
            ))
        } else {
            Ok(())
        }
    );

    (value_handling, missing_argument_checks)
}

fn no_value_expression(ident: &Ident) -> TokenStream {
    quote!(Self::#ident)
}

fn default_value_expression(ident: &Ident, default_expr: &TokenStream) -> TokenStream {
    quote!(Self::#ident(#default_expr))
}

fn optional_value_expression(ident: &Ident, default_expr: &TokenStream) -> TokenStream {
    quote!(match parser.optional_value() {
        Some(value) => Self::#ident(FromValue::from_value(&option, value)?),
        None => Self::#ident(#default_expr),
    })
}

fn required_value_expression(ident: &Ident) -> TokenStream {
    quote!(Self::#ident(FromValue::from_value(&option, parser.value()?)?))
}

fn positional_expression(ident: &Ident) -> TokenStream {
    // TODO: Add option name in this from_value call
    quote!(
        Self::#ident(FromValue::from_value("", value)?)
    )
}

fn last_positional_expression(ident: &Ident) -> TokenStream {
    // TODO: Add option name in this from_value call
    quote!({
        let raw_args = parser.raw_args()?;
        let collection = std::iter::once(value)
            .chain(raw_args)
            .map(|v| FromValue::from_value("", v))
            .collect::<Result<_,_>>()?;
        Self::#ident(collection)
    })
}
