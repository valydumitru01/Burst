use anyhow::{anyhow, Context};
use std::collections::HashSet;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::vk::{EntryV1_0, StringArray};
use vulkanalia::{vk, Version};
use vulkanalia::{Entry as VkEntry, Instance, VkResult};

/// Vulkan extensions are optional API features that are *not* part of the Vulkan Core
/// but can be used if supported by the platform, driver or device.
/// I.e. They are plug-ins that allow extra capabilities beyond the Vulkan version it is used.
///
/// Two types:
/// - Instance extensions: Extra features for the instance (chosen before its creation).
/// E.g. Platform integration or debug utils.
/// - Device extensions: Extra features for the physical device (actual GPU, chosen before creating
/// the device object).
/// E.g. Ray tracing or mesh shaders.
///
/// Extensions are stored as Strings of max size 258.
/// # Examples
/// Instance examples:
/// - `VK_KHR_surface`: Required to create rendering surfaces (cross-platform base)
/// - `VK_KHR_xcb_surface`: Linux/XCB platform surface
/// - `VK_KHR_win32_surface`: Windows-specific surface
/// - `VK_EXT_debug_utils`: Debug logging, names, markers
/// - `VK_KHR_get_physical_device_properties2`:
/// Query more GPU info
/// Device examples:
/// - `VK_KHR_swapchain`: Required to use the swapchain (must have for presenting images)
/// - `VK_KHR_timeline_semaphore`: Synchronization primitive with timeline semantics
/// - `VK_EXT_descriptor_indexing`: Advanced descriptor sets (bindless, variable count)
/// - `VK_KHR_ray_tracing_pipeline`: Full ray tracing support
/// - `VK_KHR_acceleration_structure`: Acceleration structure creation
/// - `VK_KHR_shader_draw_parameters`: Pass draw ID/push constants to shaders
type Extension = StringArray<256>;

/// # Vulkan Entry
/// A Vulkan Entry is the entry point for Vulkan.
/// Basically is the object that dynamically loads the Vulkan API.
///
/// But in more detail:
/// At the point of creating the entry, Vulkan has not been dispatched. What VkEntry does is find
/// the loader in the system (i.e. `vulkan-1.dll` and `libvulkan.so.1`) and get the global symbols
/// from the loader (these symbols are just the functions for VkEntry to work, i.e. `vkCreateInstance`)
///
/// This way, VkEntry becomes a bootstrapped API for the Vulkan Driver. Which means the minimum set
/// of functions that allow for loading Vulkan.
///
/// Entry can exist without any Vulkan loading, as it is not part of it.
#[derive(Debug, Clone)]
pub(crate) struct Entry {
    entry: VkEntry,
}
impl Entry {
    /// # Vulkan Entry Loading
    /// See [`Entry`]
    /// Two steps in order:
    /// 1. Searches the Loader inside the operating system
    /// 2. Bootstraps the functions from the loader to allow Entry to call them.
    pub(in crate::gapi) fn new() -> anyhow::Result<Self> {
        // Finds the dynamic library (e.g. `.so` or `.dll`)
        let loader = unsafe {
            LibloadingLoader::new(LIBRARY)
                .with_context(|| format!("Failed to load Vulkan library: {}", LIBRARY))?
        };
        // Dynamically dispatches the Vulkan functions
        let entry = unsafe {
            vulkanalia::Entry::new(loader)
                .map_err(|b| anyhow!("Failed to load Vulkan entry: {}", b))?
        };
        Ok(Self { entry })
    }

    /// The version it returns is the (maximum) version the loader supports.
    /// It is not the version of the loader nor the version of the Vulkan API
    pub(in crate::gapi) fn version(&self) -> anyhow::Result<Version> {
        Ok(self.entry.version()?)
    }

    /// Getter for the VkEntry
    pub(in crate::gapi) fn get(&self) -> &vulkanalia::Entry {
        &self.entry
    }

    /// Query for the available layers
    pub(in crate::gapi) fn get_available_layers(&self) -> anyhow::Result<HashSet<Extension>> {
        let available_layers: HashSet<Extension> =
            unsafe { self.entry.enumerate_instance_layer_properties() }?
                .iter()
                .map(|l| l.layer_name)
                .collect::<HashSet<_>>();
        Ok(available_layers)
    }

    /// Queries all the available extensions (features) it supports.
    /// Useful to make checks before creating a Vulkan instance with selected extensions.
    pub(in crate::gapi) fn get_available_extensions(&self) -> anyhow::Result<HashSet<Extension>> {
        let available_extensions =
            unsafe { self.entry.enumerate_instance_extension_properties(None) }?
                .iter()
                .map(|e| e.extension_name)
                .collect::<HashSet<_>>();
        Ok(available_extensions)
    }

    /// Creates a Vulkan Instance!
    /// See [`Instance`]
    /// The VkInstance calls the Loader now that VkEntry has loaded it.
    ///
    /// # Parameters
    /// - `info`, composed of the following:
    ///     - `application_info`: Metadata about the app (name, version, Vulkan version)
    ///     - `enabled_layer_names`: List of validation or debug layers to enable.
    ///     - `enabled_extension_names`: List of instance extensions to use (e.g., `VK_KHR_surface`, `VK_EXT_debug_utils`).
    /// - `allocation_callbacks`: Custom allocator hook, to override default memory management.
    ///
    /// # Examples
    /// Basic usage with default allocator:
    /// ```rust
    /// let instance = entry.create_instance(&create_info, None)?;
    /// ```
    /// Usage with custom allocator:
    /// ```rust
    /// let callbacks = vk::AllocationCallbacks {
    ///     p_user_data: std::ptr::null_mut(),
    ///     pfn_allocation: Some(my_alloc),
    ///     pfn_reallocation: Some(my_realloc),
    ///     pfn_free: Some(my_free),
    ///     pfn_internal_allocation: None,
    ///     pfn_internal_free: None,
    /// };
    ///
    /// let instance = entry.create_instance(&create_info, Some(&callbacks))?;
    pub(in crate::gapi) fn create_instance(
        &self,
        info: &vk::InstanceCreateInfo,
        allocation_callbacks: Option<&vk::AllocationCallbacks>,
    ) -> VkResult<Instance> {
        unsafe { self.entry.create_instance(info, allocation_callbacks) }
    }
}
