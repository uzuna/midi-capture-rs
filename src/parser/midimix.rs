use crate::EvKey;

use super::{FromMidi, State};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum ChButton {
    Mute(Ch, State),
    Solo(Ch, State),
    Record(Ch, State),
}

impl ChButton {
    const RANGE: (u32, u32) = (1, 27);
}

impl FromMidi for ChButton {
    fn from_midi(key: EvKey, value: i32) -> Option<Self> {
        if Self::RANGE.0 > key.param || key.param > Self::RANGE.1 {
            return None;
        }
        let class_index = (key.param - 1) % 3;
        let no_index = (key.param - 1) / 3;
        let ch = Ch::from(no_index);
        let chb = match class_index {
            0 => Self::Mute(ch, value.into()),
            1 => Self::Solo(ch, value.into()),
            2 => Self::Record(ch, value.into()),
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
}

impl FromMidi for Slider {
    fn from_midi(key: EvKey, value: i32) -> Option<Self> {
        let index = Self::RANGE.iter().position(|&x| x == key.param)?;
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
    fn from_midi(key: EvKey, value: i32) -> Option<Self> {
        let index = Self::RANGE.iter().position(|&x| {
            let (start, end) = x;
            start <= key.param && key.param <= end
        })?;
        let ch = Ch::from(index as u32);
        let no = (key.param - Self::RANGE[index].0) as u8;
        let value = value as u8;
        Some(Self { ch, no, value })
    }
}