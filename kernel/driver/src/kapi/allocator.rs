use core::alloc::GlobalAlloc;

use winapi::shared::ntdef::PVOID;

use crate::kdef::{ExAllocatePoolWithTag, POOL_TYPE, ExFreePoolWithTag};

const POOL_TAG: u32 = 0x123333;

struct NonPagedAllocator;
unsafe impl GlobalAlloc for NonPagedAllocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        ExAllocatePoolWithTag(POOL_TYPE::NonPagedPool, layout.size(), POOL_TAG) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        ExFreePoolWithTag(ptr as PVOID, POOL_TAG);
    }
}

#[global_allocator]
static GLOBAL_ALLOC: NonPagedAllocator = NonPagedAllocator;