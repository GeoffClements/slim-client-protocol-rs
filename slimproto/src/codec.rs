use bytes::{buf::BufMut, Buf, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::{
    proto::{
        AutoStart, Format, PcmChannels, PcmEndian, PcmSampleRate, PcmSampleSize, SpdifEnable,
        StreamFlags, TransType,
    },
    ClientMessage, ServerMessage,
};

use std::{convert::TryInto, io, net::Ipv4Addr, time::Duration};

pub struct SlimCodec;

impl Encoder for SlimCodec {
    type Item = ClientMessage;
    type Error = io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.extend(BytesMut::from(item));
        Ok(())
    }
}

impl Decoder for SlimCodec {
    type Item = ServerMessage;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<ServerMessage>> {
        if buf.len() <= 2 {
            return Ok(None);
        };

        let frame_size = u16::from_be_bytes(buf[..2].try_into().unwrap()) as usize;

        if buf.len() < frame_size + 2 {
            if buf.capacity() < frame_size + 2 {
                buf.reserve(frame_size);
            }
            return Ok(None);
        };

        let _ = buf.split_to(2);
        let msg = buf.split_to(frame_size);

        match msg.into() {
            ServerMessage::Error => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Server data corrupted",
            )),
            msg @ _ => Ok(Some(msg)),
        }
    }
}

impl From<ClientMessage> for BytesMut {
    fn from(src: ClientMessage) -> BytesMut {
        const FRAMESIZE: usize = 1024;

        let mut msg = Vec::with_capacity(FRAMESIZE + 2);
        let mut frame_size = Vec::with_capacity(2);
        let mut frame = Vec::with_capacity(FRAMESIZE);

        match src {
            ClientMessage::Helo {
                device_id,
                revision,
                mac,
                uuid,
                wlan_channel_list,
                bytes_received,
                capabilities,
            } => {
                msg.put("HELO".as_bytes());
                frame.put_u8(device_id);
                frame.put_u8(revision);
                frame.put(mac.bytes().as_ref());
                frame.put(uuid.as_ref());
                frame.put_u16(wlan_channel_list);
                frame.put_u64(bytes_received);
                frame.put(capabilities.as_bytes());
            }

            ClientMessage::Bye(val) => {
                msg.put("BYE!".as_bytes());
                frame.put_u8(val);
            }
            ClientMessage::Stat {
                event_code,
                stat_data,
            } => {
                msg.put("STAT".as_bytes());
                frame.put(event_code.as_bytes());
                frame.put_u8(stat_data.crlf);
                frame.put_u16(0);
                frame.put_u32(stat_data.buffer_size);
                frame.put_u32(stat_data.fullness);
                frame.put_u64(stat_data.bytes_received);
                frame.put_u16(stat_data.sig_strength);
                frame.put_u32(stat_data.jiffies);
                frame.put_u32(stat_data.output_buffer_size);
                frame.put_u32(stat_data.output_buffer_fullness);
                frame.put_u32(stat_data.elapsed_seconds);
                frame.put_u16(stat_data.voltage);
                frame.put_u32(stat_data.elapsed_milliseconds);
                frame.put_u32(stat_data.timestamp);
                frame.put_u16(stat_data.error_code);
            }

            ClientMessage::Name(name) => {
                msg.put("SETD".as_bytes());
                frame.put_u8(0);
                frame.put(name.as_bytes());
            }
        }

        frame_size.put_u32(frame.len() as u32);
        msg.append(&mut frame_size);
        msg.append(&mut frame);

        msg.as_slice().into()
    }
}

