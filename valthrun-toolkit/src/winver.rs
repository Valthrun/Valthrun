use std::mem;

use anyhow::{
    bail,
    Result,
};
use windows::Win32::{
    Foundation::{
        NTSTATUS,
        STATUS_SUCCESS,
    },
    System::SystemInformation::OSVERSIONINFOEXW,
};

type OSVERSIONINFOEX = OSVERSIONINFOEXW;

#[link(name = "ntdll")]
extern "system" {
    fn RtlGetVersion(info: &mut OSVERSIONINFOEX) -> NTSTATUS;
}

// Calls the Win32 API function RtlGetVersion to get the OS version information:
// https://msdn.microsoft.com/en-us/library/mt723418(v=vs.85).aspx
pub fn version_info() -> Result<OSVERSIONINFOEX> {
    let mut info: OSVERSIONINFOEX = unsafe { mem::zeroed() };
    info.dwOSVersionInfoSize = mem::size_of::<OSVERSIONINFOEX>() as u32;

    if unsafe { RtlGetVersion(&mut info) } == STATUS_SUCCESS {
        Ok(info)
    } else {
        bail!("Failed to get version")
    }
}
