use std::{fmt, time::Duration};

use crate::ClientMessage;

#[derive(Clone, Debug, Default)]
pub struct StatusData {
    pub(crate) crlf: u8,
    pub(crate) buffer_size: u32,
    pub(crate) fullness: u32,
    pub(crate) bytes_received: u64,
    pub(crate) sig_strength: u16,
    pub(crate) jiffies: Duration,
    pub(crate) output_buffer_size: u32,
    pub(crate) output_buffer_fullness: u32,
    pub(crate) elapsed_seconds: u32,
    pub(crate) voltage: u16,
    pub(crate) elapsed_milliseconds: u32,
    pub(crate) timestamp: Duration,
    pub(crate) error_code: u16,
}

impl StatusData {
    pub fn new(buffer_size: u32, output_buffer_size: u32) -> Self {
        let mut stat = StatusData::default();
        stat.buffer_size = buffer_size;
        stat.output_buffer_size = output_buffer_size;
        stat
    }

    pub fn set_crlf<'a>(&'a mut self, crlf: u8) -> &'a mut Self {
        self.crlf = crlf;
        self
    }

    pub fn set_fullness<'a>(&'a mut self, fullness: u32) -> &'a mut Self {
        self.fullness = fullness;
        self
    }

    pub fn add_bytes_received<'a>(&'a mut self, bytes_received: u64) -> &'a mut Self {
        self.bytes_received = self.bytes_received.wrapping_add(bytes_received);
        self
    }

    pub fn set_jiffies<'a>(&'a mut self, jiffies: Duration) -> &'a mut Self {
        self.jiffies = jiffies;
        self
    }

    pub fn set_output_buffer_fullness<'a>(
        &'a mut self,
        output_buffer_fullness: u32,
    ) -> &'a mut Self {
        self.output_buffer_fullness = output_buffer_fullness;
        self
    }

    pub fn set_elapsed_seconds<'a>(&'a mut self, elapsed_seconds: u32) -> &'a mut Self {
        self.elapsed_seconds = elapsed_seconds;
        self
    }

    pub fn set_elapsed_milli_seconds<'a>(&'a mut self, elapsed_milli_seconds: u32) -> &'a mut Self {
        self.elapsed_milliseconds = elapsed_milli_seconds;
        self
    }

    pub fn set_timestamp<'a>(&'a mut self, timestamp: Duration) -> &'a mut Self {
        self.timestamp = timestamp;
        self
    }

    pub fn set_error_code<'a>(&'a mut self, error_code: u16) -> &'a mut Self {
        self.error_code = error_code;
        self
    }

    pub fn make_status_message(&self, msgtype: StatusCode) -> ClientMessage {
        ClientMessage::Stat {
            event_code: msgtype.to_string(),
            stat_data: self.clone(),
        }
    }
}

pub enum StatusCode {
    Connect,
    DecoderReady,
    StreamEstablished,
    Flushed,
    HeadersReceived,
    BufferThreshold,
    OutputUnderrun,
    Pause,
    Resume,
    TrackStarted,
    Timer,
    Underrun,
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            StatusCode::Connect => write!(f, "STMc"),
            StatusCode::DecoderReady => write!(f, "STMd"),
            StatusCode::StreamEstablished => write!(f, "STMe"),
            StatusCode::Flushed => write!(f, "STMf"),
            StatusCode::HeadersReceived => write!(f, "STMh"),
            StatusCode::BufferThreshold => write!(f, "STMl"),
            StatusCode::OutputUnderrun => write!(f, "STMo"),
            StatusCode::Pause => write!(f, "STMp"),
            StatusCode::Resume => write!(f, "STMr"),
            StatusCode::TrackStarted => write!(f, "STMs"),
            StatusCode::Timer => write!(f, "STMt"),
            StatusCode::Underrun => write!(f, "STMu"),
        }
    }
}
