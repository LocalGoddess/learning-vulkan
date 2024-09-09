use winit::window::Window;

use super::{
    vulkan_device::VulkanLogicalDevice, vulkan_instance::VulkanInstance,
    vulkan_surface::VulkanSurfaceExt,
};

pub struct VulkanSwapchain<'a> {
    pub instance: &'a VulkanInstance,
    pub logical_device: &'a VulkanLogicalDevice,
    pub surface_ext: &'a VulkanSurfaceExt
}

impl<'a> VulkanSwapchain<'a> {
    pub fn create(instance: &'a VulkanInstance, _window: &Window) -> Self {

        Self {
            instance,
            logical_device: &instance.logical_device,
            surface_ext: &instance.surface_ext
        }
    }
}
