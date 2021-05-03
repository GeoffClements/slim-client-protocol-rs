/// Example of creating a protocol object and receiveing messages
/// We create the object and add capabilities.
/// The protocol object connects to the server and announces
/// itself with a HELO message. We then print all the messages
/// from the server.
///
/// The server will eventually disconnect because we are not
/// responding to any of the status messages.
use futures::StreamExt;
use slimproto::{Capability, SlimProto};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut proto = SlimProto::new();
    proto
        .add_capability(Capability::Modelname("Example".to_owned()))
        .add_capability(Capability::Model("Example".to_owned()));

    if let Ok((mut proto_stream, _, _)) = proto.connect().await {
        while let Some(Ok(msg)) = proto_stream.next().await {
            println!("{:?}", msg);
        }
    }
}
