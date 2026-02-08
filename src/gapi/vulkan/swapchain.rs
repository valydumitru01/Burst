use crate::gapi::vulkan::logical_device::{LogicalDevice, RealDevice};
use crate::gapi::vulkan::surface::Surface;
use crate::window::MyWindow;
use anyhow::Context;
use log::{debug, info};
use log::__private_api::loc;
use vulkanalia::vk;
use vulkanalia::vk::{Format, Handle, HasBuilder};
use crate::gapi::vulkan::image::Image;

#[derive(Debug)]
pub(crate) struct Swapchain {
    // The swapchain handle from Vulkan.
    vk_swapchain: vk::SwapchainKHR,
    /// The swapchain images are the actual images that will be presented to the screen.
    /// They are created by the driver when we create the swapchain and we can get them with
    /// vkGetSwapchainImagesKHR.
    images: Vec<vk::Image>,
    /// An Image View describes how to access the image and which part of the image to access
    /// For example, it can specify the format of the image, the color channels,
    /// and the subresource range (e.g. mip levels, array layers) that will be accessed.
    image_views: Vec<Image>,
    format: vk::Format,
    extent: vk::Extent2D,
}

impl Swapchain {
    pub(crate) fn new(
        window: &MyWindow,
        real_device: &RealDevice,
        logical_device: &LogicalDevice,
        surface: &Surface,
    ) -> anyhow::Result<Swapchain> {
        let support = real_device.get_swapchain_info(surface)?;
        let queues = logical_device.get_queues();

        // The surface format describes how the pixels in the swapchain images are stored and
        // interpreted. It includes the color format (e.g. RGBA, BGRA) and the color space
        // (e.g. sRGB).
        let surface_format = Self::get_surface_format(&support.formats).with_context(|| {
            anyhow::anyhow!(
                "Failed to find suitable swapchain surface format between: {:?}",
                support.formats
            )
        })?;

        // The present mode determines how images are presented to the screen.
        // It can affect latency, tearing, and power consumption.
        let present_mode = Self::get_present_mode(&support.present_modes).with_context(|| {
            anyhow::anyhow!(
                "Failed to find suitable swapchain present mode between: {:?}",
                support.present_modes
            )
        })?;

        // The extent is the resolution of the swapchain images, which should match the resolution
        // of the window we are rendering to.
        let extent = Self::get_extent(window, support.capabilities);

        // The implementation specifies the minimum number that it requires to function
        // However, simply sticking to this minimum means that we may sometimes have to wait on the
        // driver to complete internal operations before we can acquire another image to render to.
        // Therefore, it is recommended to request at least one more image than the minimum
        let mut image_count = support.capabilities.min_image_count + 1;

        // We should also make sure to not exceed the maximum number of images while doing this,
        // where 0 is a special value that means that there is no maximum
        if support.capabilities.max_image_count != 0
            && image_count > support.capabilities.max_image_count
        {
            image_count = support.capabilities.max_image_count;
        }

        // The Sharing Mode specifies how to handle swapchain images that will be used across
        // multiple queue families. That will be the case in our application if the graphics queue
        // family is different from the presentation queue.
        // TODO: If the queue families for graphics and presentation are different, we use
        //   CONCURRENT for simplicity, but EXCLUSIVE can offer better performance but it needs
        //   explicit ownership transfers between the queues, which can be more complex to manage.
        //   Improve this
        let mut queue_family_indices = vec![];
        let image_sharing_mode = if queues.graphics[0] != queues.present[0] {
            queue_family_indices.push(queues.graphics[0].as_raw() as u32);
            queue_family_indices.push(queues.present[0].as_raw() as u32);
            vk::SharingMode::CONCURRENT
        } else {
            vk::SharingMode::EXCLUSIVE
        };

        // This specifies the amount of layers each image consists of. This is always 1 unless you
        // are developing a stereoscopic 3D application
        let image_array_layers = 1;

        // The usage flag specifies what we intend to use the images in the swapchain for. Here we
        // are specifying that we will render directly to them, and thus we will use them as color
        // attachments in the framebuffer.
        // It is also possible to render images to a separate image first to perform
        // operations like post-processing. In that case it may be used a value like
        // vk::ImageUsageFlags::TRANSFER_DST instead and use a memory operation to transfer the
        // rendered image to a swapchain image.
        let image_usage = vk::ImageUsageFlags::COLOR_ATTACHMENT;

        // The composite_alpha method specifies if the alpha channel should be used for blending
        // with other windows in the window system.
        // OPAQUE means that the alpha channel is ignored and treated as 1.0, which is the most
        // common case for applications that don't need transparency.
        let composite_alpha = vk::CompositeAlphaFlagsKHR::OPAQUE;

        // If the clipped member is set to true then that means that we don't care about the color
        // of pixels that are obscured, for example because another window is in front of them.
        // Unless you really need to be able to read these pixels back and get predictable results,
        // you'll get the best performance by enabling clipping.
        let clipped = true;

        // This is used when you want to recreate the swapchain.
        // With Vulkan, it's possible that your swapchain becomes invalid or unoptimized while your
        // application is running, for example because the window was resized.
        // In that case the swapchain actually needs to be recreated from scratch and a reference
        // to the old one must be specified in this method (.old_swapchain) so that the driver can
        // optimize the transition between the old and the new swapchain.
        // By default is null, for now we are not implementing swapchain recreation.
        // TODO: Implement swapchain recreation and use this field properly
        let old_swapchain = vk::SwapchainKHR::null();

        debug!(
            "Creating swapchain with the following configuration:\
            \n- min_image_count: {}\
            \n- image_format: {:?}\
            \n- image_color_space: {:?}\
            \n- image_extent: {:?}\
            \n- image_array_layers: {}\
            \n- image_usage: {:?}\
            \n- image_sharing_mode: {:?}\
            \n- queue_family_indices: {:?}\
            \n- pre_transform: {:?}\
            \n- composite_alpha: {:?}\
            \n- present_mode: {:?}\
            \n- clipped: {}\
            \n- old_swapchain: {:?}",
            image_count,
            surface_format.format,
            surface_format.color_space,
            extent,
            image_array_layers,
            image_usage,
            image_sharing_mode,
            queue_family_indices,
            support.capabilities.current_transform,
            composite_alpha,
            present_mode,
            clipped,
            old_swapchain
        );

        let info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.get_vk())
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(image_array_layers)
            .image_usage(image_usage)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(queue_family_indices.as_slice())
            .pre_transform(support.capabilities.current_transform)
            .composite_alpha(composite_alpha)
            .present_mode(present_mode)
            .clipped(clipped)
            .old_swapchain(old_swapchain)
            .build();

