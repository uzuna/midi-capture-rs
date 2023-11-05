fn main() -> Result<(), Box<dyn std::error::Error>> {
    let device = hi_ctrl::CaptureDevice::new(None)?;
    let mut guard = device.get()?;
    loop {
        if let Some(ev) = guard.read_event()? {
            println!("{:?}", ev);
        }
    }
}
