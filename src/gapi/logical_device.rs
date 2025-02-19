use crate::gapi::debug;
use crate::gapi::debug::add_validation_layer;
use crate::gapi::queues::QueueFamilyIndices;
use crate::gapi::vulkan::{AppData, PORTABILITY_MACOS_VERSION, VALIDATION_ENABLED};
use vulkanalia::vk::{DeviceV1_0, HasBuilder};
use vulkanalia::{vk, Device, Entry, Instance};
#[derive(Clone, Debug)]
pub(crate) struct LogicalDevice {
    pub device: Device,
}
impl LogicalDevice {
    pub(crate) fn get_queue(&self, queue_family: u32, queue_index: u32) -> vk::Queue {
        unsafe { self.device.get_device_queue(queue_family, queue_index) }
    }
    pub(crate) fn new(
        entry: &Entry,
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
    ) -> anyhow::Result<Self> {
        let device = Self::create_logical_device(entry, instance, physical_device)?;
        Ok(Self { device })
    }
    pub(crate) fn destroy(&self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
    fn create_logical_device(
        entry: &Entry,
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
    ) -> anyhow::Result<Device> {
        log::debug!("Creating logical device...");
        // The currently available drivers will only allow you to create a small number of queues for each queue family, and
        // you don't really need more than one. That's because you can create all the command buffers on multiple threads
        // and then submit them all at once on the main thread with a single low-overhead call.
        let indices = QueueFamilyIndices::get(instance, physical_device)?;

        // Vulkan lets you assign priorities to queues to influence the scheduling of command buffer execution using
        // floating point numbers between 0.0 and 1.0. This is required even when only creating a single queue.
        let queue_priorities = &[1.0];
        // This structure describes the number of queues we want for a single queue family.
        let queue_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(indices.graphics)
            .queue_priorities(queue_priorities);
        // Previous implementations of Vulkan made a distinction between instance and device specific validation layers,
        // but this is no longer the case.
        // However, it is still a good idea to set them anyway to be compatible with older implementations.
        let mut layers = Vec::new();
        if VALIDATION_ENABLED {
            log::info!("Enabling validation layers for the logical device.");
            let available_layers = debug::get_available_layers(entry)?;
            add_validation_layer(available_layers, &mut layers)?;
        };
        let mut extensions = vec![];
        // Required by Vulkan SDK on macOS since 1.3.216.
        if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
            log::info!("Enabling extensions for macOS portability.");
            // Enable macOS support for the logical device
            // # Warning: This is a provisional extension and may change in the future.
            extensions.push(vk::KHR_PORTABILITY_SUBSET_EXTENSION.name.as_ptr());
        }
        let features = vk::PhysicalDeviceFeatures::builder();
        let queue_infos = &[queue_info];
        let info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(queue_infos)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extensions)
            .enabled_features(&features);
        let device = unsafe { instance.create_device(physical_device, &info, None) }?;
        Ok(device)
    }
}
