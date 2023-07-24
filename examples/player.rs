use std::{
    cell::RefCell,
    io::Write,
    net::{Ipv4Addr, TcpStream},
    rc::Rc,
    sync::{Arc, RwLock},
};

use libpulse_binding::{self as pa, context::Context, stream::Stream};
use pa::sample::Spec;
use slimproto::{
    buffer::SlimBuffer,
    discovery::discover,
    proto::{PcmChannels, PcmEndian, PcmSampleRate, PcmSampleSize, Server},
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
    // Set up pulse audio
    // We need control of the output with stop and pause etc.,
    // so we have to use the threaded version
    let (ml, cx) = pulse::setup()?;

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

        slim_rx_in
            .send(ServerMessage::Serv {
                ip_address: Ipv4Addr::from(server.ip_address),
                sync_group_id: None,
            })
            .ok();

        // Outer loop to reconnect to a different server and
        // update server details when a Serv message is received
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

            // Inner read loop
            while let Ok(msg) = rx.framed_read() {
                match msg {
                    // Request to change to another server
                    ServerMessage::Serv {
                        ip_address: ip,
                        sync_group_id: sgid,
                    } => {
                        server = (ip, sgid).into();
                        // Now inform the main thread
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
                pcmsamplesize,
                pcmsamplerate,
                pcmchannels,
                pcmendian,
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
                            pcmsamplesize,
                            pcmsamplerate,
                            pcmchannels,
                            pcmendian,
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
                            cx.clone(),
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
    pcmsamplesize: slimproto::proto::PcmSampleSize,
    pcmsamplerate: slimproto::proto::PcmSampleRate,
    pcmchannels: slimproto::proto::PcmChannels,
    pcmendian: slimproto::proto::PcmEndian,
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
    cx: Rc<RefCell<Context>>,
) -> anyhow::Result<()> {
    // The LMS sends an ip of 0, 0, 0, 0 when it wants us to default to it
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

    let track = match probed.format.default_track() {
        Some(track) => track,
        None => {
            if let Ok(status) = status.read() {
                let msg = status.make_status_message(StatusCode::NotSupported);
                slim_tx.send(msg).ok();
            }
            return Ok(());
        }
    };

    // Set pa sample format
    let sample_format = match (pcmsamplesize, pcmendian) {
        (PcmSampleSize::Eight, _) => pa::sample::Format::U8,
        (PcmSampleSize::Sixteen, PcmEndian::Big) => pa::sample::Format::S16be,
        (PcmSampleSize::Sixteen, PcmEndian::Little) => pa::sample::Format::S16le,
        (PcmSampleSize::ThirtyTwo, PcmEndian::Big) => pa::sample::Format::S32be,
        (PcmSampleSize::ThirtyTwo, PcmEndian::Little) => pa::sample::Format::S32le,
        (PcmSampleSize::SelfDescribing, _) => {
            let sample_format = track
                .codec_params
                .sample_format
                .unwrap_or(symphonia::core::sample::SampleFormat::F64);
            match sample_format {
                symphonia::core::sample::SampleFormat::U8 => pa::sample::Format::U8,
                symphonia::core::sample::SampleFormat::S16 => pa::sample::Format::S16NE,
                symphonia::core::sample::SampleFormat::S32 => pa::sample::Format::S32NE,
                symphonia::core::sample::SampleFormat::F32 => pa::sample::Format::FLOAT32NE,
                _ => pa::sample::Format::Invalid,
            }
        }
        _ => pa::sample::Format::Invalid,
    };

    let sample_rate = match pcmsamplerate {
        PcmSampleRate::Rate(rate) => rate,
        PcmSampleRate::SelfDescribing => track.codec_params.sample_rate.unwrap_or(44100),
    };

    let channels = match pcmchannels {
        PcmChannels::Mono => 1u8,
        PcmChannels::Stereo => 2,
        PcmChannels::SelfDescribing => match track.codec_params.channel_layout {
            Some(symphonia::core::audio::Layout::Mono) => 1,
            Some(symphonia::core::audio::Layout::Stereo) => 2,
            _ => 0,
        },
    };

    // Create a spec for the pa stream
    let spec = Spec {
        format: sample_format,
        rate: sample_rate,
        channels,
    };

    // Create a pulseaudio stream
    let pa_stream = Rc::new(RefCell::new(
        match Stream::new(&mut cx.borrow_mut(), "Music", &spec, None) {
            Some(stream) => stream,
            None => {
                if let Ok(status) = status.read() {
                    let msg = status.make_status_message(StatusCode::NotSupported);
                    slim_tx.send(msg).ok();
                }
                return Ok(());
            }
        }
    ));

    // Add callback to pa_stream to feed music
    

    Ok(())
}

mod pulse {
    use std::{cell::RefCell, ops::Deref, rc::Rc};

    use libpulse_binding::{
        self as pa,
        context::{Context, FlagSet as CxFlagSet},
        error::PAErr,
        mainloop::threaded::Mainloop,
        stream::{FlagSet as SmFlagSet, Stream},
    };

    pub fn setup() -> Result<(Rc<RefCell<Mainloop>>, Rc<RefCell<Context>>), PAErr> {
        let ml = Rc::new(RefCell::new(
            Mainloop::new().ok_or(pa::error::Code::ConnectionRefused)?,
        ));

        let cx = Rc::new(RefCell::new(
            Context::new(ml.borrow_mut().deref(), "Slimproto_example")
                .ok_or(pa::error::Code::ConnectionRefused)?,
        ));

        // Context state change callback
        {
            let ml_ref = ml.clone();
            let cx_ref = cx.clone();
            cx.borrow_mut().set_state_callback(Some(Box::new(move || {
                let state = unsafe { (*cx_ref.as_ptr()).get_state() };
                match state {
                    pa::context::State::Ready
                    | pa::context::State::Terminated
                    | pa::context::State::Failed => unsafe {
                        (*ml_ref.as_ptr()).signal(false);
                    },
                    _ => {}
                }
            })))
        }

        cx.borrow_mut().connect(None, CxFlagSet::NOFLAGS, None)?;
        ml.borrow_mut().lock();
        ml.borrow_mut().start()?;

        // Wait for context to be ready
        loop {
            match cx.borrow().get_state() {
                pa::context::State::Ready => {
                    break;
                }
                pa::context::State::Failed | pa::context::State::Terminated => {
                    ml.borrow_mut().unlock();
                    ml.borrow_mut().stop();
                    return Err(pa::error::PAErr(
                        pa::error::Code::ConnectionTerminated as i32,
                    ));
                }
                _ => ml.borrow_mut().wait(),
            }
        }

        cx.borrow_mut().set_state_callback(None);
        ml.borrow_mut().unlock();

        Ok((ml, cx))
    }

    pub fn connect_stream(ml: Rc<RefCell<Mainloop>>, sm: Rc<RefCell<Stream>>) -> Result<(), PAErr> {
        ml.borrow_mut().lock();

        // Stream state change callback
        {
            let ml_ref = ml.clone();
            let sm_ref = sm.clone();
            sm.borrow_mut().set_state_callback(Some(Box::new(move || {
                let state = unsafe { (*sm_ref.as_ptr()).get_state() };
                match state {
                    pa::stream::State::Ready
                    | pa::stream::State::Failed
                    | pa::stream::State::Terminated => unsafe {
                        (*ml_ref.as_ptr()).signal(false);
                    },
                    _ => {}
                }
            })));
        }

        sm.borrow_mut()
            .connect_playback(None, None, SmFlagSet::NOFLAGS, None, None)?;

        // Wait for stream to be ready
        loop {
            match sm.borrow_mut().get_state() {
                pa::stream::State::Ready => {
                    break;
                }
                pa::stream::State::Failed | pa::stream::State::Terminated => {
                    ml.borrow_mut().unlock();
                    ml.borrow_mut().stop();
                    return Err(pa::error::PAErr(
                        pa::error::Code::ConnectionTerminated as i32,
                    ));
                }
                _ => {
                    ml.borrow_mut().wait();
                }
            }
        }

        sm.borrow_mut().set_state_callback(None);
        ml.borrow_mut().unlock();

        Ok(())
    }
}
