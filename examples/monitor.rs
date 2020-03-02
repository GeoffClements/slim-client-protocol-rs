use tokio::stream::StreamExt;
use futures::sink::SinkExt;

use slimproto::{ClientMessage, ServerMessage, SlimProtoBuilder, StatData};

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

    // let stat = ClientMessage::Stat {
    //     event_code: "STMt".to_owned(),
    //     StatData::default(),
    // }
    

    while let Some(msg) = proto.next().await {
        println!("{:?}", msg);

        match msg {
            Ok(ServerMessage::Queryname) => {
                if let Some(name) = proto.modelname.clone() {
                    let _ = proto.send(ClientMessage::Name(name.to_owned())).await;
                }
            },

            // Ok(ServerMessage::Status) => {
            //     let _ = proto.send(ClientMessage::Stat).await;
            // },

            _ => {},
        }
    }
}
