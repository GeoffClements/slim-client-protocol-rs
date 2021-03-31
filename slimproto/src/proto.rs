// use async_stream::try_stream;
use bitflags::bitflags;
use mac_address::MacAddress;
use tokio::net::TcpStream;
use tokio_stream::Stream;
use tokio_util::codec::FramedRead;

// use crate::codec::SlimCodec;
use crate::{
    codec::SlimCodec,
    discovery::{discover, ServerTlv},
};

use std::{io, net::Ipv4Addr, time::Duration};

#[derive(Default)]
pub struct StatData {
    pub crlf: u8,
    pub buffer_size: u32,
    pub fullness: u32,
    pub bytes_received: u64,
    pub sig_strength: u16,
    pub jiffies: Duration,
    pub output_buffer_size: u32,
    pub output_buffer_fullness: u32,
    pub elapsed_seconds: u32,
    pub voltage: u16,
    pub elapsed_milliseconds: u32,
    pub timestamp: Duration,
    pub error_code: u16,
}

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
        stat_data: StatData,
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

#[derive(Debug, PartialEq)]
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
        http_headers: String,
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

pub struct SlimProto;

impl SlimProto {
    pub async fn connect(&self) -> io::Result<Box<dyn Stream<Item = io::Result<ServerMessage>>>> {
        const SLIM_PORT: u16 = 3483;
        const READBUFSIZE: usize = 1024;
        // const WRITEBUFSIZE: usize = 1024;

        let (server_addr, server_tlvs) = discover(None).await?.unwrap(); //safe unwrap with no timeout
        let port = match server_tlvs.get("JSON") {
            Some(ServerTlv::Port(port)) => Some(*port),
            _ => None,
        }
        .unwrap_or(SLIM_PORT);

        let (server_rx, _server_tx) = TcpStream::connect((server_addr, port)).await?.into_split();
        let frames = FramedRead::with_capacity(server_rx, SlimCodec, READBUFSIZE);
        Ok(Box::new(frames))
    }
}

// impl SlimProto {
//     async fn send_helo(&mut self, bytes_rx: u64, reconnect: bool) {
//         let helo = ClientMessage::Helo {
//             device_id: 12,
//             revision: 0,
//             mac: match get_mac_address() {
//                 Ok(Some(mac)) => mac,
//                 _ => MacAddress::new([1, 2, 3, 4, 5, 6]),
//             },
//             uuid: [0u8; 16],
//             wlan_channel_list: if reconnect {0x4000} else {0},
//             bytes_received: bytes_rx,
//             language: ['e', 'n'],
//             capabilities: self.make_cap_string(),
//         };

//         let _ = self.framed.send(helo).await;
//     }

//     fn make_cap_string(&self) -> String {
//         let mut caps = self.capabilities.iter().join(",");
//         if let Some(modelname) = &self.modelname {
//             caps.push_str(format!(",ModelName={}", modelname).as_str());
//         }
//         caps
//     }
// }

// #[derive(Default)]
// pub struct SlimProtoBuilder {
//     server: Option<Ipv4Addr>,
//     reconnect: bool,
//     bytes_rx: u64,
//     model_name: Option<String>,
//     capabilities: Vec<Capability>,
// }

// impl SlimProtoBuilder {
//     pub fn new() -> Self {
//         SlimProtoBuilder::default()
//     }

//     pub fn server<'a>(&'a mut self, ip: Ipv4Addr) -> &'a mut Self {
//         self.server = Some(ip);
//         self
//     }

//     pub fn wma<'a>(&'a mut self, en: bool) -> &'a mut Self {
//         self.capabilities
//             .push(Capability::new("wma", CapValue::Bool(en)));
//         self
//     }

//     pub fn wmap<'a>(&'a mut self, en: bool) -> &'a mut Self {
//         self.capabilities
//             .push(Capability::new("wmap", CapValue::Bool(en)));
//         self
//     }

//     pub fn wmal<'a>(&'a mut self, en: bool) -> &'a mut Self {
//         self.capabilities
//             .push(Capability::new("wmal", CapValue::Bool(en)));
//         self
//     }

//     pub fn ogg<'a>(&'a mut self, en: bool) -> &'a mut Self {
//         self.capabilities
//             .push(Capability::new("ogg", CapValue::Bool(en)));
//         self
//     }

//     pub fn flc<'a>(&'a mut self, en: bool) -> &'a mut Self {
//         self.capabilities
//             .push(Capability::new("flc", CapValue::Bool(en)));
//         self
//     }

