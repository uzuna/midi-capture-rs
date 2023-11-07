//! MIDIから特定のコントローラーへのマッピング実装

#[cfg(feature = "nano_kontrol2")]
pub mod nano_control2;

#[cfg(feature = "midimix")]
pub mod midimix;

pub trait FromMidi {
    fn from_midi(key: crate::MidiEvent, value: i32) -> Option<Self>
    where
        Self: Sized;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    On,
    Off,
}

impl From<i32> for State {
    fn from(v: i32) -> Self {
        match v {
            0 => Self::Off,
            _ => Self::On,
        }
    }
}
