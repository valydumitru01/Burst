use crate::gapi::entry::Entry;
use crate::gapi::instance::{Instance, Layers, PORTABILITY_MACOS_VERSION};
use crate::gapi::physical_device::PhysicalDevice;
use crate::gapi::surface::Surface;
use crate::gapi::vulkan::{SuitabilityError, VALIDATION_ENABLED};
use crate::window::window::MyWindow;

use anyhow::anyhow;
use std::collections::HashMap;
use std::ffi::CStr;
use vulkanalia::vk::{
	DeviceV1_0, HasBuilder, InstanceV1_0, KhrSurfaceExtension, PhysicalDeviceFeatures,
};
use vulkanalia::{vk, Device};

/// An enum to distinguish different queue types your application might need.
///
/// If you need more specialized queues (e.g., compute-only, transfer-only),
/// you can add variants here and include the matching logic in `find_queue_families`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum QueueType {
	/// A queue that supports graphics operations.
	Graphics,
	/// A queue that supports presenting images to a surface (swapchain).
	Present,
	// Add more variants here as needed, e.g. `Compute`, `Transfer`, etc.
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
#[derive(Clone, Debug)]
pub struct QueueRequest {
	/// The type of queue requested (e.g. [`QueueType::Graphics`]).
	pub queue_type: QueueType,
	/// Whether or not this queue needs to support presentation on the given surface.
	pub require_present: bool,
	/// How many queues of this type should be created.
	pub count: u32,
}

/// Holds metadata about a single queue family that will be created, including
/// which [`QueueType`] it corresponds to and how many queues from that family
/// will be requested.
#[derive(Clone, Debug)]
struct FamilyInfo {
	/// The queue family index in Vulkan.
	pub family_index: u32,
	/// How many queues to create in this family.
	pub count: u32,
	/// The type of queue we are satisfying (graphics, present, etc.).
	pub queue_type: QueueType,
}

/// Wraps the Vulkan logical device and the queue handles it owns.
///
/// This object is responsible for:
/// - Creating the Vulkan device from a chosen physical device.
/// - Finding and storing all queue handles (graphics, present, etc.) according to user requests.
/// - Destroying the device (and by extension, the queues) at shutdown.
#[derive(Clone, Debug)]
pub struct LogicalDevice {
	/// The Vulkan device handle.
	device: Device,
	/// A mapping from [`QueueType`] to a list of Vulkan queues of that type.
	queues: HashMap<QueueType, Vec<vk::Queue>>,
}

