use crate::{debug_success, info_success};

use crate::gapi::vulkan::commands::command_buffers::CommandBuffers;
use crate::gapi::vulkan::commands::command_pool::CommandPool;
use crate::gapi::vulkan::core::entry::Entry;
use crate::gapi::vulkan::core::instance::Instance;
use crate::gapi::vulkan::core::logical_device::LogicalDevice;
use crate::gapi::vulkan::core::queues::{QueueCapability, QueueRequest};
use crate::gapi::vulkan::core::real_device::RealDevice;
use crate::gapi::vulkan::core::surface::Surface;
use crate::gapi::vulkan::enums::extensions::{DeviceExtension, PORTABILITY_MACOS_VERSION};
use crate::gapi::vulkan::memory::framebuffer::Framebuffer;
use crate::gapi::vulkan::memory::swapchain::Swapchain;
use crate::gapi::vulkan::pipeline::pipeline::Pipeline;
use crate::gapi::vulkan::pipeline::render_pass::MyRenderPass;
use crate::gapi::vulkan::pipeline::viewport::Viewport;
use crate::window::MyWindow;
use anyhow::{anyhow, bail, Context};
use log::{debug, info, trace, warn};
use thiserror::Error;
use vulkanalia::vk;
use vulkanalia::vk::{HasBuilder, ShaderStageFlags};

const VERT_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/vert.spv"));
const FRAG_DATA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/frag.spv"));

