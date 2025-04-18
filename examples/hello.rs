/**
 Requires a Slim server on the network!

 Example of discovering a server, saying hello and receiving
 messages.

 We create minimal capabilities to send to the server.

 We then connect to the server and announce our presence
 with a HELO message. We then print all the messages
 from the server as they arrive.

 The server will eventually disconnect because we are not
 responding to any of the status messages so we use a timeout
 to quit.
*/
use slimproto::{discovery::discover, Capabilities, FramedReader};
use std::time::Duration;

fn main() {
    // Spawn a main thread to allow a timeout
    std::thread::spawn(|| {
        // Discover server
        if let Some(server) = discover(Some(Duration::from_secs(10))).unwrap() {
            // Add some minimal capabilities
            let caps = Capabilities::default();

            // Prepare the server object with the capabilities and then connect
            let (mut rx, _tx) = server.prepare(caps).connect().unwrap();

            // Print messages as we receive them
            while let Ok(msg) = rx.framed_read() {
                println!("{:?}", msg);
            }
        }
    });
    std::thread::sleep(Duration::from_secs(10));
}
