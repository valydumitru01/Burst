use crate::gapi::vulkan::commands::command_buffers::CommandBuffer;
use crate::gapi::vulkan::core::logical_device::LogicalDevice;
use crate::gapi::vulkan::pipeline::render_pass::MyRenderPass;
use crate::gapi::vulkan::pipeline::shaders::Shader;
use crate::gapi::vulkan::pipeline::stages::color_blending_stage::ColorBlendingStage;
use crate::gapi::vulkan::pipeline::stages::input_assembler_stage::InputAssemblerStage;
use crate::gapi::vulkan::pipeline::stages::per_fragment_tests_stage::PerFragmentTestsStage;
use crate::gapi::vulkan::pipeline::stages::rasterization_stage::RasterizationStage;
use crate::gapi::vulkan::pipeline::stages::shader_stage::ShaderStage;
use crate::gapi::vulkan::pipeline::viewport::Viewport;
use anyhow::Context;
use vulkanalia::vk;
use vulkanalia::vk::{Handle, HasBuilder, ShaderStageFlags};

pub struct Pipeline {
    vk_pipeline_layout: vk::PipelineLayout,
    vk_pipeline: vk::Pipeline,
}

impl Pipeline {
    pub fn new(
        device: &LogicalDevice,
        viewport: &Viewport,
        render_pass: &MyRenderPass,
    ) -> anyhow::Result<Self> {
        let vert = include_bytes!(concat!(env!("OUT_DIR"), "/vert.spv"));
        let frag = include_bytes!(concat!(env!("OUT_DIR"), "/frag.spv"));
        let vert_shader_module = Shader::new(&device, &vert[..])?;
        let frag_shader_module = Shader::new(&device, &frag[..])?;

        let input_assembly_stage = InputAssemblerStage::new();
        let vert_shader_stage = ShaderStage::new(&vert_shader_module, ShaderStageFlags::VERTEX);
        let rasterization_stage = RasterizationStage::new();
        let per_frag_tests_stage = PerFragmentTestsStage::new();
        let frag_shader_stage = ShaderStage::new(&frag_shader_module, ShaderStageFlags::FRAGMENT);
        let color_blending_stage = ColorBlendingStage::new();

        let vertex_input_state = input_assembly_stage.build_vertex_input_state();
        let input_assembly_state = input_assembly_stage.build_input_assembly_state();
        let color_blend_state = color_blending_stage.build_color_blend_state();
        let viewport_state = viewport.build_viewport_state();
        let rasterization_state = rasterization_stage.build_rasterization_state();
        let multisample_state = rasterization_stage.build_multisample_state();
        let depth_stencil_state = per_frag_tests_stage.build_depth_stencil_state();

        let vert_stage = vert_shader_stage.get_stage();
        let frag_stage = frag_shader_stage.get_stage();

        let layout_info = vk::PipelineLayoutCreateInfo::builder();
        let pipeline_layout = device.create_pipeline_layout(&layout_info)?;

        let stages = &[*vert_stage, *frag_stage];
        let info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_state)
            .layout(pipeline_layout)
            .depth_stencil_state(&depth_stencil_state)
            .render_pass(render_pass.get_vk())
            .subpass(0)
            .base_pipeline_handle(vk::Pipeline::null()) // Optional
            .base_pipeline_index(-1); // Optional

        let pipeline = device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[info])
            .with_context(|| "Failed to create graphics pipeline")?[0];

        // These need to live past pipeline creation, but can be destroyed immediately after.
        vert_shader_module.destroy(&device);
        frag_shader_module.destroy(&device);

        Ok(Pipeline {
            vk_pipeline_layout: pipeline_layout,
            vk_pipeline: pipeline,
        })
    }

    pub fn bind(&self, device: &LogicalDevice, command_buffer: &CommandBuffer) {
        device.bind_pipeline(
            *command_buffer.get_vk(),
            vk::PipelineBindPoint::GRAPHICS,
            self.vk_pipeline,
        );
    }

    pub fn destroy(&self, device: &LogicalDevice) {
        device.destroy_pipeline_layout(self.vk_pipeline_layout);
        device.destroy_pipeline(self.vk_pipeline);
    }
}
