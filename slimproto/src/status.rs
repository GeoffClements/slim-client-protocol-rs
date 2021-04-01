use std::time::Duration;

#[derive(Debug, Default)]
pub struct StatusData {
    pub crlf: u8,
    pub buffer_size: u32,
    pub fullness: u32,
    pub bytes_received: u64,
    pub sig_strength: u16,
    pub jiffies: Duration,
    pub output_buffer_size: u32,
    pub output_buffer_fullness: u32,
    pub elapsed_seconds: u32,
    pub voltage: u16,
    pub elapsed_milliseconds: u32,
    pub timestamp: Duration,
    pub error_code: u16,
}

impl StatusData {
    pub fn new(timestamp: Duration) -> Self {
        let mut stat = StatusData::default();
        stat.timestamp = timestamp;
        stat
    }
}