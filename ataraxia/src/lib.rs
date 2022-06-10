//! Ataraxia is a Rust Library for the Revolt API.
//! 
//! View the [examples] on how to make and structure a bot.
//! 
//! 
//! Once logged in, you may add handlers to your client to dispatch [`Event`]s,
//! such as [`EventHandler::message`]. This will cause your handler to be called
//! when a [`Event::Message`] is received. Each handler is given a
//! [`Context`], giving information about the event and some helpful functions. 

pub mod websocket;
pub mod framework;
pub mod http;
pub mod models;
pub mod context;

pub use async_trait::async_trait;

