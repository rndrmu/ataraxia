/*!
Procedural macros for ataraxia, e.g. [`macro@command`].
*/
mod util;

use quote::ToTokens;

/// Marks an async function as a bot command.
///
/// Early-stage — currently a pass-through that will gain argument parsing and
/// registration support in a future release.
#[proc_macro_attribute]
pub fn command(
    args: proc_macro::TokenStream,
    function: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let _args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let function = syn::parse_macro_input!(function as syn::ItemFn);
    function.into_token_stream().into()
}

