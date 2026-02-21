use log::debug;
use vulkanalia::vk;
use vulkanalia::vk::HasBuilder;

pub struct PerFragmentTestsStage {}

impl PerFragmentTestsStage {
    pub fn new() -> Self {
        Self {}
    }

    pub fn build_depth_stencil_state(&self) -> vk::PipelineDepthStencilStateCreateInfo {
        // If depth_test_enable is set to true, then fragments will be compared to the depth
        // buffer to determine if they should be discarded or not. This is essential for proper
        // rendering of 3D scenes, as it ensures that closer objects are rendered in front of
        // farther ones.
        // It is disabled for now.
        let depth_test_enable = false;

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(depth_test_enable)
            .build();

        debug!(
            "Created PipelineDepthStencilState struct: {:#?}",
            &depth_stencil_state
        );

        depth_stencil_state
    }
}
