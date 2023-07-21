/// Used to wrap around a reader.
/// Keeps the associates status data updated

use std::{
    io::{BufRead, BufReader, Read},
    sync::{Arc, RwLock},
};

use crate::status::StatusData;

pub struct SlimBuffer<R> {
    inner: BufReader<R>,
    status: Arc<RwLock<StatusData>>,
}

impl<R> SlimBuffer<R>
where
    R: Read,
{
    pub fn new(inner: R, status: Arc<RwLock<StatusData>>) -> Self {
        let buf = BufReader::new(inner);
        if let Ok(mut status) = status.write() {
            status.set_buffer_size(buf.capacity() as u32);
        }
        Self { inner: buf, status }
    }

    pub fn with_capacity(capacity: usize, inner: R, status: Arc<RwLock<StatusData>>) -> Self {
        let buf = BufReader::with_capacity(capacity, inner);
        if let Ok(mut status) = status.write() {
            status.set_buffer_size(buf.capacity() as u32);
        }
        Self { inner: buf, status }
    }
}

impl<R> Read for SlimBuffer<R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let bytes_read = self.inner.read(buf)?;
        if let Ok(mut status) = self.status.write() {
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
