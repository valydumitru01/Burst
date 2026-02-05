use vulkanalia::vk;

trait BitIter {
    fn iter(&self) -> impl Iterator<Item = u32>;
}

impl BitIter for u32 {
    fn iter(&self) -> impl Iterator<Item = u32> {
        (0..32).filter(move |i| (*self & (1u32 << i)) != 0u32)
    }
}
/// An enum to distinguish different queue types your application might need.
///
/// If you need more specialized queues (e.g., compute-only, transfer-only),
/// you can add variants here and include the matching logic in `find_queue_families`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum QueueCapability {
    /// A queue that supports graphics operations.
    Graphics,
    /// A queue that supports compute operations.
    Compute,
    /// A queue that supports transfer operations (copying data).
    Transfer,
}
impl From<QueueCapability> for vk::QueueFlags {
    fn from(cap: QueueCapability) -> Self {
        match cap {
            QueueCapability::Graphics => vk::QueueFlags::GRAPHICS,
            QueueCapability::Compute => vk::QueueFlags::COMPUTE,
            QueueCapability::Transfer => vk::QueueFlags::TRANSFER,
        }
    }
}
impl QueueCapability {}

/// A request describing how many queues of a given type your app needs.
///
/// For example, if you require two graphics queues, you can specify:
/// ```
/// QueueRequest {
///     queue_type: QueueType::Graphics,
///     queue_flags: vk::QueueFlags::GRAPHICS,
///     require_present: false,
///     count: 2,
/// }
/// ```
#[derive(Clone, Debug)]
pub struct QueueRequest {
    /// The type of queue requested (e.g. [`QueueCapability::Graphics`]).
    pub capabilities: Vec<QueueCapability>,
    /// Whether this queue needs to support presentation on the given surface.
    pub require_present: bool,
    /// How many queues of this type should be created?
    pub count: u32,
}

/// Holds metadata about a single queue family that will be created, including
/// which [`QueueCapability`] it corresponds to and how many queues from that family
/// will be requested.
#[derive(Clone, Debug)]
pub(crate) struct QueueFamily {
    /// The queue family index in Vulkan.
    pub family_index: u32,
    /// How many there are available in this family.
    pub count: u32,
    /// The type of queue we are satisfying (graphics, present, etc.).
    pub capabilities: Vec<QueueCapability>,
    /// Whether this family can present images to the surface.
    pub allows_present: bool,
}
