use std::fmt;

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
            Capability::Accurateplaypoints => write!(f, "AccuratePlayPoints"),
            Capability::Syncgroupid(v) => write!(f, "SyncgroupID={}", v),
            Capability::Hasdigitalout => write!(f, "HasDigitalOut"),
            Capability::Haspreamp => write!(f, "HasPreAmp"),
            Capability::Hasdisabledac => write!(f, "HasDisableDac"),
        }
    }
}

#[derive(Default)]
pub struct Capabilities(Vec<Capability>);

impl Capabilities {
    pub fn add(&mut self, newcap: Capability) {
        let Self(ref mut caps) = self;
        caps.push(newcap);
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
        assert_eq!(c.to_string(), "mp3");
    }

    #[test]
    fn two_bools() {
        let mut c = Capabilities::default();
        c.add(Capability::Mp3);
        c.add(Capability::Ogg);
        assert_eq!(c.to_string(), "mp3,ogg");
    }

    #[test]
    fn list_with_values() {
        let mut c = Capabilities::default();
        c.add(Capability::Mp3);
        c.add(Capability::Maxsamplerate(9600));
        c.add(Capability::Ogg);
        assert_eq!(c.to_string(), "mp3,MaxSampleRate=9600,ogg");
    }
}
