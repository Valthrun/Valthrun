use std::{
    mem,
    ptr,
};

use valthrun_driver_shared::{
    requests::{
        RequestRead,
        ResponseRead,
    },
    IO_MAX_DEREF_COUNT,
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

struct ReadContext<'a> {
    process: HANDLE,

    read_buffer: &'a mut [u8],

    resolved_offsets: [u64; IO_MAX_DEREF_COUNT],

    offsets: &'a [u64],
    offset_index: usize,
}

fn read_memory_rvm(ctx: &mut ReadContext) -> bool {
    let mut current_address = ctx.offsets[0];
    while (ctx.offset_index + 1) < ctx.offsets.len() {
        let target = &mut ctx.resolved_offsets[ctx.offset_index];
        let target = unsafe {
            core::slice::from_raw_parts_mut(target as *mut u64 as *mut u8, size_of_val(target))
        };

        let status = unsafe {
            NtReadVirtualMemory(
                ctx.process,
                current_address as *const (),
                target.as_mut_ptr() as *mut (),
                mem::size_of_val(target) as u32,
                ptr::null_mut(),
            )
        };
        if !status.is_ok() {
            return false;
        }

        // add the next offset
        current_address =
            ctx.resolved_offsets[ctx.offset_index].wrapping_add(ctx.offsets[ctx.offset_index + 1]);
        ctx.offset_index += 1;
    }

    let status = unsafe {
        NtReadVirtualMemory(
            ctx.process,
            current_address as *const (),
            ctx.read_buffer.as_mut_ptr() as *mut (),
            ctx.read_buffer.len() as u32,
            ptr::null_mut(),
        )
    };
    status.is_ok()
}

pub fn read(req: &RequestRead, res: &mut ResponseRead) -> anyhow::Result<()> {
    assert!(req.offsets.len() >= 1);
    assert!(req.offset_count <= req.offsets.len());

    let offsets = &req.offsets[0..req.offset_count];
    let read_buffer = unsafe { core::slice::from_raw_parts_mut(req.buffer, req.count) };

    let process = match util::open_process_by_id(req.process_id as u32, PROCESS_VM_READ) {
        Ok(handle) => handle,
        Err(err) => {
            log::warn!("Failed to open process {}: {}", req.process_id, err);
            *res = ResponseRead::UnknownProcess;
            return Ok(());
        }
    };

    let mut read_ctx = ReadContext {
        process: process.raw_handle(),

        read_buffer,
        resolved_offsets: [0u64; IO_MAX_DEREF_COUNT],

        offsets,
        offset_index: 0,
    };

    if !read_memory_rvm(&mut read_ctx) {
        *res = ResponseRead::InvalidAddress {
            resolved_offsets: read_ctx.resolved_offsets,
            resolved_offset_count: read_ctx.offset_index,
        };
        return Ok(());
    }

    *res = ResponseRead::Success;
    Ok(())
}
