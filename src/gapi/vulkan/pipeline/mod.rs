pub mod stages;
use crate::gapi::vulkan::logical_device::LogicalDevice;
use crate::gapi::vulkan::pipeline::stages::color_blending_stage::ColorBlendingStage;
use crate::gapi::vulkan::pipeline::stages::per_fragment_tests_stage::PerFragmentTestsStage;
use crate::gapi::vulkan::pipeline::stages::rasterization_stage::RasterizationStage;
use crate::gapi::vulkan::pipeline::stages::shader_stage::ShaderStage;
use crate::gapi::vulkan::render_pass::MyRenderPass;
use crate::gapi::vulkan::rendering::shaders::Shader;
use crate::gapi::vulkan::viewport::Viewport;
use anyhow::Context;
use vulkanalia::vk;
use vulkanalia::vk::{Handle, HasBuilder, ShaderStageFlags};

#[derive(Clone, Debug)]
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

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            // Defaults for now
            .vertex_binding_descriptions(&[] as &[vk::VertexInputBindingDescription])
            .vertex_attribute_descriptions(&[] as &[vk::VertexInputAttributeDescription]);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            // To draw voxels, we use point lists, where each vertex represents a single voxel.
            .topology(vk::PrimitiveTopology::POINT_LIST)
            .primitive_restart_enable(false);

        let layout_info = vk::PipelineLayoutCreateInfo::builder();

        let pipeline_layout = device.create_pipeline_layout(&layout_info)?;

        let vert_shader_stage = ShaderStage::new(&vert_shader_module, ShaderStageFlags::VERTEX);
        let rasterization_stage = RasterizationStage::new();

        let per_frag_tests_stage = PerFragmentTestsStage::new();
        let frag_shader_stage = ShaderStage::new(&frag_shader_module, ShaderStageFlags::FRAGMENT);
        let color_blending_stage = ColorBlendingStage::new();

        let vert_stage = vert_shader_stage.get_stage();
        let frag_stage = frag_shader_stage.get_stage();
        let stages = &[*vert_stage, *frag_stage];
        let info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(viewport.get_viewport_state())
            .rasterization_state(rasterization_stage.get_rasterization_state())
            .multisample_state(rasterization_stage.get_multisample_state())
            .color_blend_state(color_blending_stage.get_color_blend_state())
            .layout(pipeline_layout)
            .depth_stencil_state(per_frag_tests_stage.get_depth_stencil_state())
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

    pub fn destroy(&self, device: &LogicalDevice) {
        device.destroy_pipeline_layout(self.vk_pipeline_layout);
        device.destroy_pipeline(self.vk_pipeline);
    }
}
