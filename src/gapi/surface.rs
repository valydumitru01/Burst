use vulkanalia::vk;

struct Surface {
    /// Abstract type of surface to present rendered images to.
    /// This surface will be backed by the window that we've already opened with winit.
    ///
    /// The window surface needs to be created right after the instance creation, because it can actually influence
    /// the physical device selection.
    ///
    /// Although the vk::SurfaceKHR object and its usage is platform-agnostic, its creation isn't because it depends on
    /// window system details. Fortunately, the vulkanalia crate provides a way to create a surface for a winit window
    /// that handles the platform differences for us.
    vk_surface: vk::SurfaceKHR,
}
