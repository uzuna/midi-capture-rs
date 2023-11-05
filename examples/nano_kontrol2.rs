//! Exmaple for KORG nanoKONTROL2

use midi_capture::{CallcbackCtrl, EvKey};

fn main() -> midi_capture::Result<()> {
    let device = midi_capture::CaptureDevice::new(None)?;
    let mut guard = device.get()?;
    midi_capture::read_sync_cb(
        &mut guard,
        std::time::Duration::from_millis(10),
        parse_nanokontrol2,
    )
}

fn parse_nanokontrol2(m: std::collections::HashMap<midi_capture::EvKey, i32>) -> CallcbackCtrl {
    for (key, value) in m {
        if let Some((key, value)) = TransportButton::from_midi(key, value) {
            println!("{:?} {:?}", key, value);
            continue;
        }
        if let Some((key, value)) = TrackButton::from_midi(key, value) {
            println!("{:?} {:?}", key, value);
            continue;
        }
        if let Some((key, value)) = TransportButton::from_midi(key, value) {
            println!("{:?} {:?}", key, value);
            continue;
        }
        if let Some((key, value)) = MarkerButton::from_midi(key, value) {
            println!("{:?} {:?}", key, value);
            continue;
        }
        if let Some((key, value)) = ChButton::from_midi(key, value) {
            println!("{:?} {:?}", key, value);
            continue;
        }
        if let Some((key, value)) = Slider::from_midi(key, value) {
            println!("{:?} {:?}", key, value);
            continue;
        }
        if let Some((key, value)) = Knob::from_midi(key, value) {
            println!("{:?} {:?}", key, value);
            continue;
        }
        // println!("{:?}", event);
    }
    midi_capture::CallcbackCtrl::Continue
}

pub trait FromMidi {
    fn from_midi(key: EvKey, value: i32) -> Option<(Self, Value)>
    where
        Self: Sized;
}

#[derive(Debug)]
pub enum Event {
    Maker(MarkerButton, Value),
    Track(TrackButton, Value),
    TransportButton(TransportButton, Value),
}
#[derive(Debug)]

pub enum Value {
    On,
    Off,
    Value(u8),
}

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
#[derive(Debug)]

pub enum TrackButton {
    Prev,
    Next,
}

impl FromMidi for TrackButton {
    fn from_midi(key: EvKey, value: i32) -> Option<(Self, Value)> {
        let key = match key {
            EvKey {
                channel: 0,
                param: 58,
            } => TrackButton::Prev,
            EvKey {
                channel: 0,
                param: 59,
            } => TrackButton::Next,
            _ => return None,
        };
        let value = match value {
            0 => Value::Off,
            _ => Value::On,
        };
        Some((key, value))
    }
}

#[derive(Debug)]

pub enum MarkerButton {
    Set,
    Prev,
    Next,
}
impl FromMidi for MarkerButton {
    fn from_midi(key: EvKey, value: i32) -> Option<(Self, Value)> {
        let key = match key {
            EvKey {
                channel: 0,
                param: 60,
            } => MarkerButton::Set,
            EvKey {
                channel: 0,
                param: 61,
            } => MarkerButton::Prev,
            EvKey {
                channel: 0,
                param: 62,
            } => MarkerButton::Next,
            _ => return None,
        };
        let value = match value {
            0 => Value::Off,
            _ => Value::On,
        };
        Some((key, value))
    }
}

#[derive(Debug)]

pub enum TransportButton {
    Rewind,
    FastForward,
    Stop,
    Play,
    Record,
}

impl FromMidi for TransportButton {
    fn from_midi(key: EvKey, value: i32) -> Option<(Self, Value)> {
        let key = match key {
            EvKey {
                channel: 0,
                param: 43,
            } => TransportButton::Rewind,
            EvKey {
                channel: 0,
                param: 44,
            } => TransportButton::FastForward,
            EvKey {
                channel: 0,
                param: 42,
            } => TransportButton::Stop,
            EvKey {
                channel: 0,
                param: 41,
            } => TransportButton::Play,
            EvKey {
                channel: 0,
                param: 45,
            } => TransportButton::Record,
            _ => return None,
        };
        let value = match value {
            0 => Value::Off,
            _ => Value::On,
        };
        Some((key, value))
    }
}

#[derive(Debug)]
pub enum Ch {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}
impl From<u32> for Ch {
    fn from(v: u32) -> Self {
        match v {
            0 => Self::A,
            1 => Self::B,
            2 => Self::C,
            3 => Self::D,
            4 => Self::E,
            5 => Self::F,
            6 => Self::G,
            7 => Self::H,
            _ => panic!("invalid ch {} ", v),
        }
    }
}

#[derive(Debug)]

pub enum ChButton {
    Solo(Ch),
    Mute(Ch),
    Record(Ch),
}

impl ChButton {
    const SOLO_START: u32 = 32;
    const SOLO_END: u32 = 39;
    const MUTE_START: u32 = 48;
    const MUTE_END: u32 = 55;
    const RECORD_START: u32 = 64;
    const RECORD_END: u32 = 71;
}

impl FromMidi for ChButton {
    fn from_midi(key: EvKey, value: i32) -> Option<(Self, Value)> {
        let chb = match key.param {
            Self::SOLO_START..=Self::SOLO_END => Self::Solo(Ch::from(key.param - Self::SOLO_START)),
            Self::MUTE_START..=Self::MUTE_END => Self::Mute(Ch::from(key.param - Self::MUTE_START)),
            Self::RECORD_START..=Self::RECORD_END => {
                Self::Record(Ch::from(key.param - Self::RECORD_START))
            }
            _ => return None,
        };
        let value = match value {
            0 => Value::Off,
            _ => Value::On,
        };
        Some((chb, value))
    }
}

#[derive(Debug)]
pub struct Slider {
    pub ch: Ch,
}
impl Slider {
    const RANGE: (u32, u32) = (0, 7);
}

impl FromMidi for Slider {
    fn from_midi(key: EvKey, value: i32) -> Option<(Self, Value)> {
        if key.param < Self::RANGE.0 || key.param > Self::RANGE.1 {
            return None;
        }
        let ch = Ch::from(key.param);
        let value = Value::Value(value as u8);
        Some((Slider { ch }, value))
    }
}

#[derive(Debug)]
pub struct Knob {
    pub ch: Ch,
}

impl Knob {
    const RANGE: (u32, u32) = (16, 23);
}

impl FromMidi for Knob {
    fn from_midi(key: EvKey, value: i32) -> Option<(Self, Value)> {
        if key.param < Self::RANGE.0 || key.param > Self::RANGE.1 {
            return None;
        }
        let ch = Ch::from(key.param - Self::RANGE.0);
        let value = Value::Value(value as u8);
        Some((Knob { ch }, value))
    }
}
