
#[derive(Default, Debug, darling::FromMeta)]
#[darling(default)]
pub struct CommandArgs {
    name: String,
    description: String,
    aliases: crate::util::List<String>
}

pub struct Command {
    name: String,
    description: String,
    aliases: Vec<String>,
    function: syn::ItemFn,
}

/* 
pub fn cmd(
    args: CommandArgs,
    function: syn::ItemFn,
) -> Result<proc_macro::TokenStream, darling::Error> {
    let name = args.name;
    let description = args.description;
    let aliases = args.aliases.0;

    // construct the command struct
    let function_name = function.sig.ident;

    Ok(quote::quote!(
        ataraxia::macros::Command {
            name: #name,
            description: #description,
            aliases: vec![#(#aliases),*],
            function: #function_name as fn(&ataraxia::context::Context, ataraxia::models::Message) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'static>>
        }
    ).into())
}

 */