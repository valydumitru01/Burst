use crate::gapi::entry::Entry;
use crate::gapi::instance::Instance;
use crate::gapi::logical_device::LogicalDevice;
use crate::gapi::physical_device::PhysicalDevice;
use crate::gapi::queues::QueueFamilyIndices;
use crate::gapi::{debug, surface};
use crate::window::window::MyWindow;
use log::error;
use thiserror::Error;

pub(crate) const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

/// Our Vulkan app.
#[derive(Clone, Debug)]
pub struct App {
    entry: Entry,
    instance: Instance,
    debugger: debug::Debugger,
    device: LogicalDevice,
    surface: surface::Surface,
}
#[derive(Debug, Error)]
#[error("Missing {0}.")]
pub(crate) struct SuitabilityError(pub &'static str);
impl App {
    /// Creates our Vulkan app.
    pub unsafe fn create(window: &MyWindow) -> anyhow::Result<Self> {
        let mut entry = Entry::new()?;
        let instance = Instance::new(&entry, window)?;
        let debugger = debug::Debugger::new(&instance)?;
        let surface = surface::Surface::new(&instance, window)?;
        let device = LogicalDevice::new(&entry, &instance, &surface, &window)?;

        Ok(Self {
            entry,
            instance,
            debugger,
            device,
            surface,
        })
    }

    /// Renders a frame for our Vulkan app.
    pub unsafe fn render(&mut self, window: &MyWindow) -> anyhow::Result<()> {
        Ok(())
    }

    /// Destroys our Vulkan app.
    pub fn destroy(&self) {
        self.device.destroy();
        if VALIDATION_ENABLED {
            self.debugger.destroy(&self.instance);
        }
        self.surface.destroy(&self.instance);
        self.instance.destroy();
    }
}

/// The Vulkan handles and associated properties used by our Vulkan app.
#[derive(Clone, Debug, Default)]
pub(crate) struct AppData {}
