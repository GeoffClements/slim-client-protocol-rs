/// Examplle of creating a protocol object and receiveing messages
/// We create the object and add capabilities.
/// The protocol object connects to the server and announces
/// itself with a HELO message. We then print all the messages
/// from the server.
///
/// Unlike the hello example this will respond to status requests
/// with a status message and this will go on forever, doing
/// nothing, hence boring.
use futures::{SinkExt, StreamExt};
use slimproto::{Capability, ServerMessage, SlimProto, StatusCode, StatusData};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut status = StatusData::new(0, 0);

    let mut proto = SlimProto::new();
    proto
        .add_capability(Capability::Modelname("Example".to_owned()))
        .add_capability(Capability::Model("Example".to_owned()));

    if let Ok((mut proto_stream, mut proto_sink)) = proto.connect().await {
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
                _ => {}
            }
        }
    }
}
