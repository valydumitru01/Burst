use anyhow::Context;
use vulkanalia::bytecode::Bytecode;
use vulkanalia::vk;
use vulkanalia::vk::HasBuilder;
use crate::gapi::vulkan::logical_device::LogicalDevice;

pub(crate) struct Shader{
    vk_shader_module: vk::ShaderModule
}

impl Shader{
    pub fn new(device: &LogicalDevice, bytecode: &[u8]) -> anyhow::Result<Self> {
        // Vulkan expects the bytecodes in u32 format, so we need to convert the bytecode from &[u8] to &[u32].
        // luckily, Vulkanalia provides a Bytecode struct that handles this for us.
        // It will also check alignment errors.
        let bytecode = Bytecode::new(bytecode).with_context(|| "Failed to create bytecode from shader bytecode")?;
        let info = vk::ShaderModuleCreateInfo::builder()
            .code(bytecode.code())
            .code_size(bytecode.code_size());
        let vk_shader_module = device.create_shader_module(&info)
            .with_context(|| "Failed to create shader module")?;

        Ok(Self {
            vk_shader_module
        })
    }

    pub fn get_vk(&self) -> vk::ShaderModule {
        self.vk_shader_module
    }


    pub fn destroy(&self, device: &LogicalDevice) {
        unsafe {
            device.destroy_shader_module(self.vk_shader_module);
        }
    }
}
