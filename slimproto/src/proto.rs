use crate::codec::SlimCodec;
use crate::discovery;

use tokio::{io::BufStream, net::TcpStream};
use tokio_util::codec::Framed;

use std::{io, net::Ipv4Addr};

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
        let framed = Framed::new(BufStream::with_capacity(READBUFSIZE, WRITEBUFSIZE, server_tcp), SlimCodec);

        Ok(SlimProto {framed})
    }
}
