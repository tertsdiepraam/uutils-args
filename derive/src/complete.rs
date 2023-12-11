// For the full copyright and license information, please view the LICENSE
// file that was distributed with this source code.

use crate::{
    argument::{ArgType, Argument},
    flags::{Flag, Flags, Value},
};
use proc_macro2::TokenStream;
use quote::quote;

pub fn complete(args: &[Argument], file: &Option<String>) -> TokenStream {
    let mut arg_specs = Vec::new();

    let (summary, _usage, after_options) = if let Some(file) = file {
        crate::help::read_help_file(file)
    } else {
        ("".into(), "{} [OPTIONS] [ARGUMENTS]".into(), "".into())
    };

    for Argument {
        help,
        field,
        arg_type,
        ..
    } in args
    {
        let ArgType::Option {
            flags,
            hidden: false,
            ..
        } = arg_type
        else {
            continue;
        };

        let Flags { short, long, .. } = flags;
        if short.is_empty() && long.is_empty() {
            continue;
        }

        let short: Vec<_> = short
            .iter()
            .map(|Flag { flag, value }| {
                let flag = flag.to_string();
                let value = match value {
                    Value::No => quote!(::uutils_args_complete::Value::No),
                    Value::Optional(name) => quote!(::uutils_args_complete::Value::Optional(#name)),
                    Value::Required(name) => quote!(::uutils_args_complete::Value::Optional(#name)),
                };
                quote!(::uutils_args_complete::Flag {
                    flag: #flag,
                    value: #value
                })
            })
            .collect();

        let long: Vec<_> = long
            .iter()
            .map(|Flag { flag, value }| {
                let value = match value {
                    Value::No => quote!(::uutils_args_complete::Value::No),
                    Value::Optional(name) => quote!(::uutils_args_complete::Value::Optional(#name)),
                    Value::Required(name) => quote!(::uutils_args_complete::Value::Optional(#name)),
                };
                quote!(::uutils_args_complete::Flag {
                    flag: #flag,
                    value: #value
                })
            })
            .collect();

        let hint = if let Some(ty) = field {
            quote!(Some(<#ty>::value_hint()))
        } else {
            quote!(None)
        };

        arg_specs.push(quote!(
            ::uutils_args_complete::Arg {
                short: vec![#(#short),*],
                long: vec![#(#long),*],
                help: #help,
                value: #hint,
            }
        ))
    }

    quote!(::uutils_args_complete::Command {
        name: option_env!("CARGO_BIN_NAME").unwrap_or(env!("CARGO_PKG_NAME")),
        summary: #summary,
        after_options: #after_options,
        version: env!("CARGO_PKG_VERSION"),
        args: vec![#(#arg_specs),*],
        license: env!("CARGO_PKG_LICENSE"),
        authors: env!("CARGO_PKG_AUTHORS"),
    })
}
