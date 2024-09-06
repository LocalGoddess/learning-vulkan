use ash::vk::{
    self, DeviceCreateInfo, DeviceQueueCreateInfo, PhysicalDevice, PhysicalDeviceFeatures,
    PhysicalDeviceProperties, PhysicalDeviceType, QueueFlags,
};

use crate::util::str_to_cstr;

pub struct VulkanPhysicalDevice {
    pub physical_device: PhysicalDevice,
    pub properties: PhysicalDeviceProperties,
    pub features: PhysicalDeviceFeatures,
    pub queue_family_indicies: QueueFamilyIndicies,
}

impl VulkanPhysicalDevice {
    pub fn select(instance: &ash::Instance) -> Self {
        tracing::info!("Selecting a physical device");

        let physical_devices = unsafe { instance.enumerate_physical_devices() }
            .expect("Failed to find any Vulkan supported physical devices");

        let mut selected_device: Option<PhysicalDevice> = None;
        let mut highest_rating: u32 = 0;
        for physical_device in physical_devices {
            let current_rating = Self::rate_device(instance, physical_device);
            if current_rating > highest_rating {
                highest_rating = current_rating;
                selected_device = Some(physical_device);
            }
        }

        // I don't like getting the properties and features a second time after the loop but it
        // will do for now
        let properties =
            unsafe { instance.get_physical_device_properties(selected_device.unwrap()) };
        let features = unsafe { instance.get_physical_device_features(selected_device.unwrap()) };

        let queue_family_indicies =
            QueueFamilyIndicies::get_all(instance, selected_device.unwrap());

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

    fn rate_device(instance: &ash::Instance, physical_device: PhysicalDevice) -> u32 {
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let features = unsafe { instance.get_physical_device_features(physical_device) };

        let mut rating: u32 = 0u32;
        if !Self::is_suitable(instance, physical_device, features) {
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
    ) -> bool {
        features.geometry_shader == ash::vk::TRUE
            && QueueFamilyIndicies::check_graphics(instance, physical_device)
                .graphics_index
                .is_some()
    }
}

#[derive(Default)]
pub struct QueueFamilyIndicies {
    pub graphics_index: Option<u32>,
}

impl QueueFamilyIndicies {
    pub fn get_all(
        instance: &ash::Instance,
        physical_device: PhysicalDevice,
    ) -> QueueFamilyIndicies {
        let mut queue_family_indicies = QueueFamilyIndicies::default();

        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        for (index, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags.contains(QueueFlags::GRAPHICS) {
                queue_family_indicies.graphics_index = Some(index as u32);
            }
        }

        queue_family_indicies
    }

    // Might delete depending on how big the get_all function gets
    pub fn check_graphics(
        instance: &ash::Instance,
        physical_device: PhysicalDevice,
    ) -> QueueFamilyIndicies {
        let mut qf_indicies = QueueFamilyIndicies::default();

        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        for (index, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags.contains(QueueFlags::GRAPHICS) {
                qf_indicies.graphics_index = Some(index as u32);
            }
        }

        qf_indicies
    }

    pub fn update(&mut self) {
        todo!("Lazy");
    }

    pub fn has_all(&self) -> bool {
        self.graphics_index.is_some()
    }
}

pub struct VulkanLogicalDevice {
    pub logical_device: ash::Device,

    pub graphics_queue: vk::Queue,
}

impl VulkanLogicalDevice {
    pub fn create(instance: &ash::Instance, physical_device: &VulkanPhysicalDevice) -> Self {
        tracing::info!("Creating a logical device");

        // Maybe asbtract away the queue creation from here?
        let queue_priorities = [1f32];
        let logical_device_queue_create_info = DeviceQueueCreateInfo::default()
            .queue_family_index(
                physical_device
                    .queue_family_indicies
                    .graphics_index
                    .unwrap(),
            )
            .queue_priorities(&queue_priorities);

        let qcinfos = [logical_device_queue_create_info];
        let logical_device_create_info = DeviceCreateInfo::default()
            .enabled_features(&physical_device.features)
            .queue_create_infos(&qcinfos);

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

        Self {
            logical_device,
            graphics_queue,
        }
    }

    pub fn create_compatability(
        instance: &ash::Instance,
        physical_device: &VulkanPhysicalDevice,
        extension_names: Vec<*const i8>,
    ) -> Self {
        todo!("Make VulkanLogicalDevice::create more generic then implement this")
    }
}
