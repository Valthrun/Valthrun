#![allow(dead_code)]

use std::{
    any::Any,
    ffi::CStr,
    fmt::Debug,
    sync::{
        Arc,
        Weak,
    },
};

use anyhow::Context;
use cs2_schema_declaration::{
    MemoryDriver,
    MemoryHandle,
    SchemaValue,
};
use obfstr::obfstr;
use valthrun_kernel_interface::{
    requests::{
        RequestCSModule,
        RequestKeyboardState,
        RequestMouseMove,
        RequestProtectionToggle,
        ResponseCsModule,
    },
    CS2ModuleInfo,
    KInterfaceError,
    KernelInterface,
    KeyboardState,
    ModuleInfo,
    MouseState,
};

use crate::{
    Signature,
    SignatureType,
};

pub struct CSMemoryDriver(Weak<CS2Handle>);
impl MemoryDriver for CSMemoryDriver {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn read_slice(&self, address: u64, slice: &mut [u8]) -> anyhow::Result<()> {
        let cs2 = self.0.upgrade().context("cs2 handle has been dropped")?;
        cs2.read_slice(&[address], slice)
    }

    fn read_cstring(
        &self,
        address: u64,
        expected_length: Option<usize>,
        _max_length: Option<usize>,
    ) -> anyhow::Result<String> {
        let cs2 = self.0.upgrade().context("cs2 handle has been dropped")?;
        cs2.read_string(&[address], expected_length)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Module {
    Client,
    Engine,
    Schemasystem,
}

static EMPTY_MODULE_INFO: ModuleInfo = ModuleInfo {
    base_address: 0,
    module_size: usize::MAX,
};
impl Module {
    pub fn get_base_offset<'a>(&self, module_info: &'a CS2ModuleInfo) -> Option<&'a ModuleInfo> {
        Some(match self {
            Module::Client => &module_info.client,
            Module::Engine => &module_info.engine,
            Module::Schemasystem => &module_info.schemasystem,
        })
    }
}

/// Handle to the CS2 process
pub struct CS2Handle {
    weak_self: Weak<Self>,

    pub ke_interface: KernelInterface,
    pub module_info: CS2ModuleInfo,
}

impl CS2Handle {
    pub fn create() -> anyhow::Result<Arc<Self>> {
        let interface = KernelInterface::create(obfstr!("\\\\.\\GLOBALROOT\\Device\\valthrun"))?;

        /*
         * Please no not analyze me:
         * https://www.unknowncheats.me/wiki/Valve_Anti-Cheat:VAC_external_tool_detection_(and_more)
         *
         * Even tough we don't have open handles to CS2 we don't want anybody to read our process.
         */
        unsafe { interface.execute_request(&RequestProtectionToggle { enabled: true }) }?;

        let module_info =
            unsafe { interface.execute_request::<RequestCSModule>(&RequestCSModule {}) }?;
        let module_info = match module_info {
            ResponseCsModule::Success(info) => info,
            ResponseCsModule::NoProcess => return Err(KInterfaceError::ProcessDoesNotExists.into()),
            error => anyhow::bail!("failed to load module info: {:?}", error),
        };

        log::debug!(
            "{}. Process id {}",
            obfstr!("Successfully initialized CS2 handle"),
            module_info.process_id
        );
        log::debug!(
            "  {} located at {:X} ({:X} bytes)",
            obfstr!("client.dll"),
            module_info.client.base_address,
            module_info.client.module_size
        );
        log::debug!(
            "  {} located at {:X} ({:X} bytes)",
            obfstr!("engine2.dll"),
            module_info.engine.base_address,
            module_info.engine.module_size
        );

        Ok(Arc::new_cyclic(|weak_self| Self {
            weak_self: weak_self.clone(),

            ke_interface: interface,
            module_info,
        }))
    }

    pub fn protect_process(&self) -> anyhow::Result<()> {
        unsafe {
            self.ke_interface
                .execute_request(&RequestProtectionToggle { enabled: true })
        }?;
        Ok(())
    }

    pub fn send_keyboard_state(&self, states: &[KeyboardState]) -> anyhow::Result<()> {
        unsafe {
            self.ke_interface.execute_request(&RequestKeyboardState {
                buffer: states.as_ptr(),
                state_count: states.len(),
            })
        }?;

        Ok(())
    }

    pub fn send_mouse_state(&self, states: &[MouseState]) -> anyhow::Result<()> {
        unsafe {
            self.ke_interface.execute_request(&RequestMouseMove {
                buffer: states.as_ptr(),
                state_count: states.len(),
            })
        }?;

        Ok(())
    }

