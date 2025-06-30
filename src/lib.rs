pub mod api;
pub mod client;
pub mod error;
pub mod http;
pub mod models;
pub mod paths;
pub mod session;

pub mod prelude {
    pub use crate::client::Degiro;
    pub use crate::error::ClientError;
}
