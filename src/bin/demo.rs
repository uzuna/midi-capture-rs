use midi_capture::{CaptureDevice, parser};

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
            if let Some(data) = ev.get_data::<alsa::seq::EvCtrl>() {
                let event = midi_capture::parser::midimix::Event::parse((&data).into(), data.value);
                println!("{:?}", event);
            }
            midi_capture::CallcbackCtrl::Continue
        });
    } else {
        println!("Usage: {} [sync|all], got {}", args[0], args[1]);
        Ok(())
    }
}
