use midi_capture::{parser::midimix, CaptureDevice};

fn main() -> midi_capture::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let device = CaptureDevice::new(None)?;
    let mut guard = device.get()?;
    if args.len() > 1 && args[1] == "sync" {
        midi_capture::read_sync_cb(&mut guard, std::time::Duration::from_millis(500), |ev| {
            println!("{:?}", ev);
            midi_capture::CallcbackCtrl::Continue
        })
    } else if args.len() > 1 && args[1] == "all" {
        return midi_capture::read_all_cb(&mut guard, |ev| {
            println!("{:?}", ev);
            midi_capture::CallcbackCtrl::Continue
        });
    } else if args.len() > 1 && args[1] == "allp" {
        return midi_capture::read_all_cb(&mut guard, |ev| {
            if let Some((key, value)) = midi_capture::MidiEvent::parse(ev) {
                if let Some(ev) = midimix::Event::parse(key, value) {
                    println!("{:?}", ev);
                }
            }
            midi_capture::CallcbackCtrl::Continue
        });
    } else {
        println!("Usage: {} [sync|all], got {}", args[0], args[1]);
        Ok(())
    }
}
