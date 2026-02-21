use crate::gapi::vulkan::commands::command_pool::CommandPool;
use crate::gapi::vulkan::core::logical_device::LogicalDevice;
use crate::gapi::vulkan::memory::framebuffer::Framebuffer;
use crate::gapi::vulkan::memory::swapchain::Swapchain;
use crate::gapi::vulkan::pipeline::render_pass::MyRenderPass;
use crate::info_success;
use anyhow::Context;
use log::{debug, info};
use std::mem::swap;
use vulkanalia::vk;
use vulkanalia::vk::HasBuilder;

pub struct CommandBuffer {
    command_buffer: vk::CommandBuffer,
}

impl CommandBuffer {
    pub fn new(command_buffer: vk::CommandBuffer) -> Self {
        Self { command_buffer }
    }

    pub fn get_vk(&self) -> &vk::CommandBuffer {
        &self.command_buffer
    }

    pub fn begin(&self, device: &LogicalDevice) -> anyhow::Result<()> {
        // The flags parameter specifies how we're going to use the command buffer.
        // The following values are available:
        //
        // vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT – The command buffer will be rerecorded right after
        // executing it once.
        // vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE – This is a secondary command buffer that will be
        // entirely within a single render pass.
        // vk::CommandBufferUsageFlags::SIMULTANEOUS_USE – The command buffer can be resubmitted while it is also
        // already pending execution.
        // For now, it is empty
        let flags = vk::CommandBufferUsageFlags::empty();

        // The inheritance_info parameter is only relevant for secondary command buffers. It specifies which state
        // to inherit from the calling primary command buffers.
        let inheritance = vk::CommandBufferInheritanceInfo::builder().build();
        debug!(
            "Created CommandBufferInheritanceInfo struct: {:#?}",
            inheritance
        );

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(flags) // Optional.
            .inheritance_info(&inheritance) // Optional.
            .build();
        debug!("Created CommandBufferBeginInfo struct: {:#?}", info);

        device
            .begin_command_buffer(self.command_buffer, &info)
            .with_context(|| format!("Failed to begin recording command buffer {:?}", self.command_buffer))?;

        Ok(())
    }

    pub fn end(&self, device: &LogicalDevice) -> anyhow::Result<()> {
        device
            .end_command_buffer(self.command_buffer)
            .with_context(|| "Failed to end recording command buffer")?;
        Ok(())
    }

    pub fn record<F>(
        &self,
        device: &LogicalDevice,
        framebuffer: &Framebuffer,
        recording_logic: F,
    ) -> anyhow::Result<()>
    where
        F: FnOnce(&Self, &Framebuffer) -> anyhow::Result<()>,
    {
        self.begin(device)?;
        recording_logic(self, framebuffer)?;
        self.end(device)?;
        Ok(())
    }
}

pub struct CommandBuffers {
    command_buffers: Vec<CommandBuffer>,
}

impl CommandBuffers {
    pub fn new(
        device: &LogicalDevice,
        framebuffers: &[Framebuffer],
        command_pool: &CommandPool,
    ) -> anyhow::Result<Self> {
        // The level parameter specifies if the allocated command buffers are primary or secondary command buffers.
        //
        // -vk::CommandBufferLevel::PRIMARY – Can be submitted to a queue for execution, but cannot be called from other
        // command buffers.
        // - vk::CommandBufferLevel::SECONDARY – Cannot be submitted directly, but can be called from primary command
        // buffers.
        let level = vk::CommandBufferLevel::PRIMARY;
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool.get_vk())
            .level(level)
            .command_buffer_count(framebuffers.len() as u32)
            .build();

        debug!(
            "Created CommandBufferAllocateInfo struct: {:#?}",
            allocate_info
        );

        let command_buffers = device
            .allocate_command_buffers(&allocate_info)
            .with_context(|| "Failed to allocate command buffers")?
            .iter()
            .map(|command_buffer| CommandBuffer::new(*command_buffer))
            .collect::<Vec<CommandBuffer>>();

        Ok(Self { command_buffers })
    }

    pub fn get_buffers(&self) -> &[CommandBuffer] {
        &self.command_buffers
    }

    /// Records commands for all buffers.
    /// The `recording_logic` closure is called for each image index.
    pub fn record_all<F>(
        &self,
        device: &LogicalDevice,
        framebuffers: &[Framebuffer],
        recording_logic: F,
    ) -> anyhow::Result<()>
    where   F: Fn(&CommandBuffer, &Framebuffer) -> anyhow::Result<()>,
    {
        for (i, command_buffer) in self.command_buffers.iter().enumerate() {
            let framebuffer = &framebuffers[i];
            command_buffer.record(device, framebuffer, |cb, fb| recording_logic(cb, fb))?;
        }
        Ok(())
    }

}
