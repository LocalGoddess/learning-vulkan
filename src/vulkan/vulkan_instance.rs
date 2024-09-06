use std::ffi::CStr;

use ash::{
    ext::debug_utils,
    vk::{
        self, ApplicationInfo, DebugUtilsMessageSeverityFlagsEXT, DebugUtilsMessageTypeFlagsEXT,
        DebugUtilsMessengerCreateInfoEXT, DebugUtilsMessengerEXT, InstanceCreateFlags,
        InstanceCreateInfo,
    },
};
use winit::raw_window_handle::RawDisplayHandle;

use crate::{util::str_to_cstr, vulkan::vulkan_device::VulkanLogicalDevice};

use super::vulkan_device::VulkanPhysicalDevice;

#[allow(dead_code)]
pub struct VulkanInstance {
    vulkan_library: ash::Entry,
    pub vulkan_instance: ash::Instance,

    pub physical_device: VulkanPhysicalDevice,
    pub logical_device: VulkanLogicalDevice,

    debug_utils_extension: Option<DebugUtilsExtension>,
}

impl VulkanInstance {
    pub fn create(display_handle: RawDisplayHandle) -> Self {
        tracing::info!("Creating Vulkan instance");

        let vulkan_library = unsafe { ash::Entry::load() }.expect("Failed to load vulkan library");

        let application_info = ApplicationInfo::default()
            .application_name(str_to_cstr("Learning Vulkan\0"))
            .application_version(0)
            .engine_name(str_to_cstr("None\0"))
            .engine_version(0)
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
            InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            InstanceCreateFlags::default()
        };

        tracing::info!("Getting required extensions");
        let mut extension_names = if let Ok(extensions) =
            ash_window::enumerate_required_extensions(display_handle)
        {
            extensions.to_vec()
        } else {
            tracing::info!("Failed to retrieve required extensions from display handle... Attempting to apply manually");

            let mut extensions = vec![ash::khr::surface::NAME.as_ptr()];

            #[cfg(target_os = "windows")]
            extensions.push(ash::khr::win32_surface::NAME.as_ptr());

            #[cfg(target_os = "macos")]
            extensions.push(ash::mvk::macos_surface::NAME.as_ptr());

            #[cfg(target_os = "linux")]
            extensions.push(ash::khr::xcb_surface::NAME.as_ptr());

            extensions
        };

        #[cfg(debug_assertions)]
        extension_names.push(debug_utils::NAME.as_ptr());

        extension_names.append(&mut Self::get_applicable_compatability_extensions());

        let layer_names = if cfg!(debug_assertions) {
            vec![CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0")
                .unwrap()
                .as_ptr()]
        } else {
            Vec::new()
        };

        let instance_create_info = InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_layer_names(&layer_names)
            .enabled_extension_names(&extension_names)
            .flags(create_flags);

        let vulkan_instance =
            unsafe { vulkan_library.create_instance(&instance_create_info, None) }
                .expect("Failed to create Vulkan instance");

        let debug_utils_extension = DebugUtilsExtension::create(&vulkan_library, &vulkan_instance);

        let physical_device = VulkanPhysicalDevice::select(&vulkan_instance);
        tracing::info!("Selected {:?}", physical_device.get_device_info());

        let logical_device = VulkanLogicalDevice::create(&vulkan_instance, &physical_device);

        Self {
            vulkan_library,
            vulkan_instance,
            debug_utils_extension,
            physical_device,
            logical_device,
        }
    }

    /// Gets a vec of all extensions required by the target platform for compatability
    fn get_applicable_compatability_extensions() -> Vec<*const i8> {
        let mut support_extensions = Vec::new();

        if cfg!(any(target_os = "macos", target_os = "ios")) {
            support_extensions.push(ash::khr::portability_enumeration::NAME.as_ptr());
            support_extensions.push(ash::khr::get_display_properties2::NAME.as_ptr());
        }

        support_extensions
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        tracing::info!("Destroying Vulkan instance");
        unsafe {
            if self.debug_utils_extension.is_some() {
                let debug_utils = self.debug_utils_extension.as_ref().unwrap();
                debug_utils
                    .loader
                    .destroy_debug_utils_messenger(debug_utils.callback, None);
            }

            self.logical_device.logical_device.destroy_device(None);

            self.vulkan_instance.destroy_instance(None);
        }
    }
}

pub struct DebugUtilsExtension {
    pub loader: debug_utils::Instance,
    pub callback: DebugUtilsMessengerEXT,
}

impl DebugUtilsExtension {
    pub fn create(
        vulkan_library: &ash::Entry,
        vulkan_instance: &ash::Instance,
    ) -> Option<DebugUtilsExtension> {
        if cfg!(debug_assertions) {
            tracing::info!("Creating debug utils");
            let debug_create_info = DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(
                    DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | DebugUtilsMessageSeverityFlagsEXT::INFO,
                )
                .message_type(
                    DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .pfn_user_callback(Some(crate::util::vulkan_debug_extension_callback));

            let loader = debug_utils::Instance::new(vulkan_library, vulkan_instance);
            let callback = unsafe { loader.create_debug_utils_messenger(&debug_create_info, None) }
                .expect("Failed to create debug utils");

            Some(Self { loader, callback })
        } else {
            tracing::info!("Skipping creation of debug utils");
            None
        }
    }
}
