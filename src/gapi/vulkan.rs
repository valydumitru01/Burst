use crate::gapi::debug;
use crate::gapi::debug::{
    add_debug_creation_destruction, add_validation_layer, create_messenger, destroy_debug_calls,
    get_debug_info,
};
use crate::gapi::logical_device::LogicalDevice;
use crate::gapi::physical_device::pick_physical_device;
use crate::gapi::queues::QueueFamilyIndices;
use anyhow::anyhow;
use log::{error, info};
use thiserror::Error;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::{HasBuilder, InstanceV1_0};
use vulkanalia::{vk, Entry, Instance};
use vulkanalia::{window as vk_window, Version};
use winit::window::Window;

pub(crate) const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

pub(crate) const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);
/// Our Vulkan app.
#[derive(Clone, Debug)]
pub struct App {
    entry: Entry,
    instance: Instance,

    debugger: debug::DebugData,
    /// This is a handle to the physical device that our Vulkan app will use.
    /// This is automatically destroyed when the instance is destroyed.
    physical_device: vk::PhysicalDevice,
    /// This is a handle to the graphics queue that our Vulkan app will use.
    graphics_queue: vk::Queue,
    device: LogicalDevice,
}
#[derive(Debug, Error)]
#[error("Missing {0}.")]
pub(crate) struct SuitabilityError(pub &'static str);
impl App {
    /// Creates our Vulkan app.
    pub unsafe fn create(window: &Window) -> anyhow::Result<Self> {
        let loader = unsafe { LibloadingLoader::new(LIBRARY)? };
        let entry = unsafe { Entry::new(loader).map_err(|b| anyhow!("{}", b))? };
        let mut data = AppData::default();
        let instance = Self::create_instance(window, &entry)?;
        let physical_device = pick_physical_device(&instance)?;
        let device = LogicalDevice::new(&entry, &instance, physical_device)?;
        let queue_indices = QueueFamilyIndices::get(&instance, physical_device)?;
        let graphics_queue = device.get_queue(queue_indices.graphics, 0);
        let mut debugger = debug::DebugData::default();
        // Add debug callback layer for vulkan calls.
        if VALIDATION_ENABLED {
            let debug_info = get_debug_info();
            debugger.messenger = create_messenger(&instance, &debug_info, &mut debugger)?;
        }
        Ok(Self {
            entry,
            instance,
            debugger,
            physical_device,
            graphics_queue,
            device,
        })
    }

    fn create_instance(window: &Window, entry: &Entry) -> anyhow::Result<Instance> {
        let application_info = vk::ApplicationInfo::builder()
            .application_name(b"Burst\0")
            .application_version(vk::make_version(1, 0, 0))
            .engine_name(b"No Engine\0")
            .engine_version(vk::make_version(1, 0, 0))
            .api_version(vk::make_version(1, 0, 0));

        let mut extensions = vk_window::get_required_instance_extensions(window)
            .iter()
            .map(|e| e.as_ptr())
            .collect::<Vec<_>>();
        let mut layers = Vec::new();
        if VALIDATION_ENABLED {
            extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
            let available_layers = debug::get_available_layers(entry)?;
            add_validation_layer(available_layers, &mut layers)?;
        }

        // Required by Vulkan SDK on macOS since 1.3.216.
        let flags = if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
            info!("Enabling extensions for macOS portability.");
            // Allow Query extended physical device properties
            extensions.push(
                vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION
                    .name
                    .as_ptr(),
            );
            //  Enable macOS support for the physical device
            extensions.push(vk::KHR_PORTABILITY_ENUMERATION_EXTENSION.name.as_ptr());

            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::empty()
        };

        let mut info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_layer_names(&layers)
            .enabled_extension_names(&extensions)
            .flags(flags);
        let mut debug_info = get_debug_info();
        // Add debug messages for creation and destruction of the Vulkan instance.
        if VALIDATION_ENABLED {
            log::debug!("Validation layers enabled.");
            add_debug_creation_destruction(&mut info, &mut debug_info);
        }
        let instance = unsafe { entry.create_instance(&info, None) }?;

        Ok(instance)
    }

    /// Renders a frame for our Vulkan app.
    pub unsafe fn render(&mut self, window: &Window) -> anyhow::Result<()> {
        Ok(())
    }

    /// Destroys our Vulkan app.
    pub fn destroy(&mut self) {
        self.device.destroy();
        if VALIDATION_ENABLED {
            destroy_debug_calls(&self.instance, &mut self.debugger);
        }
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

/// The Vulkan handles and associated properties used by our Vulkan app.
#[derive(Clone, Debug, Default)]
pub(crate) struct AppData {}
