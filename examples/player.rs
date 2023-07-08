/*
 Player
*/

use slimproto::{
    discovery::discover,
    status::{StatusCode, StatusData},
    Capabilities, ClientMessage, ServerMessage,
};

use std::time::Duration;

fn main() {
    // Set up the audio server

    // The slim protocol loop
    if let Some(mut server) = discover(Some(Duration::from_secs(10))).unwrap() {
        loop {
            // Add some capabilities
            let mut caps = Capabilities::default();
            caps.add_name("Example");

            // Prepare the server object with the capabilities and then connect
            let (mut rx, mut tx) = server.prepare(caps).connect().unwrap();

            let mut client_name = String::from("Example_Player");

            // Make the status data so that we can respond to the server ticks
            let mut status = StatusData::new(0, 0);

            // React to messages from the server
            while let Ok(msg) = rx.recv() {
                println!("{:?}", msg);
                match msg {
                    // Server wants to know our name
                    ServerMessage::Queryname => tx
                        .send(ClientMessage::Name(String::from(&client_name)))
                        .unwrap(),
                    // Server wants to set our name
                    ServerMessage::Setname(name) => {
                        client_name = name;
                    }
                    // Status tick from the server, respond with updated status data
                    ServerMessage::Status(ts) => {
                        status.set_timestamp(ts);
                        let msg = status.make_status_message(StatusCode::Timer);
                        tx.send(msg).unwrap();
                    }
                    // Request to change to another server
                    ServerMessage::Serv {
                        ip_address: ip,
                        sync_group_id: sgid,
                    } => {
                        server = (ip, sgid).into();
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}
