use std::ffi::c_char;
use crate::gapi::vulkan::config::{API_DUMP_ENABLED, VALIDATION_ENABLED};
use crate::gapi::vulkan::debug::Debugger;
use crate::gapi::vulkan::entry::Entry;
use crate::gapi::vulkan::real_device::RealDevice;
use crate::{debug_success, info_success, trace_success};
use anyhow::anyhow;
use log::{debug, info, trace, warn};
use vulkanalia::vk::{HasBuilder, InstanceV1_0};
use vulkanalia::{vk, Instance as VkInstance, Version, VkResult};
use crate::gapi::vulkan::enums::extensions::{InstanceExtension, PORTABILITY_MACOS_VERSION};
use crate::gapi::vulkan::enums::layers::InstanceLayer;
use crate::window::MyWindow;

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
#[derive(Clone, Debug)]
pub(crate) struct Instance {
    instance: VkInstance,
}

impl Instance {
    /// # Instance Creation
    ///
    /// [`Entry`] is in charge of creating the Vulkan [Instance];
    /// this constructor handles the [flags](vk::Flags), [extensions](InstanceExtension),
    /// [layers](Instance), and [info](vk::ApplicationInfo) needed to create the instance.
    ///
    /// The configuration of the Instance is abstracted away inside the [`Instance`] class.
    ///
    /// # Details
    /// - First the constructor gathers the configuration data (flags, extensions, etc.) defined
    /// within the `Instance` class.
    /// - Then, if validation is enabled, we add a validation layer.
    ///
    /// # Errors
    ///
    /// Returns error if the machine is Mac and the Vulkan version that the machine has does not
    /// support portability to macOS.
    ///
    pub fn new(entry: &Entry, window: &MyWindow) -> anyhow::Result<Self> {

        debug!("Checking if system is compatible with Vulkan...");
        Self::check_compatibility(entry)?;
        info_success!("System is compatible with Vulkan!");

        debug!("Getting configured instance extensions...");
        let extensions = Self::get_required_extensions(window);
        let extension_names: Vec<*const c_char> = extensions
            .iter()
            .map(|ext| ext.name_ptr())
            .collect::<Vec<_>>();
        info!("Requested extensions: \n\t{:?}", extensions);

        debug!("Checking if extensions are available...");
        entry.check_instance_extensions_available(&extensions)?;
        info_success!("Requested Instance extensions are available!");

        debug!("Getting configured instance layers...");
        let layers = Self::get_required_layers();
        let layer_names: Vec<*const c_char> = layers
            .iter()
            .map(|layer| layer.name_ptr())
            .collect::<Vec<_>>();
        debug_success!("Requested layers: \n\t{:?}", layers);
        debug!(
            "All Available layers: \n\t{:?}",
            entry.get_available_layers()?
        );

        debug!("Checking if layers are available...");
        entry.check_layers_are_available(&layers)?;
        info_success!("Requested Instance layers are available!");

        debug!("Checking if requested extensions support the requested layers...");
        entry.check_layers_supported_by_extensions(&layers)?;
        info_success!("Requested Instance layers are supported by the requested extensions!");

        trace!("Building application info");
        let application_info = vk::ApplicationInfo::builder()
            .application_name(b"Burst\0")
            .application_version(vk::make_version(1, 0, 0))
            .engine_name(b"BurstG\0")
            .engine_version(vk::make_version(1, 0, 0))
            .api_version(vk::make_version(1, 2, 0))
            .build();
        trace_success!("Application info built!: \n\t{:?}", application_info);

        debug!("Getting flags to configure Instance...");
        let flags = Self::get_flags();
        info!("Flags to configure: \n\t{:?}", flags);

        trace!("Building InstanceCreateInfo...");
        let mut info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_layer_names(&layer_names)
            .enabled_extension_names(extension_names.as_slice())
            .flags(flags);
        trace_success!("InstanceCreateInfo built!: \n\t{:?}", info);

        // Add debug messages for creation and destruction of the Vulkan instance.
        if VALIDATION_ENABLED {
            debug!("{}", "Adding lifetime messenger to Instance.");
            Debugger::add_instance_lifetime_messenger(&mut info);
            debug_success!("Lifetime messenger added to Instance!");
        }
        trace!("Creating vulkan instance...");
        let instance = entry.create_instance(&info, None)?;
        info_success!("Vulkan Instance created!");

        Ok(Self { instance })
    }

    /// Checks if the system is compatible with Vulkan.
    /// All the data necessary before checking is stored inside [`Entry`]
    ///
    /// # Parameters
    /// - `entry`: The Vulkan [`Entry`] that contains the required information to check
    /// compatibility.
    /// # Returns
    /// - An error if the system is not compatible with Vulkan, containing the reason why.
    fn check_compatibility(entry: &Entry) -> anyhow::Result<()> {
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

    fn config_required_extensions(window: &MyWindow) -> Vec<InstanceExtension> {
        let mut required_exts: Vec<InstanceExtension> = window
            .get_required_extensions()
            .iter()
            .map(|ext| InstanceExtension::from_name(*ext))
            .collect::<Vec<_>>();
        if VALIDATION_ENABLED || API_DUMP_ENABLED {
            required_exts.push(InstanceExtension::ExtDebugUtils);
        }
        if cfg!(target_os = "macos") {
            required_exts.push(InstanceExtension::KhrGetPhysicalDeviceProperties2);
            required_exts.push(InstanceExtension::KhrPortabilityEnumeration);
        }
        required_exts
    }

    fn config_required_layers() -> Vec<InstanceLayer> {
        let mut layers: Vec<InstanceLayer> = vec![];
        if VALIDATION_ENABLED && API_DUMP_ENABLED {
            layers.push(InstanceLayer::ApiDump);
        }
        if VALIDATION_ENABLED {
            layers.push(InstanceLayer::Validation);
        }
        layers
    }

    /// Collects and returns the required extensions for the Vulkan instance.
    ///
    /// # Parameters
    /// - `window`: The window handler ([`MyWindow`]) that knows its required extensions.
    ///
    /// # Returns
    /// - A vector of [`ExtensionStr`] that contains the required extensions for the Vulkan instance.
    fn get_required_extensions(window: &MyWindow) -> Vec<InstanceExtension> {
        let extensions = Self::config_required_extensions(window);
        info!("Required Extension: {:?}", extensions);
        extensions
    }

    /// Collects and returns the required layers for the Vulkan instance.
    /// # Returns
    /// A list of all the [layers](Instance) required by [`Instance`]
    fn get_required_layers() -> Vec<InstanceLayer> {
        let layers = Self::config_required_layers();
        info!("Required Layers: {:?}", layers);
        layers
    }

    /// Configures the flags for [`Instance`]
    /// # Returns
    /// All the flags that will be passed to the [`Instance`] constructor.
    fn get_flags() -> vk::InstanceCreateFlags {
        if cfg!(target_os = "macos") {
            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::empty()
        }
    }

    pub fn enumerate_real_devices(&self) -> VkResult<Vec<RealDevice>> {
        trace!("Querying all physical devices...");
        unsafe {
            let physical_devices = self
                .instance
                .enumerate_physical_devices()?
                .into_iter()
                .map(|device| RealDevice::new(self, device))
                .collect::<Vec<_>>();
            trace_success!("Physical devices found: \n\t{:?}", physical_devices);
            Ok(physical_devices)
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
