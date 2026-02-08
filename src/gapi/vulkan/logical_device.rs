use crate::gapi::vulkan::enums::extensions::DeviceExtension;
use crate::gapi::vulkan::instance::Instance;
pub(crate) use crate::gapi::vulkan::queues::QueueCapability;
pub(crate) use crate::gapi::vulkan::queues::{QueueRequest, Queues};
pub(crate) use crate::gapi::vulkan::real_device::RealDevice;
use crate::gapi::vulkan::surface::Surface;
use anyhow::Context;
use vulkanalia::vk::{
    DeviceV1_0, HasBuilder, ImageViewCreateInfoBuilder, KhrSwapchainExtension,
    PhysicalDeviceFeatures, Queue, SwapchainCreateInfoKHR, SwapchainKHR,
};
use vulkanalia::{vk, Device};

/// Wraps the Vulkan logical device, and the queue handles it owns.
///
/// This object is responsible for:
/// - Creating the Vulkan device from a chosen physical device.
/// - Finding and storing all queue handles (graphics, present, etc.) according to user requests.
/// - Destroying the device (and by extension, the queues) at shutdown.
#[derive(Debug)]
pub struct LogicalDevice {
    /// The Vulkan device handle.
    device: Device,
    queues: Queues,
}

impl LogicalDevice {
    pub fn new(
        real_device: &RealDevice,
        instance: &Instance,
        surface: &Surface,
        requests: &[QueueRequest],
        extensions: &[DeviceExtension],
    ) -> anyhow::Result<Self> {
        let resolved_families = Queues::resolve_queue_requests(real_device, surface, requests)
            .with_context(|| format!("Failed to resolve queue requests: {:?}", requests))?;

        let queue_infos = Queues::create_queue_infos(&resolved_families);

        let ext_names = extensions.iter().map(|e| e.name_ptr()).collect::<Vec<_>>();
        let features = PhysicalDeviceFeatures::builder().geometry_shader(true);

        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&ext_names)
            .enabled_features(&features);

        let device = unsafe {
            instance
                .get_vk()
                .create_device(*real_device.get_vk(), &create_info, None)?
        };

        let queues = Queues::new(&device, &real_device, surface, &resolved_families)?;
        Ok(Self { device, queues })
    }

    fn get_vk_queue(&self, family_index: u32, queue_index: u32) -> Queue {
        unsafe { self.device.get_device_queue(family_index, queue_index) }
    }

    pub fn create_swapchain_khr(
        &self,
        info: &SwapchainCreateInfoKHR,
    ) -> anyhow::Result<SwapchainKHR> {
        unsafe {
            self.device
                .create_swapchain_khr(info, None)
                .map_err(|e| anyhow::anyhow!("Failed to create swapchain: {}", e))
        }
    }

    pub fn destroy_swapchain_khr(&self, swapchain: SwapchainKHR) {
        unsafe {
            self.device.destroy_swapchain_khr(swapchain, None);
        }
    }

    pub fn destroy_image_view(&self, image_view: vk::ImageView) {
        unsafe {
            self.device.destroy_image_view(image_view, None);
        }
    }

    pub(crate) fn create_image_view(
        &self,
        create_info: &ImageViewCreateInfoBuilder,
    ) -> anyhow::Result<vk::ImageView> {
        unsafe {
            self.device
                .create_image_view(create_info, None)
                .map_err(|e| anyhow::anyhow!("Failed to create image view: {}", e))
        }
    }

    pub fn get_swapchain_images_khr(
        &self,
        swapchain: SwapchainKHR,
    ) -> anyhow::Result<Vec<vk::Image>> {
        unsafe {
            self.device
                .get_swapchain_images_khr(swapchain)
                .map_err(|e| anyhow::anyhow!("Failed to get swapchain images: {}", e))
        }
    }

    /// Returns a reference to the underlying Vulkan [`Device`].
    ///
    /// # Example
    /// ```
    /// let device_handle = logical_device.get_device();
    /// // use device_handle...
    /// ```
    pub fn get_vk(&self) -> &Device {
        &self.device
    }

    pub fn get_queues(&self) -> &Queues {
        &self.queues
    }

    /// Destroys this logical device. Automatically frees all queues it owns.
    ///
    /// # Safety
    /// Must only be called when you are certain no further use of the device or
    /// its queues is needed.
    pub fn destroy(&self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}
