use bitflags::bitflags;
use futures::{Sink, SinkExt};
use http_header::RequestHeader;
use mac_address::{get_mac_address, MacAddress};
use tokio::net::TcpStream;
use tokio_stream::Stream;
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::{
    capability::{Capabilities, Capability},
    codec::SlimCodec,
    discovery::discover,
    status::StatusData,
};

use std::{io, net::Ipv4Addr, pin::Pin, time::Duration};

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
    pub struct StreamFlags: u8 {
        const INF_LOOP = 0b1000_0000;
        const NO_RESTART_DECODER = 0b0100_0000;
        const INVERT_POLARITY_LEFT = 0b0000_0001;
        const INVERT_POLARITY_RIGHT = 0b0000_0010;
    }
}

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
        http_headers: Option<RequestHeader>,
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

#[derive(Default)]
pub struct SlimProto {
    pub(crate) capabilities: Capabilities,
}

impl SlimProto {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_capability<'a>(&'a mut self, newcap: Capability) -> &'a mut Self {
        self.capabilities.add(newcap);
        self
    }

    pub async fn connect(
        self,
    ) -> io::Result<(
        Pin<Box<dyn Stream<Item = io::Result<ServerMessage>>>>,
        Pin<Box<dyn Sink<ClientMessage, Error = io::Error>>>,
        Ipv4Addr,
    )> {
        const SLIM_PORT: u16 = 3483;
        const READBUFSIZE: usize = 1024;

        let (server_addr, _server_tlvs) = discover(None).await?.unwrap(); //safe unwrap with no timeout

        let (server_rx, server_tx) = TcpStream::connect((server_addr, SLIM_PORT))
            .await?
            .into_split();
        let read_frames = FramedRead::with_capacity(server_rx, SlimCodec, READBUFSIZE);
        let mut write_frames = FramedWrite::new(server_tx, SlimCodec);

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
            capabilities: self.capabilities.to_string(),
        };
        write_frames.send(helo).await?;

        Ok((Box::pin(read_frames), Box::pin(write_frames), server_addr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buildproto() {
        let mut p = SlimProto::new();
        p.add_capability(Capability::Mp3);
        p.add_capability(Capability::Model("test".to_owned()))
            .add_capability(Capability::Ogg);
        assert_eq!(p.capabilities.to_string(), "mp3,Model=test,ogg");
    }
}
