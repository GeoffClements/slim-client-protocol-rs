/*
 Player
*/

use slimproto::{
    discovery::discover,
    status::{StatusCode, StatusData},
    Capabilities, ClientMessage, FramedReader, FramedWriter, ServerMessage,
};

use crossbeam::{channel::unbounded, select};

use std::time::Duration;

fn main() {
    let (slim_tx, slim_rx) = unbounded();

    // The slim protocol loop
    std::thread::spawn(move || {
        if let Some(mut server) = discover(Some(Duration::from_secs(10))).unwrap() {
            loop {
                // Add some capabilities
                let mut caps = Capabilities::default();
                caps.add_name("Example");

                // Prepare the server object with the capabilities and then connect
                let (mut rx, mut tx) = server.prepare(caps).connect().unwrap();

                // React to messages from the server
                while let Ok(msg) = rx.framed_read() {
                    match msg {
                        // Request to change to another server
                        ServerMessage::Serv {
                            ip_address: ip,
                            sync_group_id: sgid,
                        } => {
                            server = (ip, sgid).into();
                            break;
                        }
                        _ => {
                            slim_tx.send(msg).ok();
                        }
                    }
                }
            }
        }
    });

    // Set up the audio server
    // let main_loop = MainLoop::new().unwrap();
    // let context = Context::new(&main_loop).unwrap();
    // let core = context.connect(None).unwrap();

    // Idle waiting for messages
    loop {
        select! {
            recv(slim_rx)-> slim_msg => {
                if let Ok(msg) = slim_msg {
                    println!("{:?}", msg);
                }
            }
            default(Duration::from_secs(10)) => {
                println!("Timed out");
                break;
            }
        }
    }
}
