use std::ptr;

use valthrun_driver_protocol::{
    command::DriverCommandMemoryRead,
    types::{
        DirectoryTableType,
        MemoryAccessResult,
    },
};
use windows::Win32::{
    Foundation::{
        HANDLE,
        NTSTATUS,
    },
    System::Threading::PROCESS_VM_READ,
};

use crate::util;

extern "C" {
    fn NtReadVirtualMemory(
        ProcessHandle: HANDLE,
        BaseAddress: *const (),
        Buffer: *const (),
        NumberOfBytesToRead: u32,
        NumberOfBytesReaded: *mut u32,
    ) -> NTSTATUS;
}

pub fn read(command: &mut DriverCommandMemoryRead) -> anyhow::Result<()> {
    if !matches!(command.directory_table_type, DirectoryTableType::Default) {
        anyhow::bail!("unsupported memory read type");
    }

    let read_buffer = unsafe { core::slice::from_raw_parts_mut(command.buffer, command.count) };
    let process = match util::open_process_by_id(command.process_id as u32, PROCESS_VM_READ) {
        Ok(handle) => handle,
        Err(err) => {
            log::warn!("Failed to open process {}: {}", command.process_id, err);
            command.result = MemoryAccessResult::ProcessUnknown;
            return Ok(());
        }
    };

    let status = unsafe {
        NtReadVirtualMemory(
            process.raw_handle(),
            command.address as *const (),
            read_buffer.as_mut_ptr() as *mut (),
            read_buffer.len() as u32,
            ptr::null_mut(),
        )
    };
    if status.is_ok() {
        command.result = MemoryAccessResult::Success;
    } else {
        command.result = MemoryAccessResult::PartialSuccess { bytes_copied: 0 };
    }
    Ok(())
}
