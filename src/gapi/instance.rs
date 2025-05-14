use crate::gapi::debug::Debugger;
use crate::gapi::entry::{Entry, ExtensionStr, LayerStr};
use crate::gapi::vulkan::{SuitabilityError, VALIDATION_ENABLED};
use crate::window::window::MyWindow;
use anyhow::anyhow;
use log::{debug, info, trace};
use std::collections::HashSet;
use std::ffi::c_char;
use vulkanalia::vk::{HasBuilder, InstanceV1_0};

use crate::gapi::physical_device::PhysicalDevice;
use vulkanalia::vk::video::__BindgenBitfieldUnit;
use vulkanalia::{vk, Instance as VkInstance, Version, VkResult};

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
    /// # VK_EXT_debug_utils
    /// Structured debugging utilities for tooling and validation.
    ///
    /// # Details
    /// This extension adds:
    /// 1. A new struct, [`VkDebugUtilsMessengerEXT`](vk::DebugUtilsMessengerEXT)
    /// 2. Functions to attach metadata to Vulkan objects:
    ///     - [`vkSetDebugUtilsObjectNameEXT`](vk::PFN_vkSetDebugUtilsObjectNameEXT):  
    ///         Assigns a readable name to a Vulkan object
    ///     - [`vkSetDebugUtilsObjectTagEXT`](vk::PFN_vkSetDebugUtilsObjectTagEXT):  
    ///         Attaches an opaque tag to a Vulkan object
    /// 3. Functions to annotate queues and command buffers:
    ///     - [`vkQueueBeginDebugUtilsLabelEXT`](vk::PFN_vkQueueBeginDebugUtilsLabelEXT)  
    ///     - [`vkQueueEndDebugUtilsLabelEXT`](vk::PFN_vkQueueEndDebugUtilsLabelEXT)  
    ///     - [`vkQueueInsertDebugUtilsLabelEXT`](vk::PFN_vkQueueInsertDebugUtilsLabelEXT)  
    ///     - [`vkCmdBeginDebugUtilsLabelEXT`](vk::PFN_vkCmdBeginDebugUtilsLabelEXT)  
    ///     - [`vkCmdEndDebugUtilsLabelEXT`](vk::PFN_vkCmdEndDebugUtilsLabelEXT)  
    ///     - [`vkCmdInsertDebugUtilsLabelEXT`](vk::PFN_vkCmdInsertDebugUtilsLabelEXT)
    /// 4. Debug-messenger control:
    ///     - [`vkCreateDebugUtilsMessengerEXT`](vk::PFN_vkCreateDebugUtilsMessengerEXT):  
    ///         Registers a callback for validation messages
    ///     - [`vkDestroyDebugUtilsMessengerEXT`](vk::PFN_vkDestroyDebugUtilsMessengerEXT)  
    ///     - [`vkSubmitDebugUtilsMessageEXT`](vk::PFN_vkSubmitDebugUtilsMessageEXT)
    /// Replaces the older `VK_EXT_debug_report` and `VK_EXT_debug_marker` extensions.
    ExtDebugUtils,

    /// # VK_KHR_surface
    /// Core extension to allow Vulkan to interface with windowing systems.
    ///
    /// # Details
    /// This extension adds:
    /// 1. A new struct, [`VkSurfaceKHR`](vk::SurfaceKHR)
    /// 2. New instance-level functions to manage the surface:
    ///     - [`vkDestroySurfaceKHR`](vk::PFN_vkDestroySurfaceKHR):   
    ///         Destroys the [surface](vk::SurfaceKHR)
    ///     - [`vkGetPhysicalDeviceSurfaceSupportKHR`](vk::PFN_vkGetPhysicalDeviceSurfaceSupportKHR):   
    ///         Check if a [physical device](vk::PhysicalDevice)'s [queue](vk::Queue) can present images to
    ///         the [surface](vk::SurfaceKHR).
    ///     - [`vkGetPhysicalDeviceSurfaceCapabilitiesKHR`](vk::PFN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR):   
    ///         Queries the [capabilities](vk::SurfaceCapabilitiesKHR) of the [surface](vk::SurfaceKHR).
    ///     - [`vkGetPhysicalDeviceSurfaceFormatsKHR`](vk::PFN_vkGetPhysicalDeviceSurfaceFormatsKHR):
    ///         Queries the [color formats](vk::SurfaceFormatKHR) supported by the [surface](vk::SurfaceKHR)
    ///     - [`vkGetPhysicalDeviceSurfacePresentModesKHR`](vk::PFN_vkGetPhysicalDeviceSurfacePresentModesKHR):
    ///         Queries the supported presentation modes supported by the [surface](vk::SurfaceFormatKHR).
    /// 3. Surface-specific extensions
    ///     - `VK_KHR_win32_surface`
    ///     - `VK_KHR_xcb_surface`
    ///     - `VK_KHR_wayland_surface`
    ///     - etc.
    /// 4. Swapchain device extension ([`VK_KHR_swapchain`](vk::KHR_SWAPCHAIN_EXTENSION)) is allowed, this extension
    /// depends on [`VK_KHR_surface`](vk::KHR_SURFACE_EXTENSION) extension.
    KhrSurface,
    /// # VK_KHR_get_physical_device_properties2
    /// Extended querying for physical-device features and properties.
    ///
    /// # Details
    /// This extension adds:
    /// 1. Two root structs that accept extension chains:
    ///     - [`VkPhysicalDeviceFeatures2`](vk::PhysicalDeviceFeatures2)  
    ///     - [`VkPhysicalDeviceProperties2`](vk::PhysicalDeviceProperties2)
    /// 2. A family of query functions that take the new structs:
    ///     - [`vkGetPhysicalDeviceFeatures2KHR`](vk::PFN_vkGetPhysicalDeviceFeatures2KHR)  
    ///     - [`vkGetPhysicalDeviceProperties2KHR`](vk::PFN_vkGetPhysicalDeviceProperties2KHR)  
    ///     - [`vkGetPhysicalDeviceFormatProperties2KHR`](vk::PFN_vkGetPhysicalDeviceFormatProperties2KHR)  
    ///     - [`vkGetPhysicalDeviceImageFormatProperties2KHR`](vk::PFN_vkGetPhysicalDeviceImageFormatProperties2KHR)  
    ///     - [`vkGetPhysicalDeviceQueueFamilyProperties2KHR`](vk::PFN_vkGetPhysicalDeviceQueueFamilyProperties2KHR)  
    ///     - [`vkGetPhysicalDeviceMemoryProperties2KHR`](vk::PFN_vkGetPhysicalDeviceMemoryProperties2KHR)  
    ///     - [`vkGetPhysicalDeviceSparseImageFormatProperties2KHR`](vk::PFN_vkGetPhysicalDeviceSparseImageFormatProperties2KHR)  
    ///     - plus external-object capability queries
    /// 3. A required foundation for many later feature and property extensions
    /// Promoted to core in Vulkan 1.1; still needed when targeting `VK_API_VERSION_1_0`.
    KhrGetPhysicalDeviceProperties2,
    /// # VK_KHR_portability_enumeration
    /// Opt-in enumeration of portability-subset (non-conformant) devices.
    ///
    /// # Details
    /// This extension adds:
    /// 1. Instance-creation flag `VK_INSTANCE_CREATE_ENUMERATE_PORTABILITY_BIT_KHR` in [`VkInstanceCreateInfo`](vk::InstanceCreateInfo)  
    ///     Setting the flag makes `vkEnumeratePhysicalDevices` return devices that only implement `VK_KHR_portability_subset`
    /// 2. A change in enumeration rules: without the flag, portability devices are hidden to preserve strict conformance
    /// 3. Requirement that applications enabling the flag also enable [`VK_KHR_portability_subset`](vk::KHR_PORTABILITY_SUBSET_EXTENSION) at device-creation time
    /// Main use-case: run Vulkan applications on macOS or iOS via Metal-backed drivers such as MoltenVK.
    KhrPortabilityEnumeration,
}

