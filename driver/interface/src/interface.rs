use core::{
    mem,
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
};
use std::{
    env,
    error::Error,
    fs,
    path::PathBuf,
};

use libloading::Library;
use obfstr::obfstr;
use valthrun_driver_protocol::{
    command::{
        DriverCommand,
        DriverCommandInitialize,
        DriverCommandInputKeyboard,
        DriverCommandInputMouse,
        DriverCommandMetricsReportSend,
        DriverCommandProcessMemoryRead,
        DriverCommandProcessMemoryWrite,
        DriverCommandProcessModules,
        DriverCommandProcessProtection,
        InitializeResult,
        KeyboardState,
        MouseState,
        ProcessModulesResult,
        ProcessProtectionMode,
        VersionInfo,
    },
    types::{
        DriverFeature,
        MemoryAccessResult,
        ProcessFilter as ProtocolProcessFilter,
        ProcessId,
        ProcessModuleInfo,
    },
    CommandResult,
    FnCommandHandler,
    PROTOCOL_VERSION,
};

use crate::{
    IResult,
    InterfaceError,
};

/// Interface for a Valthrun memory driver
pub struct DriverInterface {
    _library: Library,
    fn_command_handler: FnCommandHandler,

    driver_version: VersionInfo,
    driver_features: DriverFeature,

    read_calls: AtomicUsize,
}

impl DriverInterface {
    fn populate_library_paths() -> Vec<PathBuf> {
        let mut result = Vec::with_capacity(64);
        if let Ok(path) = env::var(obfstr!("VT_DRIVER_PATH")) {
            log::debug!("Adding env driver path: {}", path);
            result.push(PathBuf::from(path));
        }

        for directory in [
            env::var(obfstr!("VT_DRIVER_DIR")).map(PathBuf::from).ok(),
            env::current_exe()
                .ok()
                .map(|v| v.parent().map(|v| v.to_owned()))
                .flatten(),
        ] {
            let Some(directory) = directory else { continue };

            if let Ok(driver_name) = env::var(obfstr!("VT_DRIVER_NAME")) {
                result.push(directory.join(driver_name));
            }

            match fs::read_dir(&directory) {
                Ok(dir) => {
                    log::debug!("Adding drivers from {}", directory.display());
                    /*
                     * Add all dlls which start with driver_/valthrun_driver_ to the candidate list.
                     * Starting the driver which has been least recently modified.
                     */
                    let mut candidates = dir
                        .filter_map(|entry| entry.ok())
                        .map(|entry| entry.file_name().to_string_lossy().to_string())
                        .filter(|file_name| {
                            (file_name.starts_with("driver_")
                                || file_name.starts_with(obfstr!("valthrun_driver_")))
                                && file_name.ends_with(".dll")
                        })
                        .map(|file_name| directory.join(file_name))
                        .filter_map(|file| Some((file.metadata().ok()?.modified().ok()?, file)))
                        .collect::<Vec<_>>();

                    candidates.sort_by_key(|(timestamp, _file)| *timestamp);
                    result.extend(candidates.into_iter().rev().map(|(_, file)| file));
                }
                Err(err) => {
                    log::debug!(
                        "Skipping looking for driver in {}: {}",
                        directory.display(),
                        err
                    );
                }
            }
        }

        result
    }

    pub fn create_from_env() -> IResult<Self> {
        for path in Self::populate_library_paths() {
            log::debug!("Trying to load driver from {}", path.display());
            match unsafe { Library::new(&path) } {
                Ok(lib) => {
                    log::debug!("    -> success.");
                    log::debug!("Initialize driver interface.",);
                    return Self::create(lib);
                }
                Err(err) => {
                    if let Some(err) = err.source() {
                        log::debug!("    error: {:#}", err);
                    } else {
                        log::debug!("    error: {:#}", err);
                    }
                }
            }
        }

        Err(InterfaceError::NoDriverFound)
    }

    pub fn create(library: Library) -> IResult<Self> {
        let fn_command_handler = unsafe { *library.get::<FnCommandHandler>(b"execute_command\0")? };

        let mut interface = Self {
            _library: library,
            fn_command_handler,

            driver_version: VersionInfo::default(),
            driver_features: DriverFeature::empty(),

            read_calls: AtomicUsize::new(0),
        };
        interface.initialize()?;
        Ok(interface)
    }

