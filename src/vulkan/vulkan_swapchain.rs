use ash::{khr::swapchain, vk::{
    self, ColorSpaceKHR, ComponentSwizzle, CompositeAlphaFlagsKHR, Extent2D, Image, ImageAspectFlags, ImageUsageFlags, ImageView, ImageViewCreateInfo, ImageViewType, PresentModeKHR, SharingMode, SurfaceCapabilitiesKHR, SurfaceFormatKHR, SwapchainCreateInfoKHR
}};
use winit::window::Window;

use super::{
    vulkan_device::{VulkanLogicalDevice, VulkanPhysicalDevice},
    vulkan_instance::VulkanInstance,
    vulkan_surface::VulkanSurfaceExt,
};

pub struct VulkanSwapchain<'a> {
    pub instance: &'a VulkanInstance,
    pub logical_device: &'a VulkanLogicalDevice,
    pub surface_ext: &'a VulkanSurfaceExt,

    pub loader: swapchain::Device,
    pub handle: vk::SwapchainKHR,

    pub extent: Extent2D,
    pub image_format: SurfaceFormatKHR,

    pub images: Vec<Image>,
    pub image_views: Vec<ImageView>,

    support_details: SwapchainSupportDetails, // Might delete
}

impl<'a> VulkanSwapchain<'a> {
    pub fn create(
        instance: &'a VulkanInstance,
        window: &Window,
        previous_swapchain: Option<VulkanSwapchain<'_>>,
    ) -> Self {
        let support_details = SwapchainSupportDetails::create(instance, &instance.physical_device);

        let mut image_count = support_details.capabilities.min_image_count + 1;
        if support_details.capabilities.max_image_count > 0
            && image_count > support_details.capabilities.max_image_count
        {
            image_count = support_details.capabilities.max_image_count;
        }

        let surface_format = support_details.choose_surface_format();
        let extent = support_details.choose_extent(window);

        let mut create_info = SwapchainCreateInfoKHR::default()
            .surface(instance.surface_ext.surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT);

        let queue_family_indicies = &instance.physical_device.queue_family_indicies;
        let indicies = [
            queue_family_indicies.graphics_index.unwrap(),
            queue_family_indicies.present_queue.unwrap(),
        ];

        if queue_family_indicies.graphics_index.unwrap()
            != queue_family_indicies.present_queue.unwrap()
        {
            create_info = create_info
                .image_sharing_mode(SharingMode::CONCURRENT)
                .queue_family_indices(&indicies);
        } else {
            create_info = create_info.image_sharing_mode(SharingMode::EXCLUSIVE);
        }

        create_info = create_info
            .pre_transform(support_details.capabilities.current_transform)
            .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(support_details.choose_present_mode())
            .clipped(true);

        if previous_swapchain.is_some() {
            create_info = create_info.old_swapchain(previous_swapchain.unwrap().handle);
        }
    
        let swapchain_loader = swapchain::Device::new(&instance.vulkan_instance, &instance.logical_device.logical_device);
        let swapchain = unsafe { swapchain_loader.create_swapchain(&create_info, None) }.expect("Failed to create Vulkan swapchain");
        
        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }.expect("Failed to retrieve swapchain images");
        let mut image_views: Vec<ImageView> = Vec::new();
        
        for image in &images {
            let image_view_create_info = ImageViewCreateInfo::default()
                .image(*image)
                .view_type(ImageViewType::TYPE_2D)
                .format(surface_format.format)
                .components(
                    vk::ComponentMapping { r: ComponentSwizzle::IDENTITY, g: ComponentSwizzle::IDENTITY, b: ComponentSwizzle::IDENTITY, a: ComponentSwizzle::IDENTITY }
                    )
                .subresource_range(
                    vk::ImageSubresourceRange { aspect_mask: ImageAspectFlags::COLOR, base_mip_level: 0, level_count: 1, base_array_layer: 0, layer_count: 1 }
                    );

            let view = unsafe { instance.logical_device.logical_device.create_image_view(&image_view_create_info, None) }.expect("Failed to create image view");
            image_views.push(view);

        }

        Self {
            instance,
            logical_device: &instance.logical_device,
            surface_ext: &instance.surface_ext,
            extent,
            image_format: surface_format,
            loader: swapchain_loader,
            handle: swapchain,
            images,
            image_views,
            support_details
        }
    }

    pub fn destroy(&mut self) {
        for view in &self.image_views {
            unsafe { self.instance.logical_device.logical_device.destroy_image_view(*view, None) };
        }
    }
}

pub struct SwapchainSupportDetails {
    pub capabilities: SurfaceCapabilitiesKHR,
    pub formats: Vec<SurfaceFormatKHR>,
    pub present_modes: Vec<PresentModeKHR>,
}

impl SwapchainSupportDetails {
    pub fn create(instance: &VulkanInstance, physical_device: &VulkanPhysicalDevice) -> Self {
        let capabilities = unsafe {
            instance
                .surface_ext
                .loader
                .get_physical_device_surface_capabilities(
                    physical_device.physical_device,
                    instance.surface_ext.surface,
                )
        }
        .expect("Failed to retrieve physical device surface capabilities");
        let formats = unsafe {
            instance
                .surface_ext
                .loader
                .get_physical_device_surface_formats(
                    physical_device.physical_device,
                    instance.surface_ext.surface,
                )
        }
        .expect("Failed to retrieve physical device formats");
        let present_modes = unsafe {
            instance
                .surface_ext
                .loader
                .get_physical_device_surface_present_modes(
                    physical_device.physical_device,
                    instance.surface_ext.surface,
                )
        }
        .expect("Failed to retrieve physical device present modes");

        Self {
            capabilities,
            formats,
            present_modes,
        }
    }

    pub fn choose_surface_format(&self) -> SurfaceFormatKHR {
        for available_format in &self.formats {
            if available_format.format == vk::Format::B8G8R8A8_SRGB
                && available_format.color_space == ColorSpaceKHR::SRGB_NONLINEAR
            {
                return *available_format;
            }
        }
        self.formats[0]
    }

    pub fn choose_present_mode(&self) -> PresentModeKHR {
        for available_present_mode in &self.present_modes {
            if *available_present_mode == PresentModeKHR::MAILBOX {
                return *available_present_mode;
            }
        }

        self.present_modes[0]
    }

    pub fn choose_extent(&self, window: &winit::window::Window) -> Extent2D {
        if self.capabilities.current_extent.width != u32::MAX {
            return self.capabilities.current_extent;
        }

        let inner_size = window.inner_size();
        let min_image = self.capabilities.min_image_extent;
        let max_image = self.capabilities.max_image_extent;

        Extent2D {
            width: u32::clamp(inner_size.width, min_image.width, max_image.width),
            height: u32::clamp(inner_size.height, min_image.height, max_image.height),
        }
    }
}
