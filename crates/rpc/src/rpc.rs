pub mod auth;

pub use proto;
pub use proto::{Receipt, TypedEnvelope, error::*};

#[cfg(feature = "gpui")]
mod proto_client;
#[cfg(feature = "gpui")]
pub use proto_client::*;

pub const PROTOCOL_VERSION: u32 = 68;
