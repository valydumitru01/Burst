use crate::window::window::MyWindow;

use crate::gapi::vulkan::config::VALIDATION_ENABLED;
use crate::gapi::vulkan::entry::Entry;
use crate::gapi::vulkan::extensions::InstanceExtension;
use crate::gapi::vulkan::instance::{Instance, PORTABILITY_MACOS_VERSION};
pub(crate) use crate::gapi::vulkan::queues::{QueueCapability, QueueFamily, QueueRequest};
pub(crate) use crate::gapi::vulkan::real_device::RealDevice;
use crate::gapi::vulkan::surface::Surface;
use anyhow::anyhow;
use std::collections::HashMap;
use vulkanalia::vk::{DeviceV1_0, HasBuilder, PhysicalDeviceFeatures};
use vulkanalia::{vk, Device};

/// Wraps the Vulkan logical device, and the queue handles it owns.
///
/// This object is responsible for:
/// - Creating the Vulkan device from a chosen physical device.
/// - Finding and storing all queue handles (graphics, present, etc.) according to user requests.
/// - Destroying the device (and by extension, the queues) at shutdown.
#[derive(Clone, Debug)]
pub struct LogicalDevice {
    /// The Vulkan device handle.
    device: Device,
    /// A mapping from [`Vec<QueueCapability>`] to a list of Vulkan queues of that type.
    queues: HashMap<Vec<QueueCapability>, Vec<vk::Queue>>,
}

impl LogicalDevice {
    /// Creates a new [`LogicalDevice`] and the requested queues.
    ///
    /// # Parameters
    /// - `entry`: The Vulkan entry, used to track available layers and extensions.
    /// - `instance`: The Vulkan instance.
    /// - `surface`: The window surface we need to present images on (if any requested queue needs it).
    /// - `window`: The winit (or similar) window wrapper, needed for instance extensions on some platforms.
    /// - `real_device`: The chosen Vulkan physical device.
    /// - `requests`: A slice of [`QueueRequest`] describing how many queues of each type your app needs.
    ///
    /// # Returns
    /// A [`LogicalDevice`] containing the underlying Vulkan device and its queue handles.
    ///
    /// # Errors
    /// Returns an error if:
    /// - No suitable queue families can be found for the requested queue types.
    /// - The device creation fails.
    pub fn new(
        entry: &Entry,
        instance: &Instance,
        surface: &Surface,
        window: &MyWindow,
        real_device: RealDevice,
        queue_requests: Vec<QueueRequest>,
    ) -> anyhow::Result<Self> {
        // Find a valid queue family for each requested queue type (graphics, present, etc.).
        let family_infos =
            Self::find_queue_families(instance, surface, &real_device, queue_requests)?;

        // Gather any required device extensions.
        let extensions = Self::get_required_extensions(entry, window)?;

        // Build up the Vulkan queue creation infos from the resolved family indices.
        // We merge same-family requests so that we only create one `DeviceQueueCreateInfo` per distinct family index.
        let queue_infos = Self::create_queue_infos(&family_infos);

        // Build the device creation info structure.
        let features = PhysicalDeviceFeatures::builder();
        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&extensions)
            .enabled_features(&features);

        // Create the logical device.
        let device = unsafe {
            instance
                .get_vk()
                .create_device(*real_device.get_vk(), &create_info, None)
        }
        .map_err(|e| anyhow!("Failed to create logical device: {}", e))?;

        // Retrieve the Vulkan queue handles, storing them in a HashMap keyed by [`QueueType`].
        let queue_map = Self::retrieve_queues(&device, &family_infos);

