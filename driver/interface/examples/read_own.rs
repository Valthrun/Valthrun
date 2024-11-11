use valthrun_driver_interface::DriverInterface;

fn read_heap_buffer(interface: &DriverInterface) -> anyhow::Result<()> {
    let mut buffer = Vec::with_capacity(10_000);
    buffer.resize(10_000, 0);

    for (index, entry) in buffer.iter_mut().enumerate() {
        *entry = index;
    }

    let mut read_buffer = Vec::new();
    read_buffer.resize(buffer.len(), 0usize);
    interface.read_slice::<usize>(std::process::id(), buffer.as_ptr() as u64, &mut read_buffer)?;

    if buffer == read_buffer {
        println!("Read head buffer successfull");
    } else {
        assert_eq!(buffer, read_buffer);
        println!("Full heap buffer read failed!");
    }

    Ok(())
}

pub fn main() -> anyhow::Result<()> {
    env_logger::builder().parse_default_env().init();
    let interface = DriverInterface::create_from_env()?;

    let target_value = 0x42u64;
    let read_value = interface.read::<u64>(std::process::id(), &target_value as *const _ as u64);

    println!("Read result: {:X?}", read_value);
    read_heap_buffer(&interface)?;
    Ok(())
}
