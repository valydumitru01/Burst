use crate::gapi::vulkan::instance::Instance;
use crate::gapi::vulkan::surface::Surface;
use vulkanalia::vk;
use vulkanalia::vk::{
    InstanceV1_0, KhrSurfaceExtension, PhysicalDevice as VkPhysicalDevice, PresentModeKHR,
    QueueFamilyProperties, SurfaceCapabilitiesKHR, SurfaceFormatKHR,
};
#[derive(Clone, Debug)]
pub(crate) struct SwapchainInfo {
    pub(crate) capabilities: SurfaceCapabilitiesKHR,
    pub(crate) formats: Vec<SurfaceFormatKHR>,
    pub(crate) present_modes: Vec<PresentModeKHR>,
}
#[derive(Clone, Debug)]
pub struct RealDevice<'a> {
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

    pub fn supported_extensions(&self) -> anyhow::Result<Vec<vk::ExtensionProperties>> {
        unsafe {
            self.instance
                .get_vk()
                .enumerate_device_extension_properties(self.vk_real_device, None)
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to enumerate device extensions for physical device \"{:#?}\": {}",
                        self.vk_real_device,
                        e
                    )
                })
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

    pub fn get_surface_capabilities(
        &self,
        surface: &Surface,
    ) -> anyhow::Result<vk::SurfaceCapabilitiesKHR> {
        unsafe {
            self.instance
                .get_vk()
                .get_physical_device_surface_capabilities_khr(
                    self.vk_real_device,
                    surface.get(),
                )
                .map_err(|e| anyhow::anyhow!("Failed to get surface capabilities for surface \"{:#?}\" and physical device \"{:#?}\": {}",
                    surface, self.vk_real_device, e))
        }
    }

    pub fn get_surface_formats(
        &self,
        surface: &Surface,
    ) -> anyhow::Result<Vec<vk::SurfaceFormatKHR>> {
        unsafe {
            self.instance
                .get_vk()
                .get_physical_device_surface_formats_khr(
                    self.vk_real_device,
                    surface.get(),
                )
                .map_err(|e| anyhow::anyhow!("Failed to get surface formats for surface \"{:#?}\" and physical device \"{:#?}\": {}",
                    surface, self.vk_real_device, e))
        }
    }

    pub fn get_surface_present_modes(
        &self,
        surface: &Surface,
    ) -> anyhow::Result<Vec<vk::PresentModeKHR>> {
        unsafe {
            self.instance
                .get_vk()
                .get_physical_device_surface_present_modes_khr(
                    self.vk_real_device,
                    surface.get(),
                )
                .map_err(|e| anyhow::anyhow!("Failed to get surface present modes for surface \"{:#?}\" and physical device \"{:#?}\": {}",
                    surface, self.vk_real_device, e))
        }
    }

    pub fn get_swapchain_info(&self, surface: &Surface) -> anyhow::Result<SwapchainInfo> {
        Ok(SwapchainInfo {
            capabilities: self.get_surface_capabilities(surface)?,
            formats: self.get_surface_formats(surface)?,
            present_modes: self.get_surface_present_modes(surface)?,
        })
    }
}
