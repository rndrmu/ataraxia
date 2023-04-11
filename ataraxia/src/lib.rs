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
pub mod http;
pub mod models;
pub mod context;

/// Re-exports the `async_trait` crate. 
/// 
/// Used in Bots to implement the [`EventHandler`] trait.
/// 
/// [`EventHandler`]: crate::websocket::EventHandler
pub use async_trait::async_trait;


pub mod macros {
    //! Procedural macros used in ataraxia, like [`command`]
    #[doc(inline)]
    pub use ataraxia_macros::*;
}

#[doc(no_inline)]
pub use {
    macros::*, 
};

