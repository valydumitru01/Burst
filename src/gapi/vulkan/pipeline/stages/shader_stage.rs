use log::debug;
use crate::gapi::vulkan::rendering::shaders::Shader;
use vulkanalia::vk;
use vulkanalia::vk::{HasBuilder, ShaderModule, ShaderStageFlags};


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
        let shader_stage_config = ShaderStageConfig {
            shader: shader.get_vk(),
            stage: stage_flag,
            name: "main\0",
        };
        debug!("Creating shader stage with config: {shader_stage_config:#?}");

        let stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(shader_stage_config.stage)
            .module(shader_stage_config.shader)
            .name(shader_stage_config.name.as_bytes());

        Self {
            stage: stage.build(),
        }
    }

    pub fn get_stage(&self) -> &vk::PipelineShaderStageCreateInfo {
        &self.stage
    }

}