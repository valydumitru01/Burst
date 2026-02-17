use log::debug;
use vulkanalia::vk;
use vulkanalia::vk::HasBuilder;

#[derive(Debug)]
struct DepthStencilStateConfig {
    depth_test_enable: bool,
}

pub struct PerFragmentTestsStage {
    depth_stencil_state: vk::PipelineDepthStencilStateCreateInfo,
}

impl PerFragmentTestsStage {
    pub fn new() -> Self {

        // Depth and Stencil Testing
        let config = DepthStencilStateConfig {
            // If depth_test_enable is set to true, then fragments will be compared to the depth
            // buffer to determine if they should be discarded or not. This is essential for proper
            // rendering of 3D scenes, as it ensures that closer objects are rendered in front of
            // farther ones.
            // It is disabled for now.
            depth_test_enable: false,
        };

        debug!("Creating Pipeline Depth Stencil State with config: {:#?}", config);
        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(config.depth_test_enable);


        Self {
            depth_stencil_state: depth_stencil_state.build(),
        }
    }

    pub fn get_depth_stencil_state(&self) -> &vk::PipelineDepthStencilStateCreateInfo {
        &self.depth_stencil_state
    }
}
