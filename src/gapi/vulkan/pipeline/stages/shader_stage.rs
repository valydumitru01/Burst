use log::debug;
use vulkanalia::vk;
use vulkanalia::vk::{HasBuilder, ShaderModule, ShaderStageFlags};
use crate::gapi::vulkan::pipeline::shaders::Shader;

#[derive(Debug)]
struct ShaderStageConfig {
    shader: ShaderModule,
    stage: ShaderStageFlags,
    name: &'static str,
}
pub struct ShaderStage {
    stage: vk::PipelineShaderStageCreateInfo,
}

impl ShaderStage {
    pub fn new(shader: &Shader, stage_flag: ShaderStageFlags) -> Self {
        let stage = stage_flag;
        let shader = shader.get_vk();
        let name = "main\0".as_bytes();

        let stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(stage)
            .module(shader)
            .name(name)
            .build();

        debug!("Creating PipelineShaderStageCreateInfo struct: {stage:#?}");

        Self {
            stage: stage,
        }
    }

    pub fn get_stage(&self) -> &vk::PipelineShaderStageCreateInfo {
        &self.stage
    }
}
