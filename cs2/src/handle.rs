#![allow(dead_code)]

use std::{
    any::Any,
    ffi::CStr,
    fmt::Debug,
    ops::Deref,
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
use utils_state::{
    State,
    StateCacheType,
    StateRegistry,
};
use valthrun_kernel_interface::{
    com_from_env,
    KernelInterface,
    KeyboardState,
    ModuleInfo,
    MouseState,
    ProcessId,
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
    Tier0,
}

impl Module {
    fn get_module_name<'a>(&self) -> &'static str {
        match self {
            Module::Client => "client.dll",
            Module::Engine => "engine2.dll",
            Module::Schemasystem => "schemasystem.dll",
            Module::Tier0 => "tier0.dll",
        }
    }
}

/// Handle to the CS2 process
pub struct CS2Handle {
    weak_self: Weak<Self>,
    metrics: bool,

    modules: Vec<ModuleInfo>,
    process_id: ProcessId,

    pub ke_interface: KernelInterface,
}

impl CS2Handle {
    pub fn create(metrics: bool) -> anyhow::Result<Arc<Self>> {
        let interface = KernelInterface::create(com_from_env()?)?;

        /*
         * Please no not analyze me:
         * https://www.unknowncheats.me/wiki/Valve_Anti-Cheat:VAC_external_tool_detection_(and_more)
         *
         * Even tough we don't have open handles to CS2 we don't want anybody to read our process.
         */
        if let Err(err) = interface.toggle_process_protection(true) {
            log::warn!("Failed to enable process protection: {}", err)
        };

        let (process_id, modules) = interface.request_cs2_modules()?;
        log::debug!(
            "{}. Process id {}",
            obfstr!("Successfully initialized CS2 handle"),
            process_id
        );

        log::trace!("{} ({})", obfstr!("CS2 modules"), modules.len());
        for module in modules.iter() {
            log::trace!(
                "  - {} ({:X} - {:X})",
                module.base_dll_name(),
                module.base_address,
                module.base_address + module.module_size
            );
        }

        Ok(Arc::new_cyclic(|weak_self| Self {
            weak_self: weak_self.clone(),
            metrics,
            modules,
            process_id,

            ke_interface: interface,
        }))
    }

    fn get_module_info(&self, target: Module) -> Option<&ModuleInfo> {
        self.modules
            .iter()
            .find(|module| module.base_dll_name() == target.get_module_name())
    }

    pub fn process_id(&self) -> ProcessId {
        self.process_id
    }

    pub fn send_keyboard_state(&self, states: &[KeyboardState]) -> anyhow::Result<()> {
        self.ke_interface.send_keyboard_state(states)?;
        Ok(())
    }

    pub fn send_mouse_state(&self, states: &[MouseState]) -> anyhow::Result<()> {
        self.ke_interface.send_mouse_state(states)?;
        Ok(())
    }

    pub fn add_metrics_record(&self, record_type: &str, record_payload: &str) {
        if !self.metrics {
            /* user opted out */
            return;
        }

        let _ = self
            .ke_interface
            .add_metrics_record(record_type, record_payload);
    }

    pub fn module_address(&self, module: Module, address: u64) -> Option<u64> {
        let module = self.get_module_info(module)?;
        if (address as usize) < module.base_address
            || (address as usize) >= (module.base_address + module.module_size)
        {
            None
        } else {
            Some(address - module.base_address as u64)
        }
    }

    pub fn memory_address(&self, module: Module, offset: u64) -> anyhow::Result<u64> {
        Ok(self
            .get_module_info(module)
            .context("invalid module")?
            .base_address as u64
            + offset)
    }

    pub fn read_sized<T: Copy>(&self, offsets: &[u64]) -> anyhow::Result<T> {
        Ok(self.ke_interface.read(self.process_id, offsets)?)
    }

    pub fn read_slice<T: Copy>(&self, offsets: &[u64], buffer: &mut [T]) -> anyhow::Result<()> {
        Ok(self
            .ke_interface
            .read_slice(self.process_id, offsets, buffer)?)
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
        let module_info = self.get_module_info(module).context("invalid module")?;

        let inst_offset = self
            .ke_interface
            .find_pattern(
                self.process_id,
                module_info.base_address as u64,
                module_info.module_size,
                &*signature.pattern,
            )?
            .with_context(|| {
                format!(
                    "{} {}",
                    obfstr!("failed to find pattern"),
                    signature.debug_name
                )
            })?;

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

pub struct CS2HandleState(Arc<CS2Handle>);

impl CS2HandleState {
    pub fn new(value: Arc<CS2Handle>) -> Self {
        Self(value)
    }

    pub fn handle(&self) -> &Arc<CS2Handle> {
        &self.0
    }
}

impl State for CS2HandleState {
    type Parameter = ();

    fn create(_states: &StateRegistry, _param: Self::Parameter) -> anyhow::Result<Self> {
        anyhow::bail!("CS2 handle state must be manually set")
    }

    fn cache_type() -> StateCacheType {
        StateCacheType::Persistent
    }
}

impl Deref for CS2HandleState {
    type Target = CS2Handle;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}
