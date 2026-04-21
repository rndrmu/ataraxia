/*!
Procedural macros for ataraxia, e.g. [`macro@command`].
*/
mod command;
mod util;

/// Marks an async `fn(Context, Message)` as a bot command.
///
/// Generates a `pub static <NAME>_COMMAND: ataraxia::command::Command` that can
/// be registered with [`ataraxia::command::CommandFramework`].
///
/// # Arguments
/// - `name` — command name (defaults to the function name)
/// - `description` — help text (defaults to `""`)
/// - `aliases` — additional trigger names, e.g. `aliases(p, pong)`
///
/// # Example
/// ```ignore
/// #[command(name = "ping", description = "Ping the bot", aliases(p))]
/// async fn ping(ctx: Context, msg: Message) {
///     ctx.reply("pong!").await;
/// }
/// // generates: pub static PING_COMMAND: Command = ...;
/// ```
#[proc_macro_attribute]
pub fn command(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    command::command(args, input)
}
