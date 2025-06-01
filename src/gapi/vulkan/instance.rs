use crate::gapi::app::SuitabilityError;
use crate::gapi::vulkan;
use crate::gapi::vulkan::config::VALIDATION_ENABLED;
use crate::gapi::vulkan::debug::Debugger;
use crate::gapi::vulkan::entry::Entry;
pub(crate) use crate::gapi::vulkan::extensions::{
    ExtensionStr, InstanceExtensions, PORTABILITY_MACOS_VERSION,
};
use crate::gapi::vulkan::layers;
use crate::gapi::vulkan::layers::{InstanceLayers, LayerStr};
use crate::gapi::vulkan::real_device::RealDevice;
use crate::window::window::MyWindow;
use anyhow::anyhow;
use log::{debug, info, trace};
use std::collections::HashSet;
use vulkanalia::vk::{HasBuilder, InstanceV1_0};
use vulkanalia::{vk, Instance as VkInstance, Version, VkResult};

/// # Vulkan Instance
/// The Vulkan instance is the connection between this program and the Vulkan driver.
/// Acts as the "context" for the entire Vulkan ecosystem.
/// It is the first object created in a Vulkan application and is responsible for managing
/// the Vulkan API's state.
///
/// # Details
/// The Vulkan instance is the constructor and source of the Vulkan API.
/// Allows configuration of the Layers, GPUs, and Extensions that will be used in the application.
///
/// More exactly:
/// - Allows global queries for physical devices (GPUs), layers, and extensions.
/// - Allows the creation of surfaces and debug utils, which are instance extensions.
/// - Handles the Vulkan lifetime, if the instance is destroyed, all objects created
/// with it are also destroyed.
///
/// > Note: Instance captures the driver state at creation time, so any changes to the driver,
/// > layers, or extensions at system level after instance creation will not be reflected in the
/// > instance.
///
///
#[derive(Clone, Debug)]
pub(crate) struct Instance {
    instance: VkInstance,
}

impl Instance {
    /// # Instance Creation
    ///
    /// [`Entry`] is in charge of creating the Vulkan [Instance];
    /// this constructor handles the [flags](vk::Flags), [extensions](InstanceExtensions),
    /// [layers](Instance), and [info](vk::ApplicationInfo) needed to create the instance.
    ///
    /// The configuration of the Instance is abstracted away inside the [`Instance`] class.
    ///
    /// # Details
    /// - First the constructor gathers the configuration data (flags, extensions, etc.) defined
    /// within the `Instance` class.
    /// - Then, if validation is enabled, we add a validation layer.
    /// -
    ///
    /// # Errors
    ///
    /// Returns error if the machine is Mac and the Vulkan version that the machine has does not
    /// support portability to macOS.
    ///
    pub fn new(entry: &Entry, window: &MyWindow) -> anyhow::Result<Self> {
        let flags = Self::get_flags();
        let extensions = Self::get_required_extensions(window);
        let extension_ptrs = extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();
        let layers = Self::get_required_layers();
        let layer_ptrs = layers
            .iter()
            .map(|layer| layer.as_ptr())
            .collect::<Vec<_>>();
        let application_info = vk::ApplicationInfo::builder()
            .application_name(b"Burst\0")
            .application_version(vk::make_version(1, 0, 0))
            .engine_name(b"BurstG\0")
            .engine_version(vk::make_version(1, 0, 0))
            .api_version(vk::make_version(1, 0, 0))
            .build();

        // Check if the requested layers are available.
        trace!("Checking if requested Instance layers are available...");
        let unavailable_layers =
            Self::find_unavailable_layers(entry.get_available_layers()?, layers);

        if !unavailable_layers.is_empty() {
            return Err(anyhow!(
                "Missing required layer(s): {:?}",
                unavailable_layers
            ));
        }
        trace!("Requested Instance layers are available!");

        trace!("Building InstanceCreateInfo...");
        let mut info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_layer_names(&layer_ptrs)
            .enabled_extension_names(&extension_ptrs)
            .flags(flags);
        trace!("InstanceCreateInfo build!");

        // Add debug messages for creation and destruction of the Vulkan instance.
        if VALIDATION_ENABLED {
            debug!("Enabling Validation Layer.");
            Debugger::add_instance_life_debug(&mut info);
        }
        debug!("Creating instance...");
        let instance = entry.create_instance(&info, None)?;

        Ok(Self { instance })
    }

