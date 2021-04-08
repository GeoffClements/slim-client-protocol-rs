/// A basic player using Rodio
use futures::{SinkExt, StreamExt};
use rodio::{self, Decoder, OutputStream, Sink};
use slimproto::{
    util::socketreader::SocketReader, Capability, ClientMessage, ServerMessage, SlimProto,
    StatusCode, StatusData,
};

use std::{io::Write, net::TcpStream};

const BUFSIZE: u32 = 8 * 1024;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut status = StatusData::new(BUFSIZE, BUFSIZE);
    let mut proto = SlimProto::new();
    proto
        // .add_capability(Capability::Flc)
        // .add_capability(Capability::Ogg)
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
                        music_out.append(Decoder::new(SocketReader::with_capacity(32 * 1024, cx)).unwrap());
                    }
                }
                _ => {}
            }
        }
    }
}
