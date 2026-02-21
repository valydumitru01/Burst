use std::collections::HashMap;
use anyhow::bail;
use log::{info, trace};
use vulkanalia::{vk, Device};
use vulkanalia::vk::{DeviceV1_0, HasBuilder, Queue};
use crate::gapi::vulkan::core::real_device::RealDevice;
use crate::gapi::vulkan::core::surface::Surface;

trait BitIter {
    fn iter(&self) -> impl Iterator<Item = u32>;
}

impl BitIter for u32 {
    fn iter(&self) -> impl Iterator<Item = u32> {
        (0..32).filter(move |i| (*self & (1u32 << i)) != 0u32)
    }
}
/// An enum to distinguish different queue types your application might need.
///
/// If you need more specialized queues (e.g., compute-only, transfer-only),
/// you can add variants here and include the matching logic in `find_queue_families`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum QueueCapability {
    /// A queue that supports graphics operations.
    Graphics,
    /// A queue that supports compute operations.
    Compute,
    /// A queue that supports transfer operations (copying data).
    Transfer,
}
impl QueueCapability {
    pub(crate) fn from_flags(flags: vk::QueueFlags) -> Vec<Self> {
        let mut capabilities = Vec::new();
        if flags.contains(vk::QueueFlags::GRAPHICS) {
            capabilities.push(Self::Graphics);
        }
        if flags.contains(vk::QueueFlags::COMPUTE) {
            capabilities.push(Self::Compute);
        }
        if flags.contains(vk::QueueFlags::TRANSFER) {
            capabilities.push(Self::Transfer);
        }
        capabilities
    }
}

impl From<QueueCapability> for vk::QueueFlags {
    fn from(cap: QueueCapability) -> Self {
        match cap {
            QueueCapability::Graphics => vk::QueueFlags::GRAPHICS,
            QueueCapability::Compute => vk::QueueFlags::COMPUTE,
            QueueCapability::Transfer => vk::QueueFlags::TRANSFER,
        }
    }
}

/// A request describing how many queues of a given type your app needs.
///
/// For example, if you require two graphics queues, you can specify:
/// ```
/// QueueRequest {
///     queue_type: QueueType::Graphics,
///     queue_flags: vk::QueueFlags::GRAPHICS,
///     require_present: false,
///     count: 2,
/// }
/// ```
#[derive(Debug)]
pub struct QueueRequest {
    /// The type of queue requested (e.g. [`QueueCapability::Graphics`]).
    pub capabilities: Vec<QueueCapability>,
    /// Whether this queue needs to support presentation on the given surface.
    pub require_present: bool,
    /// How many queues of this type should be created?
    pub count: u32,
}

/// Holds metadata about a single queue family that will be created, including
/// which [`QueueCapability`] it corresponds to and how many queues from that family
/// will be requested.
#[derive(Debug)]
pub struct QueueFamily {
    /// The queue family index in Vulkan.
    pub family_index: u32,
    /// How many there are available in this family.
    pub count: u32,
    /// The type of queue we are satisfying (graphics, present, etc.).
    pub capabilities: Vec<QueueCapability>,
    /// Whether this family can present images to the surface.
    pub allows_present: bool,
}

#[derive(Debug)]
pub struct Queues{
    pub graphics: Vec<Queue>,
    pub graphics_family_index: u32,
    pub present: Vec<Queue>,
    pub present_family_index: u32,
    pub compute: Vec<Queue>,
    pub compute_family_index: u32,
    pub transfer: Vec<Queue>,
}


impl Queues{

    pub fn new(
        device: &Device,
        families: &[QueueFamily],
    ) -> anyhow::Result<Self> {
        Self::extract_queues(device, &families)
    }

    fn extract_queues(
        device: &Device,
        resolved_families: &[QueueFamily],
    ) -> anyhow::Result<Queues> {
        let mut graphics = Vec::new();
        let mut graphics_family_index = 0;
        let mut present = Vec::new();
        let mut present_family_index = 0;
        let mut compute = Vec::new();
        let mut compute_family_index = 0;
        let mut transfer = Vec::new();
        let mut transfer_family_index = 0;

        // Track how many queues we've already pulled from each family index
        let mut family_offsets = HashMap::<u32, u32>::new();

        for res in resolved_families {
            let offset = family_offsets.entry(res.family_index).or_insert(0);
            if res.capabilities.contains(&QueueCapability::Graphics) {
                graphics_family_index = res.family_index;
            }
            if res.capabilities.contains(&QueueCapability::Compute) {
                compute_family_index = res.family_index;
            }
            if res.capabilities.contains(&QueueCapability::Transfer) {
                transfer_family_index = res.family_index;
            }
            if res.allows_present {
                present_family_index = res.family_index;
            }
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
            graphics_family_index,
            present,
            present_family_index,
            compute,
            compute_family_index,
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
                    .supports_surface(family_index, surface)
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
    pub fn create_queue_infos(families: &[QueueFamily]) -> Vec<vk::DeviceQueueCreateInfo> {
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
    pub fn resolve_queue_requests(
        real_device: &RealDevice,
        surface: &Surface,
        requests: &[QueueRequest],
    ) -> anyhow::Result<Vec<QueueFamily>> {
        info!("Finding suitable queue families for requested queues...");
        let mut results = Vec::with_capacity(requests.len());
        // We need to fulfill all requests of families for the device
        for request in requests {
            info!(
                "Finding queue family for request: \n{:#?}",
                request.capabilities
            );
            let required_flags = &request.capabilities;
            let properties = real_device.get_queue_families_properties();

            // Now we go over the queue families of the device and try to find one that matches
            for (family_index, props) in properties.iter().enumerate() {
                // Make the index a u32 from usize to match Vulkan's expectations.
                let family_index = family_index as u32;
                let supports_present =
                    real_device.supports_surface(family_index, surface)?;
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

}