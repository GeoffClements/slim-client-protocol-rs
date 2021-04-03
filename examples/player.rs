/// A basic player using Rodio
use futures::{SinkExt, StreamExt};
use slimproto::{Capability, ClientMessage, ServerMessage, SlimProto, StatusCode, StatusData};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    const BUFSIZE: u32 = 8 * 1024;
    let mut status = StatusData::new(BUFSIZE, BUFSIZE);

    let mut proto = SlimProto::new();
    proto
        .add_capability(Capability::Flc)
        .add_capability(Capability::Ogg)
        .add_capability(Capability::Mp3)
        .add_capability(Capability::Modelname("Example".to_owned()))
        .add_capability(Capability::Model("Example".to_owned()));

    if let Ok((mut proto_stream, mut proto_sink, _server_addr)) = proto.connect().await {
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
                    http_headers: headers,
                    ..
                } => {
                    if let Some(request) = headers {
                        println!("{}", request.uri());
                    }
                }
                _ => {}
            }
        }
    }
}