impl Extension {
    /// Converts the enum to the Vulkan extension name string ([`vk::ExtensionName`])
    /// # Returns
    /// A [`vk::ExtensionName`] with the name of the extension.
    pub fn name(&self) -> &'static vk::ExtensionName {
        match self {
            Extension::KhrSurface => &vk::KHR_SURFACE_EXTENSION.name,
            Extension::ExtDebugUtils => &vk::EXT_DEBUG_UTILS_EXTENSION.name,
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
    /// # `VK_LAYER_KHRONOS_validation`
    /// The **official, all-in-one validation layer** maintained by the Khronos Group.
    ///
    /// # What it adds
    /// 1. Full-spectrum API validation  
    ///    - Structural checks: correct object lifetimes, handle ownership, thread-safety.  
    ///    - Draw-time checks: descriptor-set compatibility, pipeline state, render-pass rules.  
    ///    - Synchronization validation (optionally via the _sync2_ sub-module).  
    ///    - Best-practice warnings for performance or portability issues.
    /// 2. **Message routing** through the standard debug-messenger callback
    ///    (`VK_EXT_debug_utils`).  Supports severity & type filtering, custom callbacks,
    ///    and message IDs.
    /// 3. **Configurable behaviour**  
    ///    - Runtime toggles via `VK_LAYER_SETTINGS_PATH`, `VK_INSTANCE_LAYERS`,
    ///      or JSON settings files.  
    ///    - Fine-grained disable lists (e.g. suppress known-benign message IDs).  
    ///    - GPU-assisted validation & shader-instrumentation modes for deeper analysis.
    /// 4. **Extended utilities** exposed as layer-specific extensions  
    ///    (`VK_EXT_validation_features`, `VK_EXT_validation_flags`,
    ///    `VK_EXT_tooling_info`).
    /// 5. **Safe-mode fallbacks** — if certain device features are unsupported
    ///    GPU-assisted checks gracefully downgrade to host simulation instead of failing.
    ///
    /// # Typical use
    /// Enable during development:
    /// ```text
    /// enabled_layer_names = ["VK_LAYER_KHRONOS_validation"]
    /// ```
    /// Pair with the `VK_EXT_debug_utils` extension to receive messages.
    Validation,

    /// # `VK_LAYER_LUNARG_api_dump`
    /// Human-readable trace layer that logs every Vulkan call (and its parameters)
    /// as it happens.
    ///
    /// # What it adds
    /// 1. **Automatic API call logging** to:
    ///    - **Stdout / stderr** (default)  
    ///    - A **named file** (`VK_APIDUMP_OUTPUT_FILE=<path>`)  
    ///    - The **debug-utils messenger** stream (if present)
    /// 2. **Complete parameter expansion**  
    ///    - Enumerations & flags printed by symbolic name.  
    ///    - Structs and arrays fully expanded.  
    ///    - Handles shown as hexadecimal for cross-reference with validation messages.
    /// 3. **Timing information** (optional) — per-call timestamps & thread IDs for
    ///    performance replay.
    /// 4. **JSON or C-style output** (`VK_APIDUMP_FORMAT={text|json}`) to feed
    ///    external tooling.
    /// 5. **Selective capture filters**  
    ///    - Environment variables to include / exclude function ranges  
    ///    - Live toggling via `VK_APIDUMP_ACTIVE=0|1`
    ///
    /// # Typical use
    /// Very lightweight; enable when you need a quick “what is the app really doing?”
    /// transcript or when crafting minimal repro cases for driver bugs.
    ApiDump,

    /// # `VK_LAYER_RENDERDOC_Capture`
    /// Integration layer that allows **RenderDoc** to intercept Vulkan work for
    /// frame-capture and analysis.
    ///
    /// # What it adds
    /// 1. **Hook points** for RenderDoc’s graphics debugger to:  
    ///    - Capture a frame at any time (hot-key / API trigger).  
    ///    - Serialize command buffers, resources, and pipeline state.  
    ///    - Re-inject captures for offline replay.  
    /// 2. **In-application overlay** (optional) showing frame-time, GPU/CPU stats,
    ///    and capture status.
    /// 3. **Remote-control API** (`RENDERDOC_API_1_6_0`) obtained via
    ///    `RENDERDOC_GetAPI` — lets the app trigger captures, set markers, or
    ///    inject custom thumbnails.
    /// 4. **Automatic swap-chain tracking** across WSI extensions, including
    ///    multi-GPU and VR-compositor setups.
    /// 5. **No-op pass-through** when RenderDoc is not running, adding negligible
    ///    overhead in release builds.
    ///
    /// # Typical use
    /// Ship **disabled** in production, enable only in developer/internal builds or
    /// when the RenderDoc loader detects an attached debugger.
    RenderDoc,
}

impl Layer {
    pub const VALIDATION: LayerStr = LayerStr::from_bytes("VK_LAYER_KHRONOS_validation".as_bytes());
    pub const API_DUMP: LayerStr = LayerStr::from_bytes("VK_LAYER_LUNARG_api_dump".as_bytes());
    pub const RENDERDOC: LayerStr = LayerStr::from_bytes("VK_LAYER_RENDERDOC_Capture".as_bytes());
    pub(in crate::gapi) fn as_str(&self) -> LayerStr {
        match self {
            Self::Validation => Self::VALIDATION,
            Self::ApiDump => Self::API_DUMP,
            Self::RenderDoc => Self::RENDERDOC,
        }
    }
}

pub(crate) const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);

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
    /// this constructor handles the [flags](vk::Flags), [extensions](Extension),
    /// [layers](Layer), and [info](vk::ApplicationInfo) needed to create the instance.
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
        let extensions = Self::get_extensions(window);
        let extension_ptrs = extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();
        let layers = Self::get_layers();
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
    fn get_extensions(window: &MyWindow) -> Vec<&ExtensionStr> {
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
            extensions.push(Extension::ExtDebugUtils.name());
            trace!("Configuring validation layers...");
            trace!("Required extension: {}", Extension::ExtDebugUtils.name());
        }
        // Add the required extensions for the Vulkan portability.
        if cfg!(target_os = "macos") {
            trace!("Configuring MacOS extensions...");
            // Allow Query extended physical device properties
            extensions.push(Extension::KhrGetPhysicalDeviceProperties2.name());
            trace!(
                "Required extension: {}",
                Extension::KhrGetPhysicalDeviceProperties2.name()
            );
            //  Enable macOS support for the physical device
            extensions.push(Extension::KhrPortabilityEnumeration.name());
            trace!(
                "Required extension: {}",
                Extension::KhrPortabilityEnumeration.name()
            );
        }
        extensions
    }

    /// Configures the [`Instance`] [layers](Layer)
    /// # Returns
    /// A list of all the [layers](Layer) required by [`Instance`]
    fn get_layers() -> Vec<LayerStr> {
        trace!("Configuring layers...");
        let mut layers: Vec<LayerStr> = vec![];
        layers.push(Layer::ApiDump.as_str());
        trace!("Required Layer: {}", Layer::ApiDump.as_str());
        layers.push(Layer::RenderDoc.as_str());
        trace!("Required Layer: {}", Layer::RenderDoc.as_str());
        if VALIDATION_ENABLED {
            layers.push(Layer::Validation.as_str());
            trace!("Required Layer: {}", Layer::Validation.as_str());
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

    fn has_required_queues(&self, physical_device: &PhysicalDevice) -> bool {
        // The required queues are:
        // - Graphics
        // - Present (got from the surface)
        let required_queues = [vk::QueueFlags::GRAPHICS];
        trace!("Checking if physical device has required queues...");
        let indices = self.get_queue_family_indices(physical_device);
        indices.is_ok()
    }

    /// Picks a physical device using a set of criteria and a scoring system, selects the best suited for the app.
    /// # Details
    /// Iterates over the physical devices and selects the ones that has the required queues
    /// and supports swapchain.
    /// Then, it scores the devices based on their properties and features,
    /// and selects the one with the highest score.
    /// # Returns
    /// - `Ok(vk::PhysicalDevice)` if a suitable physical device is found.
    /// It returns the physical device with the highest score.
    /// - `Err(anyhow::Error)` if no suitable physical device is found.
    pub fn pick_physical_device(&self) -> anyhow::Result<vk::PhysicalDevice> {
        let mut best_device = None;
        let mut best_score = 0;

        for pd in self.enumerate_vk_physical_devices()? {
            let props = self.get_vk_physical_device_properties(&pd);
            let feats = self.get_vk_physical_device_features(&pd);

            // 1. Hard requirements.
            if !self.has_required_queues(&pd) {
                continue;
            }
            if !self.device_supports_swapchain(&pd) {
                continue;
            }
            if feats.geometry_shader != vk::TRUE {
                continue;
            }
            if props.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
                continue;
            }

            // 2–3. Score.
            let score = rate_device(&props, &feats);
            if score > best_score {
                best_device = Some(pd);
                best_score = score;
            }
        }

        best_device.ok_or_else(|| anyhow!("No suitable GPU found"))
    }
    /// Lists all the physical devices (real GPUs) installed in the system.
    /// # Details
    /// This call is pipelined to the Loader, which calls the GDIs which then returns the list of GPUs it knows of.
    /// This list is defined within them at the start of the system.
    ///
    /// # Returns
    /// A list of [Physical Devices](PhysicalDevice)
    /// # Errors
    /// - VK_ERROR_OUT_OF_HOST_MEMORY
    /// The program ran out of stack memory.
    ///
    /// - VK_ERROR_OUT_OF_DEVICE_MEMORY
    /// The GPU ran out of VRAM
    ///
    /// - VK_ERROR_INITIALIZATION_FAILED
    ///
    fn enumerate_vk_physical_devices(&self) -> VkResult<Vec<vk::PhysicalDevice>> {
        trace!("Enumerating physical devices...");
        unsafe { self.instance.enumerate_physical_devices() }
    }

    /// Gets the physical device properties calling [`InstanceV1_0::get_physical_device_properties`].
    /// # Details
    /// This call is pipelined to the Loader, which calls the GDIs, which know the properties of each
    /// physical device and returns them.
    /// # Returns
    /// The properties of the physical device as a [`vk::PhysicalDeviceProperties`] struct.
    fn get_vk_physical_device_properties(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> vk::PhysicalDeviceProperties {
        trace!("Getting physical device properties...");
        unsafe {
            self.instance
                .get_physical_device_properties(*physical_device)
        }
    }

    /// Gets the physical device features calling [`InstanceV1_0::get_physical_device_features`].
    /// # Details
    /// This call is pipelined to the Loader, which calls the GDIs, which know the features of each
    /// physical device and returns them.
    /// # Returns
    /// The features of the physical device as a [`vk::PhysicalDeviceFeatures`] struct.
    fn get_vk_physical_device_features(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> vk::PhysicalDeviceFeatures {
        trace!("Getting physical device features...");
        unsafe { self.instance.get_physical_device_features(*physical_device) }
    }

    /// Destroys the Vulkan instance calling [`InstanceV1_0::destroy_instance`].
    /// # Details
    /// Before destroying the instance, the application is responsible for destroying all
    /// Vulkan objects created with the instance.
    /// If done right, the RAII nature of Rust will take care of this for us.
    pub fn destroy(&self) {
        debug!("Destroying instance");
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
    pub fn get(&self) -> &VkInstance {
        &self.instance
    }
}
