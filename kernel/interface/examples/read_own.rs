use valthrun_kernel_interface::KernelInterface;

fn read_heap_buffer(interface: &KernelInterface) -> anyhow::Result<()> {
    let mut buffer = Vec::with_capacity(10_000);
    buffer.resize(10_000, 0);

    for (index, entry) in buffer.iter_mut().enumerate() {
        *entry = index;
    }

    let mut read_buffer = Vec::new();
    read_buffer.resize(buffer.len(), 0usize);
    interface.read_slice::<usize>(
        std::process::id() as i32,
        &[buffer.as_ptr() as u64],
        &mut read_buffer,
    )?;

    if buffer == read_buffer {
        println!("Read successfull");
    } else {
        assert_eq!(buffer, read_buffer);
        println!("Full buffer read failed!");
    }

    Ok(())
}

pub fn main() -> anyhow::Result<()> {
    let interface = KernelInterface::create("\\\\.\\GLOBALROOT\\Device\\valthrun")?;

    let target_value = 0x42u64;
    let read_value = interface.read::<u64>(
        std::process::id() as i32,
        &[&target_value as *const _ as u64],
    );

    println!("Read result: {:X?}", read_value);
    read_heap_buffer(&interface)?;
    Ok(())
}
