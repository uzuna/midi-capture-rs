pub mod error;
use std::time::Duration;

use alsa::seq::{self, EvCtrl, EvNote, Event, EventType};
pub use error::{Error, Result};

pub mod parser;

pub struct CaptureDevice {
    seq: alsa::Seq,
}

impl CaptureDevice {
    const DEFAULT_CLIENT_NAME: &'static str = "rust_midi_capture";
    pub fn new(name: Option<&str>) -> crate::Result<Self> {
        // Open the sequencer.
        let s = alsa::Seq::open(None, Some(alsa::Direction::Capture), true)?;
        let name = name.unwrap_or(Self::DEFAULT_CLIENT_NAME);
        let cstr = std::ffi::CString::new(name).unwrap();
        s.set_client_name(&cstr)?;

        // Create a destination port we can read from
        let mut dinfo = seq::PortInfo::empty().unwrap();
        dinfo.set_capability(seq::PortCap::WRITE | seq::PortCap::SUBS_WRITE);
        dinfo.set_type(seq::PortType::MIDI_GENERIC | seq::PortType::APPLICATION);
        dinfo.set_name(&cstr);
        s.create_port(&dinfo).unwrap();
        let dport = dinfo.get_port();

        connect_midi_source_ports(&s, dport)?;
        Ok(Self { seq: s })
    }

    pub fn get(&self) -> crate::Result<CaptureGurad> {
        let input = self.seq.input();
        use alsa::PollDescriptors;
        let pollfds = (&self.seq, Some(alsa::Direction::Capture)).get().unwrap();

        Ok(CaptureGurad { input, pollfds })
    }
}

#[derive(Debug)]
pub enum PollEvent {
    Timeout,
    Ready,
}

pub struct CaptureGurad<'a> {
    input: alsa::seq::Input<'a>,
    pollfds: Vec<libc::pollfd>,
}

impl CaptureGurad<'_> {
    pub fn read_event(&mut self) -> crate::Result<Option<alsa::seq::Event>> {
        if self.input.event_input_pending(true)? == 0 {
            return Ok(None);
        }
        let ev = self.input.event_input()?;
        Ok(Some(ev))
    }
    pub fn poll(&mut self, timeout_ms: std::time::Duration) -> crate::Result<PollEvent> {
        let wait = std::cmp::max(1, timeout_ms.as_millis() as i32);
        match alsa::poll::poll(&mut self.pollfds, wait)? {
            0 => Ok(PollEvent::Timeout),
            1 => Ok(PollEvent::Ready),
            _ => unreachable!(),
        }
    }
}

fn connect_midi_source_ports(s: &alsa::Seq, our_port: i32) -> crate::Result<()> {
    // Iterate over clients and clients' ports
    let our_id = s.client_id()?;
    let ci = seq::ClientIter::new(s);
    for client in ci {
        if client.get_client() == our_id {
            continue;
        } // Skip ourselves
        let pi = seq::PortIter::new(s, client.get_client());
        for port in pi {
            let caps = port.get_capability();

            // Check that it's a normal input port
            if !caps.contains(seq::PortCap::READ) || !caps.contains(seq::PortCap::SUBS_READ) {
                continue;
            }
            if !port.get_type().contains(seq::PortType::MIDI_GENERIC) {
                continue;
            }

            // Connect source and dest ports
            let subs = seq::PortSubscribe::empty()?;
            subs.set_sender(seq::Addr {
                client: port.get_client(),
                port: port.get_port(),
            });
            subs.set_dest(seq::Addr {
                client: our_id,
                port: our_port,
            });
            log::info!("Reading from midi input {:?}", port);
            s.subscribe_port(&subs)?;
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MidiEvent {
    Control { channel: u8, param: u32 },
    NoteOn { channel: u8, note: u8, velocity: u8 },
    NoteOff { channel: u8, note: u8, velocity: u8 },
}

impl MidiEvent {
    pub fn parse(ev: Event<'_>) -> Option<(Self, i32)> {
        if let Some(data) = ev.get_data::<alsa::seq::EvCtrl>() {
            Some(((&data).into(), data.value))
        } else {
            let evtype = ev.get_type();
            match evtype {
                seq::EventType::Noteon | seq::EventType::Noteoff => {
                    let data = ev.get_data::<alsa::seq::EvNote>().unwrap();
                    Some(((evtype, &data).into(), data.velocity.into()))
                }
                _ => None,
            }
        }
    }
}

impl From<&EvCtrl> for MidiEvent {
    fn from(ev: &EvCtrl) -> Self {
        Self::Control {
            channel: ev.channel,
            param: ev.param,
        }
    }
}

impl From<(EventType, &EvNote)> for MidiEvent {
    fn from((evtype, ev): (EventType, &EvNote)) -> Self {
        match evtype {
            EventType::Noteon => Self::NoteOn {
                channel: ev.channel,
                note: ev.note,
                velocity: ev.velocity,
            },
            EventType::Noteoff => Self::NoteOff {
                channel: ev.channel,
                note: ev.note,
                velocity: ev.velocity,
            },
            _ => unreachable!(),
        }
    }
}

pub enum CallcbackCtrl {
    Continue,
    Break,
}

/// すべてのイベントを読み込む
pub fn read_all_cb(
    guard: &mut CaptureGurad<'_>,
    f: impl Fn(alsa::seq::Event) -> CallcbackCtrl,
) -> crate::Result<()> {
    loop {
        if let Some(ev) = guard.read_event()? {
            f(ev);
        }
        guard.poll(Duration::from_millis(100))?;
    }
}

/// 同じキーは更新周期の最後の値のみが有効になる
pub fn read_sync_cb(
    guard: &mut CaptureGurad<'_>,
    interval: std::time::Duration,
    f: impl Fn(std::collections::HashMap<MidiEvent, i32>) -> CallcbackCtrl,
) -> crate::Result<()> {
    use std::collections::HashMap;
    use std::time::Instant;
    let mut next = Instant::now().checked_add(interval).unwrap();
    let mut evt: HashMap<MidiEvent, i32> = HashMap::new();
    loop {
        if let Some(ev) = guard.read_event()? {
            if let Some((k, v)) = MidiEvent::parse(ev) {
                evt.insert(k, v);
            }
        }
        if next.elapsed() >= interval {
            f(evt);
            next = Instant::now().checked_add(interval).unwrap();
            evt = HashMap::new();
        }
        let wait = match next.checked_duration_since(Instant::now()) {
            Some(wait) => wait,
            None => Duration::from_millis(2),
        };
        guard.poll(wait)?;
    }
}
