use vulkanalia::vk;
/// Type alias for the layer and extension names.
/// Vulkan provides a type for Extension ([`vk::ExtensionName`]) but not for Layer.
/// This is because extensions are ingrained in the Vulkan API, therefore, they
/// have a strict type.
///
/// Although, under the hood, Vulkan does define a type for Layer, it is just not
/// so strictly defined.
/// It can be seen defined in [`VkLayerProperties`](vk::LayerProperties) as
/// `StringArray<MAX_EXTENSION_NAME_SIZE>`
/// (the same max extension name used for extensions).
///
pub(crate) type LayerStr = vk::ExtensionName;
/// # Vulkan Layers
///
/// Layers are optional components that augment the Vulkan system.
/// They can intercept, evaluate, and modify Vulkan functions, attaching behavior to the normal Vulkan API.
///
/// # Details
/// Layers are implemented as libraries that are installed on the system and enabled or disabled
/// during Vulkan SDK initialization or at runtime, during instance creation.
///
/// A layer can choose to intercept Vulkan calls and modify their behavior.
/// Not all Vulkan functions need to be intercepted by a layer, it could intercept only a subset or
/// just a single one.
///
/// Because layers are optional, you can choose to enable some layers for debugging and disable them
/// to release.
///
/// # Examples
///
/// - `VK_LAYER_KHRONOS_validation`
/// Validation layer provided by Khronos.
/// It checks for correct API usage, detects common errors, and helps in debugging.
/// - `VK_LAYER_LUNARG_api_dump`
/// Logs all Vulkan API calls along with their parameters to the standard output.
/// Useful for tracing and debugging Vulkan function calls.
/// - `VK_LAYER_RENDERDOC_Capture`
/// Integrates with RenderDoc to capture and analyze frames.
/// Useful for debugging and performance profiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstanceLayers {
    /// # `VK_LAYER_KHRONOS_validation`
    /// The **official, all-in-one validation layer** maintained by the Khronos Group.
    ///
    /// # What it adds
    /// 1. Full-spectrum API validation
    ///    - Structural checks: correct object lifetimes, handle ownership, thread-safety.
    ///    - Draw-time checks: descriptor-set compatibility, pipeline state, render-pass rules.
    ///    - Synchronization validation (optionally via the _sync2_ sub-module).
    ///    - Best-practice warnings for performance or portability issues.
    /// 2. **Message routing** through the standard debug-messenger callback
    ///    (`VK_EXT_debug_utils`).  Supports severity & type filtering, custom callbacks,
    ///    and message IDs.
    /// 3. **Configurable behaviour**
    ///    - Runtime toggles via `VK_LAYER_SETTINGS_PATH`, `VK_INSTANCE_LAYERS`,
    ///      or JSON settings files.
    ///    - Fine-grained disable lists (e.g. suppress known-benign message IDs).
    ///    - GPU-assisted validation & shader-instrumentation modes for deeper analysis.
    /// 4. **Extended utilities** exposed as layer-specific extensions
    ///    (`VK_EXT_validation_features`, `VK_EXT_validation_flags`,
    ///    `VK_EXT_tooling_info`).
    /// 5. **Safe-mode fallbacks** — if certain device features are unsupported
    ///    GPU-assisted checks gracefully downgrade to host simulation instead of failing.
    ///
    /// # Typical use
    /// Enable during development:
    /// ```text
    /// enabled_layer_names = ["VK_LAYER_KHRONOS_validation"]
    /// ```
    /// Pair with the `VK_EXT_debug_utils` extension to receive messages.
    Validation,

    /// # `VK_LAYER_LUNARG_api_dump`
    /// Human-readable trace layer that logs every Vulkan call (and its parameters)
    /// as it happens.
    ///
    /// # What it adds
    /// 1. **Automatic API call logging** to:
    ///    - **Stdout / stderr** (default)
    ///    - A **named file** (`VK_APIDUMP_OUTPUT_FILE=<path>`)
    ///    - The **debug-utils messenger** stream (if present)
    /// 2. **Complete parameter expansion**
    ///    - Enumerations & flags printed by symbolic name.
    ///    - Structs and arrays fully expanded.
    ///    - Handles shown as hexadecimal for cross-reference with validation messages.
    /// 3. **Timing information** (optional) — per-call timestamps & thread IDs for
    ///    performance replay.
    /// 4. **JSON or C-style output** (`VK_APIDUMP_FORMAT={text|json}`) to feed
    ///    external tooling.
    /// 5. **Selective capture filters**
    ///    - Environment variables to include / exclude function ranges
    ///    - Live toggling via `VK_APIDUMP_ACTIVE=0|1`
    ///
    /// # Typical use
    /// Very lightweight; enable when you need a quick “what is the app really doing?”
    /// transcript or when crafting minimal repro cases for driver bugs.
    ApiDump,

    /// # `VK_LAYER_RENDERDOC_Capture`
    /// Integration layer that allows **RenderDoc** to intercept Vulkan work for
    /// frame-capture and analysis.
    ///
    /// # What it adds
    /// 1. **Hook points** for RenderDoc’s graphics debugger to:
    ///    - Capture a frame at any time (hot-key / API trigger).
    ///    - Serialize command buffers, resources, and pipeline state.
    ///    - Re-inject captures for offline replay.
    /// 2. **In-application overlay** (optional) showing frame-time, GPU/CPU stats,
    ///    and capture status.
    /// 3. **Remote-control API** (`RENDERDOC_API_1_6_0`) obtained via
    ///    `RENDERDOC_GetAPI` — lets the app trigger captures, set markers, or
    ///    inject custom thumbnails.
    /// 4. **Automatic swap-chain tracking** across WSI extensions, including
    ///    multi-GPU and VR-compositor setups.
    /// 5. **No-op pass-through** when RenderDoc is not running, adding negligible
    ///    overhead in release builds.
    ///
    /// # Typical use
    /// Ship **disabled** in production, enable only in developer/internal builds or
    /// when the RenderDoc loader detects an attached debugger.
    RenderDoc,
}

impl InstanceLayers {
    pub const VALIDATION: LayerStr = LayerStr::from_bytes("VK_LAYER_KHRONOS_validation".as_bytes());
    pub const API_DUMP: LayerStr = LayerStr::from_bytes("VK_LAYER_LUNARG_api_dump".as_bytes());
    pub const RENDERDOC: LayerStr = LayerStr::from_bytes("VK_LAYER_RENDERDOC_Capture".as_bytes());
    pub fn as_str(&self) -> LayerStr {
        match self {
            Self::Validation => Self::VALIDATION,
            Self::ApiDump => Self::API_DUMP,
            Self::RenderDoc => Self::RENDERDOC,
        }
    }
}
