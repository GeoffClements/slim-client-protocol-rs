use mac_address::MacAddress;
use tokio::{io::BufStream, net::TcpStream};
use tokio_util::codec::Framed;

use crate::codec::SlimCodec;
use crate::discovery;

use std::{io, net::Ipv4Addr};

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
        uuid: [u8; 16],
        wlan_channel_list: u16,
        bytes_received: u64,
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
pub enum ServerMessage {
    Serv {
        ip_address: Ipv4Addr,
        sync_group_id: Option<String>,
    },
    Status(u32),
    Stream {
        autostart: bool,
        threshold: u32,
        output_threshold: u64,
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

pub struct SlimProto {
    framed: Framed<BufStream<TcpStream>, SlimCodec>,
}

impl SlimProto {
    pub async fn new(server_addr: Option<Ipv4Addr>) -> io::Result<Self> {
        const SLIM_PORT: u16 = 3483;
        const READBUFSIZE: usize = 1024;
        const WRITEBUFSIZE: usize = 1024;

        let server_addr =
            if let Some(addr) = server_addr.or(discovery::discover(None).await?.map(|(a, _)| a)) {
                addr
            } else {
                unreachable!() // because discover has no timeout
            };

        let server_tcp = TcpStream::connect((server_addr, SLIM_PORT)).await?;
        let framed = Framed::new(
            BufStream::with_capacity(READBUFSIZE, WRITEBUFSIZE, server_tcp),
            SlimCodec,
        );

        Ok(SlimProto { framed })
    }
}
