use std::{
    io::Write,
    net::{Ipv4Addr, TcpStream},
    sync::{Arc, RwLock},
};

use slimproto::{
    buffer::SlimBuffer,
    discovery::discover,
    proto::Server,
    status::{StatusCode, StatusData},
    Capabilities, Capability, ClientMessage, FramedReader, FramedWriter, ServerMessage,
};

use crossbeam::channel::Sender;
use symphonia::core::{
    formats::FormatOptions,
    io::{MediaSourceStream, ReadOnlySource},
    meta::MetadataOptions,
    probe::Hint,
};

fn main() -> anyhow::Result<()> {
    // Set up variables needed by the Slim protocol
    let mut server = Server::default();
    let name: Arc<RwLock<String>> = Arc::new(RwLock::new("Slimproto_player".to_string()));
    let status = Arc::new(RwLock::new(StatusData::default()));
    let (slim_tx_in, slim_tx_out) = crossbeam::channel::bounded(1);
    let (slim_rx_in, slim_rx_out) = crossbeam::channel::bounded(1);

    // Start the Slim protocol thread
    // Runs forever
    let name_r = name.clone();
    std::thread::spawn(move || {
        let mut server = match discover(None) {
            Ok(Some(server)) => server,
            _ => {
                return;
            }
        };

        slim_rx_in.send(ServerMessage::Serv { ip_address: Ipv4Addr::from(server.ip_address), sync_group_id: None }).ok();

        loop {
            let name = match name_r.read() {
                Ok(name) => name,
                Err(_) => {
                    return;
                }
            };
            let mut caps = Capabilities::default();
            caps.add_name(&name);
            caps.add(Capability::Maxsamplerate(192000));
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
                        slim_rx_in
                            .send(ServerMessage::Serv {
                                ip_address: ip,
                                sync_group_id: None,
                            })
                            .ok();
                        break;
                    }
                    _ => {
                        slim_rx_in.send(msg).ok();
                    }
                }
            }
        }
    });

    // Main thread Slim protocol loop
    while let Ok(msg) = slim_rx_out.recv() {
        println!("{:?}", msg);
        match msg {
            ServerMessage::Serv { ip_address, .. } => {
                server = (ip_address, None).into();
            }
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
                if let Ok(mut status) = status.write() {
                    status.set_timestamp(ts);
                    let msg = status.make_status_message(StatusCode::Timer);
                    slim_tx_in.send(msg).ok();
                }
            }

            ServerMessage::Stream {
                // autostart,
                format,
                // pcmsamplesize,
                // pcmsamplerate,
                // pcmchannels,
                // pcmendian,
                threshold,
                // spdif_enable,
                // trans_period,
                // trans_type,
                // flags,
                // output_threshold,
                // replay_gain,
                server_port,
                server_ip,
                http_headers,
                ..
            } => {
                if let Some(http_headers) = http_headers {
                    let num_crlf = http_headers.matches("\r\n").count();

                    if num_crlf > 0 {
                        if let Ok(mut status) = status.write() {
                            status.add_crlf(num_crlf as u8);
                        }

                        play_stream(
                            slim_tx_in.clone(),
                            status.clone(),
                            //     autostart,
                            format,
                            //     pcmsamplesize,
                            //     pcmsamplerate,
                            //     pcmchannels,
                            //     pcmendian,
                            threshold,
                            //     spdif_enable,
                            //     trans_period,
                            //     trans_type,
                            //     flags,
                            //     output_threshold,
                            //     replay_gain,
                            server_port,
                            server_ip,
                            http_headers,
                            &server,
                        )?
                    }
                }
            }

            _ => {}
        }
    }
    Ok(())
}

fn play_stream(
    slim_tx: Sender<ClientMessage>,
    status: Arc<RwLock<StatusData>>,
    // autostart: slimproto::proto::AutoStart,
    format: slimproto::proto::Format,
    // pcmsamplesize: slimproto::proto::PcmSampleSize,
    // pcmsamplerate: slimproto::proto::PcmSampleRate,
    // pcmchannels: slimproto::proto::PcmChannels,
    // pcmendian: slimproto::proto::PcmEndian,
    threshold: u32,
    // spdif_enable: slimproto::proto::SpdifEnable,
    // trans_period: Duration,
    // trans_type: slimproto::proto::TransType,
    // flags: slimproto::proto::StreamFlags,
    // output_threshold: Duration,
    // replay_gain: f64,
    server_port: u16,
    server_ip: Ipv4Addr,
    http_headers: String,
    server: &Server,
) -> anyhow::Result<()> {
    let ip = if server_ip == Ipv4Addr::new(0, 0, 0, 0) {
        server.ip_address
    } else {
        server_ip
    };

    let mut data_stream = TcpStream::connect((ip, server_port))?;
    data_stream.write(http_headers.as_bytes())?;
    data_stream.flush().ok();

    if let Ok(status) = status.read() {
        let msg = status.make_status_message(StatusCode::Connect);
        slim_tx.send(msg).ok();
    }

    let mss = MediaSourceStream::new(
        Box::new(ReadOnlySource::new(SlimBuffer::with_capacity(
            threshold as usize * 1024,
            data_stream,
            status.clone(),
        ))),
        Default::default(),
    );

    // Create a hint to help the format registry guess what format reader is appropriate.
    let mut hint = Hint::new();
    hint.mime_type({
        match format {
            slimproto::proto::Format::Pcm => "audio/x-adpcm",
            slimproto::proto::Format::Mp3 => "audio/mpeg3",
            slimproto::proto::Format::Aac => "audio/aac",
            slimproto::proto::Format::Ogg => "audio/ogg",
            slimproto::proto::Format::Flac => "audio/flac",
            _ => "",
        }
    });

    // Use the default options for format readers other and enable gapless playback.
    let format_opts = FormatOptions {
        enable_gapless: true,
        ..Default::default()
    };

    // Use the default options for metadata readers.
    let metadata_opts: MetadataOptions = Default::default();

    let mut probed =
        match symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts) {
            Ok(probed) => probed,
            Err(_) => {
                if let Ok(status) = status.read() {
                    let msg = status.make_status_message(StatusCode::NotSupported);
                    slim_tx.send(msg).ok();
                }
                return Ok(());
            }
        };

    if let Some(mut metadata) = probed.metadata.get() {
        if let Some(metadata) = metadata.skip_to_latest() {
            println!("Now playing:");
            for tag in metadata.tags() {
                println!("{}: {}", tag.key, tag.value);
            }
        }
    }

    // Create a decoder for the stream.
    // let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, decode_opts)?;

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
