/*
 Example of creating a protocol object and receiving messages
 doing just enough to keep the client alive and to give the
 server a meaningful name.

 We discover the server and add capabilities.

 The client connects to the server and announces
 itself with a HELO message. We then print all the messages
 from the server and react to a few message types.

 Note that we must respond to the server with a periodic
 status message to stop being ignored.

 Unlike the hello example this will respond to status requests
 with a status message and this will go on forever, doing
 nothing much so you will need to kill this process.
*/

use slimproto::{
    discovery::discover,
    status::{StatusCode, StatusData},
    Capabilities, ClientMessage, ServerMessage, FramedReader, FramedWriter,
};

use std::time::Duration;

fn main() {
    if let Some(server) = discover(Some(Duration::from_secs(10))).unwrap() {
        // Add some capabilities
        let caps = Capabilities::default();

        // Prepare the server object with the capabilities and then connect
        let (mut rx, mut tx) = server.prepare(caps).connect().unwrap();

        let mut client_name = String::from("BoringExample");

        // Make the status data so that we can respond to the server ticks
        let mut status = StatusData::default();

        // React to messages from the server
        while let Ok(msg) = rx.framed_read() {
            println!("{:?}", msg);
            match msg {
                // Server wants to know our name
                ServerMessage::Queryname => tx
                    .framed_write(ClientMessage::Name(String::from(&client_name)))
                    .unwrap(),
                // Server wants to set our name
                ServerMessage::Setname(name) => {
                    client_name = name;
                }
                // Status tick from the server, respond with updated status data
                ServerMessage::Status(ts) => {
                    status.set_timestamp(ts);
                    let msg = status.make_status_message(StatusCode::Timer);
                    tx.framed_write(msg).unwrap();
                }
                _ => {}
            }
        }
    }
}
