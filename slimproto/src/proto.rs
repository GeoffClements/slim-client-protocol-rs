use crate::discovery;

use tokio::net::TcpStream;

use std::{io, net::Ipv4Addr};

pub struct SlimProto;

impl SlimProto {
    pub async fn new(server_addr: Option<Ipv4Addr>) -> io::Result<Self> {
        let server_addr = if server_addr.is_none() {
            if let Ok(addr) = server_addr.unwrap_or(discovery::discover(None).await? {
                
            }
        }
        // let server_addr = server_addr.unwrap_or(discovery::discover(None).await?.map(|(a, _)| a);
        if let Some(addr) = server_addr {
            let server_tcp = TcpStream::connect(addr).await?;
        }
       Ok(SlimProto)
    }
}