impl LogicalDevice {
	/// Creates a new [`LogicalDevice`] and the requested queues.
	///
	/// # Parameters
	/// - `entry`: The Vulkan entry, used to track available layers and extensions.
	/// - `instance`: The Vulkan instance.
	/// - `surface`: The window surface we need to present images on (if any requested queue needs it).
	/// - `window`: The winit (or similar) window wrapper, needed for instance extensions on some platforms.
	/// - `physical_device`: The chosen Vulkan physical device.
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
	) -> anyhow::Result<Self> {
		let requests: &[QueueRequest] = &[
			QueueRequest {
				queue_type: QueueType::Graphics,
				require_present: false,
				count: 1,
			},
			QueueRequest {
				queue_type: QueueType::Present,
				require_present: true,
				count: 1,
			},
		];
		// Pick a physical device
		let physical_device = instance.pick_physical_device()?;

		// Find a valid queue family for each requested queue type (graphics, present, etc.).
		let family_infos = Self::find_queue_families(instance, surface, &physical_device, requests)?;

		// Gather any required device extensions.
		let extensions = Self::get_extensions(entry, window)?;
		let layers = Layers::get_all_c_chars();

		// Build up the Vulkan queue creation infos from the resolved family indices.
		// We merge same-family requests so that we only create one `DeviceQueueCreateInfo` per distinct family index.
		let queue_infos = Self::create_queue_infos(&family_infos);

		// Build the device creation info structure.
		let features = PhysicalDeviceFeatures::builder();
		let create_info = vk::DeviceCreateInfo::builder().queue_create_infos(&queue_infos).enabled_layer_names(&layers).enabled_extension_names(&extensions).enabled_features(&features);

		// Create the logical device.
		let device = unsafe {
			instance.get().create_device(*physical_device.get(), &create_info, None)
		}.map_err(|e| anyhow!("Failed to create logical device: {}", e))?;

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

	/// Retrieves all queues of a particular [`QueueType`].
	///
	/// Most use-cases need only one queue of each type; in that case you can do:
	/// ```
	/// let graphics_queue = logical_device.get_queues(QueueType::Graphics)[0];
	/// ```
	pub fn get_queues(&self, queue_type: QueueType) -> &[vk::Queue] {
		self.queues.get(&queue_type).map(|v| v.as_slice()).unwrap_or(&[])
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

	// ------------------------------------------------------------------------
	// Internal helper methods
	// ------------------------------------------------------------------------

	/// Finds the queue families that satisfy each requested queue. This method:
	/// 1. Inspects all available queue families of the `physical_device`.
	/// 2. For each [`QueueRequest`], tries to locate a queue family that has the requested flags
	///    and present support (if `require_present` is `true`).
	/// 3. Returns a list of [`FamilyInfo`] records describing which family index each request ended up using.
	///
	/// # Errors
	/// If any queue request cannot be satisfied by the current device (e.g., no family supports it),
	/// returns an error.
	fn find_queue_families(
		instance: &Instance,
		surface: &Surface,
		physical_device: &PhysicalDevice,
		requests: &[QueueRequest],
	) -> anyhow::Result<Vec<FamilyInfo>> {
		let properties = unsafe {
			instance.get().get_physical_device_queue_family_properties(*physical_device.get())
		};

		// We'll accumulate the resolved FamilyInfo objects for each request in this vector.
		let mut results = Vec::with_capacity(requests.len());

		'request_loop: for request in requests {
			let required_flags = match request.queue_type {
				QueueType::Graphics => vk::QueueFlags::GRAPHICS,
				QueueType::Present => vk::QueueFlags::empty(), // Present support is checked separately.
				// Add more cases if needed for compute, transfer, etc.
			};

			// Try to find a queue family matching the flags and present requirements of this request.
			for (family_index, props) in properties.iter().enumerate() {
				let family_index = family_index as u32;

				// Check if the queue flags match.
				let supports_flags = props.queue_flags.contains(required_flags);

				// If user wants present, check physical_device_surface_support_khr.
				let supports_present = if request.require_present {
					unsafe {
						instance.get().get_physical_device_surface_support_khr(
							*physical_device.get(),
							family_index,
							surface.get(),
						)?
					}
				} else {
					true
				};

				if supports_flags && supports_present {
					results.push(FamilyInfo {
						family_index,
						count: request.count,
						queue_type: request.queue_type,
					});
					// Move on to the next request.
					continue 'request_loop;
				}
			}
			// If we got here, no family could satisfy the request -> error out.
			return Err(anyhow!(
                "No suitable queue family found for {:?}",
                request.queue_type
            ));
		}

		Ok(results)
	}

	/// Creates a minimal set of [`vk::DeviceQueueCreateInfo`] objects for the discovered family indices.
	/// If multiple requests share the same family index, we merge them into one entry that requests
	/// the sum of their `count`.
	fn create_queue_infos(families: &[FamilyInfo]) -> Vec<vk::DeviceQueueCreateInfo> {
		// We merge families by index, summing up how many queues we want to create.
		let mut merged: HashMap<u32, u32> = HashMap::new();
		for info in families {
			*merged.entry(info.family_index).or_insert(0) += info.count;
		}

		// For each unique family index, create one QueueCreateInfo that asks for the needed queue count.
		let mut create_infos = Vec::with_capacity(merged.len());
		for (family_index, total_count) in merged {
			let priorities = vec![1.0_f32; total_count as usize];
			let info = vk::DeviceQueueCreateInfo::builder().queue_family_index(family_index).queue_priorities(&priorities).build();
			create_infos.push(info);
		}
		create_infos
	}

	/// Retrieves the actual Vulkan `vk::Queue` handles from the newly created device,
	/// returning a [`HashMap`] that maps each [`QueueType`] to its queues.
	///
	/// The order of the queue handles in each list corresponds to the `count` requested.
	fn retrieve_queues(
		device: &Device,
		families: &[FamilyInfo],
	) -> HashMap<QueueType, Vec<vk::Queue>> {
		// We'll store each queue in a map from QueueType -> Vec<vk::Queue>.
		let mut result: HashMap<QueueType, Vec<vk::Queue>> = HashMap::new();

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
			result.entry(info.queue_type).or_default().extend(handles);

			// Update the counter so if we have multiple requests referencing the same queue family,
			// we keep indexing sequentially.
			counters.insert(family_index, end);
		}

		result
	}

	fn get_extensions(entry: &Entry, window: &MyWindow) -> anyhow::Result<Vec<*const i8>> {
		let mut extensions = window.get_required_extensions().iter().map(|e| e.as_ptr()).collect::<Vec<_>>();

		// If validation is enabled, also enable the debug utils extension (if available).
		if VALIDATION_ENABLED {
			extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
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
