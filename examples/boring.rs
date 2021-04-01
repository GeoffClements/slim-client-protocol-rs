/// Examplle of creating a protocol object and receiveing messages
/// We create the object and add capabilities.
/// The protocol object connects to the server and announces
/// itself with a HELO message. We then print all the messages
/// from the server.
///
/// Unlike the hello example this will respond to status requests
/// with a status message and this will go on forever.
use futures::{SinkExt, StreamExt};
use slimproto::{proto::make_heartbeat, Capability, ServerMessage, SlimProto};

#[tokio::main]
async fn main() {
    let mut proto = SlimProto::new();
    proto
        .add_capability(Capability::Modelname("Example".to_owned()))
        .add_capability(Capability::Model("Example".to_owned()));

    if let Ok((mut proto_stream, mut proto_sink)) = proto.connect().await {
        while let Some(Ok(msg)) = proto_stream.next().await {
            println!("{:?}", msg);
            match msg {
                ServerMessage::Status(timestamp) => {
                    let statmsg = make_heartbeat(timestamp);
                    if let Err(_) = proto_sink.send(statmsg).await {
                        break;
                    }
                }
                _ => {}
            }
        }
    }
}
