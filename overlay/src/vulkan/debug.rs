use std::ffi::{
    c_void,
    CStr,
};

use ash::{
    ext,
    vk::{
        self,
    },
};

pub fn create_extension_info() -> vk::DebugUtilsMessengerCreateInfoEXT<'static> {
    vk::DebugUtilsMessengerCreateInfoEXT::default()
        .flags(vk::DebugUtilsMessengerCreateFlagsEXT::empty())
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(message_callback))
}

#[inline]
pub const fn extension_name() -> &'static CStr {
    ext::debug_utils::NAME
}

unsafe extern "system" fn message_callback(
    flag: vk::DebugUtilsMessageSeverityFlagsEXT,
    typ: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    use vk::DebugUtilsMessageSeverityFlagsEXT as Flag;

    let message = CStr::from_ptr((*p_callback_data).p_message);
    match flag {
        Flag::VERBOSE => log::debug!("{typ:?} - {}", message.to_string_lossy()),
        Flag::INFO => log::info!("{typ:?} - {}", message.to_string_lossy()),
        Flag::WARNING => log::warn!("{typ:?} - {}", message.to_string_lossy()),
        _ => log::error!("{typ:?} - {}", message.to_string_lossy()),
    }
    vk::FALSE
}