        let vk_swapchain = logical_device.create_swapchain_khr(&info).with_context(|| {
            anyhow::anyhow!(
                "Failed to create swapchain with the following configuration: {:?}",
                info
            )
        })?;

        let images = logical_device.get_swapchain_images_khr(vk_swapchain).with_context(|| {;
            anyhow::anyhow!(
                "Failed to get swapchain images for swapchain: {:?}",
                vk_swapchain
            )
        })?;


        let image_views = Self::create_image_views(&images, &surface_format.format, logical_device).with_context(|| {;
            anyhow::anyhow!(
                "Failed to create image views for swapchain images: {:?}",
                images
            )
        })?;

        Ok(Self { vk_swapchain, images, format: surface_format.format, extent, image_views })
    }



    fn get_vk(&self) -> vk::SwapchainKHR {
        self.vk_swapchain
    }

    fn create_image_views(images: &[vk::Image], format: &Format, logical_device: &LogicalDevice) -> anyhow::Result<Vec<Image>> {
        images
            .iter()
            .map(|img| {
                Image::new(img, format, logical_device).with_context(|| {
                    anyhow::anyhow!(
                        "Failed to create image view for swapchain image: {:?}",
                        img
                    )
                })
            })
            .collect::<anyhow::Result<Vec<Image>>>()
    }


    fn get_surface_format(
        formats: &[vk::SurfaceFormatKHR],
    ) -> anyhow::Result<vk::SurfaceFormatKHR> {
        // TODO: Rank available formats and pick the best one.
        // - B8G8R8A8_SRGB means BGR and alpha channels with 8 bits each, 32 in total per pixel.
        // - SRGB_NONLINEAR means that the color space is sRGB with nonlinear gamma correction, which
        // is the most common color space for images and displays.
        formats
            .iter()
            .cloned()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .or_else(|| Some(formats[0]))
            .ok_or_else(|| anyhow::anyhow!("Failed to find suitable swapchain format."))
    }
    fn get_present_mode(
        present_modes: &[vk::PresentModeKHR],
    ) -> anyhow::Result<vk::PresentModeKHR> {
        // Choosing mailbox if available, otherwise falling back to FIFO which is guaranteed to be supported.
        // Mailbox is preferred for low latency and no tearing at expense of potentially higher power consumption
        present_modes
            .iter()
            .cloned()
            .find(|m| *m == vk::PresentModeKHR::MAILBOX)
            .or_else(|| Some(vk::PresentModeKHR::FIFO))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Failed to find suitable swapchain present mode between: {:?}",
                    present_modes
                )
            })
    }
    fn get_extent(window: &MyWindow, capabilities: vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        // If the current_extent is not u32::MAX, it means we need to set it to current_extent
        // otherwise, we can set the windows size ourselves and configure it,
        // like clamping it to the min and max extents supported by the device.
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D::builder()
                .width(window.size().width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ))
                .height(window.size().height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ))
                .build()
        }
    }

    pub(crate) fn destroy(&self, logical_device: &LogicalDevice) {
        for image_view in &self.image_views {
            image_view.destroy(logical_device)
        }
        logical_device.destroy_swapchain_khr(self.vk_swapchain);
    }
}
