use log::debug;
use vulkanalia::vk;
use vulkanalia::vk::HasBuilder;
use crate::gapi::vulkan::core::logical_device::LogicalDevice;
use crate::gapi::vulkan::memory::image::Image;
use crate::gapi::vulkan::memory::swapchain::Swapchain;
use crate::gapi::vulkan::pipeline::render_pass::MyRenderPass;

pub struct Framebuffer{
    framebuffer: vk::Framebuffer,
}

impl Framebuffer {
    pub fn new(render_pass: &MyRenderPass, imgs: &[Image], swapchain: &Swapchain, device: &LogicalDevice) -> Self {
        let attachments = imgs.iter().map(|image| *image.get_vk()).collect::<Vec<_>>();
        let create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass.get_vk())
            .attachments(attachments.as_slice())
            .width(swapchain.extent.width)
            .height(swapchain.extent.height)
            .layers(1)
            .build();
        debug!("Created FramebufferCreateInfo struct: {create_info:#?}");

        let framebuffer = device.create_framebuffer(&create_info).unwrap();
        Self {
            framebuffer
        }
    }

    pub fn get_vk(&self) -> vk::Framebuffer {
        self.framebuffer
    }

    pub fn destroy(&self, device: &LogicalDevice) {
        device.destroy_framebuffer(self.framebuffer);
    }
}

