use std::ffi::CStr;

use ash::vk::{
    self, DeviceCreateInfo, DeviceQueueCreateInfo, ExtensionProperties, PhysicalDevice,
    PhysicalDeviceFeatures, PhysicalDeviceProperties, PhysicalDeviceType, QueueFlags,
};

use crate::util::{self, str_to_cstr};

use super::vulkan_surface::VulkanSurfaceExt;

pub struct VulkanPhysicalDevice {
    pub physical_device: PhysicalDevice,
    pub properties: PhysicalDeviceProperties,
    pub features: PhysicalDeviceFeatures,
    pub queue_family_indicies: QueueFamilyIndicies,
}

pub fn get_device_extensions() -> Vec<*const i8> {
    vec![ash::khr::swapchain::NAME.as_ptr()]
}

impl VulkanPhysicalDevice {
    pub fn select(instance: &ash::Instance, surface_ext: &VulkanSurfaceExt) -> Self {
        tracing::info!("Selecting a physical device");

        let physical_devices = unsafe { instance.enumerate_physical_devices() }
            .expect("Failed to find any Vulkan supported physical devices");

        let mut selected_device: Option<PhysicalDevice> = None;
        let mut highest_rating: u32 = 0;
        for physical_device in physical_devices {
            let current_rating = Self::rate_device(instance, physical_device, surface_ext);
            if current_rating > highest_rating {
                highest_rating = current_rating;
                selected_device = Some(physical_device);
            }
        }
        
        if selected_device.is_none() {
            panic!("Could not find a suitable physical device");
        }

        // I don't like getting the properties and features a second time after the loop but it
        // will do for now
        let properties =
            unsafe { instance.get_physical_device_properties(selected_device.unwrap()) };
        let features = unsafe { instance.get_physical_device_features(selected_device.unwrap()) };

        let queue_family_indicies =
            QueueFamilyIndicies::get_all(instance, selected_device.unwrap(), surface_ext);

        Self {
            physical_device: selected_device.unwrap(),
            properties,
            features,
            queue_family_indicies,
        }
    }

    pub fn get_device_info(&self) -> String {
        let api_version = self.properties.api_version;
        format!(
            "Device: {:?} Driver Version {:?}.{:?}.{:?}",
            self.properties
                .device_name_as_c_str()
                .unwrap_or(str_to_cstr("Unknown\0")),
            vk::api_version_major(api_version),
            vk::api_version_minor(api_version),
            vk::api_version_patch(api_version)
        )
    }

    fn rate_device(instance: &ash::Instance, physical_device: PhysicalDevice, surface_ext: &VulkanSurfaceExt) -> u32 {
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let extension_properties =
            unsafe { instance.enumerate_device_extension_properties(physical_device) }
                .expect("Failed toget device extension properties");
        let features = unsafe { instance.get_physical_device_features(physical_device) };

        let mut rating: u32 = 0u32;
        if !Self::is_suitable(instance, physical_device, features, &extension_properties, surface_ext) {
            return rating;
        }

        if properties.device_type == PhysicalDeviceType::DISCRETE_GPU {
            rating += 1000;
        }

        rating += properties.limits.max_image_dimension2_d;

        rating
    }

    fn is_suitable(
        instance: &ash::Instance,
        physical_device: PhysicalDevice,
        features: PhysicalDeviceFeatures,
        extension_properties: &[ExtensionProperties],
        surface_ext: &VulkanSurfaceExt
    ) -> bool {
        let queue_family_indicies = QueueFamilyIndicies::get_all(instance, physical_device, surface_ext);
        dbg!(&queue_family_indicies);

        features.geometry_shader == ash::vk::TRUE
            && queue_family_indicies.has_all()
            && Self::check_device_extension_support(extension_properties)
    }

    fn check_device_extension_support(extension_properties: &[ExtensionProperties]) -> bool {
        let required_extensions = get_device_extensions();
        
        todo!("Fix this");
        extension_properties.iter().all(|x| required_extensions.contains(&x.extension_name_as_c_str().unwrap().as_ptr())) 
    }
}

#[derive(Debug, Default)]
pub struct QueueFamilyIndicies {
    pub graphics_index: Option<u32>,
    pub present_queue: Option<u32>
}

impl QueueFamilyIndicies {
    pub fn get_all(
        instance: &ash::Instance,
        physical_device: PhysicalDevice,
        surface_ext: &VulkanSurfaceExt,
    ) -> Self {
        tracing::info!("Checking for all required queue families");

        let mut queue_family_indicies = QueueFamilyIndicies::default();

        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        for (index, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags.contains(QueueFlags::GRAPHICS) {
                tracing::info!("Found graphics queue family");
                queue_family_indicies.graphics_index = Some(index as u32);
            }

            if unsafe { surface_ext.loader.get_physical_device_surface_support(physical_device, index as u32, surface_ext.surface) }.unwrap() {
                tracing::info!("Found present queue family");
                queue_family_indicies.present_queue = Some(index as u32);
            }
        }

        queue_family_indicies
    }

    pub fn get_queue_create_infos<'a>(&'a self, priorities: &'a[f32]) -> Vec<DeviceQueueCreateInfo> {
        let mut create_infos = Vec::new();

        let mut families = Vec::new();
        if self.graphics_index.is_some() {
            families.push(self.graphics_index.unwrap())
        }

        if self.present_queue.is_some() {
            families.push(self.present_queue.unwrap());
        }
        


        for family in families {
            let create_info = DeviceQueueCreateInfo::default()
                .queue_family_index(family)
                .queue_priorities(priorities);

            create_infos.push(create_info)
        }

        create_infos
    }

    pub fn has_all(&self) -> bool {
        self.present_queue.is_some() && self.graphics_index.is_some()
    }
}

pub struct VulkanLogicalDevice {
    pub logical_device: ash::Device,

    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue
}

impl VulkanLogicalDevice {
    pub fn create(instance: &ash::Instance, physical_device: &VulkanPhysicalDevice) -> Self {
        let empty = [];
        Self::create_compatability(instance, physical_device, &empty)
    }

    pub fn create_compatability(
        instance: &ash::Instance,
        physical_device: &VulkanPhysicalDevice,
        extension_names: &[*const i8],
    ) -> Self {
        tracing::info!("Creating a logical device");

        // Maybe asbtract away the queue creation from here?
        let queue_priorities = [1f32];
        let qcinfos = physical_device.queue_family_indicies.get_queue_create_infos(&queue_priorities);

        let extensions = get_device_extensions();
        let mut logical_device_create_info = DeviceCreateInfo::default()
            .enabled_features(&physical_device.features)
            .queue_create_infos(&qcinfos)
            .enabled_extension_names(&extensions);

        if !extension_names.is_empty() {
            tracing::info!("Using compatability settings");
            logical_device_create_info =
                logical_device_create_info.enabled_extension_names(extension_names);
        }

        let logical_device = unsafe {
            instance.create_device(
                physical_device.physical_device,
                &logical_device_create_info,
                None,
            )
        }
        .expect("Failed to create a logical device");

        // See above comment
        tracing::info!("Creating queues");
        let graphics_queue = unsafe {
            logical_device.get_device_queue(
                physical_device
                    .queue_family_indicies
                    .graphics_index
                    .unwrap(),
                0,
            )
        };

        let present_queue = unsafe {
            logical_device.get_device_queue(physical_device.queue_family_indicies.present_queue.unwrap(), 0)
        };

        Self {
            logical_device,
            graphics_queue,
            present_queue
        }
    }
}
