pub mod proto {
    connectrpc::include_generated!();
}
pub mod bot;
pub mod client;
pub mod ctx;
pub mod error;
pub mod events;
pub mod misc;

pub use crate::bot::Bot;
pub use crate::client::Client;
pub use crate::ctx::*;
pub use crate::error::*;
pub use crate::events::*;

pub mod prelude {
    pub use crate::bot::Bot;
    pub use crate::client::Client;
    pub use crate::ctx::*;
    pub use crate::error::*;
    pub use crate::events::*;
    pub use crate::misc::*;
    pub use async_trait::async_trait;
    pub use futures_util::StreamExt;
}
