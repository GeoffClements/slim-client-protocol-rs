use slimproto::discovery::discover;
use std::time::Duration;
use tokio;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let slim_discover = match discover(Some(Duration::from_secs(3))).await {
        Ok(response) => response,
        Err(e) => {
            println!("{:?}", e);
            return;
        }
    };

    if let Some((addr, slim_tlvs)) = slim_discover {
        println!("Server Address: {:?}", addr);

        if slim_tlvs.len() > 0 {
            println!("TLV responses:");
            for val in slim_tlvs.values() {
                println!("{:?}", val);
            }
        }
    } else {
        println!("No response from server")
    }
}
