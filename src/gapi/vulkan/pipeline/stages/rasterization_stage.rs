use log::debug;
use vulkanalia::vk;
use vulkanalia::vk::HasBuilder;


#[derive(Debug)]
struct RasterizationStageConfig {
    rasterizer_discard_enable: bool,
    depth_clamp_enable: bool,
    polygon_mode: vk::PolygonMode,
    line_width: f32,
    cull_mode: vk::CullModeFlags,
    front_face: vk::FrontFace,
    depth_bias_enable: bool,
}

#[derive(Debug)]
struct MultisampleStageConfig {
    sample_shading_enable: bool,
    rasterization_samples: vk::SampleCountFlags,
}
pub struct RasterizationStage {
    rasterization_state: vk::PipelineRasterizationStateCreateInfo,
    multisample_state: vk::PipelineMultisampleStateCreateInfo,
}

impl RasterizationStage {
    pub fn new() -> Self {

        // Rasterization
        // The rasterizer takes the geometry that is shaped by the vertices from the vertex shader
        // and turns it into fragments to be colored by the fragment shader.
        let config = RasterizationStageConfig {
            // If rasterizer_discard_enable is set to true, then geometry never passes through the
            // rasterizer stage. This basically disables any output to the framebuffer.
            rasterizer_discard_enable: false,
            // If depth_clamp_enable is set to true, then fragments that are beyond the near and far
            // planes are clamped to them as opposed to discarding them. This is useful in some
            // special cases like shadow maps. Using this requires enabling a GPU feature.
            depth_clamp_enable: false,
            // The polygon_mode determines how fragments are generated for geometry. The following modes are available:
            // vk::PolygonMode::FILL – fill the area of the polygon with fragments
            // vk::PolygonMode::LINE – polygon edges are drawn as lines
            // vk::PolygonMode::POINT – polygon vertices are drawn as points
            // We use fill mode to render our voxels as solid cubes, but line or point mode could
            // be useful for debugging purposes.
            polygon_mode: vk::PolygonMode::FILL,
            // line_width describes the thickness of lines in terms of number of fragments.
            // The maximum line width that is supported depends on the hardware and any line thicker
            // than 1.0 requires you to enable the wide_lines GPU feature.
            // This only affects the rendering if polygon_mode is set to LINE.
            line_width: 1.0,
            // The cull_mode variable determines the type of face culling to use.
            // You can disable culling, cull the front faces, cull the back faces or both.
            cull_mode: vk::CullModeFlags::BACK,
            // The front_face variable specifies the vertex order for faces to be considered
            // front-facing and can be clockwise or counterclockwise.
            front_face: vk::FrontFace::CLOCKWISE,
            // The rasterizer can alter the depth values by adding a constant value or biasing them
            // based on a fragment's slope. This is used for shadow mapping to prevent shadow acne.
            // For now, it is disabled.
            depth_bias_enable: false,
        };
        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .rasterizer_discard_enable(config.rasterizer_discard_enable)
            .depth_clamp_enable(config.depth_clamp_enable)
            .polygon_mode(config.polygon_mode)
            .line_width(config.line_width)
            .cull_mode(config.cull_mode)
            .front_face(config.front_face)
            .depth_bias_enable(config.depth_bias_enable);


        // Multisampling
        // The vk::PipelineMultisampleStateCreateInfo struct configures multisampling, which is one
        // of the ways to perform anti-aliasing. It works by combining the fragment shader results
        // of multiple polygons that rasterize to the same pixel. This mainly occurs along edges,
        // which is also where the most noticeable aliasing artifacts occur. Because it doesn't need
        // to run the fragment shader multiple times if only one polygon maps to a pixel, it is
        // significantly less expensive than simply rendering to a higher resolution and then
        // downscaling. Enabling it requires enabling a GPU feature.
        // For now it is disabled.
        let config_multisample = MultisampleStageConfig {
            sample_shading_enable: false,
            rasterization_samples: vk::SampleCountFlags::_1,
        };
        debug!("Creating pipeline multisample state with config: {config_multisample:#?}");
        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(config_multisample.sample_shading_enable)
            .rasterization_samples(config_multisample.rasterization_samples);
        Self {
            rasterization_state: rasterization_state.build(),
            multisample_state: multisample_state.build(),
        }
    }
    pub fn get_rasterization_state(&self) -> &vk::PipelineRasterizationStateCreateInfo {
        &self.rasterization_state
    }
    pub fn get_multisample_state(&self) -> &vk::PipelineMultisampleStateCreateInfo {
        &self.multisample_state
    }
}