/// Our Vulkan app.
pub struct App {
    entry: Entry,
    instance: Instance,
    device: LogicalDevice,
    surface: Surface,
    swapchain: Swapchain,
    render_pass: MyRenderPass,
    pipeline: Pipeline,
    framebuffers: Vec<Framebuffer>,
    command_pool: CommandPool,
    command_buffers: CommandBuffers,
}
#[derive(Debug, Error)]
#[error("Missing {0}.")]
pub(crate) struct SuitabilityError(pub &'static str);

/// # Vulkan App
/// This handles everything in Vulkan.
///
/// Vulkan is a wrapper around the Vulkan Driver, which is a platform-agnostic abstraction for
/// the actual GPU hardware interface.
impl App {
    /// Creates our Vulkan app.
    pub fn new(window: &MyWindow) -> anyhow::Result<Self> {
        info!("Creating Entry...");
        let entry = Entry::new()?;
        info_success!("Entry Created! Loader Version: {}", entry.version()?);
        info!("Creating Instance...");
        let instance = Instance::new(&entry, window)?;
        info_success!("Instance Created!");
        info!("Creating Surface...");
        let surface = Surface::new(&instance, window)?;
        info_success!("Surface Created!");
        let requests: Vec<QueueRequest> = vec![QueueRequest {
            capabilities: vec![QueueCapability::Graphics],
            require_present: true,
            count: 1,
        }];
        info!("Required Queues: {:?}", requests);

        let mut required_extensions = vec![DeviceExtension::KhrSwapchain];
        if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
            required_extensions.push(DeviceExtension::KhrPortabilitySubset);
        }
        info!("Selecting physical device...");
        let real_device = Self::pick_real_device(&instance, &surface, window)?;
        info_success!(
            "Physical device selected: {}",
            real_device.get_properties().device_name
        );
        if real_device.get_properties().device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
            warn!("This selected physical device is not discrete.");
        }
        info!("Creating logical device...");
        let device = LogicalDevice::new(
            &real_device,
            &instance,
            &surface,
            &requests,
            &required_extensions,
        )?;
        info_success!("Logical device created!");

        info!("Creating swapchain...");
        let swapchain = Swapchain::new(&window, &real_device, &device, &surface).with_context(|| "Failed to create swapchain.")?;
        info_success!("Swapchain created!");

        info!("Creating viewport...");
        let viewport = Viewport::new(&swapchain);
        info_success!("Viewport created!");

        info!("Creating render pass...");
        let render_pass = MyRenderPass::new(&swapchain, &device).with_context(|| "Failed to create render pass.")?;
        info_success!("Render pass created!");

        info!("Creating pipeline...");
        let pipeline = Pipeline::new(&device, &viewport, &render_pass).with_context(|| "Failed to create pipeline.")?;
        info_success!("Pipeline created!");

        info!("Creating framebuffers...");
        let framebuffers = swapchain
            .image_views
            .iter()
            .map(|image_view| {
                let attachments = std::slice::from_ref(image_view);
                Framebuffer::new(&render_pass, attachments, &swapchain, &device)
            })
            .collect::<Vec<Framebuffer>>();
        info_success!("Framebuffers created!");

        info!("Creating command pool...");
        let command_pool = CommandPool::new(&device).with_context(|| "Failed to create command pool.")?;
        info_success!("Command pool created!");

        info!("Creating command buffers...");
        let command_buffers = CommandBuffers::new(&device, &framebuffers, &command_pool)
            .with_context(|| "Failed to create command buffers.")?;
        info_success!("CommandBuffers created!");


        let app = Self {
            entry,
            instance,
            device,
            surface,
            swapchain,
            render_pass,
            pipeline,
            framebuffers,
            command_pool,
            command_buffers,
        };
        info!("Recording command buffers...");
        app.record_command_buffers().with_context(|| "Failed to record command buffers.")?;
        info_success!("Command buffers recorded!");

        Ok(app)
    }

    /// Function that returns a `SuitabilityError` if a supplied physical device does not support everything we require.
    /// # Errors
    /// It returns a `SuitabilityError` if the physical device does not support everything we require.
    /// # Returns
    /// - `Ok(())` if the physical device supports everything we require.
    /// - Returns `Err(anyhow::Error)` if the physical device does not support everything we require.
    /// # Arguments
    /// - `real_device` - The physical device to check.
    fn check_real_device(
        real_device: &RealDevice,
        surface: &Surface,
        window: &MyWindow,
    ) -> anyhow::Result<()> {
        let device_name = real_device.get_properties().device_name.to_string();
        trace!("Checking \"{device_name}\"'s features...");
        // Optional features like texture compression, 64-bit floats, and multi-viewport rendering.
        let features = real_device.get_features();
        // We require support for geometry rendering.
        if features.geometry_shader != vk::TRUE {
            bail!(SuitabilityError("Missing geometry shader support."));
        }

        info!("{device_name} supports geometry rendering.");
        info!("Checking \"{device_name}\"'s extensions...");
        let supported_extensions =
            real_device
                .supported_extensions()
                .with_context(|| {
                    format!(
                        "Failed to get supported extensions for physical device \"{device_name}\".",
                    )
                })?
                .iter()
                .map(|sup_ext| sup_ext.extension_name)
                .collect::<Vec<_>>();

        // Not all graphics cards are capable of presenting images directly to a screen for various
        // reasons, for example because they are designed for servers and don't have any display
        // outputs.
        // Therefore, we need to check if the device supports the required extensions for
        // presenting images to a screen.
        let required_extensions = vec![DeviceExtension::KhrSwapchain];
        for ext in &required_extensions {
            let ext_name = ext.name_buf();
            if supported_extensions.contains(&ext.name_buf()) {
                info!("{device_name} supports required extension {ext_name}",);
            } else {
                info!("{device_name} does NOT support required extension {ext_name}",);
                bail!(SuitabilityError("Missing required device extensions."));
            }
        }

        // It is mandatory to check the swapchain information AFTER checking for the swapchain
        // extension, swapchain support is only available if the extension is supported.
        let swapchain = real_device.get_swapchain_info(surface)?;
        if swapchain.formats.is_empty() || swapchain.present_modes.is_empty() {
            bail!(SuitabilityError("Insufficient swapchain support."));
        }

        trace!("{:?} is supported by our app!", device_name);
        Ok(())
    }

    fn pick_real_device<'a>(
        instance: &'a Instance,
        surface: &Surface,
        window: &MyWindow,
    ) -> anyhow::Result<RealDevice<'a>> /* Returned RealDevice's lifetime is bound to Instance */
    {
        let available_devices = instance.enumerate_real_devices()?;
        debug!(
            "Picking physical device between available devices: {:?}.",
            available_devices
                .iter()
                .map(|d| d.get_properties().device_name.to_string())
                .collect::<Vec<_>>()
        );
        for real_dev in available_devices {
            let properties = real_dev.get_properties();
            if let Err(error) = Self::check_real_device(&real_dev, surface, window) {
                debug!(
                    "Skipping physical device (`{}`): {error}",
                    properties.device_name
                );
            } else {
                debug!("Selected physical device (`{}`).", properties.device_name);
                return anyhow::Ok(real_dev);
            }
        }

        Err(anyhow!("Failed to find suitable physical device."))
    }

    fn record_command_buffers(&self) -> anyhow::Result<()> {
        self.command_buffers.record_all(
            &self.device,
            &self.framebuffers,
            |command_buffer, framebuffer| {
                // 1. Start Render Pass
                self.render_pass.begin(&self.device, framebuffer, command_buffer, &self.swapchain);

                // 2. Bind Pipeline
                self.pipeline.bind(&self.device, command_buffer);

                // 3. Draw
                unsafe {
                    self.device.draw(*command_buffer.get_vk(), 3, 1, 0, 0);
                }

                // 4. End Render Pass
                self.render_pass.end(&self.device, *command_buffer.get_vk());

                Ok(())
            },
        )
    }

    fn select_swapchain_surface_format() {}
    /// Renders a frame for our Vulkan app.
    pub fn render(&mut self, window: &MyWindow) -> anyhow::Result<()> {


        Ok(())
    }

    /// Destroys our Vulkan app.
    pub fn destroy(&self) {
        info!("Destroying Vulkan App...");
        self.command_pool.destroy(&self.device);
        self.framebuffers
            .iter()
            .for_each(|framebuffer| framebuffer.destroy(&self.device));
        self.pipeline.destroy(&self.device);
        self.render_pass.destroy(&self.device);
        self.swapchain.destroy(&self.device);
        self.surface.destroy(&self.instance);
        self.device.destroy();
        self.instance.destroy();
    }
}
