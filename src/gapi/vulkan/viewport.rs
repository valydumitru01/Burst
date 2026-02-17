use log::debug;
use vulkanalia::vk;
use vulkanalia::vk::HasBuilder;
use crate::gapi::vulkan::swapchain::Swapchain;


#[derive(Debug)]
struct ViewportConfig {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    min_depth: f32,
    max_depth: f32,
}

#[derive(Debug)]
struct ScissorConfig {
    offset_x: i32,
    offset_y: i32,
    extent_width: u32,
    extent_height: u32,
}
pub struct Viewport {
    viewport_state: vk::PipelineViewportStateCreateInfo,
}

impl Viewport {
    pub fn new(swapchain: &Swapchain) -> Self {
        let viewport_config = ViewportConfig {
            x: 0.0,
            y: 0.0,
            width: swapchain.extent.width as f32,
            height: swapchain.extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        debug!("Creating Viewport with configuration: \n{viewport_config:#?}");

        // Viewport
        // The viewport is the region of the framebuffer that the output will be rendered to.
        let viewport = vk::Viewport::builder()
            .x(viewport_config.x)
            .y(viewport_config.y)
            .width(viewport_config.width)
            .height(viewport_config.height)
            .min_depth(viewport_config.min_depth)
            .max_depth(viewport_config.max_depth);

        let scissor_config = ScissorConfig {
            offset_x: 0,
            offset_y: 0,
            extent_width: swapchain.extent.width,
            extent_height: swapchain.extent.height,
        };

        debug!("Creating Scissor with configuration: \n{scissor_config:#?}");
        // Scissor
        // The scissor rectangle defines the area of the framebuffer that will be affected by
        // rendering operations. Basically acts like a filter for pixels.
        // Could be used for example to render only to a specific part of the screen, or to
        // implement a split-screen effect.
        let scissor = vk::Rect2D::builder()
            .offset(vk::Offset2D {
                x: scissor_config.offset_x,
                y: scissor_config.offset_y,
            })
            .extent(vk::Extent2D {
                width: scissor_config.extent_width,
                height: scissor_config.extent_height,
            });

        let viewports = &[viewport];
        let scissors = &[scissor];

        // Viewport and Scissor State
        // Some gpus allow multiple viewports, but requires enabling a GPU feature.
        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(viewports)
            .scissors(scissors);

        Self {
            viewport_state: viewport_state.build(),
        }
    }

    pub fn get_viewport_state(&self) -> &vk::PipelineViewportStateCreateInfo {
        &self.viewport_state
    }
}