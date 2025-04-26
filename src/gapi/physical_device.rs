use vulkanalia::vk::PhysicalDevice as VkPhysicalDevice;

#[derive(Clone, Debug)]
pub(crate) struct PhysicalDevice {
	vk_physical_device: VkPhysicalDevice,
	properties: vulkanalia::vk::PhysicalDeviceProperties,
}

impl PhysicalDevice {
	pub(in crate::gapi) fn new(vk_physical_device: VkPhysicalDevice, properties: vulkanalia::vk::PhysicalDeviceProperties) -> Self {
		Self { vk_physical_device, properties }
	}
	pub(in crate::gapi) fn get_vk(&self) -> &VkPhysicalDevice {
		&self.vk_physical_device
	}

	pub(in crate::gapi) fn get_properties(&self) -> &vulkanalia::vk::PhysicalDeviceProperties {
		&self.properties
	}
}
