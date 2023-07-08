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
        println!("Server Address: {:?}", server.ip_address);
        println!("Server Port: {}", server.port);
        if let Some(sgid) = server.sync_group_id {
            println!("Sync Group: {}", sgid);
        }

        if server.tlv_map.len() > 0 {
            println!("TLV responses:");
            for val in server.tlv_map.values() {
                println!("\t{:?}", val);
            }
        }
    } else {
        println!("No response from server")
    }
}
