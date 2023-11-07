use crate::MidiEvent;

use super::{FromMidi, State};

#[derive(Debug)]
pub enum Event {
    Button(Button),
    ChButton(ChButton),
    Slider(Slider),
    Knob(Knob),
}

impl Event {
    pub fn parse(key: MidiEvent, value: i32) -> Option<Self> {
        if let Some(key) = Button::from_midi(key, value) {
            Some(Self::Button(key))
        } else if let Some(key) = ChButton::from_midi(key, value) {
            Some(Self::ChButton(key))
        } else if let Some(key) = Slider::from_midi(key, value) {
            Some(Self::Slider(key))
        } else {
            Knob::from_midi(key, value).map(Self::Knob)
        }
    }
}

/// TODO CCではなくNoteなので扱いを考える
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    BankLeft(State),
    BankRight(State),
    Solo(State),
}

impl FromMidi for Button {
    fn from_midi(key: MidiEvent, _value: i32) -> Option<Self> {
        let key = match key {
            MidiEvent::NoteOn {
                channel: 0,
                note: 25,
                ..
            } => Button::BankLeft(State::On),
            MidiEvent::NoteOn {
                channel: 0,
                note: 26,
                ..
            } => Button::BankRight(State::On),
            MidiEvent::NoteOn {
                channel: 0,
                note: 27,
                ..
            } => Button::Solo(State::On),
            MidiEvent::NoteOff {
                channel: 0,
                note: 25,
                ..
            } => Button::BankLeft(State::Off),
            MidiEvent::NoteOff {
                channel: 0,
                note: 26,
                ..
            } => Button::BankRight(State::Off),
            MidiEvent::NoteOff {
                channel: 0,
                note: 27,
                ..
            } => Button::Solo(State::Off),
            _ => return None,
        };
        Some(key)
    }
}

// TODO CCではなくNoteなので扱いを考える
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChButton {
    Mute(Ch, State),
    Record(Ch, State),
}

impl ChButton {
    const RANGE: (u8, u8) = (1, 27);
}

impl FromMidi for ChButton {
    fn from_midi(key: MidiEvent, _value: i32) -> Option<Self> {
        let (note, state) = {
            if let MidiEvent::NoteOn { note, .. } = key {
                (note, State::On)
            } else if let MidiEvent::NoteOff { note, .. } = key {
                (note, State::Off)
            } else {
                return None;
            }
        };

        if Self::RANGE.0 > note || note > Self::RANGE.1 {
            return None;
        }
        let class_index = (note - 1) % 3;
        let no_index = (note - 1) / 3;
        let ch = Ch::from(no_index as u32);
        let chb = match class_index {
            0 => Self::Mute(ch, state),
            2 => Self::Record(ch, state),
            _ => unreachable!(),
        };
        Some(chb)
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
    Master,
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
pub struct Slider {
    pub ch: Ch,
    pub value: u8,
}

impl Slider {
    const RANGE: &[u32] = &[19, 23, 27, 31, 49, 53, 57, 61];
    const MASTER: u32 = 62;
}

impl FromMidi for Slider {
    fn from_midi(key: MidiEvent, value: i32) -> Option<Self> {
        let MidiEvent::Control { channel: _, param } = key else {
            return None;
        };
        if param == Self::MASTER {
            return Some(Self {
                ch: Ch::Master,
                value: value as u8,
            });
        }
        let index = Self::RANGE.iter().position(|&x| x == param)?;
        let ch = Ch::from(index as u32);
        let value = value as u8;
        Some(Self { ch, value })
    }
}

#[derive(Debug)]
pub struct Knob {
    pub ch: Ch,
    /// 上から順に0,1,2番
    pub no: u8,
    pub value: u8,
}
impl Knob {
    #[rustfmt::skip]
    const RANGE: &[(u32, u32)] = &[
        (16, 18),
        (20, 22),
        (24, 26),
        (28, 30),
        (46, 48),
        (50, 52),
        (54, 56),
        (58, 60),
    ];
}

impl FromMidi for Knob {
    fn from_midi(key: MidiEvent, value: i32) -> Option<Self> {
        let MidiEvent::Control { channel: _, param } = key else {
            return None;
        };
        let index = Self::RANGE.iter().position(|&x| {
            let (start, end) = x;
            start <= param && param <= end
        })?;
        let ch = Ch::from(index as u32);
        let no = (param - Self::RANGE[index].0) as u8;
        let value = value as u8;
        Some(Self { ch, no, value })
    }
}
