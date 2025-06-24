use crate::enum_impl;
use std::fmt;
use vulkanalia::vk::ExtensionName;
use vulkanalia::{vk, Version};

/// Type alias for the extension names.
/// Vulkan provides a type for Extension ([`vk::ExtensionName`]) that is defined as
/// `StringArray<MAX_EXTENSION_NAME_SIZE>`
pub(crate) type ExtensionStr = vk::ExtensionName;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstanceExtension {}
enum_impl! {
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
    /// - `VK_KHR_get_real_device_properties2`: Extended device property querying
    ///
    /// ## Device Extensions:
    /// - `VK_KHR_swapchain`: Required for presenting rendered images to surfaces
    /// - `VK_KHR_timeline_semaphore`: Sync primitive with host/device timeline support
    /// - `VK_EXT_descriptor_indexing`: Bindless descriptors, variable counts
    /// - `VK_KHR_ray_tracing_pipeline`: Ray tracing shader pipeline support
    /// - `VK_KHR_acceleration_structure`: GPU-accelerated BVH structures
    /// - `VK_KHR_shader_draw_parameters`: Shader access to draw parameters (gl_DrawID, etc.)
    pub enum InstanceExtension {
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
        ExtDebugUtils = vk::EXT_DEBUG_UTILS_EXTENSION.name,
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
        KhrSurface = vk::KHR_SURFACE_EXTENSION.name,
        /// # VK_KHR_get_real_device_properties2
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
        KhrGetPhysicalDeviceProperties2 = vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION.name,
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
        KhrPortabilityEnumeration = vk::KHR_PORTABILITY_ENUMERATION_EXTENSION.name,

        /// # VK_KHR_portability_subset
        /// TODO
        KhrWin32Surface = vk::KHR_WIN32_SURFACE_EXTENSION.name,

        /// # VK_KHR_device_group_creation
        /// TODO
        KhrDeviceGroupCreation = vk::KHR_DEVICE_GROUP_CREATION_EXTENSION.name,

        /// # VK_KHR_external_fence_capabilities
        /// TODO
        KhrExternalFenceCapabilities = vk::KHR_EXTERNAL_FENCE_CAPABILITIES_EXTENSION.name,

        /// # VK_KHR_external_memory_capabilities
        /// TODO
        KhrExternalMemoryCapabilities = vk::KHR_EXTERNAL_MEMORY_CAPABILITIES_EXTENSION.name,

        /// # VK_KHR_external_semaphore_capabilities
        /// TODO
        KhrExternalSemaphoreCapabilities = vk::KHR_EXTERNAL_SEMAPHORE_CAPABILITIES_EXTENSION.name,

        /// # VK_KHR_get_surface_capabilities2
        /// TODO
        KhrGetSurfaceCapabilities2 = vk::KHR_GET_SURFACE_CAPABILITIES2_EXTENSION.name,

        /// # VK_EXT_debug_report
        /// TODO
        ExtDebugReport = vk::EXT_DEBUG_REPORT_EXTENSION.name,

        /// # VK_EXT_swapchain_colorspace
        /// TODO
        ExtSwapchainColorspace = vk::EXT_SWAPCHAIN_COLORSPACE_EXTENSION.name,
    }
}
pub(crate) const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);

/// # Vulkan Device Extensions
///
/// Device extensions augment a *logical device* with functionality that may
/// not be present in the core specification (or that was added in later core
/// versions). They must be **queried** and **enabled** during
/// [`vkCreateDevice`].
///
/// ## Examples of capabilities exposed through device extensions
/// * Presenting rendered images to a surface (swapchains)
/// * Host/device‑timestep synchronisation (timeline semaphores)
/// * Ray‑tracing pipelines and acceleration structures
/// * Bindless resource binding (descriptor indexing)
/// * Portability‑subset support for Metal‑backed drivers (MoltenVK)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceExtensions {
    /// # VK_KHR_swapchain
    /// *Required* for presenting images to a window surface.
    ///
    /// ## Details
    /// 1. Introduces the opaque handle [`vk::SwapchainKHR`].
    /// 2. Adds functions such as [`vk::CreateSwapchainKHR`], [`vk::AcquireNextImageKHR`],
    ///    and [`vk::QueuePresentKHR`].
    /// 3. Enables application‑controlled presentation modes (FIFO, MAILBOX, etc.).
    KhrSwapchain,

    /// # VK_KHR_timeline_semaphore
    /// Provides monotonically‑increasing *timeline* semaphores for fine‑grained
    /// GPU⇌CPU synchronisation.
    ///
    /// ## Details
    /// 1. Adds the struct [`vk::SemaphoreTypeCreateInfo`].
    /// 2. Allows `vkWaitSemaphores` / `vkSignalSemaphore` to operate on 64‑bit values.
    /// 3. Promoted to core in Vulkan 1.2.
    KhrTimelineSemaphore,

    /// # VK_EXT_descriptor_indexing
    /// Enables **bindless** and **variable‑descriptor‑count** resource binding.
    ///
    /// ## Details
    /// 1. Adds the `VK_DESCRIPTOR_BINDING_*` flags for descriptor set layouts.
    /// 2. Allows non‑uniform indexing and partial‑binding of descriptor arrays.
    /// 3. Foundation for modern rendering techniques such as bindless textures.
    ExtDescriptorIndexing,

    /// # VK_KHR_ray_tracing_pipeline
    /// Provides programmable **ray‑tracing shader stages** and dispatch.
    ///
    /// ## Details
    /// 1. Introduces shader stages `raygen`, `closest‑hit`, `any‑hit`, `miss`,
    ///    `intersection`, and `callable`.
    /// 2. Adds SPIR‑V instructions and `VkRayTracingPipelineCreateInfoKHR`.
    /// 3. Requires [`DeviceExtensions::KhrAccelerationStructure`].
    KhrRayTracingPipeline,

    /// # VK_KHR_acceleration_structure
    /// GPU‑accelerated bottom‑level (BLAS) and top‑level (TLAS) **BVH** data structures.
    ///
    /// ## Details
    /// 1. Introduces [`vk::AccelerationStructureKHR`].
    /// 2. Functions to build, compact, and query memory for acceleration structures.
    /// 3. Foundation for the ray‑tracing pipeline extension.
    KhrAccelerationStructure,

    /// # VK_KHR_shader_draw_parameters
    /// Exposes DrawID / BaseVertex / BaseInstance directly in shaders without
    /// requiring vertex attributes.
    KhrShaderDrawParameters,

    /// # VK_KHR_portability_subset
    /// Marks the device as implementing only a *subset* of Vulkan functionality
    /// via translation layers such as MoltenVK.
    ///
    /// ## Details
    /// 1. Applications must handle the indicated feature limitations.
    /// 2. Required when targeting portability devices enumerated with
    ///    [`InstanceExtension::KhrPortabilityEnumeration`].
    KhrPortabilitySubset,
}
