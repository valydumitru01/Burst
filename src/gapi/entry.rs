use anyhow::anyhow;
use std::collections::HashSet;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::vk::{EntryV1_0, StringArray};
use vulkanalia::{vk, Version};
use vulkanalia::{Entry as VkEntry, Instance, VkResult};

#[derive(Debug, Clone)]
pub(crate) struct Entry {
    entry: VkEntry,
}
impl Entry {
    pub(in crate::gapi) fn new() -> anyhow::Result<Self> {
        let loader = unsafe { LibloadingLoader::new(LIBRARY)? };
        let entry = unsafe { vulkanalia::Entry::new(loader).map_err(|b| anyhow!("{}", b))? };
        Ok(Self { entry })
    }

    pub(in crate::gapi) fn version(&self) -> anyhow::Result<Version> {
        Ok(self.entry.version()?)
    }

    pub(in crate::gapi) fn get(&self) -> &vulkanalia::Entry {
        &self.entry
    }

    pub(in crate::gapi) fn get_available_layers(
        &self,
    ) -> anyhow::Result<HashSet<StringArray<256>>> {
        let available_layers = unsafe { self.entry.enumerate_instance_layer_properties() }?
            .iter()
            .map(|l| l.layer_name)
            .collect::<HashSet<_>>();
        Ok(available_layers)
    }

    pub(in crate::gapi) fn get_available_extensions(
        &self,
    ) -> anyhow::Result<HashSet<StringArray<256>>> {
        let available_extensions =
            unsafe { self.entry.enumerate_instance_extension_properties(None) }?
                .iter()
                .map(|e| e.extension_name)
                .collect::<HashSet<_>>();
        Ok(available_extensions)
    }

    pub(in crate::gapi) fn create_instance(
        &self,
        info: &vk::InstanceCreateInfo,
        allocation_callbacks: Option<&vk::AllocationCallbacks>,
    ) -> VkResult<Instance> {
        unsafe { self.entry.create_instance(info, allocation_callbacks) }
    }
}
