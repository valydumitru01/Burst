use log::{debug, info};
use vulkanalia::vk;
use vulkanalia::vk::HasBuilder;


pub struct ColorBlendingStage{
    attachments: Vec<vk::PipelineColorBlendAttachmentState>,
}

impl ColorBlendingStage {
    pub fn new() -> Self {
        info!("Configuring color blending");
        // Color Blending
        // After a fragment shader has returned a color, it needs to be combined with the color that
        // is already in the framebuffer. This transformation is known as color blending and there
        // are two ways to do it:
        // - Mix the old and new value to produce a final color
        // - Combine the old and new value using a bitwise operation



        let color_write_mask = vk::ColorComponentFlags::all();
        let blend_enable = true;
        let src_color_blend_factor = vk::BlendFactor::SRC_ALPHA;
        let dst_color_blend_factor = vk::BlendFactor::ONE_MINUS_SRC_ALPHA;
        let color_blend_op = vk::BlendOp::ADD;
        let src_alpha_blend_factor = vk::BlendFactor::ONE;
        let dst_alpha_blend_factor = vk::BlendFactor::ZERO;
        let alpha_blend_op = vk::BlendOp::ADD;

        // vk::PipelineColorBlendAttachmentState contains the configuration per attached framebuffer
        // and the second struct
        // We use alpha blending, where we want the new color to be blended with the old color based
        // on its opacity
        // Alpha blending's pseudocode looks like this:
        // final_color.rgb = new_alpha * new_color + (1 - new_alpha) * old_color;
        // final_color.a = new_alpha.a;
        let attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(color_write_mask)
            .blend_enable(blend_enable)
            .src_color_blend_factor(src_color_blend_factor)
            .dst_color_blend_factor(dst_color_blend_factor)
            .color_blend_op(color_blend_op)
            .src_alpha_blend_factor(src_alpha_blend_factor)
            .dst_alpha_blend_factor(dst_alpha_blend_factor)
            .alpha_blend_op(alpha_blend_op)
            .build();
        debug!("Created PipelineColorBlendAttachmentState struct: {:#?}", &attachment);
        let attachments = vec![attachment];
        Self {
            attachments
        }
    }

    pub fn build_color_blend_state(&self) -> vk::PipelineColorBlendStateCreateInfo {

        let logic_op_enable = false;
        let logic_op = vk::LogicOp::COPY;
        let blend_constants = [0.0, 0.0, 0.0, 0.0];
        // The second structure references the array of structures for all of the framebuffers and
        // allows you to set blend constants that you can use as blend factors in the aforementioned
        // calculations.
        let color_blend = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(logic_op_enable)
            .logic_op(logic_op)
            .attachments(&self.attachments)
            .blend_constants(blend_constants)
            .build();
        debug!("Created PipelineColorBlendStateCreateInfo struct: {:#?}", &color_blend);
        color_blend
    }
}