//! SocketReader
//!
//! Provides a drop-in replacement for std::io::SocketReader but
//! provides limited Seek semantics. Designed to be used with
//! Rodio in order to play a TcpStream.

use std::{
    cmp,
    convert::{TryFrom, TryInto},
    io::{self, BufRead, Read, Seek, SeekFrom},
};

pub struct SocketReader<R> {
    inner: R,
    buf: Box<[u8]>,
    pos: usize,
    cap: usize,
    pos_from_start: u64,
}

impl<R: Read> SocketReader<R> {
    pub fn new(inner: R) -> SocketReader<R> {
        const DEFAULTBUFSIZE: usize = 8 * 1024;
        SocketReader::with_capacity(DEFAULTBUFSIZE, inner)
    }

    pub fn with_capacity(capacity: usize, inner: R) -> SocketReader<R> {
        let mut buffer = Vec::with_capacity(capacity);
        buffer.resize(capacity, 0);
        SocketReader {
            inner,
            buf: buffer.into_boxed_slice(),
            pos: 0,
            cap: 0,
            pos_from_start: 0,
        }
    }

    fn unconsume(&mut self, amt: usize) {
        let oldpos = self.pos;
        self.pos = self.pos.saturating_sub(amt);
        self.pos_from_start -= (oldpos - self.pos) as u64;
    }
}

impl<R: Read> Read for SocketReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let nread = {
            let mut rem = self.fill_buf()?;
            rem.read(buf)?
        };
        self.consume(nread);
        Ok(nread)
    }
}

impl<R: Read> BufRead for SocketReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.pos >= self.cap {
            self.cap = self.inner.read(&mut self.buf)?;
            self.pos = 0;
        }
        Ok(&self.buf[self.pos..self.cap])
    }

    fn consume(&mut self, amt: usize) {
        let oldpos = self.pos;
        self.pos = cmp::min(self.pos + amt, self.cap);
        self.pos_from_start += (self.pos - oldpos) as u64;
    }
}

impl<R: Read> Seek for SocketReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let err = io::Error::from(io::ErrorKind::NotFound);
        let relpos: i64 = match pos {
            SeekFrom::Current(n) => n,
            SeekFrom::Start(n) => {
                let n = self.pos_from_start - n;
                if n < self.pos_from_start {
                    match i64::try_from(n) {
                        Ok(n) => -n,
                        Err(_) => return Err(err),
                    }
                } else {
                    match n.try_into() {
                        Ok(n) => n,
                        Err(_) => return Err(err),
                    }
                }
            }
            SeekFrom::End(_) => return Err(err),
        };

        let mut relapos = match usize::try_from(relpos.abs()) {
            Ok(n) => n,
            Err(_) => return Err(err),
        };

        if relpos.is_negative() {
            if relapos < self.pos {
                self.unconsume(relapos);
            } else {
                return Err(err);
            }
        } else {
            while relapos > self.cap {
                self.consume(self.cap - self.pos);
                self.fill_buf()?;
                relapos -= self.cap - self.pos;
            }
            println!("{}, {}, {}", relapos, self.pos, self.cap);
            self.consume(relapos);
        }

        Ok(self.pos_from_start)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let mut d = Vec::with_capacity(64);
        let testdata = {
            for i in 0..64 {
                d.push(i as u8);
            };
            d.as_slice()
        };
        
        let mut seekbuf = SocketReader::with_capacity(64, testdata);
        let mut buf = [0u8; 8];
        let nread = seekbuf.read(&mut buf).unwrap();
        assert_eq!(nread, 8usize);
        assert_eq!(buf, testdata[0..8]);
    }

    #[test]
    fn test_multpile_reads() {
        let mut d = Vec::with_capacity(64);
        let testdata = {
            for i in 0..64 {
                d.push(i as u8);
            };
            d.as_slice()
        };
        
        let mut seekbuf = SocketReader::with_capacity(64, testdata);
        let mut buf = [0u8; 8];
        let _ = seekbuf.read(&mut buf);
        let _ = seekbuf.read(&mut buf);
        assert_eq!(buf, testdata[8..16]);
    }

    #[test]
    fn seek_from_end() {
        let mut d = Vec::with_capacity(64);
        let testdata = {
            for i in 0..64 {
                d.push(i as u8);
            };
            d.as_slice()
        };
        
        let mut seekbuf = SocketReader::with_capacity(64, testdata);
        let mut buf = [0u8; 8];
        let _ = seekbuf.read(&mut buf);
        let pos = seekbuf.seek(SeekFrom::End(0));
        assert!(pos.is_err());
    }

    #[test]
    fn seek_current_in_buf() {
        let mut d = Vec::with_capacity(64);
        let testdata = {
            for i in 0..64 {
                d.push(i as u8);
            };
            d.as_slice()
        };
        
        let mut seekbuf = SocketReader::with_capacity(64, testdata);
        let mut buf = [0u8; 8];
        let _ = seekbuf.read(&mut buf);
        let _ = seekbuf.read(&mut buf);
        let pos = seekbuf.seek(SeekFrom::Current(4)).unwrap();
        assert_eq!(pos, 20u64);
    }
}
