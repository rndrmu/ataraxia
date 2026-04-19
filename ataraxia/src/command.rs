use std::future::Future;
use std::pin::Pin;

use crate::{client::EventHandler, context::Context, models::message::Message};

pub type CommandFn =
    fn(Context, Message) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
    pub aliases: &'static [&'static str],
    pub execute: CommandFn,
}

pub struct CommandFramework {
    pub prefix: String,
    commands: Vec<&'static Command>,
}

impl CommandFramework {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self { prefix: prefix.into(), commands: Vec::new() }
    }

    pub fn command(mut self, cmd: &'static Command) -> Self {
        self.commands.push(cmd);
        self
    }

    pub async fn dispatch(&self, ctx: Context, msg: Message) {
        let content = msg.content.as_str();
        let Some(rest) = content.strip_prefix(&self.prefix) else { return };
        let name = rest.split_whitespace().next().unwrap_or("");
        if name.is_empty() { return; }
        for cmd in &self.commands {
            if cmd.name == name || cmd.aliases.contains(&name) {
                (cmd.execute)(ctx, msg).await;
                return;
            }
        }
    }
}

#[async_trait::async_trait]
impl EventHandler for CommandFramework {
    async fn authenticated(&self) {}
    async fn ready(&self, _ctx: Context, _ready: crate::models::ready::Ready) {}
    async fn on_message(&self, ctx: Context, msg: Message) {
        self.dispatch(ctx, msg).await;
    }
}
