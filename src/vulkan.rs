use std::{ffi::CStr, os::raw::c_char, u32, u64};

use ash::{
    ext::debug_utils,
    khr::swapchain,
    vk::{
        self, make_api_version, AccessFlags, ApplicationInfo, AttachmentDescription,
        AttachmentLoadOp, AttachmentReference, AttachmentStoreOp, ClearColorValue, ClearValue,
        ColorComponentFlags, ColorSpaceKHR, CommandBuffer, CommandBufferAllocateInfo,
        CommandBufferBeginInfo, CommandBufferLevel, CommandBufferResetFlags, CommandPool,
        CommandPoolCreateFlags, CommandPoolCreateInfo, ComponentMapping, ComponentSwizzle,
        CompositeAlphaFlagsKHR, CullModeFlags, DebugUtilsMessageSeverityFlagsEXT,
        DebugUtilsMessageTypeFlagsEXT, DebugUtilsMessengerCallbackDataEXT,
        DebugUtilsMessengerCreateInfoEXT, DebugUtilsMessengerEXT, DeviceCreateInfo,
        DeviceQueueCreateInfo, Extent2D, Fence, FenceCreateFlags, FenceCreateInfo, Format,
        Framebuffer, FramebufferCreateInfo, FrontFace, GraphicsPipelineCreateInfo, Image,
        ImageAspectFlags, ImageLayout, ImageSubresourceRange, ImageUsageFlags, ImageView,
        ImageViewCreateInfo, ImageViewType, InstanceCreateFlags, InstanceCreateInfo, Offset2D,
        PhysicalDevice, PhysicalDeviceFeatures, PhysicalDeviceType, Pipeline, PipelineBindPoint,
        PipelineCache, PipelineColorBlendAttachmentState, PipelineColorBlendStateCreateInfo,
        PipelineInputAssemblyStateCreateInfo, PipelineLayout, PipelineLayoutCreateInfo,
        PipelineMultisampleStateCreateInfo, PipelineRasterizationStateCreateInfo,
        PipelineShaderStageCreateInfo, PipelineStageFlags, PipelineVertexInputStateCreateInfo,
        PipelineViewportStateCreateInfo, PolygonMode, PresentInfoKHR, PresentModeKHR,
        PrimitiveTopology, QueueFlags, Rect2D, RenderPass, RenderPassBeginInfo,
        RenderPassCreateInfo, SampleCountFlags, Semaphore, SemaphoreCreateInfo, ShaderModule,
        ShaderModuleCreateInfo, ShaderStageFlags, SharingMode, SubmitInfo, SubpassContents,
        SubpassDependency, SubpassDescription, SurfaceCapabilitiesKHR, SurfaceFormatKHR,
        SurfaceKHR, SwapchainCreateInfoKHR, SwapchainKHR, Viewport,
    },
};
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use crate::util::{read_shader_file, str_to_cstr};

pub struct VulkanManager {
    pub vulkan_library: ash::Entry,
    pub vulkan_instance: ash::Instance,

    pub surface: SurfaceKHR,
    pub surface_loader: ash::khr::surface::Instance,

    pub physical_device: vk::PhysicalDevice,
    pub logical_device: ash::Device,

    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,

    pub swapchain_loader: swapchain::Device,
    pub swapchain: SwapchainKHR,

    pub swapchain_images: Vec<Image>,
    pub swapchain_image_views: Vec<ImageView>,
    pub swapchain_image_format: Format,
    pub swapchain_extent: Extent2D,

    pub render_pass: RenderPass,
    pub pipeline: Pipeline,
    pub pipeline_layout: PipelineLayout,

    pub framebuffers: Vec<Framebuffer>,
    pub command_pool: CommandPool,
    pub command_buffer: CommandBuffer,

    pub image_available_semaphore: Semaphore,
    pub render_finished_semaphore: Semaphore,
    pub in_flight_fence: Fence,

    debug_utils_instance: Option<debug_utils::Instance>,
    debug_utils_callback: Option<DebugUtilsMessengerEXT>,
}