    #[must_use]
    fn execute_command<C: DriverCommand>(&self, command: &mut C) -> IResult<String> {
        let mut error_buffer = Vec::<u8>::with_capacity(0x500);
        error_buffer.resize(0x500, 0);

        let status = unsafe {
            (self.fn_command_handler)(
                C::COMMAND_ID,
                command as *mut _ as *mut u8,
                mem::size_of::<C>(),
                error_buffer.as_mut_ptr(),
                error_buffer.len(),
            )
        };
        let result = CommandResult::from_bits_retain(status);

        let error = {
            let error_length = error_buffer
                .iter()
                .position(|v| *v == 0)
                .unwrap_or(error_buffer.len());
            error_buffer.truncate(error_length);
            String::from_utf8_lossy_owned(error_buffer)
        };

        Err(match result {
            CommandResult::Success => return Ok(error),
            CommandResult::Error => InterfaceError::CommandGenericError { message: error },

            CommandResult::CommandParameterInvalid => InterfaceError::CommandGenericError {
                message: format!("parameter invalid: {}", error),
            },
            CommandResult::CommandInvalid => InterfaceError::CommandGenericError {
                message: format!("command invalid"),
            },

            _ => InterfaceError::CommandGenericError {
                message: format!("invalid command result"),
            },
        })
    }

    fn initialize(&mut self) -> IResult<()> {
        let mut command = DriverCommandInitialize::default();
        command.client_protocol_version = PROTOCOL_VERSION;
        command.client_version = {
            let mut version_info = VersionInfo::default();

            version_info.set_application_name("valthrun-kinterface");

            version_info.version_major = 0;
            version_info.version_minor = 0;
            version_info.version_patch = 0;

            version_info
        };

        self.execute_command(&mut command)?;
        if command.client_protocol_version != command.driver_protocol_version {
            return Err(InterfaceError::DriverProtocolMismatch {
                interface_protocol: command.client_protocol_version,
                driver_protocol: command.driver_protocol_version,
            });
        }

        match command.result {
            InitializeResult::Success => {}
            InitializeResult::Unavailable => {
                return Err(InterfaceError::InitializeDriverUnavailable);
            }
        };

        self.driver_version = command.driver_version;
        self.driver_features = command.driver_features;
        log::debug!(
            "Successfully initialized driver interface with driver {} (version: {}.{}.{}).",
            self.driver_version
                .get_application_name()
                .unwrap_or("unknown"),
            self.driver_version.version_major,
            self.driver_version.version_minor,
            self.driver_version.version_patch,
        );
        log::debug!("Supported features: {:?}", self.driver_features);
        Ok(())
    }

    pub fn driver_version(&self) -> &VersionInfo {
        &self.driver_version
    }

    pub fn driver_features(&self) -> DriverFeature {
        self.driver_features
    }

