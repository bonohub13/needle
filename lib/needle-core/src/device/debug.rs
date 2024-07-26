use ash::{ext::debug_utils, vk};
use std::ffi::{c_void, CStr};

pub struct DebugUtilsMessenger {
    instance: debug_utils::Instance,
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,
}

unsafe extern "system" fn debug_callback(
    msg_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    msg_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_cb_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let msg = CStr::from_ptr((*p_cb_data).p_message);
    let msg_severity = match msg_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]",
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
        _ => "[Unknown]",
    };
    let msg_type = match msg_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[GENERAL]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[VALIDATION]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[PERFORMANCE]",
        vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING => "[DEVICE_ADDRESS_BINDING]",
        _ => "[Unknown]",
    };

    eprintln!(
        "validation layers ({} | {}): {:?}",
        msg_severity, msg_type, msg
    );

    vk::FALSE
}

impl DebugUtilsMessenger {
    pub fn new(entry: &ash::Entry, instance: &ash::Instance) -> anyhow::Result<Self> {
        let instance = debug_utils::Instance::new(entry, instance);
        let debug_utils_messenger = {
            let create_info = Self::populate_debug_message_create_info();

            unsafe { instance.create_debug_utils_messenger(&create_info, None) }?
        };

        Ok(Self {
            instance,
            debug_utils_messenger,
        })
    }

    pub fn null(entry: &ash::Entry, instance: &ash::Instance) -> Self {
        let instance = debug_utils::Instance::new(entry, instance);
        let debug_utils_messenger = vk::DebugUtilsMessengerEXT::null();

        Self {
            instance,
            debug_utils_messenger,
        }
    }

    pub unsafe fn destroy(&self) {
        self.instance
            .destroy_debug_utils_messenger(self.debug_utils_messenger, None);
    }

    #[inline]
    pub fn populate_debug_message_create_info<'a>() -> vk::DebugUtilsMessengerCreateInfoEXT<'a> {
        vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .pfn_user_callback(Some(debug_callback))
    }
}
