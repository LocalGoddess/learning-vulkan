use ash::vk::SurfaceKHR;
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

use super::vulkan_instance::VulkanInstance;

pub struct VulkanSurfaceExt {
    pub loader: ash::khr::surface::Instance,
    pub surface: SurfaceKHR,
}

impl VulkanSurfaceExt {
    pub fn create(instance: &VulkanInstance, window: &Window) -> Self {
        let display_handle = window
            .display_handle()
            .expect("Failed to get the window's display handle");
        let window_handle = window
            .window_handle()
            .expect("Failed to get the window's window handle");

        let loader =
            ash::khr::surface::Instance::new(&instance.vulkan_library, &instance.vulkan_instance);
        let surface = unsafe {
            ash_window::create_surface(
                &instance.vulkan_library,
                &instance.vulkan_instance,
                display_handle.as_raw(),
                window_handle.as_raw(),
                None,
            )
        }
        .expect("Failed to create a Vulkan surface");

        Self { loader, surface }
    }
}
