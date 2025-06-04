use crate::gapi::vulkan::instance::Instance;
use crate::window::window::MyWindow;
use vulkanalia::vk::{KhrSurfaceExtension, SurfaceKHR};
use vulkanalia::window as vk_window;

#[derive(Clone, Debug)]
pub(crate) struct Surface {
    /// Abstract type of surface to present rendered images to.
    /// This surface will be backed by the window that we've already opened with winit.
    ///
    /// The window surface needs to be created right after the instance creation, because it can actually influence
    /// the physical device selection.
    ///
    /// Although the vk::SurfaceKHR object and its usage is platform-agnostic, its creation isn't because it depends on
    /// window system details. Fortunately, the vulkanalia crate provides a way to create a surface for a winit window
    /// that handles the platform differences for us.
    vk_surface: SurfaceKHR,
}

impl Surface {
    pub fn new(instance: &Instance, window: &MyWindow) -> anyhow::Result<Self> {
        let vk_surface =
            unsafe { vk_window::create_surface(&instance.get_vk(), &window.get(), &window.get())? };
        Ok(Self { vk_surface })
    }

    pub fn get(&self) -> SurfaceKHR {
        self.vk_surface
    }

    pub fn destroy(&self, instance: &Instance) {
        unsafe {
            instance.get_vk().destroy_surface_khr(self.vk_surface, None);
        }
    }
}
