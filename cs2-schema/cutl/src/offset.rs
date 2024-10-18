pub type OffsetResolverFn = dyn Fn(&OffsetInfo) -> u64;

#[derive(Debug, Clone, Copy)]
pub struct OffsetInfo {
    pub default_value: u64,
    pub module: &'static str,
    pub class_name: &'static str,
    pub member: &'static str,
}

static mut OFFSET_RESOLVER: &'static OffsetResolverFn = &default_offset_resolver;
fn default_offset_resolver(info: &OffsetInfo) -> u64 {
    info.default_value
}

#[macro_export]
macro_rules! runtime_offset {
    ($default_value:expr, $module:expr, $class_name:expr, $class_member:expr) => {{
        static mut RESOLVED_OFFSET: Option<u64> = None;

        #[allow(static_mut_refs)]
        let cached_offset = unsafe { &mut RESOLVED_OFFSET };
        if let Some(offset) = cached_offset {
            *offset
        } else {
            let resolved_value = $crate::resolve_offset(&OffsetInfo {
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
pub fn resolve_offset(info: &OffsetInfo) -> u64 {
    #[allow(static_mut_refs)]
    let offset_resolver = unsafe { &OFFSET_RESOLVER };
    offset_resolver(info)
}

pub fn set_offset_resolver(resolver: &'static OffsetResolverFn) {
    #[allow(static_mut_refs)]
    let offset_resolver = unsafe { &mut OFFSET_RESOLVER };
    *offset_resolver = resolver;
}
