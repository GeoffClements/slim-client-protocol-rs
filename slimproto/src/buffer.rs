/// Used to wrap around a reader.
/// Keeps the associates status data updated
use std::{
    io::{BufRead, BufReader, Read},
    sync::{Arc, Mutex},
};

use crate::status::StatusData;

pub struct SlimBuffer<R> {
    inner: BufReader<R>,
    status: Arc<Mutex<StatusData>>,
    threshold: u32,
    prebuf: Vec<u8>,
}

impl<R> SlimBuffer<R>
where
    R: Read,
{
    pub fn new(inner: R, status: Arc<Mutex<StatusData>>, threshold: u32) -> Self {
        let buf = BufReader::new(inner);
        if let Ok(mut status) = status.lock() {
            status.set_buffer_size(buf.capacity() as u32);
        }

        let mut this: SlimBuffer<R> = Self {
            inner: buf,
            status,
            threshold,
            prebuf: Vec::with_capacity(255 * 1024),
        };

        this.pre_buf();
        this
    }

    pub fn with_capacity(
        capacity: usize,
        inner: R,
        status: Arc<Mutex<StatusData>>,
        threshold: u32,
    ) -> Self {
        let buf = BufReader::with_capacity(capacity, inner);
        if let Ok(mut status) = status.lock() {
            status.set_buffer_size(buf.capacity() as u32);
        }

        let mut this: SlimBuffer<R> = Self {
            inner: buf,
            status,
            threshold,
            prebuf: Vec::with_capacity(255 * 1024),
        };

        this.pre_buf();
        this
    }

    fn pre_buf(&mut self) {
        let mut buf = [0u8; 1024];
        while self.prebuf.len() < self.threshold as usize {
            if let Ok(n) = self.inner.read(&mut buf) {
                if n == 0 {
                    break;
                }
                self.prebuf.extend_from_slice(&buf[..n]);
            } else {
                break;
            }
        }
    }
}

impl<R> Read for SlimBuffer<R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes_read = if self.prebuf.len() > 0 {
            let n_bytes = (&self.prebuf[..]).read(buf)?;
            self.prebuf.drain(..n_bytes);
            n_bytes
        } else {
            self.inner.read(buf)?
        };
        if let Ok(mut status) = self.status.lock() {
            status.add_bytes_received(bytes_read as u64);
            status.set_fullness(self.inner.buffer().len() as u32);
        }
        Ok(bytes_read)
    }
}

impl<R> BufRead for SlimBuffer<R>
where
    R: Read,
{
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.inner.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prebuf() {
        const BUFLEN: usize = 1024;

        let status = Arc::new(Mutex::new(StatusData::default()));
        let source: Vec<u8> = (0u8..255).into_iter().cycle().take(BUFLEN).collect();

        let sb = SlimBuffer::new(&source[..], status, 2);
        assert_eq!(sb.prebuf, source);
        assert!(sb.prebuf.len() == source.len());
    }

    #[test]
    fn prebuf_overfill() {
        const BUFLEN: usize = 1024 * 2;

        let status = Arc::new(Mutex::new(StatusData::default()));
        let source: Vec<u8> = (0u8..255).into_iter().cycle().take(BUFLEN).collect();

        let mut sb = SlimBuffer::new(&source[..], status, 2);

        let mut buf = vec![0u8; BUFLEN];
        let n = sb.read(&mut buf).unwrap();
        sb.read(&mut buf[n..]).unwrap();
        assert_eq!(buf, source);
        assert!(sb.prebuf.len() == 0);
    }
}
