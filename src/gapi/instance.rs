use crate::gapi::debug::Debugger;
use crate::gapi::entry::Entry;
use crate::gapi::physical_device::PhysicalDevice;
use crate::gapi::vulkan::{SuitabilityError, VALIDATION_ENABLED};
use crate::window::window::MyWindow;
use anyhow::anyhow;
use log::{info, warn};
use std::ffi::c_char;
use vulkanalia::vk::{ExtensionName, HasBuilder, InstanceV1_0};
use vulkanalia::vk::{PhysicalDevice as VkPhysicalDevice, StringArray};

use vulkanalia::{vk, Instance as VkInstance, Version};

/// # Vulkan Extensions
///
/// Vulkan extensions are optional, feature-specific additions to the core Vulkan API.
/// They are not part of the core specification but can be enabled if supported by the Vulkan loader,
/// instance layers, or physical device drivers.
///
/// # Details
/// Extensions expose new capabilities—such as ray tracing, debug utilities, or surface creation—
/// that are not guaranteed to be available across all platforms or GPUs.
///
/// Extensions must be explicitly **queried for availability** and **enabled during creation** of the
/// Vulkan instance or logical device, depending on the extension type.
///
/// ## Two Types:
/// - **Instance extensions**: Extend the Vulkan instance.
/// Must be enabled during `vkCreateInstance`.
///   These typically deal with window system integration (WSI) or debugging features.
///   E.g., platform surfaces (`VK_KHR_win32_surface`) or tools (`VK_EXT_debug_utils`).
///
/// - **Device extensions**: Extend the logical device (GPU-side).
/// Must be enabled during `vkCreateDevice`.
///   These provide advanced GPU functionality like ray tracing, mesh shading, or timeline semaphores.
///
/// # Examples
/// ## Instance Extensions:
/// - `VK_KHR_surface`: Cross-platform WSI base (required for rendering to surfaces)
/// - `VK_KHR_xcb_surface`: XCB-based Linux window integration
/// - `VK_KHR_win32_surface`: Windows window integration
/// - `VK_EXT_debug_utils`: Debug names, markers, message callback hooks
/// - `VK_KHR_get_physical_device_properties2`: Extended device property querying
///
/// ## Device Extensions:
/// - `VK_KHR_swapchain`: Required for presenting rendered images to surfaces
/// - `VK_KHR_timeline_semaphore`: Sync primitive with host/device timeline support
/// - `VK_EXT_descriptor_indexing`: Bindless descriptors, variable counts
/// - `VK_KHR_ray_tracing_pipeline`: Ray tracing shader pipeline support
/// - `VK_KHR_acceleration_structure`: GPU-accelerated BVH structures
/// - `VK_KHR_shader_draw_parameters`: Shader access to draw parameters (gl_DrawID, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Extension {
    KhrSurface,
    KhrGetDebugUtils,
    KhrGetPhysicalDeviceProperties2,
    KhrPortabilityEnumeration,
}

impl Extension {
    pub fn name(&self) -> &'static vk::ExtensionName {
        match self {
            Extension::KhrSurface => &vk::KHR_SURFACE_EXTENSION.name,
            Extension::KhrGetDebugUtils => &vk::EXT_DEBUG_UTILS_EXTENSION.name,
            Extension::KhrGetPhysicalDeviceProperties2 => {
                &vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION.name
            }
            Extension::KhrPortabilityEnumeration => &vk::KHR_PORTABILITY_ENUMERATION_EXTENSION.name,
        }
    }
}

