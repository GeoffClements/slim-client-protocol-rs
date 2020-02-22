use bitflags::bitflags;
use futures::SinkExt;
use itertools::Itertools;
use mac_address::{get_mac_address, MacAddress};
use std::fmt;
use tokio::{io::BufStream, net::TcpStream};
use tokio_util::codec::Framed;

use crate::codec::SlimCodec;
use crate::discovery;

use std::{io, net::Ipv4Addr, time::Duration};

pub struct StatData {
    pub crlf: u8,
    pub buffer_size: u32,
    pub fullness: u32,
    pub bytes_received: u64,
    pub sig_strength: u16,
    pub jiffies: u32,
    pub output_buffer_size: u32,
    pub output_buffer_fullness: u32,
    pub elapsed_seconds: u32,
    pub voltage: u16,
    pub elapsed_milliseconds: u32,
    pub timestamp: u32,
    pub error_code: u16,
}

pub enum ClientMessage {
    Helo {
        device_id: u8,
        revision: u8,
        mac: MacAddress,
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
    Status(u32),
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
    Skip(u32),
    Unrecognised(String),
    Error,
}

#[derive(Clone)]
enum CapValue {
    Bool(bool),
    Number(u32),
    String(String),
}

#[derive(Clone)]
pub struct Capability {
    name: String,
    value: CapValue,
}

impl Capability {
    fn new<T: fmt::Display>(name: T, value: CapValue) -> Self {
        Capability {
            name: name.to_string(),
            value: value,
        }
    }
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match &self.value {
            CapValue::Bool(ref val) => {
                if *val {
                    "1"
                } else {
                    "0"
                }
            }
            .to_string(),
            CapValue::Number(ref val) => val.to_string(),
            CapValue::String(ref val) => val.to_string(),
        };
        write!(f, "{}={}", self.name, value)
    }
}

pub struct SlimProto {
    framed: Framed<BufStream<TcpStream>, SlimCodec>,
    capabilities: Vec<Capability>,
}

impl SlimProto {
    async fn send_helo(&mut self) -> io::Result<()> {
        let helo = ClientMessage::Helo {
            device_id: 12,
            revision: 0,
            mac: match get_mac_address() {
                Ok(Some(mac)) => mac,
                _ => MacAddress::new([1, 2, 3, 4, 5, 6]),
            },
            capabilities: self.capabilities.iter().join(","),
        };

        self.framed.send(helo).await
    }
}

#[derive(Default)]
pub struct SlimProtoBuilder {
    server: Option<Ipv4Addr>,
    capabilities: Vec<Capability>,
}

impl SlimProtoBuilder {
    pub fn new() -> Self {
        SlimProtoBuilder::default()
    }

    pub fn server(&mut self, ip: Ipv4Addr) -> &mut Self {
        self.server = Some(ip);
        self
    }

    pub fn wma(&mut self, en: bool) -> &mut Self {
        self.capabilities
            .push(Capability::new("wma", CapValue::Bool(en)));
        self
    }

    pub fn wmap(&mut self, en: bool) -> &mut Self {
        self.capabilities
            .push(Capability::new("wmap", CapValue::Bool(en)));
        self
    }

    pub fn wmal(&mut self, en: bool) -> &mut Self {
        self.capabilities
            .push(Capability::new("wmal", CapValue::Bool(en)));
        self
    }

    pub fn ogg(&mut self, en: bool) -> &mut Self {
        self.capabilities
            .push(Capability::new("ogg", CapValue::Bool(en)));
        self
    }

    pub fn flc(&mut self, en: bool) -> &mut Self {
        self.capabilities
            .push(Capability::new("flc", CapValue::Bool(en)));
        self
    }

    pub fn pcm(&mut self, en: bool) -> &mut Self {
        self.capabilities
            .push(Capability::new("pcm", CapValue::Bool(en)));
        self
    }

    pub fn aif(&mut self, en: bool) -> &mut Self {
        self.capabilities
            .push(Capability::new("aif", CapValue::Bool(en)));
        self
    }

    pub fn mp3(&mut self, en: bool) -> &mut Self {
        self.capabilities
            .push(Capability::new("mp3", CapValue::Bool(en)));
        self
    }

    pub fn alc(&mut self, en: bool) -> &mut Self {
        self.capabilities
            .push(Capability::new("alc", CapValue::Bool(en)));
        self
    }

    pub fn aac(&mut self, en: bool) -> &mut Self {
        self.capabilities
            .push(Capability::new("aac", CapValue::Bool(en)));
        self
    }

    pub fn rhap(&mut self, en: bool) -> &mut Self {
        self.capabilities
            .push(Capability::new("rhap", CapValue::Bool(en)));
        self
    }

    pub fn accurateplaypoints(&mut self, en: bool) -> &mut Self {
        self.capabilities
            .push(Capability::new("accurateplaypoints", CapValue::Bool(en)));
        self
    }

    pub fn hasdigitalout(&mut self, en: bool) -> &mut Self {
        self.capabilities
            .push(Capability::new("hasdigitalout", CapValue::Bool(en)));
        self
    }

    pub fn haspreamp(&mut self, en: bool) -> &mut Self {
        self.capabilities
            .push(Capability::new("haspreamp", CapValue::Bool(en)));
        self
    }

    pub fn hasdisabledac(&mut self, en: bool) -> &mut Self {
        self.capabilities
            .push(Capability::new("hasdisabledac", CapValue::Bool(en)));
        self
    }

    pub fn model<T: fmt::Display>(&mut self, model: T) -> &mut Self {
        self.capabilities.push(Capability::new(
            "model",
            CapValue::String(model.to_string()),
        ));
        self
    }

    pub fn modelname<T: fmt::Display>(&mut self, model: T) -> &mut Self {
        self.capabilities.push(Capability::new(
            "modelname",
            CapValue::String(model.to_string()),
        ));
        self
    }

    pub fn syncgroupid<T: fmt::Display>(&mut self, model: T) -> &mut Self {
        self.capabilities.push(Capability::new(
            "syncgroupid",
            CapValue::String(model.to_string()),
        ));
        self
    }

    pub fn maxsamplerate(&mut self, val: u32) -> &mut Self {
        self.capabilities
            .push(Capability::new("maxsamplerate", CapValue::Number(val)));
        self
    }

    pub async fn build(self, helo: bool) -> io::Result<SlimProto> {
        const SLIM_PORT: u16 = 3483;
        const READBUFSIZE: usize = 1024;
        const WRITEBUFSIZE: usize = 1024;

        let server_addr = if let Some(addr) = self
            .server
            .or(discovery::discover(None).await?.map(|(a, _)| a))
        {
            addr
        } else {
            unreachable!() // because discover has no timeout
        };

        let server_tcp = TcpStream::connect((server_addr, SLIM_PORT)).await?;
        let framed = Framed::new(
            BufStream::with_capacity(READBUFSIZE, WRITEBUFSIZE, server_tcp),
            SlimCodec,
        );

        let mut slimproto = SlimProto {
            framed: framed,
            capabilities: self.capabilities,
        };

        if helo {
            slimproto.send_helo().await?;
        }

        Ok(slimproto)
    }
}
