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
        ControllerInfo,
        DriverInfo,
        DriverRequest,
        MemoryAccessMode,
        RequestInitialize,
        RequestRead,
        RequestReportSend,
        RequestWrite,
        ResponseRead,
        ResponseWrite,
        INIT_STATUS_CONTROLLER_OUTDATED,
        INIT_STATUS_DRIVER_OUTDATED,
        INIT_STATUS_SUCCESS,
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
    driver_version: u32,

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

        let mut interface = Self {
            driver_handle,
            driver_version: 0,

            read_calls: AtomicUsize::new(0),
        };
        interface.initialize()?;

        Ok(interface)
    }

    fn initialize(&mut self) -> KResult<()> {
        let controller_info = ControllerInfo {};
        let mut driver_info = DriverInfo {};

        let result = unsafe {
            self.execute_request(&RequestInitialize {
                target_version: 0x00_02_00_00,

                controller_info: &controller_info,
                controller_info_length: core::mem::size_of_val(&controller_info),

                driver_info: &mut driver_info,
                driver_info_length: core::mem::size_of_val(&driver_info),
            })?
        };

        match result.status_code {
            INIT_STATUS_SUCCESS => {}
            INIT_STATUS_CONTROLLER_OUTDATED => {
                return Err(KInterfaceError::DriverTooNew {
                    driver_version: result.driver_version,
                })
            }
            INIT_STATUS_DRIVER_OUTDATED => {
                return Err(KInterfaceError::DriverTooOld {
                    driver_version: result.driver_version,
                })
            }
            status => return Err(KInterfaceError::InitializeInvalidStatus(status)),
        };

        self.driver_version = result.driver_version;
        log::debug!(
            "Successfully initialized kernel interface with driver version: {}.{}.{}",
            (result.driver_version >> 24) & 0xFF,
            (result.driver_version >> 16) & 0xFF,
            (result.driver_version >> 8) & 0xFF
        );
        Ok(())
    }

    pub fn driver_version(&self) -> u32 {
        self.driver_version
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
                mode: MemoryAccessMode::AttachProcess,

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
            ResponseRead::AccessModeUnavailable => Err(KInterfaceError::RequestFailed),
        }
    }

    #[must_use]
    pub fn write<T: Copy>(&self, process_id: i32, address: u64, value: &T) -> KResult<()> {
        let buffer = unsafe {
            std::slice::from_raw_parts(
                std::mem::transmute::<_, *mut u8>(value),
                std::mem::size_of::<T>(),
            )
        };

        self.write_slice(process_id, address, buffer)
    }

    #[must_use]
    pub fn write_slice<T: Copy>(&self, process_id: i32, address: u64, buffer: &[T]) -> KResult<()> {
        let result = unsafe {
            self.execute_request(&RequestWrite {
                process_id,
                mode: MemoryAccessMode::AttachProcess,

                address: address as usize,
                buffer: buffer.as_ptr() as *const u8,
                count: buffer.len() * core::mem::size_of::<T>(),
            })
        }?;

        match result {
            ResponseWrite::Success => Ok(()),
            ResponseWrite::InvalidAddress => {
                let mut offsets = [0; IO_MAX_DEREF_COUNT];
                offsets[0] = address;
                Err(KInterfaceError::InvalidAddress {
                    target_address: address,
                    resolved_offsets: [0; IO_MAX_DEREF_COUNT],
                    resolved_offset_count: 0,
                    offsets,
                    offset_count: 1,
                })
            }
            ResponseWrite::UnknownProcess => Err(KInterfaceError::ProcessDoesNotExists),
            ResponseWrite::UnsuppportedAccessMode => Err(KInterfaceError::AccessModeUnavailable),
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

    pub fn add_metrics_record(&self, record_type: &str, record_payload: &str) -> KResult<()> {
        unsafe {
            self.execute_request(&RequestReportSend {
                report_type: record_type.as_ptr(),
                report_type_length: record_type.len(),

                report_payload: record_payload.as_ptr(),
                report_payload_length: record_payload.len(),
            })
        }?;

        Ok(())
    }
}