/// # Vulkan Layers
///
/// Layers are optional components that augment the Vulkan system.
/// They can intercept, evaluate, and modify Vulkan functions, attaching behavior to the normal Vulkan API.
///
/// # Details
/// Layers are implemented as libraries that are installed on the system and enabled or disabled
/// during Vulkan SDK initialization or at runtime, during instance creation.
///
/// A layer can choose to intercept Vulkan calls and modify their behavior.
/// Not all Vulkan functions need to be intercepted by a layer, it could intercept only a subset or
/// just a single one.
///
/// Because layers are optional, you can choose to enable some layers for debugging and disable them
/// to release.
///
/// # Examples
///
/// - `VK_LAYER_KHRONOS_validation`   
/// Validation layer provided by Khronos.
/// It checks for correct API usage, detects common errors, and helps in debugging.
/// - `VK_LAYER_LUNARG_api_dump`   
/// Logs all Vulkan API calls along with their parameters to the standard output.
/// Useful for tracing and debugging Vulkan function calls.
/// - `VK_LAYER_RENDERDOC_Capture`   
/// Integrates with RenderDoc to capture and analyze frames.
/// Useful for debugging and performance profiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(in crate::gapi) enum Layer {
    Validation,
    ApiDump,
    RenderDoc,
}

impl Layer {
    pub const VALIDATION_LAYER: &'static str = "VK_LAYER_KHRONOS_validation";
    pub const VALIDATION_LAYER_NAME: &'static str = "VK_LAYER_LUNARG_api_dump";
    pub const VALIDATION_LAYER_RENDERDOC: &'static str = "VK_LAYER_RENDERDOC_Capture";
    pub(in crate::gapi) fn as_cstr(&self) -> *const c_char {
        match self {
            Self::Validation => {
                ExtensionName::from_bytes(Self::VALIDATION_LAYER.as_bytes()).as_ptr()
            }
            Self::ApiDump => {
                ExtensionName::from_bytes(Self::VALIDATION_LAYER_NAME.as_bytes()).as_ptr()
            }
            Self::RenderDoc => {
                ExtensionName::from_bytes(Self::VALIDATION_LAYER_RENDERDOC.as_bytes()).as_ptr()
            }
        }
    }

    pub(in crate::gapi) fn get_all_c_chars() -> Vec<*const c_char> {
        vec![Self::Validation.as_cstr()]
    }

    pub(in crate::gapi) fn get_all_names() -> Vec<String> {
        vec![
            Self::VALIDATION_LAYER.to_string(),
            Self::VALIDATION_LAYER_NAME.to_string(),
            Self::VALIDATION_LAYER_RENDERDOC.to_string(),
        ]
    }
}

pub(crate) const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);

/// # Vulkan Instance
/// The Vulkan instance is the connection between this program and the Vulkan driver.
/// Acts as the "context" for the entire Vulkan ecosystem.
///
///
#[derive(Clone, Debug)]
pub(crate) struct Instance {
    instance: VkInstance,
}

impl Instance {
    /// # Instance Creation
    /// See [`Instance`]
    ///
    /// Entry is in charge of creating the Vulkan Instance; this call is in charge of providing the
    /// custom configuration and checking if the machine where the program is run is compatible.
    ///
    /// # Vulkan Instance Loading
    ///
    /// - First validates the requested layers and then loads them. Validation layers get wrapped around
    /// the driver calls.
    /// -
    ///
    /// # Errors
    ///
    /// Returns error if the machine is Mac and the Vulkan version that the machine has does not
    /// support portability to macOS.
    ///
    pub fn new(entry: &Entry, window: &MyWindow) -> anyhow::Result<Self> {
        let entry_version = entry.version()?;
        // Required by Vulkan SDK on macOS since 1.3.216.
        if cfg!(target_os = "macos") && entry_version < PORTABILITY_MACOS_VERSION {
            return Err(anyhow!(
                "MacOS portability requires Vulkan {}",
                PORTABILITY_MACOS_VERSION
            ));
        }

        let flags = Self::get_flags();
        let extensions = Self::get_extensions(window);
        let layers = Layer::get_all_c_chars();
        let application_info = Self::make_application_info();

        let mut info = Self::make_instance_info(&application_info, &layers, &extensions, flags);

        // Add debug messages for creation and destruction of the Vulkan instance.
        if VALIDATION_ENABLED {
            log::debug!("Enabling Validation Layer.");
            Debugger::add_instance_life_debug(&mut info);
        }
        log::debug!("Creating instance...");
        let instance = entry.create_instance(&info, None)?;

        Ok(Self { instance })
    }

