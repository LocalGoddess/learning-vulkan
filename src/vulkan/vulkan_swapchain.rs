use winit::window::Window;

use super::{
    vulkan_device::VulkanLogicalDevice, vulkan_instance::VulkanInstance,
    vulkan_surface::VulkanSurfaceExt,
};

pub struct VulkanSwapchain<'a> {
    pub instance: &'a VulkanInstance,
    pub logical_device: &'a VulkanLogicalDevice,

    pub surface_extension: VulkanSurfaceExt,
}

impl<'a> VulkanSwapchain<'a> {
    pub fn create(instance: &'a VulkanInstance, window: &Window) -> Self {
        let surface_extension = VulkanSurfaceExt::create(instance, window);

        Self {
            instance,
            logical_device: &instance.logical_device,
            surface_extension,
        }
    }
}
