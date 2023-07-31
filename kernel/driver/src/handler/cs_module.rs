use alloc::format;
use anyhow::Context;
use obfstr::obfstr;
use valthrun_driver_shared::{requests::{RequestCSModule, ResponseCsModule}, CS2ModuleInfo};

use crate::kapi::find_processes_by_name;

pub fn handler_get_modules(_req: &RequestCSModule, res: &mut ResponseCsModule) -> anyhow::Result<()> {
    log::debug!("{}", obfstr!("Searching for CS2 process."));
    let cs2_process_candidates = find_processes_by_name(obfstr!("cs2.exe"))?;
    let cs2_process = match cs2_process_candidates.len() {
        0 => {
            *res = ResponseCsModule::NoProcess;
            return Ok(());
        },
        1 => {
            cs2_process_candidates.first().unwrap()
        },
        count => {
            *res = ResponseCsModule::UbiquitousProcesses(count);
            return Ok(());
        }
    };
    
    let cs2_process_id = cs2_process.get_id();
    log::trace!("{} process id {}. PEP at {:X}", obfstr!("CS2"), cs2_process_id, cs2_process.eprocess() as u64);

    let mut module_info: CS2ModuleInfo = Default::default();
    module_info.process_id = cs2_process_id;

    let attached_process = cs2_process.attach();
    module_info.client = attached_process.get_module(obfstr!("client.dll"))
        .with_context(|| format!("missing {}", obfstr!("client.dll")))?;

    module_info.engine = attached_process.get_module(obfstr!("engine2.dll"))
        .with_context(|| format!("missing {}", obfstr!("engine2.dll")))?;

    module_info.schemasystem = attached_process.get_module(obfstr!("schemasystem.dll"))
        .with_context(|| format!("missing {}", obfstr!("schemasystem.dll")))?;
    drop(attached_process);
    
    *res = ResponseCsModule::Success(module_info);
    Ok(())
}
