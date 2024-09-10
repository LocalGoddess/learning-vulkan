use ash::vk::{
    self, DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT,
    DebugUtilsMessengerCallbackDataEXT,
};

use std::{ffi::CStr, fs::File, io::Read};

#[inline]
pub fn str_to_cstr(rusty: &str) -> &CStr {
    CStr::from_bytes_with_nul(rusty.as_bytes()).expect("Failed to convert rustaican str to c str")
}

#[inline]
pub fn str_slice_to_cstr_vec<'a>(rusty: &'a [&'a str]) -> Vec<&'a CStr> {
    rusty.iter().map(|x| str_to_cstr(x)).collect()
}

pub fn read_shader_file(path: &str) -> Vec<u32> {
    let mut file = File::open(path).expect("Failed to open shader file");
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .expect("Failed to read shader file");

    if buffer.len() % 4 != 0 {
        panic!("File size is not a multiple of 4 bytes");
    }

    buffer
        .chunks(4)
        .map(|x| u32::from_le_bytes(x.try_into().unwrap()))
        .collect()
}

/// A debug extension callback function
/// # Safety
/// please only use for vulkan
pub unsafe extern "system" fn vulkan_debug_extension_callback(
    message_severity: DebugUtilsMessageSeverityFlagsEXT,
    message_type: DebugUtilsMessageTypeFlagsEXT,
    callback_data: *const DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data_owned = *callback_data;
    let message_id_number = callback_data_owned.message_id_number;

    let message_id_name = if callback_data_owned.p_message_id_name.is_null() {
        ""
    } else {
        CStr::from_ptr(callback_data_owned.p_message_id_name)
            .to_str()
            .unwrap()
    };

    let message_content = if callback_data_owned.p_message.is_null() {
        ""
    } else {
        CStr::from_ptr(callback_data_owned.p_message)
            .to_str()
            .unwrap()
    };

    tracing::info!("{message_severity:?}:{message_type:?} [{message_id_name} ({message_id_number})] : {message_content}");

    vk::FALSE
}
