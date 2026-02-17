use log::debug;
use vulkanalia::vk;
use vulkanalia::vk::HasBuilder;


#[derive(Debug)]
struct ColorBlendAttConfig {
    color_write_mask: vk::ColorComponentFlags,
    blend_enable: bool,
    src_color_blend_factor: vk::BlendFactor,
    dst_color_blend_factor: vk::BlendFactor,
    color_blend_op: vk::BlendOp,
    src_alpha_blend_factor: vk::BlendFactor,
    dst_alpha_blend_factor: vk::BlendFactor,
    alpha_blend_op: vk::BlendOp,
}

#[derive(Debug)]
struct ColorBlendStateConfig {
    logic_op_enable: bool,
    logic_op: vk::LogicOp,
    attachments: Vec<vk::PipelineColorBlendAttachmentStateBuilder>,
    blend_constants: [f32; 4],
}
pub struct ColorBlendingStage{
    color_blend_state: vk::PipelineColorBlendStateCreateInfo,
}

impl ColorBlendingStage {
    pub fn new() -> Self {
        // Color Blending
        // After a fragment shader has returned a color, it needs to be combined with the color that
        // is already in the framebuffer. This transformation is known as color blending and there
        // are two ways to do it:
        // - Mix the old and new value to produce a final color
        // - Combine the old and new value using a bitwise operation

        // vk::PipelineColorBlendAttachmentState contains the configuration per attached framebuffer
        // and the second struct
        // We use alpha blending, where we want the new color to be blended with the old color based
        // on its opacity
        // Alpha blending's pseudocode looks like this:
        // final_color.rgb = new_alpha * new_color + (1 - new_alpha) * old_color;
        // final_color.a = new_alpha.a;
        let color_blend_config = ColorBlendAttConfig {
            color_write_mask: vk::ColorComponentFlags::all(),
            blend_enable: true,
            src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
        };

        debug!("Creating color_blend with config: {:#?}", color_blend_config);
        let attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(color_blend_config.color_write_mask)
            .blend_enable(color_blend_config.blend_enable)
            .src_color_blend_factor(color_blend_config.src_color_blend_factor)
            .dst_color_blend_factor(color_blend_config.dst_color_blend_factor)
            .color_blend_op(color_blend_config.color_blend_op)
            .src_alpha_blend_factor(color_blend_config.src_alpha_blend_factor)
            .dst_alpha_blend_factor(color_blend_config.dst_alpha_blend_factor)
            .alpha_blend_op(color_blend_config.alpha_blend_op);


        // The second structure references the array of structures for all of the framebuffers and
        // allows you to set blend constants that you can use as blend factors in the aforementioned
        // calculations.
        let color_blend_state_config = ColorBlendStateConfig {
            logic_op_enable: false,
            logic_op: vk::LogicOp::COPY,
            attachments: vec![attachment],
            blend_constants: [0.0, 0.0, 0.0, 0.0],
        };
        debug!("Creating color_blend_state with config: {:#?}", color_blend_state_config);
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(color_blend_state_config.logic_op_enable)
            .logic_op(color_blend_state_config.logic_op)
            .attachments(&color_blend_state_config.attachments)
            .blend_constants(color_blend_state_config.blend_constants);

        Self {
            color_blend_state: color_blend_state.build(),
        }
    }

    pub fn get_color_blend_state(&self) -> &vk::PipelineColorBlendStateCreateInfo {
        &self.color_blend_state
    }
}