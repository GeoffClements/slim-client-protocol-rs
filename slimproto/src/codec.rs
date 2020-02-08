use bytes::{buf::BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::{ClientMessage, ServerMessage};

use std::{convert::TryInto, io};

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
        const MSGSIZE: usize = 1024;

        let mut msg = Vec::with_capacity(MSGSIZE + 2);
        let mut frame_size = Vec::with_capacity(2);
        let mut frame = Vec::with_capacity(MSGSIZE);

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
        //         const GAIN_FACTOR: f64 = 65536.0;
        let msg: String = src.split_to(4).into_iter().map(|c| c as char).collect();

        match msg.as_str() {
            //             "serv" => {
            //                 if src.len() < 4 {
            //                     ServerMessage::Error
            //                 } else {
            //                     let ip_addr = Ipv4Addr::from(src.split_to(4).into_buf().get_u32_be());
            //                     let sync_group = if src.len() > 0 {
            //                         Some(
            //                             src.take()
            //                                 .into_iter()
            //                                 .map(|c| c as char)
            //                                 .collect::<String>(),
            //                         )
            //                     } else {
            //                         None
            //                     };
            //                     ServerMessage::Serv {
            //                         ip_address: ip_addr,
            //                         sync_group_id: sync_group,
            //                     }
            //                 }
            //             }

            //             "strm" => {
            //                 if src.len() < 24 {
            //                     return ServerMessage::Error;
            //                 }

            //                 match src[0] as char {
            //                     't' => {
            //                         let timestamp = src[14..18].into_buf().get_u32_be();
            //                         ServerMessage::Status(timestamp)
            //                     }

            //                     's' => {
            //                         let replay_gain = src[14..18].into_buf().get_u32_be() as f64 / GAIN_FACTOR;
            //                         let http_headers = if src.len() >= 24 {
            //                             src[24..].into_iter().map(|c| *c as char).collect()
            //                         } else {
            //                             String::new()
            //                         };
            //                         ServerMessage::Stream {
            //                             autostart: src[1] == b'1' || src[1] == b'3',
            //                             threshold: src[7] as u32, // kbytes
            //                             output_threshold: src[12] as u64 * 100_000_000, // nanoseconds
            //                             replay_gain: replay_gain,
            //                             server_port: src[18..20].into_buf().get_u16_be(),
            //                             server_ip: Ipv4Addr::from(src[20..24].into_buf().get_u32_be()),
            //                             http_headers: http_headers,
            //                         }
            //                     }

            //                     'q' => ServerMessage::Stop,

            //                     'p' => {
            //                         let timestamp = src[14..18].into_buf().get_u32_be();
            //                         ServerMessage::Pause(timestamp)
            //                     }

            //                     'u' => {
            //                         let timestamp = src[14..18].into_buf().get_u32_be();
            //                         ServerMessage::Unpause(timestamp)
            //                     }

            //                     'a' => {
            //                         let timestamp = src[14..18].into_buf().get_u32_be();
            //                         ServerMessage::Skip(timestamp)
            //                     }

            //                     cmd @ _ => {
            //                         let mut msg = msg.to_owned();
            //                         msg.push('_');
            //                         msg.push(cmd);
            //                         ServerMessage::Unrecognised(msg)
            //                     }
            //                 }
            //             }
            //             "aude" => ServerMessage::Enable(!(src[1].into_buf().get_u8() == 0)),

            //             "audg" => ServerMessage::Gain(
            //                 src[10..14].into_buf().get_u32_be() as f64 / GAIN_FACTOR,
            //                 src[14..18].into_buf().get_u32_be() as f64 / GAIN_FACTOR,
            //             ),

            //             "setd" => {
            //                 if src.len() > 1 {
            //                     let name: String = src[1..].into_iter().map(|c| *c as char).collect();
            //                     ServerMessage::Setname(name)
            //                 } else {
            //                     if src[0] == 0 {
            //                         ServerMessage::Queryname
            //                     } else {
            //                         ServerMessage::Unknownsetd(src[0])
            //                     }
            //                 }
            //             }
            cmd @ _ => ServerMessage::Unrecognised(cmd.to_owned()),
        }
    }
}
