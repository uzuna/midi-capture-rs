//! MIDIから特定のコントローラーへのマッピング実装

#[cfg(feature = "nano_kontrol2")]
pub mod nano_control2;

pub trait FromMidi {
    fn from_midi(key: crate::EvKey, value: i32) -> Option<Self>
    where
        Self: Sized;
}
