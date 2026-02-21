use crate::gapi::vulkan::core::instance::Instance;
use crate::gapi::vulkan::core::queues::{QueueRequest, Queues};
use crate::gapi::vulkan::core::real_device::RealDevice;
use crate::gapi::vulkan::core::surface::Surface;
use crate::gapi::vulkan::enums::extensions::DeviceExtension;
use anyhow::Context;
use log::{info, trace};
use vulkanalia::vk::{
    Cast, DeviceV1_0, GraphicsPipelineCreateInfo, HasBuilder, ImageViewCreateInfoBuilder,
    KhrSwapchainExtension, PhysicalDeviceFeatures, Pipeline, PipelineCache, Queue,
    SwapchainCreateInfoKHR, SwapchainKHR,
};
use vulkanalia::{vk, Device};

/// Wraps the Vulkan logical device, and the queue handles it owns.
///
/// This object is responsible for:
/// - Creating the Vulkan device from a chosen physical device.
/// - Finding and storing all queue handles (graphics, present, etc.) according to user requests.
/// - Destroying the device (and by extension, the queues) at shutdown.
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
        let resolved_families = Queues::resolve_queue_requests(real_device, surface, requests)
            .with_context(|| format!("Failed to resolve queue requests: {:?}", requests))?;

        let queue_infos = Queues::create_queue_infos(&resolved_families);

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

        let queues = Queues::new(&device, &resolved_families)?;

        Ok(Self { device, queues })
    }

    fn get_vk_queue(&self, family_index: u32, queue_index: u32) -> Queue {
        unsafe { self.device.get_device_queue(family_index, queue_index) }
    }

    pub fn create_graphics_pipelines(
        &self,
        pipeline_cache: PipelineCache,
        create_info: &[impl Cast<Target = GraphicsPipelineCreateInfo> + std::fmt::Debug],
    ) -> anyhow::Result<Vec<Pipeline>> {
        trace!(
            "Calling create_graphics_pipelines with info: {:?}",
            create_info
        );
        let (pipelines, success_code) = unsafe {
            self.device
                .create_graphics_pipelines(pipeline_cache, create_info, None)
                .map_err(|e| anyhow::anyhow!("Failed to create graphics pipeline: {}", e))?
        };

        let () = match success_code {
            vk::SuccessCode::SUCCESS => (),
            vk::SuccessCode::NOT_READY => info!("Pipeline creation not ready"),
            vk::SuccessCode::TIMEOUT => info!("Pipeline creation timed out"),
            vk::SuccessCode::EVENT_SET => info!("Pipeline creation event set"),
            vk::SuccessCode::EVENT_RESET => info!("Pipeline creation event reset"),
            vk::SuccessCode::INCOMPLETE => info!("Pipeline creation incomplete"),
            vk::SuccessCode::PIPELINE_COMPILE_REQUIRED => {
                info!("Pipeline compilation required")
            }
            vk::SuccessCode::SUBOPTIMAL_KHR => info!("Pipeline creation suboptimal"),
            vk::SuccessCode::THREAD_IDLE_KHR => {
                info!("Pipeline creation thread idle")
            }
            vk::SuccessCode::THREAD_DONE_KHR => {
                info!("Pipeline creation thread done")
            }
            vk::SuccessCode::OPERATION_DEFERRED_KHR => {
                info!("Pipeline creation operation deferred")
            }
            vk::SuccessCode::OPERATION_NOT_DEFERRED_KHR => {
                info!("Pipeline creation operation not deferred")
            }
            vk::SuccessCode::INCOMPATIBLE_SHADER_BINARY_EXT => {
                info!("Pipeline creation incompatible shader binary")
            }
            vk::SuccessCode::PIPELINE_BINARY_MISSING_KHR => {
                info!("Pipeline creation binary missing")
            }
            _ => info!(
                "Pipeline creation failed with unknown success code: {:?}",
                success_code
            ),
        };

        Ok(pipelines)
    }
    pub fn create_pipeline_layout(
        &self,
        create_info: &vk::PipelineLayoutCreateInfo,
    ) -> anyhow::Result<vk::PipelineLayout> {
        trace!(
            "Calling create_pipeline_layout with info: {:?}",
            create_info
        );
        unsafe {
            self.device
                .create_pipeline_layout(create_info, None)
                .map_err(|e| anyhow::anyhow!("Failed to create pipeline layout: {}", e))
        }
    }

    pub fn destroy_pipeline(&self, pipeline: vk::Pipeline) {
        trace!("Calling destroy_pipeline for pipeline: {:?}", pipeline);
        unsafe {
            self.device.destroy_pipeline(pipeline, None);
        }
    }

    pub fn destroy_pipeline_layout(&self, layout: vk::PipelineLayout) {
        trace!(
            "Calling destroy_pipeline_layout for pipeline layout: {:?}",
            layout
        );
        unsafe {
            self.device.destroy_pipeline_layout(layout, None);
        }
    }

    pub fn create_swapchain_khr(
        &self,
        info: &SwapchainCreateInfoKHR,
    ) -> anyhow::Result<SwapchainKHR> {
        trace!("Calling create_swapchain_khr with info: {:?}", info);
        unsafe {
            self.device
                .create_swapchain_khr(info, None)
                .map_err(|e| anyhow::anyhow!("Failed to create swapchain: {}", e))
        }
    }

    pub fn create_render_pass(
        &self,
        create_info: &vk::RenderPassCreateInfo,
    ) -> anyhow::Result<vk::RenderPass> {
        trace!("Calling create_render_pass with info: {:?}", create_info);
        unsafe {
            self.device
                .create_render_pass(create_info, None)
                .map_err(|e| anyhow::anyhow!("Failed to create render pass: {}", e))
        }
    }

    pub fn destroy_render_pass(&self, render_pass: vk::RenderPass) {
        trace!(
            "Calling destroy_render_pass for render pass: {:?}",
            render_pass
        );
        unsafe {
            self.device.destroy_render_pass(render_pass, None);
        }
    }

    pub fn create_shader_module(
        &self,
        create_info: &vk::ShaderModuleCreateInfo,
    ) -> anyhow::Result<vk::ShaderModule> {
        trace!("Calling create_shader_module with info: {:?}", create_info);
        unsafe {
            self.device
                .create_shader_module(create_info, None)
                .map_err(|e| anyhow::anyhow!("Failed to create shader module: {}", e))
        }
    }

    pub fn destroy_shader_module(&self, shader_module: vk::ShaderModule) {
        trace!(
            "Calling destroy_shader_module for shader module: {:?}",
            shader_module
        );
        unsafe {
            self.device.destroy_shader_module(shader_module, None);
        }
    }

    pub fn destroy_swapchain_khr(&self, swapchain: SwapchainKHR) {
        trace!(
            "Calling destroy_swapchain_khr for swapchain: {:?}",
            swapchain
        );
        unsafe {
            self.device.destroy_swapchain_khr(swapchain, None);
        }
    }

    pub fn destroy_image_view(&self, image_view: vk::ImageView) {
        trace!(
            "Calling destroy_image_view for image view: {:?}",
            image_view
        );
        unsafe {
            self.device.destroy_image_view(image_view, None);
        }
    }

    pub(crate) fn create_image_view(
        &self,
        create_info: &ImageViewCreateInfoBuilder,
    ) -> anyhow::Result<vk::ImageView> {
        trace!("Calling create_image_view with info: {:?}", create_info);
        unsafe {
            self.device
                .create_image_view(create_info, None)
                .map_err(|e| anyhow::anyhow!("Failed to create image view: {}", e))
        }
    }

    pub fn get_swapchain_images_khr(
        &self,
        swapchain: SwapchainKHR,
    ) -> anyhow::Result<Vec<vk::Image>> {
        trace!(
            "Calling get_swapchain_images_khr for swapchain: {:?}",
            swapchain
        );
        unsafe {
            self.device
                .get_swapchain_images_khr(swapchain)
                .map_err(|e| anyhow::anyhow!("Failed to get swapchain images: {}", e))
        }
    }

    pub fn create_framebuffer(
        &self,
        create_info: &vk::FramebufferCreateInfo,
    ) -> anyhow::Result<vk::Framebuffer> {
        trace!("Calling create_framebuffer with info: {:?}", create_info);
        unsafe {
            self.device
                .create_framebuffer(create_info, None)
                .map_err(|e| anyhow::anyhow!("Failed to create framebuffer: {}", e))
        }
    }

    pub fn destroy_framebuffer(&self, framebuffer: vk::Framebuffer) {
        trace!(
            "Calling destroy_framebuffer for framebuffer: {:?}",
            framebuffer
        );
        unsafe {
            self.device.destroy_framebuffer(framebuffer, None);
        }
    }

    pub fn create_command_pool(
        &self,
        create_info: &vk::CommandPoolCreateInfo,
    ) -> anyhow::Result<vk::CommandPool> {
        trace!("Calling create_command_pool with info: {:?}", create_info);
        unsafe {
            self.device
                .create_command_pool(create_info, None)
                .map_err(|e| anyhow::anyhow!("Failed to create command pool: {}", e))
        }
    }

    pub fn destroy_command_pool(&self, command_pool: vk::CommandPool) {
        trace!(
            "Calling destroy_command_pool for command pool: {:?}",
            command_pool
        );
        unsafe {
            self.device.destroy_command_pool(command_pool, None);
        }
    }

    pub fn allocate_command_buffers(
        &self,
        create_info: &vk::CommandBufferAllocateInfo,
    ) -> anyhow::Result<Vec<vk::CommandBuffer>> {
        trace!("Calling create_command_buffers with info: {:?}", create_info);
        unsafe {
            self.device
                .allocate_command_buffers(create_info)
                .map_err(|e| anyhow::anyhow!("Failed to allocate command buffers: {}", e))
        }
    }

    pub fn begin_command_buffer(
        &self,
        command_buffer: vk::CommandBuffer,
        begin_info: &vk::CommandBufferBeginInfo,
    ) -> anyhow::Result<()> {
        trace!(
            "Calling begin_command_buffer for command buffer: {:?} with info: {:?}",
            command_buffer,
            begin_info
        );
        unsafe {
            self.device
                .begin_command_buffer(command_buffer, begin_info)
                .map_err(|e| anyhow::anyhow!("Failed to begin command buffer: {}", e))
        }
    }

    pub fn draw(&self, command_buffer: vk::CommandBuffer, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
        trace!(
            "Calling draw for command buffer: {:?} with vertex count: {}, instance count: {}, first vertex: {}, first instance: {}",
            command_buffer,
            vertex_count,
            instance_count,
            first_vertex,
            first_instance
        );
        unsafe {
            self.device.cmd_draw(command_buffer, vertex_count, instance_count, first_vertex, first_instance);
        }
    }

    pub fn end_command_buffer(&self, command_buffer: vk::CommandBuffer) -> anyhow::Result<()> {
        trace!(
            "Calling end_command_buffer for command buffer: {:?}",
            command_buffer
        );
        unsafe {
            self.device
                .end_command_buffer(command_buffer)
                .map_err(|e| anyhow::anyhow!("Failed to end command buffer: {}", e))
        }
    }

    pub fn bind_pipeline(
        &self,
        command_buffer: vk::CommandBuffer,
        pipeline_bind_point: vk::PipelineBindPoint,
        pipeline: vk::Pipeline,
    ) {
        trace!(
            "Calling bind_pipeline for command buffer: {:?} with pipeline: {:?} at bind point: {:?}",
            command_buffer,
            pipeline,
            pipeline_bind_point
        );
        unsafe {
            self.device.cmd_bind_pipeline(command_buffer, pipeline_bind_point, pipeline);
        }
    }

    pub fn begin_render_pass(
        &self,
        command_buffer: vk::CommandBuffer,
        begin_info: &vk::RenderPassBeginInfo,
        contents: vk::SubpassContents,
    ) {
        trace!(
            "Calling begin_render_pass with info: {:?} and contents: {:?}",
            begin_info,
            contents
        );
        unsafe {
            self.device.cmd_begin_render_pass(
                command_buffer, &begin_info, contents);

        }
    }

    pub fn end_render_pass(&self, command_buffer: vk::CommandBuffer) {
        trace!(
            "Calling end_render_pass for command buffer: {:?}",
            command_buffer
        );
        unsafe {
            self.device.cmd_end_render_pass(command_buffer);
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
