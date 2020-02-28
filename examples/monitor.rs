use slimproto::SlimProtoBuilder;

#[tokio::main]
async fn main() {
    let proto = SlimProtoBuilder::new()
        .flc(true)
        .mp3(true)
        .pcm(true)
        .model("rusty")
        .modelname("Example")
        .build(true)
        .await;
}