    fn make_instance_info<'a>(
        application_info: &'a vk::ApplicationInfo,
        layers: &'a Vec<*const i8>,
        extensions: &'a Vec<*const i8>,
        flags: vk::InstanceCreateFlags,
    ) -> vk::InstanceCreateInfoBuilder<'a> {
        vk::InstanceCreateInfo::builder()
            .application_info(application_info)
            .enabled_layer_names(layers)
            .enabled_extension_names(extensions)
            .flags(flags)
    }
    fn make_application_info() -> vk::ApplicationInfo {
        vk::ApplicationInfo::builder()
            .application_name(b"Burst\0")
            .application_version(vk::make_version(1, 0, 0))
            .engine_name(b"No Engine\0")
            .engine_version(vk::make_version(1, 0, 0))
            .api_version(vk::make_version(1, 0, 0))
            .build()
    }

    /// Collects all required device-level extensions in a simple vector of C-strings.
    ///
    /// Includes platform-specific or validation-specific extensions, if necessary.
    /// Instance extensions do not depend on the GPU, they just tell Vulkan how to interact with the
    /// system.
    fn get_extensions(window: &MyWindow) -> Vec<*const i8> {
        let mut extensions = window
            .get_required_extensions()
            .iter()
            .map(|e| e.as_ptr())
            .collect::<Vec<_>>();
        if VALIDATION_ENABLED {
            extensions.push(Extension::KhrGetDebugUtils.name().as_ptr());
        }
        if cfg!(target_os = "macos") {
            // Allow Query extended physical device properties
            extensions.push(Extension::KhrGetPhysicalDeviceProperties2.name().as_ptr());
            //  Enable macOS support for the physical device
            extensions.push(Extension::KhrPortabilityEnumeration.name().as_ptr());
        }
        extensions
    }
    ///
    fn check_layers(entry: Entry) -> anyhow::Result<()> {
        let available_layers = entry.get_available_layers()?;
        let layers = Layer::get_all_names();
        available_layers
            .iter()
            .find(|layer| layers.contains(&layer.to_string()))
            .ok_or_else(|| anyhow!("Missing required layer."))
            .map(|_| ())?;
        Ok(())
    }

    fn get_flags() -> vk::InstanceCreateFlags {
        if cfg!(target_os = "macos") {
            vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
        } else {
            vk::InstanceCreateFlags::empty()
        }
    }

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
    fn check_physical_device(&self, physical_device: VkPhysicalDevice) -> anyhow::Result<()> {
        log::debug!("Checking physical device suitability.");
        // Basic properties like name, type, and supported Vulkan version.
        let properties = unsafe {
            self.instance
                .get_physical_device_properties(physical_device)
        };
        // We only want to use discrete (dedicated) GPUs.
        if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
            return Err(anyhow!(SuitabilityError(
                "Only discrete GPUs are supported."
            )));
        }

        // Optional features like texture compression, 64-bit floats, and multi-viewport rendering.
        let features = unsafe { self.instance.get_physical_device_features(physical_device) };
        // We require support for geometry shaders.
        if features.geometry_shader != vk::TRUE {
            return Err(anyhow!(SuitabilityError(
                "Missing geometry shader support."
            )));
        }

        Ok(())
    }

    pub(in crate::gapi) fn pick_physical_device(&self) -> anyhow::Result<PhysicalDevice> {
        unsafe {
            for vk_physical_device in self.instance.enumerate_physical_devices()? {
                let properties = self
                    .instance
                    .get_physical_device_properties(vk_physical_device);

                if let Err(error) = self.check_physical_device(vk_physical_device) {
                    warn!(
                        "Skipping physical device (`{}`): {}",
                        properties.device_name, error
                    );
                } else {
                    info!("Selected physical device (`{}`).", properties.device_name);
                    return anyhow::Ok(PhysicalDevice::new(vk_physical_device));
                }
            }
        }

        Err(anyhow!("Failed to find suitable physical device."))
    }

    pub fn destroy(&self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
    pub fn get(&self) -> &VkInstance {
        &self.instance
    }
}
