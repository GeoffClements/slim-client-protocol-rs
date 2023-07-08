// //! Contains the protocol object with which we interact with the server.
// //!
// //! This module also holds the `ClientMessage` and `ServerMessage` types that
// //! are sent to and received from the server.

use bitflags::bitflags;
use http_tiny::Header;
use mac_address::{get_mac_address, MacAddress};

use crate::{
    codec::SlimCodec,
    framing,
    framing::{FramedRead, FramedWrite},
    status::StatusData,
    Capabilities, Capability,
};

use std::{
    collections::HashMap,
    io::{self, Read, Write},
    net::{Ipv4Addr, TcpStream},
    time::Duration,
};

/// An enum which describes the various [TLV](https://en.wikipedia.org/wiki/Type%E2%80%93length%E2%80%93value)
/// values with which the server can respond.
#[derive(Debug)]
pub enum ServerTlv {
    Name(String),
    Version(String),
    Address(Ipv4Addr),
    Port(u16),
}

/// A hashmap to hold all TLVs from the server
pub(crate) type ServerTlvMap = HashMap<String, ServerTlv>;

pub struct Server {
    pub ip_address: std::net::Ipv4Addr,
    pub port: u16,
    pub tlv_map: ServerTlvMap,
    pub sync_group_id: Option<String>,
}

pub struct PreparedServer {
    server: Server,
    caps: Capabilities,
}

impl Server {
    pub fn prepare(self, mut caps: Capabilities) -> PreparedServer {
        if let Some(sgid) = &self.sync_group_id {
            caps.add(Capability::Syncgroupid(sgid.to_owned()));
        }
        PreparedServer { server: self, caps }
    }
}

impl PreparedServer {
    pub fn connect(
        self,
    ) -> io::Result<(
        FramedRead<SlimCodec, impl Read>,
        FramedWrite<SlimCodec, impl Write>,
    )> {
        const SLIM_PORT: u16 = 3483;
        let cx = TcpStream::connect((self.server.ip_address, SLIM_PORT))?;

        let helo = ClientMessage::Helo {
            device_id: 12,
            revision: 0,
            mac: match get_mac_address() {
                Ok(Some(mac)) => mac,
                _ => MacAddress::new([1, 2, 3, 4, 5, 6]),
            },
            uuid: [0u8; 16],
            wlan_channel_list: 0,
            bytes_received: 0,
            language: ['e', 'n'],
            capabilities: self.caps.to_string(),
        };

        let (rx, mut tx) = framing::make_frames(cx, SlimCodec)?;
        tx.send(helo)?;
        Ok((rx, tx))
    }
}

/// A type that describes all messages that are sent from the client to
/// the server.
#[derive(Debug)]
pub enum ClientMessage {
    Helo {
        device_id: u8,
        revision: u8,
        mac: MacAddress,
        uuid: [u8; 16],
        wlan_channel_list: u16,
        bytes_received: u64,
        language: [char; 2],
        capabilities: String,
    },
    Stat {
        event_code: String,
        stat_data: StatusData,
    },
    Bye(u8),
    Name(String),
}

#[derive(Debug, PartialEq)]
pub enum AutoStart {
    None,
    Auto,
    Direct,
    AutoDirect,
}

#[derive(Debug, PartialEq)]
pub enum Format {
    Pcm,
    Mp3,
    Flac,
    Wma,
    Ogg,
    Aac,
    Alac,
}
#[derive(Debug, PartialEq)]
pub enum PcmSampleSize {
    Eight,
    Sixteen,
    Twenty,
    ThirtyTwo,
    SelfDescribing,
}

#[derive(Debug, PartialEq)]
pub enum PcmSampleRate {
    Rate(u32),
    SelfDescribing,
}

#[derive(Debug, PartialEq)]
pub enum PcmChannels {
    Mono,
    Stereo,
    SelfDescribing,
}

#[derive(Debug, PartialEq)]
pub enum PcmEndian {
    Big,
    Little,
    SelfDescribing,
}

#[derive(Debug, PartialEq)]
pub enum SpdifEnable {
    Auto,
    On,
    Off,
}

#[derive(Debug, PartialEq)]
pub enum TransType {
    None,
    Crossfade,
    FadeIn,
    FadeOut,
    FadeInOut,
}

bitflags! {
    #[derive(Debug, PartialEq)]
    pub struct StreamFlags: u8 {
        const INF_LOOP = 0b1000_0000;
        const NO_RESTART_DECODER = 0b0100_0000;
        const INVERT_POLARITY_LEFT = 0b0000_0001;
        const INVERT_POLARITY_RIGHT = 0b0000_0010;
    }
}

/// A type that describes all messages that are sent from the server to
/// the client.
#[derive(Debug)]
pub enum ServerMessage {
    Serv {
        ip_address: Ipv4Addr,
        sync_group_id: Option<String>,
    },
    Status(Duration),
    Stream {
        autostart: AutoStart,
        format: Format,
        pcmsamplesize: PcmSampleSize,
        pcmsamplerate: PcmSampleRate,
        pcmchannels: PcmChannels,
        pcmendian: PcmEndian,
        threshold: u32,
        spdif_enable: SpdifEnable,
        trans_period: Duration,
        trans_type: TransType,
        flags: StreamFlags,
        output_threshold: Duration,
        replay_gain: f64,
        server_port: u16,
        server_ip: Ipv4Addr,
        http_headers: Option<Header>,
    },
    Gain(f64, f64),
    Enable(bool, bool),
    Stop,
    Pause(u32),
    Unpause(u32),
    Queryname,
    Setname(String),
    DisableDac,
    Skip(u32),
    Unrecognised(String),
    Error,
}
