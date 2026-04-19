pub mod client;
pub mod context;
pub mod gateway;
pub mod http;
pub mod models;
pub mod websocket;

pub use async_trait::async_trait;

pub mod macros {
    //! Procedural macros for ataraxia (e.g. [`command`]).
    #[doc(inline)]
    pub use ataraxia_macros::*;
}

#[doc(no_inline)]
pub use macros::*;
