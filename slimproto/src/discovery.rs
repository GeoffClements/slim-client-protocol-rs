use tokio::{
    net::{udp::SendHalf, UdpSocket},
    time,
};

use std::{
    collections::HashMap,
    io,
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};

#[derive(Debug)]
pub enum ServerTlv {
    Name(String),
    Version(String),
    Address(Ipv4Addr),
    Port(u16),
}

type ServerTlvMap = HashMap<String, ServerTlv>;

pub async fn discover(timeout: Option<Duration>) -> io::Result<Option<(Ipv4Addr, ServerTlvMap)>> {
    const UDPMAXSIZE: usize = 1450; // as defined in LMS code

    let cx = UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), 0)).await?;
    cx.set_broadcast(true)?;
    let (mut udp_rx, udp_tx) = cx.split();
    let pings = send_pings(udp_tx);

    let mut server_addr = None;
    let mut server_tlv = HashMap::new();
    let mut buf = [0u8; UDPMAXSIZE];
    tokio::select! {
        _ = time::delay_for(timeout.unwrap_or(Duration::from_secs(1))), if timeout.is_some() => {},
        res = pings => {
            if let Err(err) = res {
                return Err(err);
            }
        },
        res = udp_rx.recv_from(&mut buf) => {
            match res {
                Ok((len, socket_addr)) => {
                    server_addr = if let IpAddr::V4(addr) = socket_addr.ip() {
                        Some(addr)
                    } else {
                        None
                    };
                    if len > 0 && buf[0] == b'E' {
                        server_tlv = decode_tlv(&buf[1..]);
                    }
                },
                Err(e) => return Err(e),
            }
        },
    }

    if let Some(server_addr) = server_addr {
        Ok(Some((server_addr, server_tlv)))
    } else {
        Ok(None)
    }
}

async fn send_pings(mut udp_tx: SendHalf) -> tokio::io::Result<()> {
    const PING_INTERVAL: u64 = 5;
    const SLIM_PORT: u16 = 3483;

    let buf = "eNAME\0IPAD\0JSON\0VERS\0".as_bytes();
    let bcaddr = Ipv4Addr::new(255, 255, 255, 255);
    let mut interval = time::interval(Duration::from_secs(PING_INTERVAL));
    loop {
        interval.tick().await;
        udp_tx.send_to(&buf, &(bcaddr, SLIM_PORT).into()).await?;
    }
}

fn decode_tlv(buf: &[u8]) -> ServerTlvMap {
    let mut ret = HashMap::new();
    let mut view = &buf[..];

    while view.len() > 4 && view[0].is_ascii() {
        let token: String = view[..4].iter().map(|c| *c as char).collect();
        let valen = view[4] as usize;
        view = &view[5..];

        let value = if view.len() >= valen {
            &view[..valen]
        } else {
            break;
        }
        .iter()
        .map(|c| *c as char)
        .collect::<String>();

        let value = match token.as_str() {
            "NAME" => ServerTlv::Name(value),
            "VERS" => ServerTlv::Version(value),
            "IPAD" => {
                if let Ok(addr) = value.parse::<Ipv4Addr>() {
                    ServerTlv::Address(addr)
                } else {
                    break;
                }
            }
            "JSON" => {
                if let Ok(port) = value.parse::<u16>() {
                    ServerTlv::Port(port)
                } else {
                    break;
                }
            }
            _ => {
                break;
            }
        };

        ret.insert(token, value);
        view = &view[valen..];
    }

    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discover() {
        let res = discover(Some(Duration::from_secs(1))).await;
        assert!(res.is_ok());

        if let Ok(Some((ip, r))) = res {
            assert!(!ip.is_unspecified());
            assert!(r.len() > 0);
        }
    }
}
