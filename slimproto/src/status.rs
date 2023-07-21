//! A convenience module for working with client status data.
//! 
//! The Logitech Media Server requires regular status messages from
//! the client. This module provides convenience types for this.

use std::{fmt, time::Duration};

use crate::ClientMessage;

/// A struct to hold the status data as required by the server
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
    // pub fn new(buffer_size: u32, output_buffer_size: u32) -> Self {
    //     let mut stat = StatusData::default();
    //     stat.buffer_size = buffer_size;
    //     stat.output_buffer_size = output_buffer_size;
    //     stat
    // }

    // pub fn set_crlf<'a>(&'a mut self, crlf: u8) -> &'a mut Self {
    //     self.crlf = crlf;
    //     self
    // }

    pub fn set_fullness(&mut self, fullness: u32) {
        self.fullness = fullness;
    }

    pub fn add_bytes_received(&mut self, bytes_received: u64) {
        self.bytes_received = self.bytes_received.wrapping_add(bytes_received);
    }

    // pub fn set_jiffies<'a>(&'a mut self, jiffies: Duration) -> &'a mut Self {
    //     self.jiffies = jiffies;
    //     self
    // }

    // pub fn set_output_buffer_fullness<'a>(
    //     &'a mut self,
    //     output_buffer_fullness: u32,
    // ) -> &'a mut Self {
    //     self.output_buffer_fullness = output_buffer_fullness;
    //     self
    // }

    // pub fn set_elapsed_seconds<'a>(&'a mut self, elapsed_seconds: u32) -> &'a mut Self {
    //     self.elapsed_seconds = elapsed_seconds;
    //     self
    // }

    // pub fn set_elapsed_milli_seconds<'a>(&'a mut self, elapsed_milli_seconds: u32) -> &'a mut Self {
    //     self.elapsed_milliseconds = elapsed_milli_seconds;
    //     self
    // }

    pub fn set_buffer_size(&mut self, size: u32) {
        self.buffer_size = size;
    }

    pub fn set_timestamp(&mut self, timestamp: Duration) {
        self.timestamp = timestamp;
    }

    // pub fn set_error_code<'a>(&'a mut self, error_code: u16) -> &'a mut Self {
    //     self.error_code = error_code;
    //     self
    // }

    /// Create a status message for sending to the server
    pub fn make_status_message(&self, msgtype: StatusCode) -> ClientMessage {
        ClientMessage::Stat {
            event_code: msgtype.to_string(),
            stat_data: self.clone(),
        }
    }
}

/// Status code to send as part of the status message
pub enum StatusCode {
    Connect,
    DecoderReady,
    StreamEstablished,
    Flushed,
    HeadersReceived,
    BufferThreshold,
    NotSupported,
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
            StatusCode::NotSupported => write!(f, "STMn"),
            StatusCode::OutputUnderrun => write!(f, "STMo"),
            StatusCode::Pause => write!(f, "STMp"),
            StatusCode::Resume => write!(f, "STMr"),
            StatusCode::TrackStarted => write!(f, "STMs"),
            StatusCode::Timer => write!(f, "STMt"),
            StatusCode::Underrun => write!(f, "STMu"),
        }
    }
}
