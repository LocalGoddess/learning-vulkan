use ash::vk::SurfaceKHR;
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

pub struct VulkanSurfaceExt {
    pub loader: ash::khr::surface::Instance,
    pub surface: SurfaceKHR,
}

impl VulkanSurfaceExt {
    pub fn create(library: &ash::Entry, instance: &ash::Instance, window: &Window) -> Self {
        let display_handle = window
            .display_handle()
            .expect("Failed to get the window's display handle");
        let window_handle = window
            .window_handle()
            .expect("Failed to get the window's window handle");

        let loader =
            ash::khr::surface::Instance::new(library, instance);
        let surface = unsafe {
            ash_window::create_surface(
                library,
                instance,
                display_handle.as_raw(),
                window_handle.as_raw(),
                None,
            )
        }
        .expect("Failed to create a Vulkan surface");

        Self { loader, surface }
    }
}
