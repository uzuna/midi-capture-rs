use super::{FromMidi, State};
use crate::EvKey;

pub fn parse(m: std::collections::HashMap<EvKey, i32>) -> Vec<Event> {
    m.iter()
        .filter_map(|(k, v)| Event::parse(*k, *v))
        .collect::<Vec<Event>>()
}

#[derive(Debug)]
pub enum Event {
    Maker(MarkerButton),
    Track(TrackButton),
    TransportButton(TransportButton),
    ChButton(ChButton),
    Slider(Slider),
    Knob(Knob),
}

impl Event {
    fn parse(key: EvKey, value: i32) -> Option<Self> {
        if let Some(key) = TransportButton::from_midi(key, value) {
            Some(Self::TransportButton(key))
        } else if let Some(key) = TrackButton::from_midi(key, value) {
            Some(Self::Track(key))
        } else if let Some(key) = MarkerButton::from_midi(key, value) {
            Some(Self::Maker(key))
        } else if let Some(key) = ChButton::from_midi(key, value) {
            Some(Self::ChButton(key))
        } else if let Some(key) = Slider::from_midi(key, value) {
            Some(Self::Slider(key))
        } else {
            Knob::from_midi(key, value).map(Self::Knob)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackButton {
    Prev(State),
    Next(State),
}

impl FromMidi for TrackButton {
    fn from_midi(key: EvKey, value: i32) -> Option<Self> {
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
        Some(key(value.into()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum MarkerButton {
    Set(State),
    Prev(State),
    Next(State),
}
impl FromMidi for MarkerButton {
    fn from_midi(key: EvKey, value: i32) -> Option<Self> {
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
        Some(key(value.into()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum TransportButton {
    Rewind(State),
    FastForward(State),
    Stop(State),
    Play(State),
    Record(State),
}

impl FromMidi for TransportButton {
    fn from_midi(key: EvKey, value: i32) -> Option<Self> {
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
        Some(key(value.into()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum ChButton {
    Solo(Ch, State),
    Mute(Ch, State),
    Record(Ch, State),
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
    fn from_midi(key: EvKey, value: i32) -> Option<Self> {
        let chb = match key.param {
            Self::SOLO_START..=Self::SOLO_END => {
                Self::Solo(Ch::from(key.param - Self::SOLO_START), value.into())
            }
            Self::MUTE_START..=Self::MUTE_END => {
                Self::Mute(Ch::from(key.param - Self::MUTE_START), value.into())
            }
            Self::RECORD_START..=Self::RECORD_END => {
                Self::Record(Ch::from(key.param - Self::RECORD_START), value.into())
            }
            _ => return None,
        };
        Some(chb)
    }
}

#[derive(Debug)]
pub struct Slider {
    pub ch: Ch,
    pub value: u8,
}
impl Slider {
    const RANGE: (u32, u32) = (0, 7);
}

impl FromMidi for Slider {
    fn from_midi(key: EvKey, value: i32) -> Option<Self> {
        if key.param < Self::RANGE.0 || key.param > Self::RANGE.1 {
            return None;
        }
        let ch = Ch::from(key.param);
        let value = value as u8;
        Some(Slider { ch, value })
    }
}

#[derive(Debug)]
pub struct Knob {
    pub ch: Ch,
    pub value: u8,
}

impl Knob {
    const RANGE: (u32, u32) = (16, 23);
}

impl FromMidi for Knob {
    fn from_midi(key: EvKey, value: i32) -> Option<Self> {
        if key.param < Self::RANGE.0 || key.param > Self::RANGE.1 {
            return None;
        }
        let ch = Ch::from(key.param - Self::RANGE.0);
        let value = value as u8;
        Some(Knob { ch, value })
    }
}
