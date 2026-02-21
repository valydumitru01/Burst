use log::debug;
use vulkanalia::vk;
use vulkanalia::vk::HasBuilder;
use crate::gapi::vulkan::memory::swapchain::Swapchain;

#[derive(Debug)]
pub struct Viewport {
    /// Viewport must be kept alive as long as the pipeline that uses it is alive, because the
    /// pipeline holds a reference to it.
    viewport: vk::Viewport,
    /// Scissor must be kept alive as long as the pipeline that uses it is alive, because the
    /// pipeline holds a reference to it.
    scissor: vk::Rect2D,
}

impl Viewport {
    pub fn new(swapchain: &Swapchain) -> Self {
        // Viewport
        // The viewport is the region of the framebuffer that the output will be rendered to.
        // This will almost always be (0, 0) to (width, height)
        let viewport = vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(swapchain.extent.width as f32)
            .height(swapchain.extent.height as f32)
            // The min_depth and max_depth values specify the range of depth values to use for the
            // framebuffer.
            .min_depth(0.0)
            .max_depth(1.0)
            .build();
        debug!("Created Viewport struct: \n{viewport:#?}");

        // Scissor
        // The scissor rectangle defines the area of the framebuffer that will be affected by
        // rendering operations. Basically acts like a filter for pixels.
        // Could be used for example to render only to a specific part of the screen, or to
        // implement a split-screen effect.
        let scissor = vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(swapchain.extent)
            .build();
        debug!("Created Scissor (Rect2D) struct: \n{scissor:#?}");

        Self {
            viewport,
            scissor,
        }
    }

    pub fn build_viewport_state(&self) -> vk::PipelineViewportStateCreateInfo {
        // We need to reference the viewport and scissor in the viewport state because we need
        // to keep them alive as long as the pipeline that uses them is alive.
        let viewports = std::slice::from_ref(&self.viewport);
        let scissors = std::slice::from_ref(&self.scissor);

        // Viewport and Scissor State
        // Some gpus allow multiple viewports, but requires enabling a GPU feature.
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(viewports)
            .scissors(scissors)
            .build();
        debug!("Created PipelineViewportStateCreateInfo struct: \n{:#?}", viewport_state);

        viewport_state
    }


}
