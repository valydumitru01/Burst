use vulkanalia::vk::ExtensionName;
use vulkanalia::window as vk_window;
use winit::dpi::LogicalSize;
use winit::window::{Window, WindowBuilder};

pub struct MyWindow {
    winit_window: Window,
}

impl MyWindow {
    pub fn get(&self) -> &Window {
        &self.winit_window
    }
    pub fn get_required_extensions(&self) -> &'static [&'static ExtensionName] {
        vk_window::get_required_instance_extensions(&self.winit_window)
    }
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let window = WindowBuilder::new()
            .with_title("Vulkan Tutorial (Rust)")
            .with_inner_size(LogicalSize::new(1024, 768))
            .build(&event_loop);
        Self {
            winit_window: window.unwrap(),
        }
    }
}
