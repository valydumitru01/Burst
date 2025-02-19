use crate::gapi::vulkan::VALIDATION_ENABLED;
use anyhow::anyhow;
use log::{debug, error, trace, warn};
use std::collections::HashSet;
use std::ffi::CStr;
use std::os::raw::c_void;
use vulkanalia::vk::{
    Cast, DebugUtilsMessengerCreateInfoEXT, DebugUtilsMessengerCreateInfoEXTBuilder,
    DebugUtilsMessengerEXT, EntryV1_0, ExtDebugUtilsExtension, ExtendsInstanceCreateInfo,
    HasBuilder, StringArray,
};
use vulkanalia::{vk, Entry, Instance};

#[derive(Clone, Debug, Default)]
pub(crate) struct DebugData {
    /// The messenger is in charge of handling the debug callback and it's lifetime.
    /// This needs manual destruction and initialization.
    /// Calling:
    /// ```rust
    /// /// Creating the instance
    /// let messenger = instance.create_debug_utils_messenger_ext(&debug_info, None)?;
    /// ...
    /// /// Destroying the instance
    /// instance.destroy_debug_utils_messenger_ext(messenger, None);
    /// ```
    pub messenger: vk::DebugUtilsMessengerEXT,
}

pub(crate) const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

pub(crate) fn get_available_layers(entry: &Entry) -> anyhow::Result<HashSet<StringArray<256>>> {
    let available_layers = unsafe { entry.enumerate_instance_layer_properties() }?
        .iter()
        .map(|l| l.layer_name)
        .collect::<HashSet<_>>();
    Ok(available_layers)
}
pub(crate) fn add_validation_layer(
    available_layers: HashSet<StringArray<256>>,
    layers: &mut Vec<*const i8>,
) -> anyhow::Result<()> {
    if VALIDATION_ENABLED && !available_layers.contains(&VALIDATION_LAYER) {
        return Err(anyhow!("Validation layer requested but not supported."));
    }
    layers.push(VALIDATION_LAYER.as_ptr());
    Ok(())
}

pub(crate) fn get_debug_info<'b>() -> DebugUtilsMessengerCreateInfoEXTBuilder<'b> {
    let debug_info = DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .user_callback(Some(debug_callback));
    debug_info
}

pub(crate) fn add_debug_creation_destruction<'b, T>(
    info: &mut vk::InstanceCreateInfoBuilder<'b>,
    debug_info: &'b mut impl Cast<Target = T>,
) where
    T: ExtendsInstanceCreateInfo,
{
    info.push_next(debug_info);
}

pub(crate) fn create_messenger<'b>(
    instance: &Instance,
    debug_info: &DebugUtilsMessengerCreateInfoEXT,
    debugger: &mut DebugData,
) -> anyhow::Result<DebugUtilsMessengerEXT> {
    debug!("Adding debug callback.");
    unsafe { Ok(instance.create_debug_utils_messenger_ext(&debug_info, None)?) }
}

pub(crate) fn destroy_debug_calls(instance: &Instance, debugger: &mut DebugData) {
    unsafe {
        instance.destroy_debug_utils_messenger_ext(debugger.messenger, None);
    }
}

/// The debug callback used by our Vulkan app.
/// We need extern "system" so we can expose this function to the (external) Vulkan loader.
///
/// The first parameter `severity` specifies the severity of the message, which is one of the following flags:
/// - `vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE` – Diagnostic message
/// - `vk::DebugUtilsMessageSeverityFlagsEXT::INFO` – Informational message like the creation of a resource
/// - `vk::DebugUtilsMessageSeverityFlagsEXT::WARNING` – Message about behavior that is not necessarily an error,
/// but very likely a bug in your application
/// - `vk::DebugUtilsMessageSeverityFlagsEXT::ERROR` – Message about behavior that is invalid and may cause crashes
///
/// The `type_` parameter can have the following values:
/// - `vk::DebugUtilsMessageTypeFlagsEXT::GENERAL` – Some event has happened that is unrelated to the specification
/// or performance
/// - `vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION` – Something has happened that violates the specification or
/// indicates a possible mistake
/// - `vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE` – Potential non-optimal use of Vulkan
///
/// The `data` parameter refers to a `vk::DebugUtilsMessengerCallbackDataEXT` struct containing the details of the
/// message itself, with the most important members being:
///
/// - `message` – The debug message as a null-terminated string (*const c_char)
/// - `objects` – Array of Vulkan object handles related to the message
/// - `object_count` – Number of objects in array
///
/// Finally, the last parameter, here ignored as `_`, contains a pointer that was specified during the setup of the
/// callback and allows you to pass your own data to it.
///
/// The callback returns a (Vulkan) boolean that indicates if the Vulkan call that triggered the validation layer
/// message should be aborted. If the callback returns `true`, then the call is aborted with the
/// `vk::ErrorCode::VALIDATION_FAILED_EXT` error code. This is normally only used to test the validation layers
/// themselves, so you should always return `vk::FALSE`.
extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    let data = unsafe { *data };
    let message = unsafe { CStr::from_ptr(data.message) }.to_string_lossy();

    if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
        error!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
        warn!("({:?}) {}", type_, message);
    } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
        debug!("({:?}) {}", type_, message);
    } else {
        trace!("({:?}) {}", type_, message);
    }

    vk::FALSE
}
