use crate::gapi::vulkan::instance::Instance;
use crate::gapi::vulkan::surface::Surface;
use vulkanalia::vk;
use vulkanalia::vk::{
    InstanceV1_0, KhrSurfaceExtension, PhysicalDevice as VkPhysicalDevice, QueueFamilyProperties,
};

#[derive(Clone, Debug)]
pub(crate) struct RealDevice<'a> {
    vk_real_device: VkPhysicalDevice,
    instance: &'a Instance,
}

impl<'a> RealDevice<'a> {
    pub fn new(instance: &'a Instance, vk_real_device: VkPhysicalDevice) -> Self {
        Self {
            vk_real_device,
            instance,
        }
    }
    pub fn get_vk(&self) -> &VkPhysicalDevice {
        &self.vk_real_device
    }

    // TODO: Maybe store properties inside and return a reference to it?
    pub fn get_properties(&self) -> vk::PhysicalDeviceProperties {
        unsafe {
            self.instance
                .get_vk()
                .get_physical_device_properties(self.vk_real_device)
        }
    }

    pub fn get_features(&self) -> vk::PhysicalDeviceFeatures {
        unsafe {
            self.instance
                .get_vk()
                .get_physical_device_features(self.vk_real_device)
        }
    }

    pub fn get_queue_families_properties(&self) -> Vec<QueueFamilyProperties> {
        unsafe {
            self.instance
                .get_vk()
                .get_physical_device_queue_family_properties(self.vk_real_device)
        }
    }

    pub fn supports_surface(&self, family_index: u32, surface: Surface) -> anyhow::Result<bool> {
        unsafe {
            self.instance
                .get_vk()
                .get_physical_device_surface_support_khr(
                    self.vk_real_device,
                    family_index,
                    surface.get(),
                )
                .map_err(|e| anyhow::anyhow!("Failed to get surface \"{:#?}\" support for family \"{:#?}\" and physical device \"{:#?}\": {}",
                    surface, family_index, self.vk_real_device, e))
        }
    }
}
