use crate::gapi::queues;
use crate::gapi::vulkan::SuitabilityError;
use anyhow::anyhow;
use log::{info, warn};
use vulkanalia::vk::{InstanceV1_0, PhysicalDevice};
use vulkanalia::{vk, Instance};

/// Function that returns a `SuitabilityError` if a supplied physical device does not support everything we require.
/// # Safety
/// This function is unsafe because it dereferences raw pointers and uses Vulkan.
/// # Errors
/// Returns a `SuitabilityError` if the physical device does not support everything we require.
/// # Returns
/// * `Ok(())` if the physical device supports everything we require.
/// * Returns `Err(anyhow::Error)` if the physical device does not support everything we require.
/// # Arguments
/// * `instance` - The Vulkan instance.
/// * `physical_device` - The physical device to check.
///
unsafe fn check_physical_device(
    instance: &Instance,
    physical_device: PhysicalDevice,
) -> anyhow::Result<()> {
    log::debug!("Checking physical device suitability.");
    queues::QueueFamilyIndices::get(instance, physical_device)?;
    // Basic properties like name, type, and supported Vulkan version.
    let properties = unsafe { instance.get_physical_device_properties(physical_device) };
    // We only want to use discrete (dedicated) GPUs.
    if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
        return Err(anyhow!(SuitabilityError(
            "Only discrete GPUs are supported."
        )));
    }

    // Optional features like texture compression, 64-bit floats, and multi-viewport rendering.
    let features = unsafe { instance.get_physical_device_features(physical_device) };
    // We require support for geometry shaders.
    if features.geometry_shader != vk::TRUE {
        return Err(anyhow!(SuitabilityError(
            "Missing geometry shader support."
        )));
    }

    Ok(())
}

pub fn pick_physical_device(instance: &Instance) -> anyhow::Result<PhysicalDevice> {
    unsafe {
        for physical_device in instance.enumerate_physical_devices()? {
            let properties = instance.get_physical_device_properties(physical_device);

            if let Err(error) = check_physical_device(instance, physical_device) {
                warn!(
                    "Skipping physical device (`{}`): {}",
                    properties.device_name, error
                );
            } else {
                info!("Selected physical device (`{}`).", properties.device_name);
                return anyhow::Ok(physical_device);
            }
        }
    }

    Err(anyhow!("Failed to find suitable physical device."))
}