    #[must_use]
    pub fn total_read_calls(&self) -> usize {
        self.read_calls.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn read<T: Copy>(&self, process_id: ProcessId, address: u64) -> IResult<T> {
        let mut result = unsafe { std::mem::zeroed::<T>() };
        let result_buff = unsafe {
            std::slice::from_raw_parts_mut(
                std::mem::transmute::<_, *mut u8>(&mut result),
                std::mem::size_of::<T>(),
            )
        };

        self.read_slice(process_id, address, result_buff)?;
        Ok(result)
    }

    #[must_use]
    pub fn read_slice<T: Copy>(
        &self,
        process_id: ProcessId,
        address: u64,
        buffer: &mut [T],
    ) -> IResult<()> {
        self.read_calls.fetch_add(1, Ordering::Relaxed);

        let mut command = DriverCommandProcessMemoryRead::default();
        command.process_id = process_id;
        command.address = address;

        command.buffer = buffer.as_mut_ptr() as *mut u8;
        command.count = mem::size_of::<T>() * buffer.len();

        self.execute_command(&mut command)?;
        match command.result {
            MemoryAccessResult::Success => Ok(()),
            MemoryAccessResult::ProcessUnknown => Err(InterfaceError::ProcessUnknown),
            MemoryAccessResult::PartialSuccess { bytes_copied } => {
                log::trace!(
                    "Mem access failed for src {:X} to dst {:X} (len {:X}, copied: {:X})",
                    command.address,
                    command.buffer as u64,
                    command.count,
                    bytes_copied
                );
                Err(InterfaceError::MemoryAccessFailed)
            }
        }
    }

    #[must_use]
    pub fn write<T: Copy>(&self, process_id: ProcessId, address: u64, value: &T) -> IResult<()> {
        let buffer = unsafe {
            std::slice::from_raw_parts(
                std::mem::transmute::<_, *mut u8>(value),
                std::mem::size_of::<T>(),
            )
        };

        self.write_slice(process_id, address, buffer)
    }

    #[must_use]
    pub fn write_slice<T: Copy>(
        &self,
        process_id: ProcessId,
        address: u64,
        buffer: &[T],
    ) -> IResult<()> {
        let mut command = DriverCommandProcessMemoryWrite::default();
        command.process_id = process_id;
        command.address = address;

        command.buffer = buffer.as_ptr() as *const u8;
        command.count = mem::size_of::<T>() * buffer.len();

        self.execute_command(&mut command)?;
        match command.result {
            MemoryAccessResult::Success => Ok(()),
            MemoryAccessResult::ProcessUnknown => Err(InterfaceError::ProcessUnknown),
            MemoryAccessResult::PartialSuccess { .. } => Err(InterfaceError::MemoryAccessFailed),
        }
    }

    pub fn add_metrics_record(&self, record_type: &str, record_payload: &str) -> IResult<()> {
        let mut command = DriverCommandMetricsReportSend::default();
        if !command.set_report_type(record_type) {
            return Err(InterfaceError::ReportTypeTooLong);
        }

        command.report_payload = record_payload.as_ptr();
        command.report_payload_length = record_payload.as_bytes().len();

        self.execute_command(&mut command)?;
        Ok(())
    }

    pub fn toggle_process_protection(&self, mode: ProcessProtectionMode) -> IResult<()> {
        let mut command = DriverCommandProcessProtection::default();
        command.mode = mode;

        self.execute_command(&mut command)?;
        Ok(())
    }

    pub fn request_modules(
        &self,
        filter: &ProcessFilter,
    ) -> IResult<(ProcessId, Vec<ProcessModuleInfo>)> {
        let mut module_buffer = Vec::with_capacity(512);
        module_buffer.resize_with(512, Default::default);

        let mut command = DriverCommandProcessModules::default();
        command.target_process = match filter {
            ProcessFilter::Id { id } => ProtocolProcessFilter::Id { id: *id },
            ProcessFilter::Name { name } => ProtocolProcessFilter::ImageBaseName {
                name: name.as_ptr(),
                name_length: name.as_bytes().len(),
            },
        };

        let mut retry = 0;
        while retry <= 3 {
            command.module_buffer = module_buffer.as_mut_ptr();
            command.module_buffer_length = module_buffer.len();
            command.module_count = 0;

            self.execute_command(&mut command)?;

            match command.result {
                ProcessModulesResult::Success => {
                    module_buffer.truncate(command.module_count);
                    return Ok((command.process_id, module_buffer));
                }
                ProcessModulesResult::BufferTooSmall => {
                    module_buffer.resize_with(command.module_count, Default::default);
                    retry += 1;
                    continue;
                }
                ProcessModulesResult::ProcessUnknown => {
                    return Err(InterfaceError::ProcessUnknown);
                }
                ProcessModulesResult::ProcessUbiquitous => {
                    return Err(InterfaceError::ProcessUbiquitous);
                }
            }
        }

        Err(InterfaceError::CommandGenericError {
            message: format!("failed to allocate module buffer tree times"),
        })
    }

    pub fn send_keyboard_state(&self, states: &[KeyboardState]) -> IResult<()> {
        let mut command = DriverCommandInputKeyboard::default();
        command.buffer = states.as_ptr();
        command.state_count = states.len();

        self.execute_command(&mut command)?;
        Ok(())
    }

    pub fn send_mouse_state(&self, states: &[MouseState]) -> IResult<()> {
        let mut command = DriverCommandInputMouse::default();
        command.buffer = states.as_ptr();
        command.state_count = states.len();

        self.execute_command(&mut command)?;
        Ok(())
    }
}

pub enum ProcessFilter {
    Id { id: u32 },
    Name { name: String },
}