//     pub fn pcm<'a>(&'a mut self, en: bool) -> &'a mut Self {
//         self.capabilities
//             .push(Capability::new("pcm", CapValue::Bool(en)));
//         self
//     }

//     pub fn aif<'a>(&'a mut self, en: bool) -> &'a mut Self {
//         self.capabilities
//             .push(Capability::new("aif", CapValue::Bool(en)));
//         self
//     }

//     pub fn mp3<'a>(&'a mut self, en: bool) -> &'a mut Self {
//         self.capabilities
//             .push(Capability::new("mp3", CapValue::Bool(en)));
//         self
//     }

//     pub fn alc<'a>(&'a mut self, en: bool) -> &'a mut Self {
//         self.capabilities
//             .push(Capability::new("alc", CapValue::Bool(en)));
//         self
//     }

//     pub fn aac<'a>(&'a mut self, en: bool) -> &'a mut Self {
//         self.capabilities
//             .push(Capability::new("aac", CapValue::Bool(en)));
//         self
//     }

//     pub fn rhap<'a>(&'a mut self, en: bool) -> &'a mut Self {
//         self.capabilities
//             .push(Capability::new("rhap", CapValue::Bool(en)));
//         self
//     }

//     pub fn accurateplaypoints<'a>(&'a mut self, en: bool) -> &'a mut Self {
//         self.capabilities
//             .push(Capability::new("accurateplaypoints", CapValue::Bool(en)));
//         self
//     }

//     pub fn hasdigitalout<'a>(&'a mut self, en: bool) -> &'a mut Self {
//         self.capabilities
//             .push(Capability::new("hasdigitalout", CapValue::Bool(en)));
//         self
//     }

//     pub fn haspreamp<'a>(&'a mut self, en: bool) -> &'a mut Self {
//         self.capabilities
//             .push(Capability::new("haspreamp", CapValue::Bool(en)));
//         self
//     }

//     pub fn hasdisabledac<'a>(&'a mut self, en: bool) -> &'a mut Self {
//         self.capabilities
//             .push(Capability::new("hasdisabledac", CapValue::Bool(en)));
//         self
//     }

//     pub fn model<'a, D: fmt::Display>(&'a mut self, model: D) -> &'a mut Self {
//         self.capabilities.push(Capability::new(
//             "model",
//             CapValue::String(model.to_string()),
//         ));
//         self
//     }

//     pub fn modelname<'a, D: fmt::Display>(&'a mut self, model: D) -> &'a mut Self {
//         self.model_name = Some(model.to_string());
//         self
//     }

//     pub fn syncgroupid<'a, D: fmt::Display>(&'a mut self, model: D) -> &'a mut Self {
//         self.capabilities.push(Capability::new(
//             "syncgroupid",
//             CapValue::String(model.to_string()),
//         ));
//         self
//     }

//     pub fn maxsamplerate<'a>(&'a mut self, val: u32) -> &'a mut Self {
//         self.capabilities
//             .push(Capability::new("maxsamplerate", CapValue::Number(val)));
//         self
//     }

//     pub fn reconnect<'a>(&'a mut self, reconnect: bool) -> &'a mut Self {
//         self.reconnect = reconnect;
//         self
//     }

//     pub fn bytes_received<'a>(&'a mut self, bytes_rx: u64) -> &'a mut Self {
//         self.bytes_rx= bytes_rx;
//         self
//     }

//     pub async fn build(&self, helo: bool) -> io::Result<SlimProto> {
//         const SLIM_PORT: u16 = 3483;
//         const READBUFSIZE: usize = 1024;
//         const WRITEBUFSIZE: usize = 1024;

//         let server_addr = if let Some(addr) = self
//             .server
//             .or(discovery::discover(None).await?.map(|(a, _)| a))
//         {
//             addr
//         } else {
//             unreachable!() // because discover has no timeout
//         };

//         let server_tcp = TcpStream::connect((server_addr, SLIM_PORT)).await?;
//         let framed = Framed::new(
//             BufStream::with_capacity(READBUFSIZE, WRITEBUFSIZE, server_tcp),
//             SlimCodec,
//         );

//         let mut slimproto = SlimProto {
//             framed: framed,
//             modelname: self.model_name.clone(),
//             capabilities: self.capabilities.clone(),
//         };

//         if helo {
//             slimproto.send_helo(self.bytes_rx, self.reconnect).await;
//         }

//         Ok(slimproto)
//     }
// }
