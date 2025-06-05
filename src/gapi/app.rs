use crate::gapi::vulkan::entry::Entry;
use crate::gapi::vulkan::instance::Instance;
use crate::gapi::vulkan::logical_device::{
    LogicalDevice, QueueCapability, QueueRequest, RealDevice,
};
use crate::gapi::vulkan::surface::Surface;
use crate::window::window::MyWindow;
use anyhow::anyhow;
use log::{debug, info, trace};
use thiserror::Error;
use vulkanalia::vk;

/// Our Vulkan app.
#[derive(Clone, Debug)]
pub struct App {
    entry: Entry,
    instance: Instance,
    device: LogicalDevice,
    surface: Surface,
}
#[derive(Debug, Error)]
#[error("Missing {0}.")]
pub(crate) struct SuitabilityError(pub &'static str);

/// # Vulkan App
/// This handles everything in Vulkan.
///
/// Vulkan is a wrapper around the Vulkan Driver, which is a platform-agnostic abstraction for
/// the actual GPU hardware interface.
impl App {
    /// Creates our Vulkan app.
    pub fn new(window: &MyWindow) -> anyhow::Result<Self> {
        let entry = Entry::new()?;
        let instance = Instance::new(&entry, window)?;
        let surface = Surface::new(&instance, window)?;
        let requests: Vec<QueueRequest> = vec![QueueRequest {
            capabilities: vec![QueueCapability::Graphics],
            require_present: true,
            count: 1,
        }];
        let device = LogicalDevice::new(
            &entry,
            &instance,
            &surface,
            &window,
            Self::pick_real_device(&instance)?,
            requests,
        )?;

        Ok(Self {
            entry,
            instance,
            device,
            surface,
        })
    }

    /// Function that returns a `SuitabilityError` if a supplied physical device does not support everything we require.
    /// # Errors
    /// It returns a `SuitabilityError` if the physical device does not support everything we require.
    /// # Returns
    /// - `Ok(())` if the physical device supports everything we require.
    /// - Returns `Err(anyhow::Error)` if the physical device does not support everything we require.
    /// # Arguments
    /// - `real_device` - The physical device to check.
    fn check_real_device(real_device: &RealDevice) -> anyhow::Result<()> {
        trace!("Checking physical device suitability...");
        let properties = real_device.get_properties();
        trace!("Checking if the physical device is discrete.");
        // We only want to use discrete (dedicated) GPUs.
        if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
            return Err(anyhow!(SuitabilityError(
                "Only discrete GPUs are supported."
            )));
        }

        // Optional features like texture compression, 64-bit floats, and multi-viewport rendering.
        let features = real_device.get_features();
        trace!("Checking for geometry shader feature.");
        // We require support for geometry shaders.
        if features.geometry_shader != vk::TRUE {
            return Err(anyhow!(SuitabilityError(
                "Missing geometry shader support."
            )));
        }
        trace!("This physical device is supported by our app!");
        Ok(())
    }
    fn pick_real_device(instance: &Instance) -> anyhow::Result<RealDevice> {
        trace!("Picking physical device...");
        for real_dev in instance.enumerate_real_devices()? {
            let properties = real_dev.get_properties();
            if let Err(error) = Self::check_real_device(&real_dev) {
                debug!(
                    "Skipping physical device (`{}`): {}",
                    properties.device_name, error
                );
            } else {
                info!("Selected physical device (`{}`).", properties.device_name);
                return anyhow::Ok(real_dev);
            }
        }

        Err(anyhow!("Failed to find suitable physical device."))
    }
    /// Renders a frame for our Vulkan app.
    pub fn render(&mut self, window: &MyWindow) -> anyhow::Result<()> {
        Ok(())
    }

    /// Destroys our Vulkan app.
    pub fn destroy(&self) {
        self.device.destroy();
        self.surface.destroy(&self.instance);
        self.instance.destroy();
    }
}
