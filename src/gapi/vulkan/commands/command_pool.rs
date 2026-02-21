use crate::gapi::vulkan::core::logical_device::LogicalDevice;
use anyhow::Context;
use log::debug;
use vulkanalia::vk;
use vulkanalia::vk::HasBuilder;

pub struct CommandPool {
    /// Command pools manage the memory that is used to store the buffers and command buffers are
    /// allocated from them
    command_pool: vk::CommandPool,
}

impl CommandPool {
    pub fn new(device: &LogicalDevice) -> anyhow::Result<Self> {
        let queues = device.get_queues();

        let info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::empty()) // Optional.
            .queue_family_index(queues.graphics_family_index).build();
        debug!("Created CommandPoolCreateInfo struct: {:#?}", info);
        let command_pool = device.create_command_pool(&info)
            .with_context(|| "Failed to create command pool")?;
        Ok(Self {
            command_pool,
        })
    }


    pub fn destroy(&self, device: &LogicalDevice) {
        unsafe {
            device.destroy_command_pool(self.command_pool);
        }
    }

    pub fn get_vk(&self) -> vk::CommandPool {
        self.command_pool
    }
}
