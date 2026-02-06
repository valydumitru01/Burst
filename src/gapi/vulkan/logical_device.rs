use crate::gapi::vulkan::enums::extensions::DeviceExtension;
use crate::gapi::vulkan::instance::Instance;
pub(crate) use crate::gapi::vulkan::queues::{QueueCapability, QueueFamily};
pub(crate) use crate::gapi::vulkan::queues::{QueueRequest, Queues};
pub(crate) use crate::gapi::vulkan::real_device::RealDevice;
use crate::gapi::vulkan::surface::Surface;
use crate::gapi::vulkan::swapchain::Swapchain;
use crate::window::MyWindow;
use anyhow::bail;
use log::trace;
use std::collections::HashMap;
use vulkanalia::vk::{
    DeviceV1_0, HasBuilder, KhrSwapchainExtension, PhysicalDeviceFeatures, Queue,
    SwapchainCreateInfoKHR, SwapchainKHR,
};
use vulkanalia::{vk, Device};

/// Wraps the Vulkan logical device, and the queue handles it owns.
///
/// This object is responsible for:
/// - Creating the Vulkan device from a chosen physical device.
/// - Finding and storing all queue handles (graphics, present, etc.) according to user requests.
/// - Destroying the device (and by extension, the queues) at shutdown.
#[derive(Debug)]
pub struct LogicalDevice {
    /// The Vulkan device handle.
    device: Device,
    queues: Queues,
}

impl LogicalDevice {
    pub fn new(
        real_device: &RealDevice,
        instance: &Instance,
        surface: &Surface,
        requests: &[QueueRequest],
        extensions: &[DeviceExtension],
    ) -> anyhow::Result<Self> {
        // 1. Resolve which families satisfy which requests
        let resolved_families = Self::resolve_queue_requests(real_device, surface, requests)?;

        // 2. Prepare Create Infos (Handling the Priority Lifetime Problem)
        // We must keep the priorities in a stable memory location until create_device is called.
        let mut family_priorities: HashMap<u32, Vec<f32>> = HashMap::new();
        for res in &resolved_families {
            family_priorities
                .entry(res.family_index)
                .or_default()
                .extend(vec![1.0; res.count as usize]);
        }

        let queue_infos = family_priorities
            .iter()
            .map(|(&index, priorities)| {
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(index)
                    .queue_priorities(priorities)
            })
            .collect::<Vec<_>>();

        // 3. Device Creation
        let ext_names = extensions.iter().map(|e| e.name_ptr()).collect::<Vec<_>>();
        let features = PhysicalDeviceFeatures::builder().geometry_shader(true);

        let create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_infos)
            .enabled_extension_names(&ext_names)
            .enabled_features(&features);

        let device = unsafe {
            instance
                .get_vk()
                .create_device(*real_device.get_vk(), &create_info, None)?
        };

        let queues = Self::extract_queues(&device, &resolved_families)?;

        Ok(Self { device, queues })
    }

    fn extract_queues(
        device: &Device,
        resolved_families: &[QueueFamily],
    ) -> anyhow::Result<Queues> {
        let mut graphics = Vec::new();
        let mut present = Vec::new();
        let mut compute = Vec::new();
        let mut transfer = Vec::new();

        // Track how many queues we've already pulled from each family index
        let mut family_offsets = HashMap::<u32, u32>::new();

        for res in resolved_families {
            let offset = family_offsets.entry(res.family_index).or_insert(0);

            for i in 0..res.count {
                let queue_index = *offset + i;
                let handle = unsafe { device.get_device_queue(res.family_index, queue_index) };

                // Map the handle to the specific struct fields based on capability
                for &cap in &res.capabilities {
                    match cap {
                        QueueCapability::Graphics => graphics.push(handle),
                        QueueCapability::Compute => compute.push(handle),
                        QueueCapability::Transfer => transfer.push(handle),
                    }
                }

                if res.allows_present {
                    present.push(handle);
                }
            }

            // Advance the offset for this family so the next request gets unique handles
            *offset += res.count;
        }

        Ok(Queues {
            graphics,
            present,
            compute,
            transfer,
        })
    }

    fn extract_family_queues(real_device: &RealDevice, surface: &Surface) -> Vec<QueueFamily> {
        real_device
            .get_queue_families_properties()
            .iter()
            .enumerate()
            .filter_map(|(family_index, family)| {
                let family_index = family_index as u32;
                let capabilities = QueueCapability::from_flags(family.queue_flags);
                let allows_present = real_device
                    .supports_surface(family_index, surface.clone())
                    .unwrap_or(false);
                let count = family.queue_count;
                Some(QueueFamily {
                    family_index,
                    capabilities,
                    allows_present,
                    count,
                })
            })
            .collect::<Vec<_>>()
            .into()
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

    /// Finds the queue families that satisfy each requested queue. This method:
    /// 1. Inspects all available queue families of the `real_device`.
    /// 2. For each [`QueueRequest`], tries to locate a queue family that has the requested flags
    ///    and present support (if `require_present` is `true`).
    /// 3. Returns a list of [`QueueFamily`] records describing which family index each request ended up using.
    ///
    /// # Errors
    /// If any queue request cannot be satisfied by the current device (e.g., no family supports it),
    /// returns an error.
    fn resolve_queue_requests(
        real_device: &RealDevice,
        surface: &Surface,
        requests: &[QueueRequest],
    ) -> anyhow::Result<Vec<QueueFamily>> {
        trace!("Finding suitable queue families for requested queues...");
        let mut results = Vec::with_capacity(requests.len());
        // We need to fulfill all requests of families for the device
        for request in requests {
            trace!(
                "Finding queue family for request: {:#?}",
                request.capabilities
            );
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
                bail!(
                    "No suitable queue family found for {:#?}",
                    request.capabilities
                );
            }
        }

        Ok(results)
    }

    pub fn create_swapchain_khr(
        &self,
        info: &SwapchainCreateInfoKHR,
    ) -> anyhow::Result<SwapchainKHR> {
        unsafe {
            self.device
                .create_swapchain_khr(info, None)
                .map_err(|e| anyhow::anyhow!("Failed to create swapchain: {}", e))
        }
    }

    /// Returns a reference to the underlying Vulkan [`Device`].
    ///
    /// # Example
    /// ```
    /// let device_handle = logical_device.get_device();
    /// // use device_handle...
    /// ```
    pub fn get_vk(&self) -> &Device {
        &self.device
    }

    pub fn get_queues(&self) -> &Queues {
        &self.queues
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
}
