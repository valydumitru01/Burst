use crate::gapi::vulkan::extensions::ExtensionStr;
use crate::gapi::vulkan::layers::LayerStr;
use anyhow::{anyhow, Context};
use std::collections::HashSet;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::vk::EntryV1_0;
use vulkanalia::{vk, Version};
use vulkanalia::{Entry as VkEntry, Instance, VkResult};

/// # Vulkan Entry
/// A Vulkan Entry is the entry point for Vulkan.
/// It is the object that dynamically loads the Vulkan API.
///
/// # Details
/// At the point of creating the `Entry`, the ICDs have not been loaded yet (the ICDs are Vulkan
/// front-ends for the GPU driver).
/// Therefore, Vulkan symbols are not loaded yet either.
///
/// What `Entry` does is find the loader in the system (i.e. `vulkan-1.dll` and `libvulkan.so.1`)
/// and get the global symbols from the loader
/// (these symbols are just the functions for VkEntry to work, i.e. `vkCreateInstance`).
///
/// In other words, `Entry` is a bootstrapper for the Vulkan ICDs.
#[derive(Debug, Clone)]
pub(crate) struct Entry {
    entry: VkEntry,
}

impl Entry {
    /// Constructs a new [Vulkan Entry](Entry) object.
    ///
    /// # Details
    ///
    /// Two steps in order:
    /// 1. It searches the [Loader](https://github.com/KhronosGroup/Vulkan-Loader)
    /// inside the OS (the name and location of the loader is platform-dependent) and load it.
    /// 2. It dynamically dispatches the Vulkan symbols and stores them in a VTable inside
    /// the Loader.
    ///
    /// # Errors
    ///
    /// - If the loader is not found, it returns an error.
    /// - If it fails to load the Vulkan entry, it returns an error.
    ///
    pub fn new() -> anyhow::Result<Self> {
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

    /// The version it returns is the (maximum) Vulkan version the [Loader](https://github
    /// .com/KhronosGroup/Vulkan-Loader)
    /// supports.
    /// > Note:
    /// > The ICDs could support a different version.
    /// > If the ICDs support a higher version, the Instance will be created with the Loader supported
    /// version.
    /// > If the ICDs support a lower version, when creating the Instance it will return a
    /// `VK_ERROR_INCOMPATIBLE_DRIVER` error.
    ///
    /// # Returns
    ///
    /// The version of the Vulkan API that the loader supports.
    ///
    /// # Errors
    ///
    /// - [`VK_ERROR_OUT_OF_HOST_MEMORY`](crate::gapi::errors::VK_ERROR_OUT_OF_HOST_MEMORY)
    /// Should never happen, if it does, it means the Loader or the Layers (wrongly) allocated memory.
    ///
    pub fn version(&self) -> anyhow::Result<Version> {
        Ok(self.entry.version()?)
    }

    /// Getter for the VkEntry
    pub fn get(&self) -> &vulkanalia::Entry {
        &self.entry
    }

    /// Get the available [layers](Layer) for the instance before its creation.
    /// Useful to check if the layer is available before creating the instance.
    ///
    /// # Details
    /// This calls `vkEnumerateInstanceLayerProperties` underneath.
    /// It returns a set of all the available global layers (layers inside the Vulkan Layers Manifest, whose
    /// location is OS specific).
    /// It follows this procedure:
    /// - This calls the Loader which scans for the layer manifest (JSON file) which contains the list of
    /// available layers.
    /// - The loader parses the manifest, validates it and reads and loads the metadata.
    /// - It then builds a list of available layers in memory.
    /// - Finally, it returns it to the caller.
    ///
    /// # Returns
    ///
    /// A set of all the available layers for the instance.
    /// A HashSet of [`Layer`] names.
    ///
    /// # Errors
    ///
    /// VK_ERROR_OUT_OF_HOST_MEMORY
    /// This error is thrown if the loader fails to allocate memory for the layer properties.
    ///
    /// VK_ERROR_OUT_OF_DEVICE_MEMORY
    ///
    ///
    /// # Examples
    ///
    pub fn get_available_layers(&self) -> anyhow::Result<HashSet<LayerStr>> {
        let available_layers: HashSet<LayerStr> =
            unsafe { self.entry.enumerate_instance_layer_properties() }?
                .iter()
                .map(|l| l.layer_name)
                .collect::<HashSet<_>>();
        Ok(available_layers)
    }

    /// Query for the available [extensions](Extension) for the instance
    ///
    /// # Details
    ///
    /// See [`Self::get_available_extensions`] for more details.
    ///
    /// # Returns
    ///
    /// See [`Self::get_available_extensions`]
    ///
    /// # Errors
    ///
    /// See [`Self::get_available_extensions`]
    ///
    pub fn get_available_extensions_instance(&self) -> anyhow::Result<HashSet<ExtensionStr>> {
        self.get_available_extensions(None)
    }

    /// Query for the available [extensions](Extension) for a specific layer.
    /// # Details
    ///
    /// See [`Self::get_available_extensions`] for more details.
    ///
    /// # Parameters
    ///
    /// - `layer`: The layer to query the extensions for.
    ///
    /// # Returns
    ///
    /// See [`Self::get_available_extensions`]
    ///
    /// # Errors
    ///
    /// See [`Self::get_available_extensions`]
    pub fn get_available_extensions_layer(
        &self,
        layer: LayerStr,
    ) -> anyhow::Result<HashSet<LayerStr>> {
        self.get_available_extensions(Some(layer.as_bytes()))
    }

    /// Queries all the available [extensions](Extension) (features) the
    /// [`Instance`](crate::gapi::instance::Instance) or selected [`Layer`] supports.
    ///
    /// Useful to make checks before creating a [Vulkan instance](crate::gapi::instance::Instance)
    /// with selected extensions.
    ///
    /// > Note:
    /// > This function is private and used internally called by [`Self::get_available_extensions_instance`] and
    /// [`Self::get_available_extensions_layer`].
    ///
    /// # Details
    /// This function queries for all extensions available to extend the functionality of
    /// `Instance` or `Layer`.
    /// More exactly:
    /// - For [`Instance`](crate::gapi::instance::Instance):
    ///     - It calls the Loader which scans the ICD manifest (JSON file) which contains the list of
    ///       static extensions, safe to expose pre-instance.
    ///     - The loader exposes its own built-in extensions (e.g., `VK_EXT_debug_utils` or `VK_KHR_surface`).
    ///       The loader works like a mini-ICD and mini-manifest, it defines and implements its own extensions.
    /// - For [`Layer`]:
    ///     - It calls the loader which loads the layer and calls the
    /// layer's extension query function.
    ///
    ///
    /// # Returns
    ///
    /// A set of all the available extensions for `Instance` or `Layer`.
    ///
    /// # Parameters
    ///
    /// - `optional_layer`: Optional layer to query the extensions for.
    ///     - If `None`, it queries for extensions of the instance.
    ///     - If `Some`, it queries for the extensions of the layer specified inside the `Option`.
    /// # Errors
    ///
    /// - [`VK_ERROR_OUT_OF_HOST_MEMORY`](crate::gapi::errors::VK_ERROR_OUT_OF_HOST_MEMORY)
    /// - [`VK_ERROR_OUT_OF_DEVICE_MEMORY`](crate::gapi::errors::VK_ERROR_OUT_OF_DEVICE_MEMORY)
    /// Can only happen after instance creation if the `Layer` for which we query the extensions is badly implemented.
    /// Theoretically, this should never be thrown.
    /// - [`VK_ERROR_LAYER_NOT_PRESENT`](crate::gapi::errors::VK_ERROR_LAYER_NOT_PRESENT)
    /// The layer for which we query the extensions does not exist.
    fn get_available_extensions(
        &self,
        optional_layer: Option<&[u8]>,
    ) -> anyhow::Result<HashSet<ExtensionStr>> {
        let available_extensions = unsafe {
            self.entry
                .enumerate_instance_extension_properties(optional_layer)
        }?
        .iter()
        .map(|e| e.extension_name)
        .collect::<HashSet<_>>();
        Ok(available_extensions)
    }

    /// Creates a [`Vulkan Instance`](crate::gapi::instance::Instance).
    /// It needs to be called after the [`Entry`] is created.
    /// This will be called by [`Instance::new`](crate::gapi::instance::Instance::new) (Instance constructor).
    ///
    /// # Parameters
    /// - `info`, composed of the following:
    ///     - `application_info`: Metadata about the app (name, version, Vulkan version)
    ///     - `enabled_layer_names`: List of validation or debug layers to enable.
    ///     - `enabled_extension_names`: List of instance extensions to use (e.g., `VK_KHR_surface`).
    /// - `allocation_callbacks`: Custom allocator hook, to override default memory management.
    ///
    /// # Returns
    ///
    /// A [`Vulkan Instance`](crate::gapi::instance::Instance) object.
    ///
    ///
    /// # Errors
    ///
    /// - [`VK_ERROR_OUT_OF_HOST_MEMORY`](crate::gapi::errors::VK_ERROR_OUT_OF_HOST_MEMORY)
    /// - [`VK_ERROR_OUT_OF_DEVICE_MEMORY`](crate::gapi::errors::VK_ERROR_OUT_OF_DEVICE_MEMORY)
    /// This shouldn't happen, but if it does, it means the ICD is badly implemented as this step shouldn't allocate.
    /// - [`VK_ERROR_INITIALIZATION_FAILED`](crate::gapi::errors::VK_ERROR_INITIALIZATION_FAILED)
    /// Catch-all error for initialization failures.
    /// - [`VK_ERROR_LAYER_NOT_PRESENT`](crate::gapi::errors::VK_ERROR_LAYER_NOT_PRESENT)
    /// This error is thrown if the layer specified in `enabled_layer_names` does not exist.
    /// - [`VK_ERROR_EXTENSION_NOT_PRESENT`](crate::gapi::errors::VK_ERROR_EXTENSION_NOT_PRESENT)
    /// This error is thrown if the extension specified in `enabled_extension_names` does not exist.
    /// - [`VK_ERROR_INCOMPATIBLE_DRIVER`](crate::gapi::errors::VK_ERROR_INCOMPATIBLE_DRIVER)
    /// This error is thrown if the driver is incompatible with the requested Vulkan version.
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
    ///
    pub fn create_instance(
        &self,
        info: &vk::InstanceCreateInfo,
        allocation_callbacks: Option<&vk::AllocationCallbacks>,
    ) -> VkResult<Instance> {
        unsafe { self.entry.create_instance(info, allocation_callbacks) }
    }
}