impl VulkanManager {
    pub fn new(
        window_width: u32,
        window_height: u32,
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
    ) -> Self {
        tracing::info!("Initializing Vulkan");

        let vulkan_library = unsafe { ash::Entry::load() }.expect("Failed to load Vulkan library");
        let vulkan_instance = Self::create_instance(&vulkan_library, display_handle);

        #[cfg(debug_assertions)]
        let (debug_utils_instance, debug_utils_callback) =
            Self::initialize_debug_utils(&vulkan_library, &vulkan_instance);

        #[cfg(not(debug_assertions))]
        let (debug_utils_instance, debug_utils_callback) = (None, None);

        let (surface, surface_loader) = Self::create_surface(
            &vulkan_library,
            &vulkan_instance,
            display_handle,
            window_handle,
        );

        let physical_device =
            Self::select_physical_device(&vulkan_instance, &surface_loader, &surface);
        let (logical_device, graphics_queue, present_queue) = Self::create_logical_device(
            &vulkan_instance,
            &physical_device,
            &surface_loader,
            &surface,
        );

        let (
            swapchain_loader,
            swapchain,
            swapchain_images,
            swapchain_image_format,
            swapchain_extent,
        ) = Self::create_swapchain(
            &vulkan_instance,
            window_width,
            window_height,
            &physical_device,
            &logical_device,
            &surface_loader,
            &surface,
        );

        let swapchain_image_views =
            Self::create_image_views(&logical_device, &swapchain_images, &swapchain_image_format);

        let render_pass = Self::create_render_pass(&logical_device, &swapchain_image_format);

        let (pipeline_layout, pipeline) =
            Self::create_graphics_pipeline(&logical_device, &swapchain_extent, &render_pass);

        let framebuffers = Self::create_framebuffers(
            &logical_device,
            &swapchain_image_views,
            &render_pass,
            &swapchain_extent,
        );

        let command_pool = Self::create_command_pool(
            &vulkan_instance,
            &physical_device,
            &logical_device,
            &surface_loader,
            &surface,
        );

        let command_buffer = Self::create_command_buffer(&logical_device, &command_pool);

        let (image_available_semaphore, render_finished_semaphore, in_flight_fence) =
            Self::create_sync_objects(&logical_device);

        Self {
            vulkan_library,
            vulkan_instance,

            surface,
            surface_loader,

            physical_device,
            logical_device,

            graphics_queue,
            present_queue,

            swapchain_loader,
            swapchain,

            swapchain_images,
            swapchain_image_views,
            swapchain_image_format,
            swapchain_extent,

            render_pass,
            pipeline_layout,
            pipeline,

            framebuffers,
            command_pool,
            command_buffer,

            image_available_semaphore,
            render_finished_semaphore,
            in_flight_fence,

            debug_utils_instance,
            debug_utils_callback,
        }
    }

    fn create_instance(
        vulkan_library: &ash::Entry,
        display_handle: RawDisplayHandle,
    ) -> ash::Instance {
        #[cfg(debug_assertions)]
        let layer_names_raw: Vec<*const c_char> =
            vec![CStr::from_bytes_with_nul(b"VK_LAYER_KHRONOS_validation\0")
                .unwrap()
                .as_ptr()];

        #[cfg(not(debug_assertions))]
        let layer_names_raw: Vec<*const c_char> = Vec::new();

        let mut extension_names = ash_window::enumerate_required_extensions(display_handle)
            .expect("Failed to fetch required extension names")
            .to_vec();

        if cfg!(debug_assertions) {
            extension_names.push(debug_utils::NAME.as_ptr());
        }

        if cfg!(any(target_os = "macos", target_os = "ios")) {
            extension_names.push(ash::khr::portability_enumeration::NAME.as_ptr());
            extension_names.push(ash::khr::get_physical_device_properties2::NAME.as_ptr());
        }

        let application_info = ApplicationInfo::default()
            .application_name(str_to_cstr("Learning Vulkan\0"))
            .application_version(0)
            .engine_name(str_to_cstr("Learning Vulkan\0"))
            .engine_version(0)
            .api_version(make_api_version(0, 1, 0, 0));

        let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
            InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            InstanceCreateFlags::default()
        };

        let create_info = InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_layer_names(&layer_names_raw)
            .enabled_extension_names(&extension_names)
            .flags(create_flags);

