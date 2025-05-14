use crate::gapi::instance::Instance;
use vulkanalia::vk;
use vulkanalia::vk::{InstanceV1_0, PhysicalDevice as VkPhysicalDevice, PhysicalDeviceProperties};

#[derive(Clone, Debug)]
pub(crate) struct PhysicalDevice<'a> {
    vk_physical_device: VkPhysicalDevice,
    instance: &'a Instance,
}

impl<'a> PhysicalDevice<'a> {
    pub(in crate::gapi) fn new(
        vk_physical_device: VkPhysicalDevice,
        instance: &'a Instance,
    ) -> Self {
        Self {
            vk_physical_device,
            instance,
        }
    }

    pub(in crate::gapi) fn get_vk(&self) -> &VkPhysicalDevice {
        &self.vk_physical_device
    }

    pub(in crate::gapi) fn get_properties(&self) -> PhysicalDeviceProperties {
        unsafe {
            self.instance
                .get()
                .get_physical_device_properties(self.vk_physical_device)
        }
    }

    pub(in crate::gapi) fn get_features(&self) -> vk::PhysicalDeviceFeatures {
        unsafe {
            self.instance
                .get()
                .get_physical_device_features(self.vk_physical_device)
        }
    }

    pub(in crate::gapi) fn get_queue_family_properties(&self) -> Vec<vk::QueueFamilyProperties> {
        unsafe {
            self.instance
                .get()
                .get_physical_device_queue_family_properties(self.vk_physical_device)
        }
    }

    pub(in crate::gapi) fn get_queue_families_properties(&self) -> Vec<vk::QueueFamilyProperties> {
        unsafe {
            self.instance
                .get()
                .get_physical_device_queue_family_properties(self.vk_physical_device)
        }
    }
}
