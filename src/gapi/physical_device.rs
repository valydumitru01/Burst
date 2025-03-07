use vulkanalia::vk::PhysicalDevice as VkPhysicalDevice;

#[derive(Clone, Debug)]
pub(crate) struct PhysicalDevice {
    vk_physical_device: VkPhysicalDevice,
}

impl PhysicalDevice {
    pub(in crate::gapi) fn new(vk_physical_device: VkPhysicalDevice) -> Self {
        Self { vk_physical_device }
    }
    pub(in crate::gapi) fn get(&self) -> &VkPhysicalDevice {
        &self.vk_physical_device
    }
}
