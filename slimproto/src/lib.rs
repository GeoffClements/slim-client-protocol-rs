//! Slim Protocol using [Tokio][tokio] and [Futures][futures]
//!
//!
//! [tokio]: https://docs.rs/tokio/tokio/
//! [futures]: https://docs.rs/futures/futures/

pub mod capability;
pub(crate) mod codec;
pub mod discovery;
pub mod proto;
pub mod status;
pub mod util;

pub use capability::{Capabilities, Capability};
pub use proto::{ClientMessage, ServerMessage, SlimProto};
pub use status::{StatusCode, StatusData};