impl From<BytesMut> for ServerMessage {
    fn from(mut src: BytesMut) -> ServerMessage {
        const GAIN_FACTOR: f64 = 65536.0;

        let msg: String = src.split_to(4).into_iter().map(|c| c as char).collect();
        let mut buf = src.split();

        match msg.as_str() {
            "serv" => {
                if buf.len() < 4 {
                    return ServerMessage::Error;
                }

                let ip_addr = Ipv4Addr::from(buf.split_to(4).get_u32());
                let sync_group = if buf.len() > 0 {
                    Some(buf.into_iter().map(|c| c as char).collect::<String>())
                } else {
                    None
                };
                ServerMessage::Serv {
                    ip_address: ip_addr,
                    sync_group_id: sync_group,
                }
            }

            "strm" => {
                if buf.len() < 24 {
                    return ServerMessage::Error;
                }

                match buf.split_to(1)[0] as char {
                    't' => {
                        let _ = buf.split_to(14);
                        let timestamp = buf.get_u32();
                        ServerMessage::Status(timestamp)
                    }

                    's' => {
                        let autostart = match buf.split_to(1)[0] as char {
                            '0' => AutoStart::None,
                            '1' => AutoStart::Auto,
                            '2' => AutoStart::Direct,
                            '3' => AutoStart::AutoDirect,
                            _ => return ServerMessage::Error,
                        };

                        let format = match buf.split_to(1)[0] as char {
                            'p' => Format::Pcm,
                            'm' => Format::Mp3,
                            'f' => Format::Flac,
                            'w' => Format::Wma,
                            'o' => Format::Ogg,
                            'a' => Format::Aac,
                            'l' => Format::Alac,
                            _ => return ServerMessage::Error,
                        };

                        let pcmsamplesize = match buf.split_to(1)[0] as char {
                            '0' => PcmSampleSize::Eight,
                            '1' => PcmSampleSize::Sixteen,
                            '2' => PcmSampleSize::Twenty,
                            '3' => PcmSampleSize::ThirtyTwo,
                            '?' => PcmSampleSize::SelfDescribing,
                            _ => return ServerMessage::Error,
                        };

                        let pcmsamplerate = match buf.split_to(1)[0] as char {
                            '0' => PcmSampleRate::Rate(11_000),
                            '1' => PcmSampleRate::Rate(22_000),
                            '2' => PcmSampleRate::Rate(32_000),
                            '3' => PcmSampleRate::Rate(44_100),
                            '4' => PcmSampleRate::Rate(48_000),
                            '5' => PcmSampleRate::Rate(8_000),
                            '6' => PcmSampleRate::Rate(12_000),
                            '7' => PcmSampleRate::Rate(16_000),
                            '8' => PcmSampleRate::Rate(24_000),
                            '9' => PcmSampleRate::Rate(96_000),
                            '?' => PcmSampleRate::SelfDescribing,
                            _ => return ServerMessage::Error,
                        };

                        let pcmchannels = match buf.split_to(1)[0] as char {
                            '1' => PcmChannels::Mono,
                            '2' => PcmChannels::Stereo,
                            '?' => PcmChannels::SelfDescribing,
                            _ => return ServerMessage::Error,
                        };

                        let pcmendian = match buf.split_to(1)[0] as char {
                            '0' => PcmEndian::Big,
                            '1' => PcmEndian::Little,
                            '?' => PcmEndian::SelfDescribing,
                            _ => return ServerMessage::Error,
                        };

                        let threshold = buf.split_to(1)[0] as u32 * 1024u32;

                        let spdif_enable = match buf.split_to(1)[0] as char {
                            '0' => SpdifEnable::Auto,
                            '1' => SpdifEnable::On,
                            '2' => SpdifEnable::Off,
                            _ => return ServerMessage::Error,
                        };

                        let trans_period = Duration::from_secs(buf.split_to(1)[0] as u64);

                        let trans_type = match buf.split_to(1)[0] as char {
                            '0' => TransType::None,
                            '1' => TransType::Crossfade,
                            '2' => TransType::FadeIn,
                            '3' => TransType::FadeOut,
                            '4' => TransType::FadeInOut,
                            _ => return ServerMessage::Error,
                        };

                        let flags = StreamFlags::from_bits(buf.split_to(1)[0])
                            .unwrap_or(StreamFlags::empty());

                        let output_threshold = Duration::from_millis(buf.split_to(1)[0] as u64);

                        let _ = buf.split_to(1);

                        let replay_gain = buf.split_to(4).get_u32() as f64 / GAIN_FACTOR;

                        let server_port = buf.split_to(2).get_u16();

                        let server_ip = Ipv4Addr::from(buf.split_to(4).get_u32());

                        let http_headers = if buf.len() > 0 {
                            buf[..].into_iter().map(|c| *c as char).collect()
                        } else {
                            String::new()
                        };

                        ServerMessage::Stream {
                            autostart,
                            format,
                            pcmsamplesize,
                            pcmsamplerate,
                            pcmchannels,
                            pcmendian,
                            threshold,
                            spdif_enable,
                            trans_period,
                            trans_type,
                            flags,
                            output_threshold,
                            replay_gain,
                            server_port,
                            server_ip,
                            http_headers,
                        }
                    }

                    'q' => ServerMessage::Stop,

                    'p' => {
                        let _ = buf.split_to(14);
                        let timestamp = buf.get_u32();
                        ServerMessage::Pause(timestamp)
                    }

                    'u' => {
                        let _ = buf.split_to(14);
                        let timestamp = buf.get_u32();
                        ServerMessage::Unpause(timestamp)
                    }

                    'a' => {
                        let _ = buf.split_to(14);
                        let timestamp = buf.get_u32();
                        ServerMessage::Skip(timestamp)
                    }

                    cmd @ _ => {
                        let mut msg = msg.to_owned();
                        msg.push('_');
                        msg.push(cmd);
                        ServerMessage::Unrecognised(msg)
                    }
                }
            }

            "aude" => {
                if buf.len() < 2 {
                    return ServerMessage::Error;
                }

                let (spdif, dac) = (buf[0] != 0, buf[1] != 0);
                ServerMessage::Enable(spdif, dac)
            }

            "audg" => {
                if buf.len() < 18 {
                    return ServerMessage::Error;
                }

                let _ = buf.split_to(10);
                let left = buf.split_to(4).get_u32() as f64 / GAIN_FACTOR;
                let right = buf.split_to(4).get_u32() as f64 / GAIN_FACTOR;
                ServerMessage::Gain(left, right)
            }

            "setd" => {
                if buf.len() == 0 {
                    return ServerMessage::Error;
                }

                if buf.len() > 1 {
                    let name: String = buf[1..].into_iter().map(|c| *c as char).collect();
                    ServerMessage::Setname(name)
                } else {
                    if buf[0] == 0 {
                        ServerMessage::Queryname
                    } else {
                        ServerMessage::Error
                    }
                }
            }

            cmd @ _ => ServerMessage::Unrecognised(cmd.to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::StatData;
    use futures::{SinkExt, StreamExt};
    use mac_address::MacAddress;
    use std::io::Cursor;
    use tokio_util::codec::{FramedRead, FramedWrite};

    #[tokio::test]
    async fn test_send_helo() {
        let helo = ClientMessage::Helo {
            device_id: 0,
            revision: 1,
            mac: MacAddress::new([1, 2, 3, 4, 5, 6]),
            uuid: [0u8; 16],
            wlan_channel_list: 3333,
            bytes_received: 0,
            capabilities: "abcd".to_owned(),
        };

        let mut buf = [0u8; 46];
        do_send(&mut buf, helo).await;
        assert_eq!(
            &buf[..32],
            &[
                b'H', b'E', b'L', b'O', 0, 0, 0, 38, 0, 1, 1, 2, 3, 4, 5, 6, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
        assert_eq!(
            &buf[32..],
            &[13, 5, 0, 0, 0, 0, 0, 0, 0, 0, b'a', b'b', b'c', b'd']
        );
    }

    #[tokio::test]
    async fn test_send_bye() {
        let bye = ClientMessage::Bye(55);

        let mut buf = [0u8; 9];
        do_send(&mut buf, bye).await;

        assert_eq!(&buf[..], &[b'B', b'Y', b'E', b'!', 0, 0, 0, 1, 55]);
    }

    #[tokio::test]
    async fn test_send_stat() {
        let stat_data = StatData {
            crlf: 0,
            buffer_size: 1234,
            fullness: 5678,
            bytes_received: 9123,
            sig_strength: 45,
            jiffies: 6789,
            output_buffer_size: 1234,
            output_buffer_fullness: 5678,
            elapsed_seconds: 9012,
            voltage: 3456,
            elapsed_milliseconds: 7890,
            timestamp: 1234,
            error_code: 5678,
        };
        let stat = ClientMessage::Stat {
            event_code: "STMt".to_owned(),
            stat_data: stat_data,
        };

        let mut buf = [0u8; 61];
        do_send(&mut buf, stat).await;

        assert_eq!(
            &buf[..32],
            &[
                b'S', b'T', b'A', b'T', 0, 0, 0, 53, b'S', b'T', b'M', b't', 0, 0, 0, 0, 0, 4, 210,
                0, 0, 22, 46, 0, 0, 0, 0, 0, 0, 35, 163, 0
            ]
        );
        assert_eq!(
            &buf[32..],
            &[
                45, 0, 0, 26, 133, 0, 0, 4, 210, 0, 0, 22, 46, 0, 0, 35, 52, 13, 128, 0, 0, 30,
                210, 0, 0, 4, 210, 22, 46
            ]
        );
    }

    #[tokio::test]
    async fn test_send_name() {
        let name = ClientMessage::Name("BadBoy".to_owned());

        let mut buf = [0u8; 15];
        do_send(&mut buf, name).await;

        assert_eq!(
            &buf[..],
            &[b'S', b'E', b'T', b'D', 0, 0, 0, 7, 0, b'B', b'a', b'd', b'B', b'o', b'y']
        );
    }

    #[tokio::test]
    async fn test_recv_serv() {
        let buf = [
            0u8, 12, b's', b'e', b'r', b'v', 172, 16, 1, 2, b's', b'y', b'n', b'c',
        ];
        let mut framed = FramedRead::new(&buf[..], SlimCodec);
        if let Some(Ok(msg)) = framed.next().await {
            assert_eq!(
                msg,
                ServerMessage::Serv {
                    ip_address: Ipv4Addr::new(172, 16, 1, 2),
                    sync_group_id: Some("sync".to_owned())
                }
            );
        } else {
            panic!("SERV message not received");
        }
    }

    #[tokio::test]
    async fn test_recv_status() {
        let buf = [
            0u8, 28, b's', b't', b'r', b'm', b't', 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
            15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        ];
        let mut framed = FramedRead::new(&buf[..], SlimCodec);
        if let Some(Ok(msg)) = framed.next().await {
            assert_eq!(msg, ServerMessage::Status(252711186));
        } else {
            panic!("STRMt message not received");
        }
    }

    #[tokio::test]
    async fn test_recv_stop() {
        let buf = [
            0u8, 28, b's', b't', b'r', b'm', b'q', 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
            15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        ];
        let mut framed = FramedRead::new(&buf[..], SlimCodec);
        if let Some(Ok(msg)) = framed.next().await {
            assert_eq!(msg, ServerMessage::Stop);
        } else {
            panic!("STRMq message not received");
        }
    }

    #[tokio::test]
    async fn test_recv_pause() {
        let buf = [
            0u8, 28, b's', b't', b'r', b'm', b'p', 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
            15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        ];
        let mut framed = FramedRead::new(&buf[..], SlimCodec);
        if let Some(Ok(msg)) = framed.next().await {
            assert_eq!(msg, ServerMessage::Pause(252711186));
        } else {
            panic!("STRMp message not received");
        }
    }

    #[tokio::test]
    async fn test_recv_unpause() {
        let buf = [
            0u8, 28, b's', b't', b'r', b'm', b'u', 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
            15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        ];
        let mut framed = FramedRead::new(&buf[..], SlimCodec);
        if let Some(Ok(msg)) = framed.next().await {
            assert_eq!(msg, ServerMessage::Unpause(252711186));
        } else {
            panic!("STRMu message not received");
        }
    }

    #[tokio::test]
    async fn test_recv_skip() {
        let buf = [
            0u8, 28, b's', b't', b'r', b'm', b'a', 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
            15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        ];
        let mut framed = FramedRead::new(&buf[..], SlimCodec);
        if let Some(Ok(msg)) = framed.next().await {
            assert_eq!(msg, ServerMessage::Skip(252711186));
        } else {
            panic!("STRMa message not received");
        }
    }

    #[tokio::test]
    async fn test_recv_unrecognised() {
        let buf = [
            0u8, 28, b's', b't', b'r', b'm', b'x', 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
            15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
        ];
        let mut framed = FramedRead::new(&buf[..], SlimCodec);
        if let Some(Ok(msg)) = framed.next().await {
            assert_eq!(msg, ServerMessage::Unrecognised("strm_x".to_owned()));
        } else {
            panic!("STRMx message not received");
        }
    }

    #[tokio::test]
    async fn test_recv_enable() {
        let buf = [0u8, 6, b'a', b'u', b'd', b'e', 0, 1];
        let mut framed = FramedRead::new(&buf[..], SlimCodec);
        if let Some(Ok(msg)) = framed.next().await {
            assert_eq!(msg, ServerMessage::Enable(false, true));
        } else {
            panic!("AUDE message not received");
        }
    }

    #[tokio::test]
    async fn test_recv_gain() {
        let buf = [
            0u8, 22, b'a', b'u', b'd', b'g', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 128, 0,
        ];
        let mut framed = FramedRead::new(&buf[..], SlimCodec);
        if let Some(Ok(msg)) = framed.next().await {
            match msg {
                ServerMessage::Gain(left, right) => {
                    assert_eq!(left, 1.0);
                    assert_eq!(right, 0.5);
                }
                _ => panic!("GAIN message incorrect"),
            }
        } else {
            panic!("AUDG message not received");
        }
    }
    #[tokio::test]
    async fn test_recv_setname() {
        let buf = [
            0u8, 12, b's', b'e', b't', b'd', 1, b'n', b'e', b'w', b'n', b'a', b'm', b'e',
        ];
        let mut framed = FramedRead::new(&buf[..], SlimCodec);
        if let Some(Ok(msg)) = framed.next().await {
            match msg {
                ServerMessage::Setname(name) => {
                    assert_eq!(name, "newname".to_owned());
                }
                _ => panic!("SETNAME message incorrect"),
            }
        } else {
            panic!("SETD message not received");
        }
    }

    #[tokio::test]
    async fn test_recv_strm() {
        let buf = [
            0u8, 31, b's', b't', b'r', b'm', b's', b'1', b'm', b'2', b'3', b'?', b'0', 1, b'2', 3,
            b'4', 1, 2, 0, 0, 1, 128, 0, 35, 41, 172, 16, 1, 2, b'a', b'b', b'c',
        ];
        let mut framed = FramedRead::new(&buf[..], SlimCodec);
        if let Some(Ok(msg)) = framed.next().await {
            assert_eq!(
                msg,
                ServerMessage::Stream {
                    autostart: AutoStart::Auto,
                    format: Format::Mp3,
                    pcmsamplesize: PcmSampleSize::Twenty,
                    pcmsamplerate: PcmSampleRate::Rate(44100),
                    pcmchannels: PcmChannels::SelfDescribing,
                    pcmendian: PcmEndian::Big,
                    threshold: 1024,
                    spdif_enable: SpdifEnable::Off,
                    trans_period: Duration::from_secs(3),
                    trans_type: TransType::FadeInOut,
                    flags: StreamFlags::INVERT_POLARITY_LEFT,
                    output_threshold: Duration::from_millis(2),
                    replay_gain: 1.5,
                    server_port: 9001,
                    server_ip: Ipv4Addr::new(172, 16, 1, 2),
                    http_headers: String::from("abc"),
                }
            );
        } else {
            panic!("SERV message not received");
        }
    }

    async fn do_send(buf: &mut [u8], frame: ClientMessage) {
        let buf = Cursor::new(&mut buf[..]);
        let mut framed = FramedWrite::new(buf, SlimCodec);
        let _ = framed.send(frame).await;
    }
}
