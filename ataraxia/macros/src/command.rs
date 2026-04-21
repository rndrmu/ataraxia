use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn};

#[derive(Debug, Default, FromMeta)]
#[darling(default)]
pub struct CommandArgs {
    name: Option<String>,
    description: Option<String>,
    aliases: crate::util::AliasList,
}

pub fn command(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(args as AttributeArgs);
    let func = parse_macro_input!(input as ItemFn);

    let cmd_args = match CommandArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let func_name = &func.sig.ident;
    let func_name_str = func_name.to_string();

    let cmd_name = cmd_args.name.unwrap_or_else(|| func_name_str.clone());
    let description = cmd_args.description.unwrap_or_default();
    let aliases = cmd_args.aliases.0;

    let static_ident =
        quote::format_ident!("{}_COMMAND", func_name_str.to_uppercase());

    quote! {
        #func

        pub static #static_ident: ::ataraxia::command::Command = ::ataraxia::command::Command {
            name: #cmd_name,
            description: #description,
            aliases: &[#(#aliases),*],
            execute: |ctx, msg| ::std::boxed::Box::pin(#func_name(ctx, msg)),
        };
    }
    .into()
}
