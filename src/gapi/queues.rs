use crate::gapi::instance::Instance;
use crate::gapi::logical_device::LogicalDevice;
use crate::gapi::physical_device::PhysicalDevice;
use crate::gapi::surface::Surface;
use crate::gapi::vulkan::SuitabilityError;
use anyhow::anyhow;
use std::collections::HashMap;
use vulkanalia::vk::{DeviceV1_0, InstanceV1_0, KhrSurfaceExtension, Queue};
use vulkanalia::{vk, Device};

/// Stores indices (numbers that point to the queue family)
/// of the queue families that our Vulkan app will use.
///
/// # Notes
/// * We store the graphics queue family and the presentation queue family
/// separately, but it is very likely that they will be the same.
/// It is possible to explicitly check if they are the same and then
/// use the same index for both to improve performance.
#[derive(Copy, Clone, Debug)]
pub(crate) struct QueueFamilyIndices {
    /// The graphics queue family, which stores graphic operations
    pub(crate) graphics: u32,
    /// The presentation queue family, which stores presentation operations
    /// for the vulkan window surface
    pub(crate) present: u32,
}

#[derive(Clone, Debug)]
pub(crate) enum QueueFamily {
    Graphics = 0,
    Present,
}
#[derive(Clone, Debug)]
pub(crate) struct QueueFamilies {
    /// This is a handle to the graphics queue that our Vulkan app will use.
    families: Vec<Queue>,
}

impl QueueFamilyIndices {
    pub fn get(
        instance: &Instance,
        surface: &Surface,
        physical_device: &PhysicalDevice,
    ) -> anyhow::Result<Self> {
        log::debug!("Getting queue family indices");
        // Get various details about the queue families supported by the physical device
        // including the type of operations supported and the number of queues that can be created based on that family
        let properties = unsafe {
            instance
                .get()
                .get_physical_device_queue_family_properties(*physical_device.get())
        };

        // Find the first queue that supports graphic operations (`vk::QueueFlags::GRAPHICS`)
        let graphics = properties
            .iter()
            .position(|p| p.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            .map(|i| i as u32);

        let mut present = None;
        for (index, properties) in properties.iter().enumerate() {
            unsafe {
                if instance.get().get_physical_device_surface_support_khr(
                    *physical_device.get(),
                    index as u32,
                    surface.get(),
                )? {
                    present = Some(index as u32);
                    break;
                }
            }
        }

        match (graphics, present) {
            (Some(graphics), Some(present)) => Ok(Self { graphics, present }),
            _ => Err(anyhow!(SuitabilityError("Missing queue family."))),
        }
    }

    pub(in crate::gapi) fn as_array(&self) -> [u32; 2] {
        [self.graphics, self.present]
    }
}

impl QueueFamilies {
    pub(in crate::gapi) fn get(
        device: &Device,
        queue_indices: &QueueFamilyIndices,
    ) -> anyhow::Result<Self> {
        log::debug!("Getting queue families");
        let queues = Self::get_queues(device, queue_indices);
        Ok(Self { families: queues })
    }
    pub(in crate::gapi) fn get_queue_families(&self) -> &Vec<Queue> {
        &self.families
    }
    pub(in crate::gapi) fn get_queue_family(&self, queue_family: QueueFamily) -> &Queue {
        &self.families[queue_family as usize]
    }
    fn get_queue(device: &Device, queue_family: u32, queue_index: u32) -> vk::Queue {
        unsafe { device.get_device_queue(queue_family, queue_index) }
    }

    fn get_queues(device: &Device, queue_indices: &QueueFamilyIndices) -> Vec<Queue> {
        let mut families = Vec::new();
        for index in queue_indices.as_array().iter() {
            let queue = Self::get_queue(device, *index, 0);
            families.push(queue);
        }
        families
    }
}
