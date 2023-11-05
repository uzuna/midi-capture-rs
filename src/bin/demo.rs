fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let device = hi_ctrl::CaptureDevice::new(None)?;
    let mut guard = device.get()?;
    if args.len() > 1 && args[1] == "sync" {
        read_sync_frame(&mut guard, std::time::Duration::from_millis(100))
    } else if args.len() > 1 && args[1] == "all" {
        return read_all(&mut guard);
    } else {
        println!("Usage: {} [sync|all], got {}", args[0], args[1]);
        Ok(())
    }
}
fn read_all(guard: &mut hi_ctrl::CaptureGurad) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        if let Some(ev) = guard.read_event()? {
            println!("{:?}", ev);
        }
    }
}

fn read_sync_frame(
    guard: &mut hi_ctrl::CaptureGurad,
    interval: std::time::Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut next = std::time::Instant::now().checked_add(interval).unwrap();
    let mut evt = None;
    loop {
        if let Some(ev) = guard.read_event()? {
            evt = Some(ev.into_owned());
        }
        if next.elapsed() >= interval {
            println!("{:?}", evt);
            next = std::time::Instant::now().checked_add(interval).unwrap();
            evt = None;
        }
    }
}
