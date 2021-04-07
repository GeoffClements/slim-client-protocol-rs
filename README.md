# slim-client-protocol-rs

A Tokio / Futures based crate for the Slim protocol

The [Slim Protocol][slimtcpwiki] is a TCP protocol for streaming audio files
to a [slim device][slimdevices].

This crate simplifies writing of a client for this protocol by providing an
asynchronous library that sends and receives messages to a slim server.

[slimtcpwiki]: https://wiki.slimdevices.com/index.php/SlimProto_TCP_protocol
[slimdevices]: https://en.wikipedia.org/wiki/Slim_Devices

## Supported Rust Versions

slim-client-protocol-rs is built against the latest stable release.

## License

This project is licensed under the [MIT license].

<!-- TODO: fix reference -->
[MIT license]: https://github.com/tokio-rs/tokio/blob/master/LICENSE
