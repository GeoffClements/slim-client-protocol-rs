# slim-client-protocol-rs

A crate for the Slim protocol

The [Slim Protocol][slimtcpwiki] is a TCP protocol for streaming audio files
to a [slim device][slimdevices].

This crate simplifies writing of a client for this protocol by providing a
library that sends and receives messages to a slim server.

[slimtcpwiki]: https://wiki.slimdevices.com/index.php/SlimProto_TCP_protocol
[slimdevices]: https://en.wikipedia.org/wiki/Slim_Devices

## Supported Rust Versions

slim-client-protocol-rs is built against the latest stable release.

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/GeoffClements/slim-client-protocol-rs/blob/master/LICENSE.txt

[![MIT licensed][mit-badge]][mit-url]
[![Crate](https://img.shields.io/crates/v/slimproto.svg)](https://crates.io/crates/slimproto)
[![GitHub last commit](https://img.shields.io/github/last-commit/GeoffClements/slim-client-protocol-rs.svg)][github]
[![Build Status][actions-badge]][actions-url]

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/slimproto/blob/master/LICENSE
[github]: https://github.com/GeoffCLements/slim-client-protocol-rs
[actions-badge]: https://github.com/GeoffClements/slim-client-protocol-rs/actions/workflows/rust.yml/badge.svg
[actions-url]: https://github.com/GeoffClements/slim-client-protocol-rs/actions?query=workflow%3Arust+branch%3Amaster
