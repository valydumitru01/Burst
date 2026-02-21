use vulkanalia::vk;
use vulkanalia::vk::HasBuilder;

pub struct InputAssemblerStage{
    vertex_binding_descriptions: Vec<vk::VertexInputBindingDescription>,
    vertex_attribute_descriptions: Vec<vk::VertexInputAttributeDescription>,
}

impl InputAssemblerStage {
    pub fn new() -> Self {
        let vertex_binding_descriptions = (&[] as &[vk::VertexInputBindingDescription]).to_vec();
        let vertex_attribute_descriptions = (&[] as &[vk::VertexInputAttributeDescription]).to_vec();
        Self {
            vertex_binding_descriptions,
            vertex_attribute_descriptions,
        }
    }

    pub fn build_vertex_input_state(&self) -> vk::PipelineVertexInputStateCreateInfo {
        vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&self.vertex_binding_descriptions)
            .vertex_attribute_descriptions(&self.vertex_attribute_descriptions)
            .build()
    }

    pub fn build_input_assembly_state(&self) -> vk::PipelineInputAssemblyStateCreateInfo {
        vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::POINT_LIST)
            .primitive_restart_enable(false)
            .build()
    }
}