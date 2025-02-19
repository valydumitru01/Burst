use crate::gapi::vulkan::SuitabilityError;
use anyhow::anyhow;
use vulkanalia::vk::InstanceV1_0;
use vulkanalia::{vk, Instance};

/// Stores indices (numbers that point to the queue family)
#[derive(Copy, Clone, Debug)]
pub(crate) struct QueueFamilyIndices {
    /// The graphics queue family, which stores graphic operations
    pub(crate) graphics: u32,
}

impl QueueFamilyIndices {
    pub fn get(instance: &Instance, physical_device: vk::PhysicalDevice) -> anyhow::Result<Self> {
        log::debug!("Getting queue family indices");
        // Get various details about the queue families supported by the physical device
        // including the type of operations supported and the number of queues that can be created based on that family
        let properties =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        // Find the first queue that supports graphic operations (`vk::QueueFlags::GRAPHICS`)
        let graphics = properties
            .iter()
            .position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|i| i as u32);

        if let Some(graphics) = graphics {
            Ok(Self { graphics })
        } else {
            Err(anyhow!(SuitabilityError(
                "Missing required queue families."
            )))
        }
    }
}
