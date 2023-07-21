//! Slim Protocol for Rust Clients
//!
//! This library simplifies communicating with a [Logitech Media Server, aka LMS, aka Slim Server][lms].
//!
//! Primarily, communicating with the server is done by instantiating a `Server` object, connecting to
//! the LMS server and then reading from and writing to supplied connection objects.
//! See [SlimProto][slimproto].
//!
//! This library also provides a [discover][discover] function to enable auto-discovery of LMS
//! servers on the network and a [StatusData][statusdata] struct to simplify the creation of the
//! regular status messages the server requires.
//!
//! In order to use this library it's a good idea to have studied the [Slim TCP Protocol][slimtcp] first
//! so that this library makes sense.
//!
//! [lms]: https://en.wikipedia.org/wiki/Logitech_Media_Server
//! [slimproto]: crate::proto::SlimProto
// [discover]: crate::discovery::discover
// [statusdata]: crate::status::StatusData
// [slimtcp]: https://wiki.slimdevices.com/index.php/SlimProto_TCP_protocol
//!

pub mod capability;
pub mod codec;
pub mod discovery;
pub mod proto;
pub mod status;
pub mod buffer;

pub use capability::{Capabilities, Capability};
pub use proto::{ClientMessage, ServerMessage};
pub use framous::*;
// pub use status::{StatusCode, StatusData};
