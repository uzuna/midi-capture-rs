//! Exmaple for KORG nanoKONTROL2

use midi_capture::parser::nano_control2;
use midi_capture::{CallcbackCtrl, EvKey};

fn callback(ev: std::collections::HashMap<EvKey, i32>) -> CallcbackCtrl {
    println!("{:?}", nano_control2::parse(ev));
    CallcbackCtrl::Continue
}

fn main() -> midi_capture::Result<()> {
    let device = midi_capture::CaptureDevice::new(None)?;
    let mut guard = device.get()?;
    midi_capture::read_sync_cb(&mut guard, std::time::Duration::from_millis(100), callback)
}
