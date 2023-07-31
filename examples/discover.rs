use slimproto::discovery::discover;
use std::time::Duration;

fn main() {
    let slim_discover = match discover(Some(Duration::from_secs(3))) {
        Ok(response) => response,
        Err(e) => {
            println!("{:?}", e);
            return;
        }
    };

    if let Some(server) = slim_discover {
        println!("Server Address: {:?}", server.socket.ip());
        println!("Server Port: {}", server.socket.port());
        if let Some(sgid) = server.sync_group_id {
            println!("Sync Group: {}", sgid);
        }

        if let Some(tlv_map) = server.tlv_map {
            if tlv_map.len() > 0 {
                println!("TLV responses:");
                for val in tlv_map.values() {
                    println!("\t{:?}", val);
                }
            }
        }
    } else {
        println!("No response from server")
    }
}
