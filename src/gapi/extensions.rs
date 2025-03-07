use anyhow::anyhow;
use std::collections::HashSet;
use vulkanalia::vk;
use vulkanalia::vk::StringArray;

pub(in crate::gapi) fn add_extension_if_available(
    available_extensions: &HashSet<StringArray<256>>,
    extensions: &mut Vec<*const i8>,
    extension_to_add: &vk::ExtensionName,
) -> anyhow::Result<()> {
    if !available_extensions.contains(&extension_to_add) {
        return Err(anyhow!(format!(
            "Missing required layer: {:?}",
            extension_to_add
        )));
    }
    extensions.push(extension_to_add.as_ptr());
    Ok(())
}
pub(in crate::gapi) fn add_extension(
    extensions: &mut Vec<*const i8>,
    extension_to_add: &vk::ExtensionName,
) -> anyhow::Result<()> {
    extensions.push(extension_to_add.as_ptr());
    Ok(())
}
