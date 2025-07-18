/// Contains the Server type with which we connect to the server.
///
/// This module also holds the `ClientMessage` and `ServerMessage` types that
/// are sent to and received from the server.
use bitflags::bitflags;
use framous::{FramedRead, FramedWrite, FramedWriter};
use mac_address::{get_mac_address, MacAddress};
pub const SLIM_PORT: u16 = 3483;

use crate::{codec::SlimCodec, status::StatusData, Capabilities};

use std::{
    collections::HashMap,
    io::{self, BufReader, BufWriter},
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
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

/// A Server struct to hold the connection details
pub struct Server {
    pub socket: SocketAddrV4,
    pub tlv_map: Option<ServerTlvMap>,
    pub sync_group_id: Option<String>,
    pub(crate) caps: Capabilities,
}

/// Allow to clone the server.
/// We'll lose the TLV map but it's not needed for connecting to the server
impl Clone for Server {
    fn clone(&self) -> Self {
        Self {
            socket: self.socket,
            tlv_map: None,
            sync_group_id: self.sync_group_id.as_ref().map(String::from),
            caps: self.caps.clone(),
        }
    }
}

/// Useful for conversions from a Serv message
impl From<(Ipv4Addr, Option<String>)> for Server {
    fn from(value: (Ipv4Addr, Option<String>)) -> Self {
        Self {
            socket: SocketAddrV4::new(value.0, SLIM_PORT),
            tlv_map: None,
            sync_group_id: value.1,
            caps: Capabilities(Vec::new()),
        }
    }
}

impl From<SocketAddrV4> for Server {
    fn from(value: SocketAddrV4) -> Self {
        Self {
            socket: value,
            tlv_map: None,
            sync_group_id: None,
            caps: Capabilities(Vec::new()),
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        Self {
            socket: SocketAddrV4::new([0, 0, 0, 0].into(), 9000),
            tlv_map: None,
            sync_group_id: None,
            caps: Capabilities(Vec::new()),
        }
    }
}

impl Server {
    pub fn connect(
        &self,
    ) -> io::Result<(
        FramedRead<BufReader<TcpStream>, SlimCodec>,
        FramedWrite<BufWriter<TcpStream>, SlimCodec>,
    )> {
        let cx = TcpStream::connect(self.socket)?;
        cx.set_nodelay(true)?;
        // cx.set_nonblocking(true)?;
        // cx.set_read_timeout(Some(Duration::from_secs(30)))?;
        // cx.set_write_timeout(Some(Duration::from_secs(30)))?;

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

        let rx = FramedRead::new(BufReader::new(cx.try_clone()?), SlimCodec);
        let mut tx = FramedWrite::new(BufWriter::new(cx), SlimCodec);

        tx.framed_write(helo)?;
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
    #[derive(Debug, Default, PartialEq)]
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
        http_headers: Option<String>,
    },
    Gain(f64, f64),
    Enable(bool, bool),
    Flush,
    Stop,
    Pause(Duration),
    Unpause(Duration),
    Queryname,
    Setname(String),
    DisableDac,
    Skip(Duration),
    Unrecognised(String),
    Error,
}

pub type ServerMessages = Vec<ServerMessage>;