    fn check_compatibility(entry: Entry) -> anyhow::Result<()> {
        if !cfg!(target_os = "macos") {
            return Ok(());
        }
        trace!(
            "MacOS detected, checking Entry version ({} required)",
            PORTABILITY_MACOS_VERSION
        );
        let entry_version = entry.version()?;
        // Required by Vulkan SDK on macOS since 1.3.216.
        if entry_version < PORTABILITY_MACOS_VERSION {
            return Err(anyhow!(
                "MacOS portability requires Vulkan {}",
                PORTABILITY_MACOS_VERSION
            ));
        }
        trace!(
            "MacOS version compatible! (current: {}, expected: {})",
            entry_version, PORTABILITY_MACOS_VERSION
        );
        Ok(())
    }

    /// Collects all the extensions that will be used for the Vulkan [`Instance`] creation.
    ///
    /// # Parameters
    /// - `window`: The window handler ([`MyWindow`]) that knows its required extensions.
    ///
    /// # Returns
    /// - A vector of [`ExtensionStr`] that contains the required extensions for the Vulkan instance.
    fn get_required_extensions(window: &MyWindow) -> Vec<&ExtensionStr> {
        trace!("Configuring extensions...");
        trace!("Configuring windows extensions...");
        // Query for the extensions required by the window system.
        let mut extensions = window
            .get_required_extensions()
            .iter()
            .map(|ext| *ext)
            .collect::<Vec<_>>();
        window.get_required_extensions().iter().for_each(|ext| {
            trace!("Required extension: {}", ext);
        });
        // Add the required extensions for the Vulkan validation.
        if VALIDATION_ENABLED {
            extensions.push(InstanceExtensions::ExtDebugUtils.name());
            trace!("Configuring validation layers...");
            trace!(
                "Required extension: {}",
                InstanceExtensions::ExtDebugUtils.name()
            );
        }
        // Add the required extensions for the Vulkan portability.
        if cfg!(target_os = "macos") {
            trace!("Configuring MacOS extensions...");
            // Allow Query extended physical device properties
            extensions.push(InstanceExtensions::KhrGetPhysicalDeviceProperties2.name());
            trace!(
                "Required extension: {}",
                InstanceExtensions::KhrGetPhysicalDeviceProperties2.name()
            );
            //  Enable macOS support for the physical device
            extensions.push(InstanceExtensions::KhrPortabilityEnumeration.name());
            trace!(
                "Required extension: {}",
                InstanceExtensions::KhrPortabilityEnumeration.name()
            );
        }
        extensions
    }

    /// Configures the [`Instance`] [layers](Instance)
    /// # Returns
    /// A list of all the [layers](Instance) required by [`Instance`]
    fn get_required_layers() -> Vec<LayerStr> {
        trace!("Configuring layers...");
        let mut layers: Vec<LayerStr> = vec![];
        layers.push(InstanceLayers::ApiDump.as_str());
        trace!("Required Layer: {}", InstanceLayers::ApiDump.as_str());
        layers.push(InstanceLayers::RenderDoc.as_str());
        trace!("Required Layer: {}", InstanceLayers::RenderDoc.as_str());
        if VALIDATION_ENABLED {
            layers.push(InstanceLayers::Validation.as_str());
            trace!("Required Layer: {}", InstanceLayers::Validation.as_str());
        }
        layers
    }

    /// Finds all unavailable layers before creating [`Instance`].
    /// The available layers must be returned by [`Entry`].
    ///
    /// # Parameters
    /// - `available_layers`: The layers available in the system (queried through the Vulkan [`Entry`]).
    /// - `instance_layers`: The layers to be used in the [`Instance`], configured in the instance creation.
    ///
    /// # Returns
    /// - A list of all the unavailable layers in the configuration.
    fn find_unavailable_layers(
        available_layers: HashSet<LayerStr>,
        instance_layers: Vec<LayerStr>,
    ) -> Vec<LayerStr> {
        trace!("Searching for unavailable layers...");
        instance_layers
            .into_iter()
            .filter(|layer| !available_layers.contains(layer))
            .collect()
    }

    /// Configures the flags for [`Instance`]
    /// # Returns
    /// All the flags that will be passed to the [`Instance`] constructor.
    fn get_flags() -> vk::InstanceCreateFlags {
        trace!("Configuring instance flags...");
        if cfg!(target_os = "macos") {
            trace!(
                "Flag configured: {:?}",
                vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
            );
            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::empty()
        }
    }

    pub fn enumerate_real_devices(&self) -> VkResult<Vec<RealDevice>> {
        trace!("Enumerating physical devices...");
        unsafe {
            Ok(self
                .instance
                .enumerate_physical_devices()?
                .into_iter()
                .map(|device| RealDevice::new(self, device))
                .collect::<Vec<_>>())
        }
    }

    pub fn destroy(&self) {
        debug!("Destroying instance");
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
    pub fn get_vk(&self) -> &VkInstance {
        &self.instance
    }
}
