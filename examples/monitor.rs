use tokio::stream::StreamExt;
use futures::sink::SinkExt;

use slimproto::{ClientMessage, ServerMessage, SlimProtoBuilder};

#[tokio::main]
async fn main() {
    let mut proto = SlimProtoBuilder::new()
        .flc(true)
        .mp3(true)
        .pcm(true)
        .model("rusty")
        .modelname("Example")
        .build(true)
        .await
        .unwrap();

    while let Some(msg) = proto.next().await {
        println!("{:?}", msg);

        match msg {
            Ok(ServerMessage::Queryname) => {
                if let Some(name) = proto.modelname.clone() {
                    tokio::spawn(proto.send(ClientMessage::Name(name.to_owned())));
                }
            },
            _ => {},
        }
    }
}