        unsafe { vulkan_library.create_instance(&create_info, None) }
            .expect("Failed to create Vulkan instance")
    }

    fn initialize_debug_utils(
        vulkan_library: &ash::Entry,
        vulkan_instance: &ash::Instance,
    ) -> (
        Option<debug_utils::Instance>,
        Option<DebugUtilsMessengerEXT>,
    ) {
        tracing::info!("Initialzing Vulkan debug utils extension");

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
            .pfn_user_callback(Some(vulkan_debug_extension_callback));

        let debug_utils_instance = debug_utils::Instance::new(vulkan_library, vulkan_instance);
        let debug_utils_callback =
            unsafe { debug_utils_instance.create_debug_utils_messenger(&debug_create_info, None) }
                .expect("Failed to create debug utils callback");

        (Some(debug_utils_instance), Some(debug_utils_callback))
    }

    fn select_physical_device(
        vulkan_instance: &ash::Instance,
        surface_loader: &ash::khr::surface::Instance,
        surface: &SurfaceKHR,
    ) -> vk::PhysicalDevice {
        let possible_devices = unsafe { vulkan_instance.enumerate_physical_devices() }
            .expect("Could not find any physical devices with Vulkan support");

        let chosen_device = possible_devices
            .first()
            .expect("No physical devices with Vulkan support found"); // This this project is just for learning,
                                                                      // I'm not going to do a whole rating system
                                                                      // its just a waste of time atp

        if !Self::is_device_suitable(vulkan_instance, chosen_device, surface_loader, surface) {
            panic!("Default GPU is not suitable for this application");
        }

        chosen_device.to_owned()
    }

    fn is_device_suitable(
        vulkan_instance: &ash::Instance,
        physical_device: &vk::PhysicalDevice,
        surface_loader: &ash::khr::surface::Instance,
        surface: &SurfaceKHR,
    ) -> bool {
        let physical_device_properties =
            unsafe { vulkan_instance.get_physical_device_properties(*physical_device) };
        let physical_device_features =
            unsafe { vulkan_instance.get_physical_device_features(*physical_device) };
        let queue_family_indicies =
            QueueFamilyIndicies::new(vulkan_instance, physical_device, surface_loader, surface);
        let swapchain_support_details =
            SwapChainSupportDetails::new(physical_device, surface_loader, surface);

        physical_device_properties.device_type == PhysicalDeviceType::DISCRETE_GPU
            && physical_device_features.geometry_shader == 1
            && queue_family_indicies.has_all()
            && Self::check_device_extension_support(vulkan_instance, physical_device)
            && swapchain_support_details.is_adequate()
    }

    fn check_device_extension_support(
        vulkan_instance: &ash::Instance,
        physical_device: &vk::PhysicalDevice,
    ) -> bool {
        let required_extensions = unsafe {
            [
                swapchain::NAME.as_ptr(),
                #[cfg(any(target_os = "macos", target_os = "ios"))]
                ash::khr::portability_subset::NAME.as_ptr(),
            ]
            .map(|x| CStr::from_ptr(x))
            .to_vec()
        };

        let mut total = 0;
        let available_extension =
            unsafe { vulkan_instance.enumerate_device_extension_properties(*physical_device) }
                .expect("Failed to retrieve physical device extensions");
        for extension in available_extension {
            if required_extensions.contains(&extension.extension_name_as_c_str().unwrap()) {
                total += 1;
            }
        }

        total == required_extensions.len()
    }

    fn create_logical_device(
        vulkan_instance: &ash::Instance,
        physical_device: &vk::PhysicalDevice,
        surface_loader: &ash::khr::surface::Instance,
        surface: &SurfaceKHR,
    ) -> (ash::Device, vk::Queue, vk::Queue) {
        let device_extension_names_raw = [
            swapchain::NAME.as_ptr(),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            ash::khr::portability_subset::NAME.as_ptr(),
        ];

        let qf_index =
            QueueFamilyIndicies::new(vulkan_instance, physical_device, surface_loader, surface);
        let queue_priorities = [1.0f32];

        let mut queue_create_infos = Vec::new();
        for index in qf_index.get_unique_queue_families() {
            let queue_create_info = DeviceQueueCreateInfo::default()
                .queue_family_index(index)
                .queue_priorities(&queue_priorities);

            queue_create_infos.push(queue_create_info);
        }

        let device_features = PhysicalDeviceFeatures::default();

        let logical_device_create_info = DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&device_features)
            .enabled_extension_names(&device_extension_names_raw);

        let logical_device = unsafe {
            vulkan_instance.create_device(*physical_device, &logical_device_create_info, None)
        }
        .expect("Failed to create a logical device");
        let graphics_queue =
            unsafe { logical_device.get_device_queue(qf_index.graphics_family.unwrap(), 0) };
        let present_queue =
            unsafe { logical_device.get_device_queue(qf_index.present_family.unwrap(), 0) };

        (logical_device, graphics_queue, present_queue)
    }

    fn create_surface(
        vulkan_library: &ash::Entry,
        vulkan_instance: &ash::Instance,
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
    ) -> (SurfaceKHR, ash::khr::surface::Instance) {
        let surface = unsafe {
            ash_window::create_surface(
                vulkan_library,
                vulkan_instance,
                display_handle,
                window_handle,
                None,
            )
        }
        .expect("Failed to create a render surface");
        let surface_loader = ash::khr::surface::Instance::new(vulkan_library, vulkan_instance);

        (surface, surface_loader)
    }

    fn create_swapchain(
        vulkan_instance: &ash::Instance,
        window_width: u32,
        window_height: u32,
        physical_device: &vk::PhysicalDevice,
        logical_device: &ash::Device,
        surface_loader: &ash::khr::surface::Instance,
        surface: &SurfaceKHR,
    ) -> (
        swapchain::Device,
        SwapchainKHR,
        Vec<Image>,
        Format,
        Extent2D,
    ) {
        let swap_chain_support =
            SwapChainSupportDetails::new(physical_device, surface_loader, surface);

        let surface_format = swap_chain_support.choose_swapchain_surface_format();
        let present_mode = swap_chain_support.choose_swapchain_present_mode();
        let extent = swap_chain_support.choose_swapchain_extent(window_width, window_height);

        let mut image_count = swap_chain_support.capabilities.min_image_count + 1; // We want to make
                                                                                   // sure we have more
                                                                                   // than one image to
                                                                                   // prevent tearing
        if swap_chain_support.capabilities.max_image_count > 0
            && image_count > swap_chain_support.capabilities.max_image_count
        {
            image_count = swap_chain_support.capabilities.max_image_count;
        }

        let mut swapchain_create_info = SwapchainCreateInfoKHR::default()
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1) // Always one unless the application is a stereoscopic 3D
            // application
            .image_usage(ImageUsageFlags::COLOR_ATTACHMENT) // This means we are rendering
            // directly to the swapchain in order
            // to render from a texture for post
            // processing purposes we would use
            // IMaGE_USAGE_TRANSFER_DST_BIT
            .pre_transform(swap_chain_support.capabilities.current_transform)
            .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE) // We ignore the alpha blending
            .present_mode(present_mode)
            .clipped(true) // We don't care about the color of pixels on the window when another
            // window is covering the this one
            .old_swapchain(SwapchainKHR::null())
            .min_image_count(image_count)
            .surface(*surface);

        let qf_indicies =
            QueueFamilyIndicies::new(vulkan_instance, physical_device, surface_loader, surface);
        let total_family_indicies = [
            qf_indicies.graphics_family.unwrap(),
            qf_indicies.present_family.unwrap(),
        ];

        if qf_indicies.graphics_family.unwrap() != qf_indicies.present_family.unwrap() {
            swapchain_create_info = swapchain_create_info
                .image_sharing_mode(SharingMode::CONCURRENT) // shared accross multiple queues without explicit transfers
                .queue_family_indices(&total_family_indicies);
        } else {
            swapchain_create_info =
                swapchain_create_info.image_sharing_mode(SharingMode::EXCLUSIVE);
            // Owned
            // by
            // one
            // queue
            // family
            // at
            // a
            // time
        }

        let swapchain_loader = swapchain::Device::new(vulkan_instance, logical_device);
        let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None) }
            .expect("Failed to create swapchain");

        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain) }
            .expect("Failed to get swapchain images");

        (
            swapchain_loader,
            swapchain,
            images,
            surface_format.format,
            extent,
        )
    }

    fn create_image_views(
        logical_device: &ash::Device,
        swapchain_images: &[Image],
        swapchain_image_format: &Format,
    ) -> Vec<ImageView> {
        let mut image_views = Vec::new();

        for (index, _image) in swapchain_images.iter().enumerate() {
            let create_info = ImageViewCreateInfo::default()
                .image(*swapchain_images.get(index).unwrap())
                .view_type(ImageViewType::TYPE_2D)
                .format(*swapchain_image_format)
                .components(
                    ComponentMapping::default()
                        .r(ComponentSwizzle::IDENTITY)
                        .g(ComponentSwizzle::IDENTITY)
                        .b(ComponentSwizzle::IDENTITY)
                        .a(ComponentSwizzle::IDENTITY),
                )
                .subresource_range(
                    ImageSubresourceRange::default()
                        .aspect_mask(ImageAspectFlags::COLOR)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(0)
                        .layer_count(1),
                );
            let image_view = unsafe { logical_device.create_image_view(&create_info, None) }
                .expect("Failed to create image view");
            image_views.push(image_view);
        }

        image_views
    }

    fn create_render_pass(
        logical_device: &ash::Device,
        swapchain_image_format: &Format,
    ) -> RenderPass {
        let color_attachment = AttachmentDescription::default()
            .format(*swapchain_image_format)
            .samples(SampleCountFlags::TYPE_1)
            .load_op(AttachmentLoadOp::CLEAR)
            .store_op(AttachmentStoreOp::STORE)
            .stencil_load_op(AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(AttachmentStoreOp::DONT_CARE)
            .initial_layout(ImageLayout::UNDEFINED)
            .final_layout(ImageLayout::PRESENT_SRC_KHR);

        let color_attachment_reference = AttachmentReference::default()
            .attachment(0)
            .layout(ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

        let color_attachment_references = [color_attachment_reference];
        let subpass = SubpassDescription::default()
            .pipeline_bind_point(PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_attachment_references);

        let color_attachments = [color_attachment];
        let subpasses = [subpass];
        let mut render_pass_info = RenderPassCreateInfo::default()
            .attachments(&color_attachments)
            .subpasses(&subpasses);

        let dependency = SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(AccessFlags::NONE)
            .dst_stage_mask(PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(AccessFlags::COLOR_ATTACHMENT_WRITE);

        let dependencies = [dependency];
        render_pass_info = render_pass_info.dependencies(&dependencies);

        unsafe { logical_device.create_render_pass(&render_pass_info, None) }
            .expect("Failed to create render pass")
    }

    fn create_graphics_pipeline(
        logical_device: &ash::Device,
        swapchain_extent: &Extent2D,
        render_pass: &RenderPass,
    ) -> (PipelineLayout, Pipeline) {
        let vertex_shader_code = read_shader_file("vert.spv");
        let fragme_shader_code = read_shader_file("frag.spv");

        // Clean up at the end of the file similar to OpenGL
        let vert_shader_module = Self::create_shader_module(logical_device, vertex_shader_code);
        let frag_shader_module = Self::create_shader_module(logical_device, fragme_shader_code);

        let vert_pipeline_shader_stage_create_info = PipelineShaderStageCreateInfo::default()
            .stage(ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(str_to_cstr("main\0"));
        // .specialization_info(specialization_info) Allows the passage of shader constants
        let frag_pipeline_shader_stage_create_info = PipelineShaderStageCreateInfo::default()
            .stage(ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(str_to_cstr("main\0"));

        let total_stages = vec![
            vert_pipeline_shader_stage_create_info,
            frag_pipeline_shader_stage_create_info,
        ];

        let vertex_input_info = PipelineVertexInputStateCreateInfo::default(); // Pretty shallow
                                                                               // since no vbos rn
        let input_assembly = PipelineInputAssemblyStateCreateInfo::default()
            .topology(PrimitiveTopology::TRIANGLE_LIST) // The classic OpenGL way of drawing
            // triangles
            .primitive_restart_enable(false); // Setting to true allows us to break up triangle
                                              // strips and other _STRIP types

        let viewport = Viewport::default()
            .x(0f32)
            .y(0f32)
            .width(swapchain_extent.width as f32) // These could vary from the actual window so we
            // use the extent instead
            .height(swapchain_extent.height as f32)
            .min_depth(0f32)
            .max_depth(1f32);

        let scissor = Rect2D::default()
            .offset(Offset2D { x: 0, y: 0 })
            .extent(*swapchain_extent); // we want to cover the entire framebuffer

        let viewports = [viewport];
        let scissors = [scissor];
        let viewport_state = PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .viewports(&viewports)
            .scissor_count(1)
            .scissors(&scissors); // Creating this here since I don't need/want dynamic viewport
                                  // and scissor for this little learning project

        let rasterizer = PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false) // If true pixels beyond the near and far planes are
            // clamped rather than discarded (requires a GPU feature)
            .rasterizer_discard_enable(false) // If true geometry never passes through the
            // rasterizer stage. Disabling output to the
            // framebuffer
            .polygon_mode(PolygonMode::FILL) // Any other mode than fill requires a GPU feature
            .line_width(1f32) // Anything thicker than 1 requires a GPU feature
            .cull_mode(CullModeFlags::BACK) // Funny face culling
            .front_face(FrontFace::CLOCKWISE) // The required vertex order for faces to be
            // considered front-facing
            .depth_bias_enable(false);

        let multisampling = PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(SampleCountFlags::TYPE_1); // Keeping this off for now

        let color_blend_attachment = PipelineColorBlendAttachmentState::default()
            .blend_enable(false)
            .color_write_mask(
                ColorComponentFlags::R
                    | ColorComponentFlags::G
                    | ColorComponentFlags::B
                    | ColorComponentFlags::A,
            );

        let attachments = [color_blend_attachment];
        let color_blending = PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(&attachments);

        let pipeline_layout_info = PipelineLayoutCreateInfo::default();

        let pipeline_layout =
            unsafe { logical_device.create_pipeline_layout(&pipeline_layout_info, None) }
                .expect("Failed to create pipeline layout");

        let pipeline_info = GraphicsPipelineCreateInfo::default()
            .stages(&total_stages)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .layout(pipeline_layout)
            .render_pass(*render_pass)
            .vertex_input_state(&vertex_input_info)
            .subpass(0);

        let binding = unsafe {
            logical_device.create_graphics_pipelines(PipelineCache::null(), &[pipeline_info], None)
        }
        .expect("Failed to create graphics pipeline");
        let graphics_pipeline = binding.first().unwrap().to_owned(); // we only created one

        unsafe {
            logical_device.destroy_shader_module(vert_shader_module, None);
            logical_device.destroy_shader_module(frag_shader_module, None);
        }

        (pipeline_layout, graphics_pipeline)
    }

    fn create_shader_module(logical_device: &ash::Device, code: Vec<u32>) -> ShaderModule {
        let create_info = ShaderModuleCreateInfo::default().code(&code);

        unsafe { logical_device.create_shader_module(&create_info, None) }
            .expect("Failed to create shader module")
    }

    fn create_framebuffers(
        logical_device: &ash::Device,
        image_views: &[ImageView],
        render_pass: &RenderPass,
        swapchain_extent: &Extent2D,
    ) -> Vec<Framebuffer> {
        let mut framebuffers = Vec::new();

        for view in image_views.iter() {
            let attachments = [*view];
            let create_info = FramebufferCreateInfo::default()
                .render_pass(*render_pass)
                .attachments(&attachments)
                .width(swapchain_extent.width)
                .height(swapchain_extent.height)
                .layers(1);

            let framebuffer = unsafe { logical_device.create_framebuffer(&create_info, None) }
                .expect("Failed to create framebuffer");
            framebuffers.push(framebuffer);
        }

        framebuffers
    }

    fn create_command_pool(
        vulkan_instance: &ash::Instance,
        physical_device: &vk::PhysicalDevice,
        logical_device: &ash::Device,
        surface_loader: &ash::khr::surface::Instance,
        surface: &SurfaceKHR,
    ) -> CommandPool {
        let qf_indicies =
            QueueFamilyIndicies::new(vulkan_instance, physical_device, surface_loader, surface);

        let pool_info = CommandPoolCreateInfo::default()
            .flags(CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(qf_indicies.graphics_family.unwrap());

        unsafe { logical_device.create_command_pool(&pool_info, None) }
            .expect("Failed to craete command pool")
    }

    fn create_command_buffer(
        logical_device: &ash::Device,
        command_pool: &CommandPool,
    ) -> CommandBuffer {
        tracing::info!("Creating command buffer");

        let alloc_info = CommandBufferAllocateInfo::default()
            .command_pool(*command_pool)
            .level(CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffers = unsafe { logical_device.allocate_command_buffers(&alloc_info) }
            .expect("Failed to allocate command buffers");
        command_buffers.first().unwrap().to_owned() // We are only creating one
    }

    fn write_command_buffer(&self, image_index: usize) {
        let begin_info = CommandBufferBeginInfo::default();

        unsafe {
            self.logical_device
                .begin_command_buffer(self.command_buffer, &begin_info)
        }
        .expect("Failed to begin recording command buffer");

        let mut render_pass_info = RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers.get(image_index).unwrap().to_owned())
            .render_area(
                Rect2D::default()
                    .offset(Offset2D { x: 0, y: 0 })
                    .extent(self.swapchain_extent),
            );

        let clear_color = [ClearValue {
            color: ClearColorValue {
                float32: [0f32, 0f32, 0f32, 1f32],
            },
        }];
        render_pass_info = render_pass_info.clear_values(&clear_color);

        unsafe {
            self.logical_device.cmd_begin_render_pass(
                self.command_buffer,
                &render_pass_info,
                SubpassContents::INLINE,
            )
        };
        unsafe {
            self.logical_device.cmd_bind_pipeline(
                self.command_buffer,
                PipelineBindPoint::GRAPHICS,
                self.pipeline,
            )
        };

        unsafe {
            self.logical_device
                .cmd_draw(self.command_buffer, 3, 1, 0, 0)
        };

        unsafe { self.logical_device.cmd_end_render_pass(self.command_buffer) };

        unsafe { self.logical_device.end_command_buffer(self.command_buffer) }
            .expect("Failed to record command buffer");
    }

    fn create_sync_objects(logical_device: &ash::Device) -> (Semaphore, Semaphore, Fence) {
        let semaphore_info = SemaphoreCreateInfo::default();
        let fence_create_info = FenceCreateInfo::default().flags(FenceCreateFlags::SIGNALED);

        let image_available_semaphore =
            unsafe { logical_device.create_semaphore(&semaphore_info, None) }
                .expect("Failed to create semaphore");
        let render_finished_semaphore =
            unsafe { logical_device.create_semaphore(&semaphore_info, None) }
                .expect("Failed to create semaphore");
        let fence = unsafe { logical_device.create_fence(&fence_create_info, None) }
            .expect("Failed to create fence");

        (image_available_semaphore, render_finished_semaphore, fence)
    }

    pub fn draw_frame(&self) {
        tracing::info!("Beginning Frame");
        let fences = [self.in_flight_fence];

        unsafe { self.logical_device.wait_for_fences(&fences, true, u64::MAX) }
            .expect("Failed to wait for fence");
        unsafe { self.logical_device.reset_fences(&fences) }.expect("Failed to reset fences");

        let image_index = unsafe {
            self.swapchain_loader.acquire_next_image(
                self.swapchain,
                u64::MAX,
                self.image_available_semaphore,
                Fence::null(),
            )
        }
        .expect("Failed to acquite next image")
        .0;

        unsafe {
            self.logical_device
                .reset_command_buffer(self.command_buffer, CommandBufferResetFlags::empty())
        }
        .expect("Failed to reset command buffers");

        tracing::info!("Pre write");
        Self::write_command_buffer(self, image_index as usize);
        tracing::info!("Post write");

        let wait_stages = [PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let wait_semaphores = [self.image_available_semaphore];
        let command_buffers = [self.command_buffer];

        let signal_semaphores = [self.render_finished_semaphore];

        let submit_info = SubmitInfo::default()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        let submit_infos = [submit_info];
        unsafe {
            self.logical_device.queue_submit(
                self.graphics_queue,
                &submit_infos,
                self.in_flight_fence,
            )
        }
        .expect("Failed to submit draw command buffer");

        let swap_chains = [self.swapchain];
        let image_indicies = [image_index];

        let present_info = PresentInfoKHR::default()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swap_chains)
            .image_indices(&image_indicies);

        unsafe {
            self.swapchain_loader
                .queue_present(self.present_queue, &present_info)
        }
        .expect("Failed to present queue");
    }
}

impl Drop for VulkanManager {
    fn drop(&mut self) {
        tracing::info!("Cleaning up Vulkan (Dropped)");

        if self.debug_utils_instance.is_some() {
            unsafe {
                self.debug_utils_instance
                    .as_ref()
                    .unwrap()
                    .destroy_debug_utils_messenger(self.debug_utils_callback.unwrap(), None)
            };
        }

        for image_view in &self.swapchain_image_views {
            unsafe { self.logical_device.destroy_image_view(*image_view, None) };
        }

        for framebuffer in &self.framebuffers {
            unsafe { self.logical_device.destroy_framebuffer(*framebuffer, None) };
        }

        unsafe {
            self.logical_device
                .destroy_semaphore(self.image_available_semaphore, None);
            self.logical_device
                .destroy_semaphore(self.render_finished_semaphore, None);
            self.logical_device
                .destroy_fence(self.in_flight_fence, None);
        }
        unsafe {
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None)
        }
        unsafe { self.logical_device.destroy_pipeline(self.pipeline, None) };
        unsafe {
            self.logical_device
                .destroy_pipeline_layout(self.pipeline_layout, None)
        };
        unsafe {
            self.logical_device
                .destroy_render_pass(self.render_pass, None)
        };
        unsafe { self.logical_device.destroy_device(None) };
        unsafe { self.surface_loader.destroy_surface(self.surface, None) };
        unsafe { self.vulkan_instance.destroy_instance(None) };
    }
}

#[derive(Default)]
pub struct QueueFamilyIndicies {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndicies {
    pub fn new(
        vulkan_instance: &ash::Instance,
        physical_device: &vk::PhysicalDevice,
        surface_loader: &ash::khr::surface::Instance,
        surface: &SurfaceKHR,
    ) -> Self {
        let mut qf_indicies = QueueFamilyIndicies::default();

        let queue_families = unsafe {
            vulkan_instance.get_physical_device_queue_family_properties(*physical_device)
        };
        for (index, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags.contains(QueueFlags::GRAPHICS) {
                qf_indicies.graphics_family = Some(index as u32);
            }

            if unsafe {
                surface_loader.get_physical_device_surface_support(
                    *physical_device,
                    index as u32,
                    *surface,
                )
            }
            .expect("Failed to find surface support")
            {
                qf_indicies.present_family = Some(index as u32);
            }
        }

        qf_indicies
    }

    pub fn get_unique_queue_families(&self) -> Vec<u32> {
        if !self.has_all() {
            panic!("Physical device doesn't support the proper queues");
        }
        vec![self.graphics_family.unwrap(), self.present_family.unwrap()]
    }

    pub fn has_all(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

pub struct SwapChainSupportDetails {
    pub capabilities: SurfaceCapabilitiesKHR,
    pub formats: Vec<SurfaceFormatKHR>,
    pub present_modes: Vec<PresentModeKHR>,
}

impl SwapChainSupportDetails {
    pub fn new(
        physical_device: &PhysicalDevice,
        surface_loader: &ash::khr::surface::Instance,
        surface: &SurfaceKHR,
    ) -> Self {
        let capabilities = unsafe {
            surface_loader.get_physical_device_surface_capabilities(*physical_device, *surface)
        }
        .expect("Failed to get surface capabilities");
        let formats = unsafe {
            surface_loader.get_physical_device_surface_formats(*physical_device, *surface)
        }
        .expect("Failed to get surface formats");
        let present_modes = unsafe {
            surface_loader.get_physical_device_surface_present_modes(*physical_device, *surface)
        }
        .expect("Failed to get surface present modes");

        Self {
            capabilities,
            formats,
            present_modes,
        }
    }

    pub fn choose_swapchain_surface_format(&self) -> SurfaceFormatKHR {
        // Use SRGB
        for format in &self.formats {
            if format.format == Format::B8G8R8A8_SRGB
                && format.color_space == ColorSpaceKHR::SRGB_NONLINEAR
            {
                return *format;
            }
        }
        self.formats.first().unwrap().to_owned()
    }

    pub fn choose_swapchain_extent(&self, window_width: u32, window_height: u32) -> Extent2D {
        match self.capabilities.current_extent.width {
            u32::MAX => Extent2D {
                width: window_width,
                height: window_height,
            },
            _ => self.capabilities.current_extent,
        }
    }

    pub fn choose_swapchain_present_mode(&self) -> PresentModeKHR {
        for present_mode in &self.present_modes {
            if present_mode == &PresentModeKHR::MAILBOX {
                return *present_mode;
            }
        }
        PresentModeKHR::FIFO // the only garaunteed one
    }

    pub fn is_adequate(&self) -> bool {
        !self.formats.is_empty() && !self.present_modes.is_empty()
    }
}

unsafe extern "system" fn vulkan_debug_extension_callback(
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
