use crate::gapi::debug::Debugger;
use crate::gapi::entry::{Entry, ExtensionStr, LayerStr};
use crate::gapi::physical_device::PhysicalDevice;
use crate::gapi::vulkan::{SuitabilityError, VALIDATION_ENABLED};
use crate::window::window::MyWindow;
use anyhow::anyhow;
use log::{info, warn};
use std::collections::HashSet;
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
	pub const VALIDATION_LAYER: LayerStr = LayerStr::from_bytes("VK_LAYER_KHRONOS_validation".as_bytes());
	pub const VALIDATION_LAYER_NAME: LayerStr = LayerStr::from_bytes("VK_LAYER_LUNARG_api_dump".as_bytes());
	pub const VALIDATION_LAYER_RENDERDOC: LayerStr = LayerStr::from_bytes("VK_LAYER_RENDERDOC_Capture".as_bytes());
	pub(in crate::gapi) fn as_cstr(&self) -> *const c_char {
		match self {
			Self::Validation => {
				Self::VALIDATION_LAYER.as_ptr()
			}
			Self::ApiDump => {
				Self::VALIDATION_LAYER_NAME.as_ptr()
			}
			Self::RenderDoc => {
				Self::VALIDATION_LAYER_RENDERDOC.as_ptr()
			}
		}
	}

	pub(in crate::gapi) fn get_all_names_c_char() -> Vec<*const c_char> {
		Self::get_all_names().iter()
				.map(|layer| layer.as_ptr())
				.collect()
	}

	pub(in crate::gapi) fn get_all_names() -> Vec<LayerStr> {
		vec![
			LayerStr::from_bytes(Self::VALIDATION_LAYER.as_bytes()),
			LayerStr::from_bytes(Self::VALIDATION_LAYER_NAME.as_bytes()),
			LayerStr::from_bytes(Self::VALIDATION_LAYER_RENDERDOC.as_bytes())
		]
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
	/// [`Entry`] is in charge of creating the Vulkan Instance; this constructor handles the flags, extensions,
	/// layers, and info needed to create the instance.
	/// I.e., The configuration of the Instance is abstracted away inside the Instance class.
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
		let layers = Self::get_layers();
		let application_info = Self::make_application_info();

		// Check if the requested layers are available.
		if VALIDATION_ENABLED {
			Self::are_layers_available(entry.get_available_layers()?, layers);
		}
		let mut info = vk::InstanceCreateInfo::builder()
				.application_info(&application_info)
				.enabled_layer_names(&layers)
				.enabled_extension_names(&extensions)
				.flags(flags);

		// Add debug messages for creation and destruction of the Vulkan instance.
		if VALIDATION_ENABLED {
			log::debug!("Enabling Validation Layer.");
			Debugger::add_instance_life_debug(&mut info);
		}
		log::debug!("Creating instance...");
		let instance = entry.create_instance(&info, None)?;

		Ok(Self { instance })
	}


	fn make_application_info() -> vk::ApplicationInfo {
		vk::ApplicationInfo::builder().application_name(b"Burst\0")
				.application_version(vk::make_version(1, 0, 0))
				.engine_name(b"BurstG\0")
				.engine_version(vk::make_version(1, 0, 0))
				.api_version(vk::make_version(1, 0, 0))
				.build()
	}

	/// Collects all the extensions that will be used for the Vulkan [`Instance`] creation.
	///
	/// # Parameters
	/// - `window`: The window handler ([`MyWindow`]) that knows its required extensions.
	///
	/// # Returns
	/// - A vector of [`ExtensionStr`] that contains the required extensions for the Vulkan instance.
	fn get_extensions(window: &MyWindow) -> Vec<&ExtensionStr> {
		// Query for the extensions required by the window system.
		let mut extensions = window
				.get_required_extensions()
				.iter()
				.map(|ext| *ext)
				.collect::<Vec<_>>();
		// Add the required extensions for the Vulkan validation.
		if VALIDATION_ENABLED {
			extensions.push(Extension::KhrGetDebugUtils.name());
		}
		// Add the required extensions for the Vulkan portability.
		if cfg!(target_os = "macos") {
			// Allow Query extended physical device properties
			extensions.push(Extension::KhrGetPhysicalDeviceProperties2.name());
			//  Enable macOS support for the physical device
			extensions.push(Extension::KhrPortabilityEnumeration.name());
		}
		extensions
	}
	/// Checks if the layers to be used are available in the system.
	///
	/// # Parameters
	/// - `available_layers`: The layers available in the system (queried through the Vulkan [`Entry`]).
	/// - `instance_layers`: The layers to be used in the [`Instance`], configured in the instance creation.
	///
	/// # Returns
	/// - `true` if all the layers are available (all the layers in `instance_layers` are in `available_layers`).
	/// - `false` if any of the layers are not available
	fn are_layers_available(available_layers: HashSet<LayerStr>, instance_layers: Vec<LayerStr>) -> bool {
		instance_layers.iter().all(|layer| available_layers.contains(layer))
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
	/// - `Ok(())` if the physical device supports everything we require.
	/// - Returns `Err(anyhow::Error)` if the physical device does not support everything we require.
	/// # Arguments
	/// - `physical_device` - The physical device to check.
	fn check_physical_device(&self, physical_device: PhysicalDevice) -> anyhow::Result<()> {
		log::debug!("Checking physical device suitability.");
		let properties = unsafe {
			self.instance.get_physical_device_properties(physical_device)
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
				let physical_device = PhysicalDevice::new(vk_physical_device,
					self.instance.get_physical_device_properties(vk_physical_device));

				if let Err(error) = self.check_physical_device(physical_device) {
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
