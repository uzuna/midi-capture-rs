pub mod error;
use alsa::seq;
pub use error::{Error, Result};

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
        Ok(CaptureGurad { input })
    }
}

pub struct CaptureGurad<'a> {
    input: alsa::seq::Input<'a>,
}

impl CaptureGurad<'_> {
    pub fn read_event(&mut self) -> crate::Result<Option<alsa::seq::Event>> {
        if self.input.event_input_pending(true)? == 0 {
            return Ok(None);
        }
        let ev = self.input.event_input()?;
        Ok(Some(ev))
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
