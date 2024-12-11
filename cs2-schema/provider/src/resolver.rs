use crate::{
    OffsetInfo,
    PROVIDER_INSTANCE,
};

#[macro_export]
macro_rules! runtime_offset {
    ($default_value:expr, $module:expr, $class_name:expr, $class_member:expr) => {{
        static mut RESOLVED_OFFSET: Option<u64> = None;

        #[allow(static_mut_refs)]
        let cached_offset = unsafe { &mut RESOLVED_OFFSET };
        if let Some(offset) = cached_offset {
            *offset
        } else {
            let resolved_value = $crate::resolve_offset(&$crate::OffsetInfo {
                default_value: $default_value,
                module: $module,
                class_name: $class_name,
                member: $class_member,
            });
            *cached_offset = Some(resolved_value);

            resolved_value
        }
    }};
}
pub fn resolve_offset(offset: &OffsetInfo) -> u64 {
    log::trace!(
        "Resolving offset {}::{}.{}",
        offset.module,
        offset.class_name,
        offset.member
    );
    let instance = PROVIDER_INSTANCE.read().unwrap();
    let Some(instance) = instance.as_ref() else {
        panic!("no schema provider set");
    };

    let Some(value) = instance.resolve_offset(offset) else {
        panic!("could not resolve offset for {:?}", offset);
    };

    log::trace!(" -> 0x{:X}", value);
    value
}
