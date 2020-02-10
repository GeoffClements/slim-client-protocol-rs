use bytes::{buf::BufMut, Buf, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::{ClientMessage, ServerMessage};

use std::{convert::TryInto, io, net::Ipv4Addr};

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

        msg.iter().as_slice().into()
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
                if src.len() < 24 {
                    return ServerMessage::Error;
                }

                match buf[0] as char {
                    't' => {
                        let timestamp = buf.split_to(14).get_u32();
                        ServerMessage::Status(timestamp)
                    }

                    's' => {
                        let frame = buf.split_to(14);
                        let replay_gain = buf.split_to(4).get_u32() as f64 / GAIN_FACTOR;
                        let server_port = buf.split_to(2).get_u16();
                        let server_ip = Ipv4Addr::from(buf.split_to(4).get_u32());
                        let http_headers = if buf.len() > 0 {
                            buf[..].into_iter().map(|c| *c as char).collect()
                        } else {
                            String::new()
                        };
                        ServerMessage::Stream {
                            autostart: frame[1] == b'1' || frame[1] == b'3',
                            threshold: frame[7] as u32, // kbytes
                            output_threshold: frame[12] as u64 * 100_000_000, // nanoseconds
                            replay_gain: replay_gain,
                            server_port: server_port,
                            server_ip: server_ip,
                            http_headers: http_headers,
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
                if buf.len() < 22 {
                    return ServerMessage::Error;
                }

                let mut buf = buf.split_to(10);
                ServerMessage::Gain(
                    buf.split_to(4).get_u32() as f64 / GAIN_FACTOR,
                    buf.split_to(4).get_u32() as f64 / GAIN_FACTOR,
                )
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
    use futures::SinkExt;
    use mac_address::MacAddress;
    use std::io::Cursor;
    use tokio_util::codec::FramedWrite;

    #[tokio::test]
    async fn test_helo() {
        let helo = ClientMessage::Helo {
            device_id: 0,
            revision: 1,
            mac: MacAddress::new([1, 2, 3, 4, 5, 6]),
            uuid: [0u8; 16],
            wlan_channel_list: 3333,
            bytes_received: 0,
            capabilities: "abcd".to_owned(),
        };

        let mut buf_inner = [0u8; 46];
        let buf = Cursor::new(&mut buf_inner[..]);
        let mut framed = FramedWrite::new(buf, SlimCodec);
        let _ = framed.send(helo).await;

        assert_eq!(
            &buf_inner[..32],
            &[
                b'H', b'E', b'L', b'O', 0, 0, 0, 38, 0, 1, 1, 2, 3, 4, 5, 6, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
        assert_eq!(
            &buf_inner[32..],
            &[13, 5, 0, 0, 0, 0, 0, 0, 0, 0, b'a', b'b', b'c', b'd']
        );
    }
}
