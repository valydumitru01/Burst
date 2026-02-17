use crate::gapi::vulkan::logical_device::LogicalDevice;
use crate::gapi::vulkan::swapchain::Swapchain;
use anyhow::Context;
use log::debug;
use vulkanalia::vk;
use vulkanalia::vk::{Format, HasBuilder};

#[derive(Debug)]
struct AttachmentDescriptionConfig {
    format: vk::Format,
    samples: vk::SampleCountFlags,
    load_op: vk::AttachmentLoadOp,
    store_op: vk::AttachmentStoreOp,
    stencil_load_op: vk::AttachmentLoadOp,
    stencil_store_op: vk::AttachmentStoreOp,
    initial_layout: vk::ImageLayout,
    final_layout: vk::ImageLayout,
}

#[derive(Debug)]
struct AttachmentReferenceConfig {
    attachment: u32,
    layout: vk::ImageLayout,
}

#[derive(Debug)]
struct SubpassDescriptionConfig {
    pipeline_bind_point: vk::PipelineBindPoint,
    color_attachments: Vec<vk::AttachmentReference>,
}

/// RenderPass is a specification of:
/// - How many color and depth buffers there will be
/// - How many samples to use for each of them
/// - How their contents should be handled throughout the rendering operations
pub struct MyRenderPass {
    handle: vk::RenderPass,
}

impl MyRenderPass {
    pub fn new(swapchain: &Swapchain, device: &LogicalDevice) -> anyhow::Result<Self> {
        let col_att_config = AttachmentDescriptionConfig {
            // The format of the color attachment should match the format of the swapchain images,
            // and we're not doing anything with multisampling yet, so we'll stick to 1 sample.
            format: swapchain.format,
            samples: vk::SampleCountFlags::_1,
            // The load_op and store_op determine what to do with the data in the attachment before
            // rendering and after rendering.
            //
            // We have the following choices for load_op:
            // - vk::AttachmentLoadOp::LOAD – Preserve the existing contents of the attachment
            // - vk::AttachmentLoadOp::CLEAR – Clear the values to a constant at the start
            // - vk::AttachmentLoadOp::DONT_CARE – Existing contents are undefined; we don't care
            // about them
            // In our case we're going to use the clear operation to clear the framebuffer to black
            // before drawing a new frame.
            load_op: vk::AttachmentLoadOp::CLEAR,
            // There are only two possibilities for the store_op:
            // - vk::AttachmentStoreOp::STORE – Rendered contents will be stored in memory and can
            // be read later
            // - vk::AttachmentStoreOp::DONT_CARE – Contents of the framebuffer will be undefined
            // after the rendering operation
            // We're interested in seeing the rendered triangle on the screen, so we're going with
            // the store operation here.
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            // The load_op and store_op apply to color and depth data, and
            // stencil_load_op / stencil_store_op apply to stencil data.
            //
            // Our application won't do  anything with the stencil buffer, so the results of loading
            // and storing are irrelevant.
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            // Textures and framebuffers in Vulkan are represented by vk::Image objects with a
            // certain pixel format, however the layout of the pixels in memory can change based on
            // what you're trying to do with an image.
            //
            // Some of the most common layouts are:
            // - vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL – Images used as color attachment
            // - vk::ImageLayout::PRESENT_SRC_KHR – Images to be presented in the swapchain
            // - vk::ImageLayout::TRANSFER_DST_OPTIMAL – Images to be used as destination for a
            // memory copy operation
            //
            // initial_layout specifies which layout the image will have before the render pass
            // begins
            // Using vk::ImageLayout::UNDEFINED for initial_layout means that we don't care what
            // previous layout the image was in.
            initial_layout: vk::ImageLayout::UNDEFINED,
            // final_layout specifies the layout to automatically transition to when the render pass finishes
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        };

        debug!("Creating render pass with color attachment config: \n{col_att_config:#?}");
        let color_attachment = vk::AttachmentDescription::builder()
            .format(col_att_config.format)
            .samples(col_att_config.samples)
            .load_op(col_att_config.load_op)
            .store_op(col_att_config.store_op)
            .stencil_load_op(col_att_config.stencil_load_op)
            .stencil_store_op(col_att_config.stencil_store_op)
            .initial_layout(col_att_config.initial_layout)
            .final_layout(col_att_config.final_layout);

        // A single render pass can consist of multiple subpasses.
        // Subpasses are subsequent rendering operations that depend on the contents of framebuffers
        // in previous passes, for example a sequence of post-processing effects that are applied
        // one after another.
        // If these rendering operations are grouped into one render pass then Vulkan is able to
        // reorder the operations and conserve memory bandwidth for possibly better performance.
        // But for now, we're just going to have a single subpass that renders the triangle
        // directly to the swapchain images.
        let col_att_ref_config = AttachmentReferenceConfig {
            // The attachment parameter specifies which attachment to reference by its index in the
            // attachment descriptions array. Our array consists of a single vk::AttachmentDescription,
            // so its index is 0.
            attachment: 0,
            // The layout specifies which layout we would like the attachment to have during a
            // subpass that uses this reference.
            // Vulkan will automatically transition the attachment to this layout when the subpass
            // is started.
            // We intend to use the attachment to function as a color buffer and the
            // vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        debug!(
            "Creating render pass with color attachment reference config: \n{col_att_ref_config:#?}"
        );

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(col_att_ref_config.attachment)
            .layout(col_att_ref_config.layout);

        let subpass_config = SubpassDescriptionConfig {
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            // The index of the attachment in this array is directly referenced from the fragment
            // shader with the layout(location = 0) out vec4 outColor directive
            color_attachments: vec![*color_attachment_ref],
        };

        debug!("Creating render pass with color attachments: \n{subpass_config:#?}");
        let subpass = vk::SubpassDescription::builder()
            // Vulkan may also support compute subpasses in the future, so we have to be explicit
            // about this being a graphics subpass.
            .pipeline_bind_point(subpass_config.pipeline_bind_point)
            .color_attachments(&subpass_config.color_attachments);

        let attachments = &[color_attachment];
        let subpasses = &[subpass];
        debug!("Creating render pass with attachments: \
            \n{attachments:#?} \
            and subpasses: \
            \n{subpasses:#?}");
        let info = vk::RenderPassCreateInfo::builder()
            .attachments(attachments)
            .subpasses(subpasses);

        let render_pass = device
            .create_render_pass(&info)
            .with_context(|| "creating render pass with info: \n\t\"\"\"\n{info:#?}\n\t\"\"\"")?;

        Ok(Self {
            handle: render_pass,
        })
    }

    pub fn get_vk(&self) -> vk::RenderPass {
        self.handle
    }
}
