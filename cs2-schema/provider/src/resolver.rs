use crate::{
    OffsetInfo,
    PROVIDER_INSTANCE,
};

#[macro_export]
macro_rules! runtime_offset {
    ($default_value:expr, $module:expr, $class_name:expr, $class_member:expr) => {{
        static mut RESOLVED_OFFSET: Option<u64> = None;

        #[allow(static_mut_refs)]
        $crate::resolve_offset(
            unsafe { &mut RESOLVED_OFFSET },
            &$crate::OffsetInfo {
                default_value: $default_value,
                module: $module,
                class_name: $class_name,
                member: $class_member,
            },
        )
    }};
}
pub fn resolve_offset(cache: &mut Option<u64>, offset: &OffsetInfo) -> u64 {
    *cache.get_or_insert_with(|| {
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
    })
}
