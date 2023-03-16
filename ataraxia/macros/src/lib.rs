/*!
Procedural macros used in ataraxia, like [`macro@command`]
*/

mod command;
mod util;

use proc_macro::TokenStream;

/**
# These macros are largely taken from the [poise] crate, and are used to create commands for the bot.
[poise]: https://github.com/serenity-rs/poise
This macro transforms plain functions into bot commands.

Documentation comments are used as help text. The first line is a single-line description,
displayed in listings of your bot's commands (i.e. `~help`). Following paragraphs are detailed explanations,
for example for command-specific help (i.e. `~help command_name`). Escape newlines with `\`

# Macro arguments

`#[ataraxia::command]` accepts a number of arguments to configure the command:
- `prefix_command`: Generate a prefix command
- `slash_command`: Generate a slash command
- `context_menu_command`: Generate a context menu command
- `description_localized`: Adds localized description of the parameter `description_localized("locale", "Description")` (slash-only)
- `name_localized`: Adds localized name of the parameter `name_localized("locale", "new_name")` (slash-only)
- `subcommands`: List of subcommands `subcommands("foo", "bar", "baz")`
- `aliases`: Command name aliases (only applies to prefix commands)
- `invoke_on_edit`: Reruns the command if an existing invocation message is edited (prefix only)
- `track_deletion`: Deletes the bot response to a command if the command message is deleted (prefix only)
- `reuse_response`: After the first response, post subsequent responses as edits to the initial message (prefix only)
- `track_edits`: Shorthand for `invoke_on_edit`, `track_deletion`, and `reuse_response` (prefix only)
- `broadcast_typing`: Trigger a typing indicator while command runs (only applies to prefix commands I think)
- `help_text_fn`: Path to a string-returning function which is used for command help text instead of documentation comments
    - Useful if you have many commands with very similar help messages: you can abstract the common parts into a function
- `check`: Path to a function which is invoked for every invocation. If the function returns false, the command is not executed (can be used multiple times)
- `on_error`: Error handling function
- `rename`: Choose an alternative command name instead of the function name
    - Useful if your command name is a Rust keyword, like `move`
- `discard_spare_arguments`: Don't throw an error if the user supplies too many arguments
- `hide_in_help`: Hide this command in help menus
- `ephemeral`: Make bot responses ephemeral if possible
    - Only ataraxia's function, like `ataraxia::send_reply`, respect this preference
- `required_permissions`: Permissions which the command caller needs to have
- `required_bot_permissions`: Permissions which the bot is known to need
- `owners_only`: Restricts command callers to a configurable list of owners (see FrameworkOptions)
- `guild_only`: Restricts command callers to only run on a guild
- `dm_only`: Restricts command callers to only run on a DM
- `nsfw_only`: Restricts command callers to only run on a NSFW channel
- `identifying_name`: Optionally, a unique identifier for this command for your personal usage
- `category`: Category of this command which affects placement in the help command
- `custom_data`: Arbitrary expression that will be boxed and stored in `Command::custom_data`
- `global_cooldown`: Minimum duration in seconds between invocations, globally
- `user_cooldown`: Minimum duration in seconds between invocations, per user
- `guild_cooldown`: Minimum duration in seconds between invocations, per guild
- `channel_cooldown`: Minimum duration in seconds between invocations, per channel
- `member_cooldown`: Minimum duration in seconds between invocations, per guild member

# Function parameters

`Context` is the first parameter of all command functions. It's an enum over either PrefixContext or
SlashContext, which contain a variety of context data each. Context provides some utility methods to
access data present in both PrefixContext and SlashContext, like `author()` or `created_at()`.

All following parameters are inputs to the command. You can use all types that implement
`ataraxia::PopArgumentAsync`, `ataraxia::PopArgument`, `serenity::ArgumentConvert` or `std::str::FromStr`.
You can also wrap types in `Option` or `Vec` to make them optional or variadic. In addition, there
are multiple attributes you can use on parameters:
- `#[description = ""]`: Sets description of the parameter (slash-only)
- `#[description_localized("locale", "Description")]`: Adds localized description of the parameter (slash-only)
- `#[name_localized("locale", "new_name")]`: Adds localized name of the parameter (slash-only)
- `#[autocomplete = "callback()"]`: Sets the autocomplete callback (slash-only)
- `#[channel_types("", "")]`: For channel parameters, restricts allowed channel types (slash-only)
- `#[rename = "new_name"]`: Changes the user-facing name of the parameter (slash-only)
- `#[min = 0]`: Minimum value for this number parameter (slash-only)
- `#[max = 0]`: Maximum value for this number parameter (slash-only)
- `#[rest]`: Use the entire rest of the message for this parameter (prefix-only)
- `#[lazy]`: Can be used on Option and Vec parameters and is equivalent to regular expressions' laziness (prefix-only)
- `#[flag]`: Can be used on a bool parameter to set the bool to true if the user typed the parameter name literally (prefix-only)
    - For example with `async fn my_command(ctx: Context<'_>, #[flag] my_flag: bool)`, `~my_command` would set my_flag to false, and `~my_command my_flag` would set my_flag to true

# Help text

Documentation comments are used as command help text. The first paragraph is the command
description (`Command::description`) and all following paragraphs are the multiline help text
(`Command::help_text`).

In the multiline help text, put `\` at the end of a line to escape the newline.

Example:

```rust
/// This is the description of my cool command, it can span multiple
/// lines if you need to
///
/// Here in the following paragraphs, you can give information on how \
/// to use the command that will be shown in your command's help.
///
/// You could also put example invocations here:
/// `~coolcommand test`
#[atarxia::command(slash_command)]
pub async fn coolcommand(ctx: Context<'_>, s: String) -> Result<(), Error> { ... }
```
results in
```rust
ataraxia::Command {
    description: Some("This is the description of my cool command, it can span multiple lines if you need to".into()),
    help_text: Some("Here in the following paragraphs, you can give information on how to use the command that will be shown in your command's help.\n\nYou could also put example invocations here:\n`~coolcommand test`".into()),
    ...
}
```

# Internals

Internally, this attribute macro generates a function with a single `ataraxia::Command`
return type, which contains all data about this command. For example, it transforms a function of
this form:
```rust
/// This is a command
#[ataraxia::command(slash_command, prefix_command)]
async fn my_command(ctx: Context<'_>) -> Result<(), Error> {
    // code
}
```
into something like
```rust
fn my_command() -> ataraxia::Command<Data, Error> {
    async fn inner(ctx: Context<'_>) -> Result<(), Error> {
        // code
    }

    ataraxia::Command {
        name: "my_command",
        description: "This is a command",
        prefix_action: Some(|ctx| Box::pin(async move {
            inner(ctx.into()).await
        })),
        slash_action: Some(|ctx| Box::pin(async move {
            inner(ctx.into()).await
        })),
        context_menu_action: None,
        // ...
    }
}
```

If you're curious, you can use [`cargo expand`](https://github.com/dtolnay/cargo-expand) to see the
exact desugaring
*/
#[proc_macro_attribute]
pub fn command(args: TokenStream, function: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as Vec<syn::NestedMeta>);
    let args = match <command::CommandArgs as darling::FromMeta>::from_list(&args) {
        Ok(x) => x,
        Err(e) => return e.write_errors().into(),
    };

    let function = syn::parse_macro_input!(function as syn::ItemFn);

    match command::command(args, function) {
        Ok(x) => x,
        Err(e) => e.write_errors().into(),
    }
}

