/// A basic player using Rodio
use futures::{SinkExt, StreamExt};
use rodio::{self, Decoder, OutputStream, Sink};
use slimproto::{Capability, ClientMessage, ServerMessage, SlimProto, StatusCode, StatusData};

use std::{
    cmp,
    convert::TryInto,
    io::{self, BufRead, Read, Seek, SeekFrom, Write},
    net::TcpStream,
};

const BUFSIZE: u32 = 8 * 1024;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut status = StatusData::new(BUFSIZE, BUFSIZE);
    let mut proto = SlimProto::new();
    proto
        .add_capability(Capability::Mp3)
        .add_capability(Capability::Modelname("Example".to_owned()))
        .add_capability(Capability::Model("Example".to_owned()));
    let (_music_stream, music_handle) = OutputStream::try_default().unwrap();
    let music_out = Sink::try_new(&music_handle).unwrap();
    if let Ok((mut proto_stream, mut proto_sink, server_addr)) = proto.connect().await {
        while let Some(Ok(msg)) = proto_stream.next().await {
            println!("{:?}", msg);
            match msg {
                ServerMessage::Status(timestamp) => {
                    status.set_timestamp(timestamp);
                    let msg = status.make_status_message(StatusCode::Timer);
                    if let Err(_) = proto_sink.send(msg).await {
                        break;
                    }
                }
                ServerMessage::Queryname => {
                    if let Err(_) = proto_sink
                        .send(ClientMessage::Name("Rodio".to_owned()))
                        .await
                    {
                        break;
                    }
                }
                ServerMessage::Stream {
                    server_ip,
                    server_port,
                    http_headers,
                    ..
                } => {
                    let server_addr = if server_ip.octets() == [0u8; 4] {
                        server_addr
                    } else {
                        server_ip
                    };
                    if let Ok(mut cx) = TcpStream::connect((server_addr, server_port)) {
                        if let Some(request) = http_headers {
                            let _ = write!(
                                cx,
                                "{} {} {}\r\n\r\n",
                                request.method(),
                                request.uri(),
                                request.version()
                            );
                        }
                        music_out.append(Decoder::new(BufReader::new(cx)).unwrap());
                    }
                }
                _ => {}
            }
        }
    }
}

struct BufReader<R> {
    inner: R,
    buf: Box<[u8]>,
    pos: usize,
    cap: usize,
    pos_from_start: u64,
}

impl<R: Read> BufReader<R> {
    fn new(inner: R) -> BufReader<R> {
        BufReader::with_capacity(BUFSIZE as usize, inner)
    }

    fn with_capacity(capacity: usize, inner: R) -> BufReader<R> {
        let mut buffer = Vec::with_capacity(capacity);
        buffer.resize(capacity, 0);
        BufReader {
            inner,
            buf: buffer.into_boxed_slice(),
            pos: 0,
            cap: 0,
            pos_from_start: 0,
        }
    }
}

impl<R: Read> Read for BufReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let nread = {
            let mut rem = self.fill_buf()?;
            rem.read(buf)?
        };
        self.consume(nread);
        Ok(nread)
    }
}

impl<R: Read> BufRead for BufReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.pos >= self.cap {
            self.cap = self.inner.read(&mut self.buf)?;
            self.pos = 0;
        }
        Ok(&self.buf[self.pos..self.cap])
    }

    fn consume(&mut self, amt: usize) {
        self.pos = cmp::min(self.pos + amt, self.cap);
        self.pos_from_start += self.pos as u64;
    }
}

impl<R: Read> Seek for BufReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let mut result = 0u64;
        if let SeekFrom::Current(n) = pos {
            let remainder = (self.cap - self.pos) as i64;
            if n <= remainder {
                self.consume(n.try_into().unwrap());
                result = self.pos_from_start;
            } else {
                return Err(io::Error::from(io::ErrorKind::NotFound));
            }
        }
        Ok(result)
    }
}