    pub fn module_address(&self, module: Module, address: u64) -> Option<u64> {
        let module = module.get_base_offset(&self.module_info)?;
        if (address as usize) < module.base_address
            || (address as usize) >= (module.base_address + module.module_size)
        {
            None
        } else {
            Some(address - module.base_address as u64)
        }
    }

    pub fn memory_address(&self, module: Module, offset: u64) -> anyhow::Result<u64> {
        Ok(module
            .get_base_offset(&self.module_info)
            .context("invalid module")?
            .base_address as u64
            + offset)
    }

    pub fn read_sized<T: Copy>(&self, offsets: &[u64]) -> anyhow::Result<T> {
        Ok(self
            .ke_interface
            .read(self.module_info.process_id, offsets)?)
    }

    pub fn read_slice<T: Copy>(&self, offsets: &[u64], buffer: &mut [T]) -> anyhow::Result<()> {
        Ok(self
            .ke_interface
            .read_slice(self.module_info.process_id, offsets, buffer)?)
    }

    pub fn read_string(
        &self,
        offsets: &[u64],
        expected_length: Option<usize>,
    ) -> anyhow::Result<String> {
        let mut expected_length = expected_length.unwrap_or(8); // Using 8 as we don't know how far we can read
        let mut buffer = Vec::new();

        // FIXME: Do cstring reading within the kernel driver!
        loop {
            buffer.resize(expected_length, 0u8);
            self.read_slice(offsets, buffer.as_mut_slice())
                .context("read_string")?;

            if let Ok(str) = CStr::from_bytes_until_nul(&buffer) {
                return Ok(str.to_str().context("invalid string contents")?.to_string());
            }

            expected_length += 8;
        }
    }

    fn create_memory_driver(&self) -> Arc<dyn MemoryDriver> {
        Arc::new(CSMemoryDriver(self.weak_self.clone())) as Arc<(dyn MemoryDriver + 'static)>
    }

    /// Read the whole schema class and return a wrapper around the data.
    pub fn read_schema<T: SchemaValue>(&self, offsets: &[u64]) -> anyhow::Result<T> {
        let address = if offsets.len() == 1 {
            offsets[0]
        } else {
            let base = self.read_sized::<u64>(&offsets[0..offsets.len() - 1])?;
            base + offsets[offsets.len() - 1]
        };

        let schema_size = T::value_size().context("schema must have a size")?;
        let mut memory = MemoryHandle::from_driver(&self.create_memory_driver(), address);
        memory.cache(schema_size as usize)?;

        T::from_memory(memory)
    }

    /// Reference an address in memory and wrap the schema class around it.
    /// Every member accessor will read the current bytes from the process memory.
    ///
    /// This function should be used if a class is only accessed once or twice.
    pub fn reference_schema<T: SchemaValue>(&self, offsets: &[u64]) -> anyhow::Result<T> {
        let address = if offsets.len() == 1 {
            offsets[0]
        } else {
            let base = self.read_sized::<u64>(&offsets[0..offsets.len() - 1])?;
            base + offsets[offsets.len() - 1]
        };

        T::from_memory(MemoryHandle::from_driver(
            &self.create_memory_driver(),
            address,
        ))
    }

    pub fn resolve_signature(&self, module: Module, signature: &Signature) -> anyhow::Result<u64> {
        log::trace!("Resolving '{}' in {:?}", signature.debug_name, module);
        let module_info = module
            .get_base_offset(&self.module_info)
            .context("invalid module")?;

        let inst_offset = self
            .ke_interface
            .find_pattern(
                self.module_info.process_id,
                module_info.base_address as u64,
                module_info.module_size,
                &*signature.pattern,
            )?
            .context("failed to find pattern")?;

        let value = self.reference_schema::<u32>(&[inst_offset + signature.offset])? as u64;
        let value = match &signature.value_type {
            SignatureType::Offset => value,
            SignatureType::RelativeAddress { inst_length } => inst_offset + value + inst_length,
        };

        match &signature.value_type {
            SignatureType::Offset => log::trace!(
                " => {:X} (inst at {:X})",
                value,
                self.module_address(module, inst_offset).unwrap_or(u64::MAX)
            ),
            SignatureType::RelativeAddress { .. } => log::trace!(
                "  => {:X} ({:X})",
                value,
                self.module_address(module, value).unwrap_or(u64::MAX)
            ),
        }
        Ok(value)
    }
}
