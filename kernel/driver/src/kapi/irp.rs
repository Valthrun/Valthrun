use winapi::{shared::ntdef::NTSTATUS, km::wdm::{IRP, IoCompleteRequest, IO_PRIORITY::IO_NO_INCREMENT}};

pub trait IrpEx {
    fn complete_request(&mut self, status: NTSTATUS) -> NTSTATUS;
}

impl IrpEx for IRP {
    fn complete_request(&mut self, status: NTSTATUS) -> NTSTATUS {
        self.IoStatus.Information = status as usize;
        unsafe { IoCompleteRequest(self, IO_NO_INCREMENT) };
        return status;
    }
}