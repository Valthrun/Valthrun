#![allow(dead_code)]

use anyhow::Context;
use obfstr::obfstr;
use std::{ffi::CStr, fmt::Debug};
use kinterface::{
    requests::{RequestCSModule, ResponseCsModule, RequestProtectionToggle},
    CS2ModuleInfo, KernelInterface, ModuleInfo, SearchPattern,
};

#[derive(Debug, Clone, Copy)]
pub enum Module {
    /// Read the absolute address in memory
    Absolute,

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
            Module::Absolute => &EMPTY_MODULE_INFO,
            Module::Client => &module_info.client,
            Module::Engine => &module_info.engine,
            Module::Schemasystem => &module_info.schemasystem,
        })
    }
}

/// Handle to the CS2 process
pub struct CS2Handle {
    pub ke_interface: KernelInterface,
    pub module_info: CS2ModuleInfo,
}

impl CS2Handle {
    pub fn create() -> anyhow::Result<Self> {
        let interface = KernelInterface::create(obfstr!("\\\\.\\valthrun"))?;

        /*
         * Please no not analyze me:
         * https://www.unknowncheats.me/wiki/Valve_Anti-Cheat:VAC_external_tool_detection_(and_more)
         *
         * Even tough we don't have open handles to CS2 we don't want anybody to read our process.
         */
        interface.execute_request(&RequestProtectionToggle{ enabled: true })?;
        
        let module_info = interface.execute_request::<RequestCSModule>(&RequestCSModule {})?;
        let module_info = match module_info {
            ResponseCsModule::Success(info) => info,
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

        Ok(Self {
            ke_interface: interface,
            module_info,
        })
    }

    pub fn protect_process(&self) -> anyhow::Result<()> {
        self.ke_interface
            .execute_request(&RequestProtectionToggle { enabled: true })?;
        Ok(())
    }

    pub fn module_address(&self, module: Module, address: u64) -> Option<u64> {
        let module = module.get_base_offset(&self.module_info)?;
        if (address as usize) < module.base_address || (address as usize) >= (module.base_address + module.module_size) {
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

    pub fn read<T>(&self, module: Module, offsets: &[u64]) -> anyhow::Result<T> {
        let mut offsets = offsets.to_vec();
        offsets[0] += module
            .get_base_offset(&self.module_info)
            .context("invalid module")?
            .base_address as u64;

        Ok(self
            .ke_interface
            .read(self.module_info.process_id, offsets.as_slice())?)
    }

    pub fn read_slice<T: Sized>(
        &self,
        module: Module,
        offsets: &[u64],
        buffer: &mut [T],
    ) -> anyhow::Result<()> {
        let mut offsets = offsets.to_vec();
        offsets[0] += module
            .get_base_offset(&self.module_info)
            .context("invalid module")?
            .base_address as u64;

        Ok(self
            .ke_interface
            .read_slice(self.module_info.process_id, offsets.as_slice(), buffer)?)
    }

    pub fn read_vec<T: Sized>(
        &self,
        module: Module,
        offsets: &[u64],
        length: usize,
    ) -> anyhow::Result<Vec<T>> {
        let mut offsets = offsets.to_vec();
        offsets[0] += module
            .get_base_offset(&self.module_info)
            .context("invalid module")?
            .base_address as u64;

        Ok(self
            .ke_interface
            .read_vec(self.module_info.process_id, offsets.as_slice(), length)?)
    }

    pub fn read_string(
        &self,
        module: Module,
        offsets: &[u64],
        expected_length: Option<usize>,
    ) -> anyhow::Result<String> {
        let mut expected_length = expected_length.unwrap_or(8); // Using 8 as we don't know how far we can read
        let mut buffer = Vec::new();

        // FIXME: Do cstring reading within the kernel driver!
        loop {
            buffer.resize(expected_length, 0u8);
            self.read_slice(module, offsets, buffer.as_mut_slice())?;
            if let Ok(str) = CStr::from_bytes_until_nul(&buffer) {
                return Ok(str.to_str().context("invalid string contents")?.to_string());
            }

            expected_length += 8;
        }
    }

    pub fn find_pattern(
        &self,
        module: Module,
        pattern: &dyn SearchPattern,
    ) -> anyhow::Result<Option<u64>> {
        let module = module
            .get_base_offset(&self.module_info)
            .context("invalid module")?;
        let address = self.ke_interface.find_pattern(
            self.module_info.process_id,
            module.base_address as u64,
            module.module_size,
            pattern,
        )?;
        Ok(address.map(|addr| addr.wrapping_sub(module.base_address as u64)))
    }
}
