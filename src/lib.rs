pub mod proto {
    connectrpc::include_generated!();
}
pub mod bot;

pub use bot::Bot;
