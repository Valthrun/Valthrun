use anyhow::Context;
use valthrun_driver_shared::{requests::{RequestCSModule, ResponseCsModule}, CS2ModuleInfo};

use crate::kapi::find_processes_by_name;

pub fn handler_get_modules(_req: &RequestCSModule, res: &mut ResponseCsModule) -> anyhow::Result<()> {
    log::debug!("Searching for CS process.");
    let cs2_process_candidates = find_processes_by_name("cs2.exe")?;
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
    log::trace!("CS2 process id {}. PEP at {:X}", cs2_process_id, cs2_process.eprocess() as u64);

    let mut module_info: CS2ModuleInfo = Default::default();
    module_info.process_id = cs2_process_id;

    let attached_process = cs2_process.attach();
    module_info.client = attached_process.get_module("client.dll")
        .context("missing client.dll")?;

    module_info.engine = attached_process.get_module("engine2.dll")
        .context("missing engine2.dll")?;

    module_info.schemasystem = attached_process.get_module("schemasystem.dll")
        .context("missing schemasystem.dll")?;

    *res = ResponseCsModule::Success(module_info);
    Ok(())
}
