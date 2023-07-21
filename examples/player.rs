/*
 Player
*/

use std::sync::{Arc, RwLock};

use slimproto::{
    discovery::discover,
    status::{StatusCode, StatusData},
    Capabilities, Capability, ClientMessage, FramedReader, FramedWriter, ServerMessage,
};

// use symphonia::{
//     core::{
//         codecs::CodecType,
//         formats::FormatOptions,
//         io::{MediaSource, MediaSourceStream, ReadOnlySource},
//         meta::MetadataOptions,
//         probe::Hint,
//     },
//     default::get_codecs,
// };

fn main() -> anyhow::Result<()> {
    let name: Arc<RwLock<String>> = Arc::new(RwLock::new("Slimproto_player".to_string()));
    let mut status = StatusData::default();

    let (slim_tx_in, slim_tx_out) = crossbeam::channel::bounded(1);
    let (slim_rx_in, slim_rx_out) = crossbeam::channel::bounded(1);

    // Slim protocol thread
    // Runs forever
    let name_r = name.clone();
    std::thread::spawn(move || {
        let mut server = match discover(None) {
            Ok(Some(server)) => server,
            _ => {
                return;
            }
        };

        loop {
            let name = match name_r.read() {
                Ok(name) => name,
                Err(_) => {
                    return;
                }
            };
            let mut caps = Capabilities::default();
            caps.add_name(&name);
            caps.add(Capability::Pcm);
            caps.add(Capability::Mp3);
            caps.add(Capability::Aac);
            caps.add(Capability::Ogg);
            caps.add(Capability::Flc);

            // Connect to the server
            let (mut rx, mut tx) = match server.clone().prepare(caps).connect() {
                Ok((rx, tx)) => (rx, tx),
                Err(_) => {
                    return;
                }
            };

            // Start write thread
            // Continues until connection is dropped
            let slim_tx_out_r = slim_tx_out.clone();
            std::thread::spawn(move || {
                while let Ok(msg) = slim_tx_out_r.recv() {
                    // println!("{:?}", msg);
                    if tx.framed_write(msg).is_err() {
                        return;
                    }
                }
            });

            // Read loop
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
                        slim_rx_in.send(msg).ok();
                    }
                }
            }
        }
    });

    // Main Slim protocol loop
    while let Ok(msg) = slim_rx_out.recv() {
        println!("{:?}", msg);
        match msg {
            ServerMessage::Queryname => {
                if let Ok(name) = name.read() {
                    slim_tx_in
                        .send(ClientMessage::Name((*name).to_owned()))
                        .ok();
                }
            }

            ServerMessage::Setname(new_name) => {
                if let Ok(mut name) = name.write() {
                    *name = new_name;
                }
            }

            ServerMessage::Status(ts) => {
                status.set_timestamp(ts);
                let msg = status.make_status_message(StatusCode::Timer);
                slim_tx_in.send(msg).ok();
            }

            ServerMessage::Stream {
                // autostart,
                // format,
                // pcmsamplesize,
                // pcmsamplerate,
                // pcmchannels,
                // pcmendian,
                // threshold,
                // spdif_enable,
                // trans_period,
                // trans_type,
                // flags,
                // output_threshold,
                // replay_gain,
                // server_port,
                // server_ip,
                http_headers,
                ..
            } => {
                if let Some(_http_headers) = http_headers {
                    play_stream(
                        //     &mut tx,
                        //     &status,
                        //     autostart,
                        //     format,
                        //     pcmsamplesize,
                        //     pcmsamplerate,
                        //     pcmchannels,
                        //     pcmendian,
                        //     threshold,
                        //     spdif_enable,
                        //     trans_period,
                        //     trans_type,
                        //     flags,
                        //     output_threshold,
                        //     replay_gain,
                        //     server_port,
                        //     server_ip,
                        //     http_headers,
                        )?
                }
            }

            _ => {}
        }
    }
    Ok(())
}

fn play_stream(// tx: &mut FramedWrite<BufWriter<TcpStream>, slimproto::codec::SlimCodec>,
    // status: &StatusData,
    // autostart: slimproto::proto::AutoStart,
    // format: slimproto::proto::Format,
    // pcmsamplesize: slimproto::proto::PcmSampleSize,
    // pcmsamplerate: slimproto::proto::PcmSampleRate,
    // pcmchannels: slimproto::proto::PcmChannels,
    // pcmendian: slimproto::proto::PcmEndian,
    // threshold: u32,
    // spdif_enable: slimproto::proto::SpdifEnable,
    // trans_period: Duration,
    // trans_type: slimproto::proto::TransType,
    // flags: slimproto::proto::StreamFlags,
    // output_threshold: Duration,
    // replay_gain: f64,
    // server_port: u16,
    // server_ip: Ipv4Addr,
    // http_headers: String,
) -> anyhow::Result<()> {
    // let msg = status.make_status_message(StatusCode::Connect);
    // tx.framed_write(msg).ok();

    // let mut stream = TcpStream::connect((server_ip, server_port))?;
    // stream.write(http_headers.as_bytes())?;
    // stream.flush().ok();

    // let mut hint = Hint::new();
    // let mss = MediaSourceStream::new(Box::new(ReadOnlySource::new(stream)), Default::default());

    // let format_opts = FormatOptions {
    //     enable_gapless: true,
    //     ..Default::default()
    // };

    // let metadata_opts: MetadataOptions = Default::default();
    // match symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts) {
    //     Ok(probed) => {}
    //     Err(_) => {
    //         let msg = status.make_status_message(StatusCode::NotSupported);
    //         tx.framed_write(msg).ok();
    //     }
    // }
    Ok(())
}
