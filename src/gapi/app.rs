use crate::debug_success;
use crate::gapi::vulkan::entry::Entry;
use crate::gapi::vulkan::enums::extensions::DeviceExtension;
use crate::gapi::vulkan::instance::Instance;
use crate::gapi::vulkan::logical_device::{
    LogicalDevice, QueueCapability, QueueRequest, RealDevice,
};
use crate::gapi::vulkan::surface::Surface;
use anyhow::anyhow;
use log::{debug, info, trace, warn};
use thiserror::Error;
use vulkanalia::vk;
use crate::window::MyWindow;

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
        debug!("Creating Entry...");
        let entry = Entry::new()?;
        debug_success!("Entry Created! Loader Version: {}", entry.version()?);
        debug!("Creating Instance...");
        let instance = Instance::new(&entry, window)?;
        debug_success!("Instance Created!");
        debug!("Creating Surface...");
        let surface = Surface::new(&instance, window)?;
        debug_success!("Surface Created!");
        let requests: Vec<QueueRequest> = vec![QueueRequest {
            capabilities: vec![QueueCapability::Graphics],
            require_present: true,
            count: 1,
        }];
        info!("Required Queues: {:?}", requests);

        let device = LogicalDevice::new(
            &entry,
            &instance,
            &surface,
            Self::pick_real_device(&instance, &surface)?,
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
    fn check_real_device(real_device: &RealDevice, surface: &Surface) -> anyhow::Result<()> {
        let device_name = real_device.get_properties().device_name.to_string();
        trace!("Checking \"{:?}\"'s features...", device_name);
        // Optional features like texture compression, 64-bit floats, and multi-viewport rendering.
        let features = real_device.get_features();
        // We require support for geometry shaders.
        if features.geometry_shader != vk::TRUE {
            return Err(anyhow!(SuitabilityError(
                "Missing geometry shader support."
            )));
        }
        trace!("\t{:?} supports geometry shaders.", device_name);
        trace!("Checking \"{:?}\"'s extensions...", device_name);
        let supported_extensions = real_device
            .supported_extensions()?
            .iter()
            .map(|sup_ext| sup_ext.extension_name)
            .collect::<Vec<_>>();
        
        // Not all graphics cards are capable of presenting images directly to a screen for various
        // reasons, for example because they are designed for servers and don't have any display
        // outputs.
        // Therefore, we need to check if the device supports the required extensions for
        // presenting images to a screen.
        let required_extensions = vec![DeviceExtension::KhrSwapchain];
        for ext in &required_extensions {
            if supported_extensions.contains(&ext.name_buf()) {
                trace!(
                    "\t{:?} supports required extension {:?}",
                    device_name,
                    ext.name_buf().to_string()
                );
            } else {
                trace!(
                    "\t{:?} does NOT support required extension {:?}",
                    device_name,
                    ext.name_buf().to_string()
                );
                return Err(anyhow!(SuitabilityError(
                    "Missing required device extensions."
                )));
            }
        }

        // It is mandatory to check the swapchain information AFTER checking for the swapchain
        // extension, swapchain support is only available if the extension is supported.
        let swapchain = real_device.get_swapchain_info(surface)?;
        if swapchain.formats.is_empty() || swapchain.present_modes.is_empty() {
            return Err(anyhow!(SuitabilityError("Insufficient swapchain support.")));
        }


        trace!("{:?} is supported by our app!", device_name);
        Ok(())
    }
    fn pick_real_device<'a>(instance: &'a Instance, surface: &Surface) -> anyhow::Result<RealDevice<'a>> /* Returned RealDevice's lifetime is bound to Instance */
    {
        let available_devices = instance.enumerate_real_devices()?;
        info!(
            "Picking physical device between available devices: {:?}.",
            available_devices
                .iter()
                .map(|d| d.get_properties().device_name.to_string())
                .collect::<Vec<_>>()
        );
        for real_dev in available_devices {
            let properties = real_dev.get_properties();
            if let Err(error) = Self::check_real_device(&real_dev, surface) {
                debug!(
                    "Skipping physical device (`{}`): {}",
                    properties.device_name, error
                );
            } else {
                info!("Selected physical device (`{}`).", properties.device_name);
                if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
                    warn!("This selected physical device is not discrete.");
                }
                return anyhow::Ok(real_dev);
            }
        }

        Err(anyhow!("Failed to find suitable physical device."))
    }
    
    fn select_swapchain_surface_format(){
        
    }
    /// Renders a frame for our Vulkan app.
    pub fn render(&mut self, window: &MyWindow) -> anyhow::Result<()> {

        Ok(())
    }

    /// Destroys our Vulkan app.
    pub fn destroy(&self) {
        info!("Destroying Vulkan App...");
        self.device.destroy();
        self.surface.destroy(&self.instance);
        self.instance.destroy();
    }
}
