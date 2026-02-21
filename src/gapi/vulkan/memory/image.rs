use anyhow::Context;
use log::debug;
use vulkanalia::vk;
use vulkanalia::vk::HasBuilder;
use crate::gapi::vulkan::core::logical_device::LogicalDevice;

#[derive(Debug)]
pub struct Image{
    /// Image View is owned by use, and is referenced by the framebuffer.
    /// It describes how the image should be accessed.
    /// It points to an actual image, but with additional information about how to interpret the
    /// image data (e.g., format, color component mapping, subresource range).
    vk_image_view: vk::ImageView,
    /// Image owned by the OS. Represents the actual heap of pixels in memory.
    /// It does not contain any information about how to interpret the data, that's why we use the
    /// ImageView to access it.
    vk_image: vk::Image
}

impl Image{
    pub fn new(image: &vk::Image, format: &vk::Format, device: &LogicalDevice) -> anyhow::Result<Self> {
        // Define the color component mapping for the image view
        // This allows swizzle the color channels around.
        // For example, it allows to map all the channels to the red channel for a monochrome texture.
        let components = vk::ComponentMapping::builder()
            .r(vk::ComponentSwizzle::IDENTITY)
            .g(vk::ComponentSwizzle::IDENTITY)
            .b(vk::ComponentSwizzle::IDENTITY)
            .a(vk::ComponentSwizzle::IDENTITY)
            .build();
        debug!("Created ComponentMapping struct: {components:#?}");

        // The subresource range for the image view describes the image's purpose and which part of
        // the image should be accessed.
        // Our images will be used as color targets without any mipmapping levels or multiple layers.
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);

        debug!("Created ImageSubresourceRange struct: {subresource_range:#?}");

        let info = vk::ImageViewCreateInfo::builder()
            .image(*image)
            // The view type represents how the image data should be interpreted
            // In this case, as it is a 2D image, we use the 2D view type
            .view_type(vk::ImageViewType::_2D)
            .format(*format)
            .components(components)
            .subresource_range(subresource_range);

        debug!("Created ImageView struct: {info:#?}");

        let vk_image_view = device.create_image_view(&info).with_context(|| "Failed to create image view")?;

        Ok(Self {
            vk_image_view,
            vk_image: *image
        })

    }


    pub fn get_vk(&self) -> &vk::ImageView {
        &self.vk_image_view
    }

    pub fn destroy(&self, device: &LogicalDevice) {
        unsafe {
            device.destroy_image_view(self.vk_image_view);
        }
    }
}