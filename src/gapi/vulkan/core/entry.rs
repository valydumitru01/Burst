use crate::gapi::vulkan::enums::extensions::InstanceExtension;
use crate::gapi::vulkan::enums::layers::{InstanceLayer, LayerStr};
use anyhow::{anyhow, Context};
use log::{trace, warn};
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

    /// The version it returns is the (maximum) Vulkan version the
    ///     [Loader](https://github.com/KhronosGroup/Vulkan-Loader)
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
    pub fn get_available_layers(&self) -> anyhow::Result<HashSet<InstanceLayer>> {
        let available_layers: HashSet<InstanceLayer> =
            unsafe { self.entry.enumerate_instance_layer_properties() }?
                .iter()
                .map(|l| InstanceLayer::from_name(&l.layer_name))
                .collect::<HashSet<_>>();
        Ok(available_layers)
    }

    /// Checks if the provided layers are available for the instance.
    ///
    /// # Returns
    /// - `Ok()` if all layers are available.
    /// - `Err()` if any layer is not available, with a message indicating which layers are missing.
    /// # Parameters
    ///
    pub fn check_layers_are_available(
        &self,
        required_layers: &Vec<InstanceLayer>,
    ) -> anyhow::Result<()> {
        let missing_layers = self.find_unavailable_layers(required_layers)?;
        if missing_layers.is_empty() {
            Ok(())
        } else {
            if missing_layers.contains(&InstanceLayer::RenderDoc) {
                warn!("{}", "RenderDoc layer is not available.");
                warn!(
                    "{}",
                    "You can install it from https://renderdoc.org/, or disable it in the configuration."
                );
            }
            Err(anyhow!(
                "The following layers are not available: {:?}",
                missing_layers
            ))
        }
    }

    fn is_layer_supported_by_extensions(
        &self,
        layer: &InstanceLayer,
        available_extensions: &HashSet<InstanceExtension>,
    ) -> bool {
        layer
            .required_extensions()
            .iter()
            .all(|ext| available_extensions.contains(ext))
    }
    pub fn check_layers_supported_by_extensions(
        &self,
        layers: &Vec<InstanceLayer>,
    ) -> anyhow::Result<()> {
        let available_extensions = self.get_available_instance_extensions()?;
        for layer in layers {
            trace!(
                "Checking if layer `{}` is supported by extensions...",
                layer
            );
            if !self.is_layer_supported_by_extensions(&layer, &available_extensions) {
                return Err(anyhow!(
                    "The layer `{}` is not supported by the available extensions.",
                    layer
                ));
            }
            trace!("Layer `{}` is supported by extensions.", layer);
        }
        Ok(())
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
    pub fn get_available_instance_extensions(&self) -> anyhow::Result<HashSet<InstanceExtension>> {
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
    pub fn get_available_layer_extensions(
        &self,
        layer: LayerStr,
    ) -> anyhow::Result<HashSet<InstanceExtension>> {
        self.get_available_extensions(Some(layer.as_bytes()))
    }

    /// # Are Extensions Available?
    /// Checks if the provided extensions are available for the instance or a specific layer.
    /// # Details
    /// This method is hidden from the public API and is used internally.
    /// To use it, you can call:
    /// - [`Self::check_instance_extensions_available`] for instance extensions.
    /// - [`Self::check_layer_extensions_are_available`] for layer extensions.
    /// # Parameters
    /// - `extensions`: An iterable collection of extensions to check.
    /// - `optional_layer`:
    ///     - If `Some(layer)`, checks the extensions for that specific layer.
    ///     - If `None`, checks the extensions for the instance.
    /// # Returns
    /// - `Ok` if all the extensions are available.
    /// - `Err(anyhow::Error)` if any of the extensions are not available,
    ///     with a message indicating which ones are missing.
    fn check_extensions_are_available(
        &self,
        extensions: &Vec<InstanceExtension>,
        optional_layer: Option<&[u8]>,
    ) -> anyhow::Result<()> {
        let available_extensions = self.get_available_extensions(optional_layer)?;
        let missing_extensions: Vec<_> = extensions
            .into_iter()
            .inspect(|ext| {
                log::trace!("Checking instance extension: {}", ext);
            })
            .filter(|ext| !available_extensions.contains(ext))
            .collect();
        if missing_extensions.is_empty() {
            Ok(())
        } else {
            Err(anyhow!(
                "The following extensions are not available: {:?}",
                missing_extensions
            ))
        }
    }

    /// Finds all unavailable layers before creating [`crate::gapi::vulkan::instance::Instance`].
    /// The available layers must be returned by [`Entry`].
    ///
    /// # Parameters
    /// - `available_layers`: The layers available in the system (queried through the Vulkan [`Entry`]).
    /// - `instance_layers`: The layers to be used in the [`crate::gapi::vulkan::instance::Instance`], configured in the instance creation.
    ///
    /// # Returns
    /// - A list of all the unavailable layers in the configuration.
    pub fn find_unavailable_layers(
        &self,
        required_layers: &Vec<InstanceLayer>,
    ) -> anyhow::Result<Vec<InstanceLayer>> {
        let available_layers = self.get_available_layers()?;
        Ok(required_layers
            .iter()
            .map(|l| l.clone())
            .filter(|l| !available_layers.contains(l))
            .collect::<Vec<_>>())
    }

    pub fn find_unavailable_extensions(
        &self,
        required_extensions: Vec<InstanceExtension>,
    ) -> anyhow::Result<Vec<InstanceExtension>> {
        Ok(self
            .get_available_instance_extensions()?
            .into_iter()
            .filter(|ext| !required_extensions.contains(ext))
            .collect::<Vec<_>>())
    }

    /// Checks if the provided instance extensions are available.
    /// # Details
    /// See [`Self::check_extensions_are_available`] for more details.
    /// # Parameters
    /// - `extensions`: A vector of extensions to check.
    /// # Returns
    /// - `Ok(())` if all the extensions are available for the instance.
    /// - `Err(anyhow::Error)` if any of the extensions are not available,
    ///   with a message indicating which ones are missing.
    pub fn check_instance_extensions_available(
        &self,
        extensions: &Vec<InstanceExtension>,
    ) -> anyhow::Result<()> {
        self.check_extensions_are_available(extensions, None)
    }

    /// Checks if the provided layer extensions are available.
    /// # Details
    /// See [`Self::check_extensions_are_available`] for more details.
    /// # Parameters
    /// - `layer`: The layer to check the extensions for.
    /// - `extensions`: An iterable collection of extensions to check.
    /// # Returns
    /// - `Ok(())` if all the extensions are available for the layer.
    /// - `Err(anyhow::Error)` if any of the extensions are not available,
    ///  with a message indicating which ones are missing.
    pub fn check_layer_extensions_are_available(
        &self,
        layer: LayerStr,
        extensions: &Vec<InstanceExtension>,
    ) -> anyhow::Result<()> {
        self.check_extensions_are_available(extensions, Some(layer.as_bytes()))
    }

    /// Queries all the available [extensions](Extension) (features) the
    /// [`Instance`](crate::gapi::instance::Instance) or selected [`Layer`] supports.
    ///
    /// Useful to make checks before creating a [Vulkan instance](crate::gapi::instance::Instance)
    /// with selected extensions.
    ///
    /// > Note:
    /// > This function is private and used internally called by [`Self::get_available_instance_extensions`] and
    /// [`Self::get_available_layer_extensions`].
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
    pub(crate) fn get_available_extensions(
        &self,
        optional_layer: Option<&[u8]>,
    ) -> anyhow::Result<HashSet<InstanceExtension>> {
        let available_extensions = unsafe {
            self.entry
                .enumerate_instance_extension_properties(optional_layer)
        }?
        .iter()
        .map(|e| InstanceExtension::from_name(&e.extension_name))
        .collect::<HashSet<_>>();
        Ok(available_extensions)
    }
}
