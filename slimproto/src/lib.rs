//! Slim Protocol using [Tokio][tokio] and [Futures][futures]
//!
//! This library simplifies communicating with a [Logitech Media Server, aka LMS, aka Slim Server][lms]
//! by providing suitable asynchronous objects.
//!
//! Primarily, communicating with the server is done by instantiating asynchronous `Stream` and `Sink`
//! objects and then reading from and writing to them. Creating these protocol objects is done with
//! [SlimProto][slimproto].
//!
//! This library also provides a [discover][discover] function to enable auto-discovery of LMS
//! servers on the network and a [StatusData][statusdata] struct to simplify the creation of the
//! regular status messages the server requires.
//!
//! In order to use this library it's a good idea to have studied the [Slim TCP Protocol][slimtcp] first
//! so that this library makes sense.
//!
//! [tokio]: https://docs.rs/tokio/tokio/
//! [futures]: https://docs.rs/futures/futures/
//! [lms]: https://en.wikipedia.org/wiki/Logitech_Media_Server
//! [slimproto]: crate::proto::SlimProto
//! [discover]: crate::discovery::discover
//! [statusdata]: crate::status::StatusData
//! [slimtcp]: https://wiki.slimdevices.com/index.php/SlimProto_TCP_protocol
//!

pub mod capability;
pub(crate) mod codec;
pub mod discovery;
pub mod proto;
pub mod status;

pub use capability::{Capabilities, Capability};
pub use proto::{ClientMessage, ServerMessage, SlimProto};
pub use status::{StatusCode, StatusData};
