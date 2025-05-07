/// Provides the types needed to send capability data to the server.

use std::fmt;

/// A client capability as recognised by by the server. Sent as a list of capabilities
/// when the client announces itself to the server
pub enum Capability {
    Wma,
    Wmap,
    Wmal,
    Ogg,
    Flc,
    Pcm,
    Aif,
    Mp3,
    Alc,
    Aac,
    Maxsamplerate(u32),
    Model(String),
    Modelname(String),
    Rhap,
    Accurateplaypoints,
    Syncgroupid(String),
    Hasdigitalout,
    Haspreamp,
    Hasdisabledac,
}

/// When sent to the server a capability is sent as text
impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Capability::Wma => write!(f, "wma"),
            Capability::Wmap => write!(f, "wmap"),
            Capability::Wmal => write!(f, "wmal"),
            Capability::Ogg => write!(f, "ogg"),
            Capability::Flc => write!(f, "flc"),
            Capability::Pcm => write!(f, "pcm"),
            Capability::Aif => write!(f, "aif"),
            Capability::Mp3 => write!(f, "mp3"),
            Capability::Alc => write!(f, "alc"),
            Capability::Aac => write!(f, "aac"),
            Capability::Maxsamplerate(v) => write!(f, "MaxSampleRate={}", v.to_string()),
            Capability::Model(v) => write!(f, "Model={}", v),
            Capability::Modelname(v) => write!(f, "Modelname={}", v),
            Capability::Rhap => write!(f, "Rhap"),
            Capability::Accurateplaypoints => write!(f, "AccuratePlayPoints=1"),
            Capability::Syncgroupid(v) => write!(f, "SyncgroupID={}", v),
            Capability::Hasdigitalout => write!(f, "HasDigitalOut=1"),
            Capability::Haspreamp => write!(f, "HasPreAmp=1"),
            Capability::Hasdisabledac => write!(f, "HasDisableDac=1"),
        }
    }
}

/// A list of capabilities which is sent to the server when the client announces itself.
/// See [SlimpProto](crate::proto::SlimProto) for more details.
pub struct Capabilities(Vec<Capability>);

impl Capabilities {
    /// Add a new capability to the list. Note that capabilities are sent to the server
    /// in the order that they are added to the list.
    ///
    /// Normally you will not need to use this method as capabilities are usually added
    /// using the [add_capability](crate::proto::SlimProto::add_capability) method.
    pub fn add(&mut self, newcap: Capability) {
        let Self(ref mut caps) = self;
        caps.push(newcap);
    }

    pub fn add_name(&mut self, name: &str) {
        self.add(Capability::Modelname(name.to_owned()));
    }
}

impl Default for Capabilities {
    fn default() -> Self {
        let mut caps = Vec::new();
        caps.push(Capability::Accurateplaypoints);
        caps.push(Capability::Model("squeezelite".to_owned()));
        Self(caps)
    }
}

impl fmt::Display for Capabilities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(ref caps) = self;
        let capstr = caps.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        write!(f, "{}", capstr.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single() {
        let mut c = Capabilities::default();
        c.add(Capability::Mp3);
        assert_eq!(c.to_string(), "AccuratePlayPoints=1,Model=squeezelite,mp3");
    }

    #[test]
    fn list_with_values() {
        let mut c = Capabilities::default();
        c.add(Capability::Mp3);
        c.add(Capability::Maxsamplerate(9600));
        c.add(Capability::Ogg);
        assert_eq!(c.to_string(), "AccuratePlayPoints=1,Model=squeezelite,mp3,MaxSampleRate=9600,ogg");
    }

    #[test]
    fn name() {
        let mut c = Capabilities::default();
        c.add_name("Testing");
        assert_eq!(c.to_string(), "AccuratePlayPoints=1,Model=squeezelite,Modelname=Testing");
    }
}