        Ok(Self {
            device,
            queues: queue_map,
        })
    }

    /// Returns a reference to the underlying Vulkan [`Device`].
    ///
    /// # Example
    /// ```
    /// let device_handle = logical_device.get_device();
    /// // use device_handle...
    /// ```
    pub fn get_device(&self) -> &Device {
        &self.device
    }

    /// Retrieves all queues of a particular [`QueueCapability`].
    ///
    /// Most use-cases need only one queue of each type; in that case you can do:
    /// ```
    /// let graphics_queue = logical_device.get_queues(QueueType::Graphics)[0];
    /// ```
    pub fn get_queues(&self, queue_type: &Vec<QueueCapability>) -> &[vk::Queue] {
        self.queues
            .get(queue_type)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Destroys this logical device. Automatically frees all queues it owns.
    ///
    /// # Safety
    /// Must only be called when you are certain no further use of the device or
    /// its queues is needed.
    pub fn destroy(&self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }

    /// Finds the queue families that satisfy each requested queue. This method:
    /// 1. Inspects all available queue families of the `real_device`.
    /// 2. For each [`QueueRequest`], tries to locate a queue family that has the requested flags
    ///    and present support (if `require_present` is `true`).
    /// 3. Returns a list of [`QueueFamily`] records describing which family index each request ended up using.
    ///
    /// # Errors
    /// If any queue request cannot be satisfied by the current device (e.g., no family supports it),
    /// returns an error.
    fn find_queue_families(
        _instance: &Instance,
        surface: &Surface,
        real_device: &RealDevice,
        requests: Vec<QueueRequest>,
    ) -> anyhow::Result<Vec<QueueFamily>> {
        let mut results = Vec::with_capacity(requests.len());
        // We need to fulfill all requests of families for the device
        for request in requests {
            let required_flags = &request.capabilities;
            let properties = real_device.get_queue_families_properties();

            // Now we go over the queue families of the device and try to find one that matches
            for (family_index, props) in properties.iter().enumerate() {
                // Make the index a u32 from usize to match Vulkan's expectations.
                let family_index = family_index as u32;
                let supports_present =
                    real_device.supports_surface(family_index, surface.clone())?;
                // If we require present support, but this family doesn't support it, skip it.
                if request.require_present && !supports_present {
                    continue;
                };

                let supports_flags = required_flags
                    .iter()
                    .all(|&flag| props.queue_flags.contains(flag.into()));

                // If the family doesn't support all required flags, skip it.
                if !supports_flags {
                    continue;
                }

                // The first one that matches our requirements is the one we store to then use
                results.push(QueueFamily {
                    family_index,
                    count: request.count,
                    capabilities: required_flags.clone(),
                    allows_present: supports_present,
                });
                // Then we stop searching a queue for this request, we go to the next one.
                break;
            }

            if results.is_empty() {
                return Err(anyhow!(
                    "No suitable queue family found for {:#?}",
                    request.capabilities
                ));
            }
        }

        Ok(results)
    }
    /// Creates a minimal set of [`vk::DeviceQueueCreateInfo`] objects for the discovered family indices.
    /// If multiple requests share the same family index, we merge them into one entry that requests
    /// the sum of their `count`.
    fn create_queue_infos(families: &[QueueFamily]) -> Vec<vk::DeviceQueueCreateInfo> {
        // We merge families by index, summing up how many queues we want to create.
        let mut merged: HashMap<u32, u32> = HashMap::new();
        for info in families {
            *merged.entry(info.family_index).or_insert(0) += info.count;
        }

        // For each unique family index, create one QueueCreateInfo that asks for the needed queue count.
        let mut create_infos = Vec::with_capacity(merged.len());
        for (family_index, total_count) in merged {
            let priorities = vec![1.0_f32; total_count as usize];
            let info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(family_index)
                .queue_priorities(&priorities)
                .build();
            create_infos.push(info);
        }
        create_infos
    }

    /// Retrieves the actual Vulkan `vk::Queue` handles from the newly created device,
    /// returning a [`HashMap`] that maps each [`QueueCapability`] to its queues.
    ///
    /// The order of the queue handles in each list corresponds to the `count` requested.
    fn retrieve_queues(
        device: &Device,
        families: &[QueueFamily],
    ) -> HashMap<Vec<QueueCapability>, Vec<vk::Queue>> {
        // We'll store each queue in a map from QueueType -> Vec<vk::Queue>.
        let mut result: HashMap<Vec<QueueCapability>, Vec<vk::Queue>> = HashMap::new();

        // We also need to track, for each family index, how many queues we've already retrieved
        // so we can retrieve them in order: e.g., `get_device_queue(family_index, 0)`, then `(family_index, 1)`, etc.
        let mut counters: HashMap<u32, u32> = HashMap::new();

        for info in families {
            let family_index = info.family_index;
            let start = *counters.get(&family_index).unwrap_or(&0);
            let end = start + info.count;

            let mut handles = Vec::with_capacity(info.count as usize);
            for queue_idx in start..end {
                // Retrieve queue handle from the device.
                let q = unsafe { device.get_device_queue(family_index, queue_idx) };
                handles.push(q);
            }
            // Store them under the correct queue type in our map.
            result
                .entry(info.capabilities.clone())
                .or_default()
                .extend(handles);

            // Update the counter so if we have multiple requests referencing the same queue family,
            // we keep indexing sequentially.
            counters.insert(family_index, end);
        }

        result
    }

    fn get_required_extensions(entry: &Entry, window: &MyWindow) -> anyhow::Result<Vec<*const i8>> {
        let mut extensions = window
            .get_required_extensions()
            .iter()
            .map(|e| e.as_ptr())
            .collect::<Vec<_>>();

        // If validation is enabled, also enable the debug utils extension (if available).
        if VALIDATION_ENABLED {
            extensions.push(InstanceExtension::ExtDebugUtils.name().as_ptr());
        }

        // macOS portability extension if needed.
        if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
            extensions.push(vk::KHR_PORTABILITY_SUBSET_EXTENSION.name.as_ptr());
        }

        // Surface extension to interact with the window system.
        extensions.push(vk::KHR_SURFACE_EXTENSION.name.as_ptr());

        Ok(extensions)
    }
}
