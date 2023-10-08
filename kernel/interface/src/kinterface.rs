use std::{
    ffi::{
        c_void,
        CString,
    },
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
};

use valthrun_driver_shared::{
    requests::{
        DriverRequest,
        RequestRead,
        ResponseRead,
    },
    IO_MAX_DEREF_COUNT,
};
use windows::{
    core::PCSTR,
    Win32::{
        Foundation,
        Storage::FileSystem::{
            self,
            CreateFileA,
            FILE_FLAGS_AND_ATTRIBUTES,
        },
        System::IO::DeviceIoControl,
    },
};

use crate::{
    KInterfaceError,
    KResult,
    SearchPattern,
};

/// Interface for our kernel driver
pub struct KernelInterface {
    driver_handle: Foundation::HANDLE,
    read_calls: AtomicUsize,
}

impl KernelInterface {
    pub fn create(path: &str) -> KResult<Self> {
        let driver_handle = unsafe {
            let path = CString::new(path).map_err(KInterfaceError::DeviceInvalidPath)?;
            CreateFileA(
                PCSTR::from_raw(path.as_bytes().as_ptr()),
                Foundation::GENERIC_READ.0 | Foundation::GENERIC_WRITE.0,
                FileSystem::FILE_SHARE_READ | FileSystem::FILE_SHARE_WRITE,
                None,
                FileSystem::OPEN_EXISTING,
                FILE_FLAGS_AND_ATTRIBUTES(0),
                None,
            )
            .map_err(KInterfaceError::DeviceUnavailable)?
        };

        Ok(Self {
            driver_handle,
            read_calls: AtomicUsize::new(0),
        })
    }

    #[must_use]
    pub fn total_read_calls(&self) -> usize {
        self.read_calls.load(Ordering::Relaxed)
    }

    /// Execute an action with kernel privilidges
    /// Note: It's unsafe, as the caller must validate all parameters given for the target action.
    #[must_use]
    pub unsafe fn execute_request<R: DriverRequest>(&self, payload: &R) -> KResult<R::Result> {
        let mut result: R::Result = Default::default();
        let success = unsafe {
            DeviceIoControl(
                self.driver_handle,
                R::control_code(),
                Some(payload as *const _ as *const c_void),
                std::mem::size_of::<R>() as u32,
                Some(&mut result as *mut _ as *mut c_void),
                std::mem::size_of::<R::Result>() as u32,
                None,
                None,
            )
            .as_bool()
        };

        if success {
            Ok(result)
        } else {
            /* TOOD: GetLastErrorCode? */
            Err(KInterfaceError::RequestFailed)
        }
    }

    #[must_use]
    pub fn read<T: Copy>(&self, process_id: i32, offsets: &[u64]) -> KResult<T> {
        let mut result = unsafe { std::mem::zeroed::<T>() };
        let result_buff = unsafe {
            std::slice::from_raw_parts_mut(
                std::mem::transmute::<_, *mut u8>(&mut result),
                std::mem::size_of::<T>(),
            )
        };

        self.read_slice(process_id, offsets, result_buff)?;
        Ok(result)
    }

    #[must_use]
    pub fn read_slice<T: Copy>(
        &self,
        process_id: i32,
        offsets: &[u64],
        buffer: &mut [T],
    ) -> KResult<()> {
        let mut offset_buffer = [0u64; IO_MAX_DEREF_COUNT];
        if offsets.len() > offset_buffer.len() {
            return Err(KInterfaceError::TooManyOffsets {
                provided: offsets.len(),
                limit: offset_buffer.len(),
            });
        }

        self.read_calls.fetch_add(1, Ordering::Relaxed);
        offset_buffer[0..offsets.len()].copy_from_slice(offsets);
        let result = unsafe {
            /*
             * Safety:
             * All parameters are checked and verified to point to valid memory.
             * The buffer ptr is guranteed to hold at least `count` bytes.
             */
            self.execute_request::<RequestRead>(&RequestRead {
                process_id,

                offsets: offset_buffer.clone(),
                offset_count: offsets.len(),

                buffer: buffer.as_mut_ptr() as *mut u8,
                count: buffer.len() * std::mem::size_of::<T>(),
            })
        }?;

        match result {
            ResponseRead::Success => Ok(()),
            ResponseRead::InvalidAddress {
                resolved_offset_count,
                resolved_offsets,
            } => {
                //log::trace!("Invalid read {:?}: {:?} -> {:?}", offsets, resolved_offsets, resolved_offset_count);
                Err(KInterfaceError::InvalidAddress {
                    target_address: if resolved_offset_count == 0 {
                        offsets[0]
                    } else {
                        resolved_offsets[resolved_offset_count - 1]
                    },
                    resolved_offsets,
                    resolved_offset_count,
                    offsets: offset_buffer,
                    offset_count: offsets.len(),
                })
            }
            ResponseRead::UnknownProcess => Err(KInterfaceError::ProcessDoesNotExists),
        }
    }

    #[must_use]
    pub fn find_pattern(
        &self,
        process_id: i32,
        address: u64,
        length: usize,
        pattern: &dyn SearchPattern,
    ) -> KResult<Option<u64>> {
        if pattern.length() > length {
            return Ok(None);
        }

        let mut buffer = Vec::<u8>::with_capacity(length);
        buffer.resize(length, 0);
        self.read_slice(process_id, &[address], &mut buffer)?;

        for (index, window) in buffer.windows(pattern.length()).enumerate() {
            if !pattern.is_matching(window) {
                continue;
            }

            return Ok(Some(address + index as u64));
        }

        Ok(None)
    }
}
