/*!
Procedural macros used in ataraxia, like [`macro@command`]
*/
mod util;
mod command;
use quote::ToTokens;

#[proc_macro_attribute]
pub fn command(args: proc_macro::TokenStream, function: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // dummy fn that prints the args and function
    let args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let function = syn::parse_macro_input!(function as syn::ItemFn);

    println!("args: {:#?}", args);
    println!("function: {:#?}", function);

    function.into_token_stream().into()

}
