use std::{
    io::{self, BufReader, BufWriter, Read, Write},
    net::TcpStream,
};

use bytes::BytesMut;

const INITCAP: usize = 8 * 1024;

pub struct FramedRead<U, R> {
    inner: R,
    codec: U,
    read_frame: BytesMut,
}

pub struct FramedWrite<U, W> {
    inner: W,
    codec: U,
}

impl<U, R> FramedRead<U, R> {
    pub fn new(inner: R, codec: U) -> Self {
        Self {
            inner,
            codec,
            read_frame: BytesMut::with_capacity(INITCAP),
        }
    }
}

impl<U, W> FramedWrite<U, W> {
    pub fn new(inner: W, codec: U) -> Self {
        Self { inner, codec }
    }
}

impl<U, R> FramedRead<U, R>
where
    U: Decoder,
    R: Read,
{
    pub fn recv(&mut self) -> io::Result<U::Item> {
        let mut buf = [0u8; INITCAP];
        loop {
            let bytes_read = self.inner.read(&mut buf)?;
            self.read_frame.extend_from_slice(&buf[..bytes_read]);
            match self.codec.decode(&mut self.read_frame) {
                Ok(Some(item)) => return Ok(item),
                Ok(None) => continue,
                Err(e) => return Err(e),
            }
        }
    }
}

impl<U, W> FramedWrite<U, W>
where
    U: Encoder,
    W: Write,
{
    pub fn send(&mut self, item: U::Item) -> io::Result<()> {
        let mut dst = BytesMut::with_capacity(INITCAP);
        self.codec.encode(item, &mut dst)?;
        self.inner.write(&dst[..dst.len()])?;
        self.inner.flush()
    }
}

pub trait Decoder {
    type Item;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<Self::Item>>;
}

pub trait Encoder {
    type Item;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> io::Result<()>;
}

pub fn make_frames<U>(
    socket: TcpStream,
    codec: U,
) -> io::Result<(FramedRead<U, impl Read>, FramedWrite<U, impl Write>)>
where
    U: Clone,
{
    let write_sock = socket.try_clone()?;
    let codec2 = codec.clone();
    Ok((
        FramedRead::new(BufReader::new(socket), codec),
        FramedWrite::new(BufWriter::new(write_sock), codec2),
    ))
}

#[cfg(test)]
mod tests {
    use bytes::{Buf, BufMut};
    use socket_server_mocker::{
        server_mocker::ServerMocker,
        server_mocker_instruction::{ServerMockerInstruction, ServerMockerInstructionsList},
        tcp_server_mocker::TcpServerMocker,
    };

    use super::*;

    #[derive(Clone)]
    struct TestCodec;

    #[derive(Debug, PartialEq)]
    enum TestMsg {
        Number(u32),
        Error,
    }

    impl Encoder for TestCodec {
        type Item = TestMsg;

        fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> io::Result<()> {
            match item {
                TestMsg::Number(n) => {
                    dst.put(b"Number:".as_slice());
                    dst.put_u32(n);
                }
                _ => {}
            }
            Ok(())
        }
    }

    impl Decoder for TestCodec {
        type Item = TestMsg;

        fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<Self::Item>> {
            if src.len() < 11 {
                return Ok(None);
            }

            let msg = String::from_utf8(src.split_to(7).to_vec()).unwrap();
            match msg.as_str() {
                "Number:" => {
                    let n = src.get_u32();
                    Ok(Some(TestMsg::Number(n)))
                }
                _ => Ok(Some(TestMsg::Error)),
            }
        }
    }

    #[test]
    fn write_frame() {
        let mut buf = [0u8; 32];
        let mut writer = FramedWrite::new(&mut buf[..], TestCodec);
        writer.send(TestMsg::Number(1)).unwrap();
        println!("{:?}", buf);
        assert_eq!(
            buf[..11],
            [b'N', b'u', b'm', b'b', b'e', b'r', b':', 0, 0, 0, 1]
        )
    }

    #[test]
    fn read_frame() {
        let buf = [b'N', b'u', b'm', b'b', b'e', b'r', b':', 0, 0, 0, 1];
        let mut reader = FramedRead::new(&buf[..], TestCodec);
        let msg = reader.recv().unwrap();
        assert_eq!(msg, TestMsg::Number(1));
    }

    #[test]
    fn frames_over_tcp() {
        let test_buf = [b'N', b'u', b'm', b'b', b'e', b'r', b':', 0, 0, 0, 1];

        let tcp_server_mocker = TcpServerMocker::new(35642).unwrap();
        let client = TcpStream::connect("127.0.0.1:35642").unwrap();

        let (mut rx, mut tx) = make_frames(client, TestCodec).unwrap();

        tcp_server_mocker
            .add_mock_instructions_list(ServerMockerInstructionsList::new_with_instructions(
                [
                    ServerMockerInstruction::ReceiveMessage,
                    ServerMockerInstruction::SendMessage(Vec::from(test_buf)),
                ]
                .as_slice(),
            ))
            .unwrap();

        tx.send(TestMsg::Number(1)).unwrap();
        let response = rx.recv().unwrap();

        assert_eq!(response, TestMsg::Number(1));
        assert_eq!(tcp_server_mocker.pop_received_message().unwrap(), test_buf);
    }
}
