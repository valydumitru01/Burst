/// Host memory allocation has failed.
/// > This global member only exists to document the error code. 
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_ERROR_OUT_OF_HOST_MEMORY: () = ();

/// Device memory allocation has failed.
/// This usually happens when the GPU heap is exhausted or memory is too fragmented 
/// to fulfill an allocation request. It May occur during image/buffer creation or other 
/// resource-heavy operations.
/// > This global member only exists to document the error code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_ERROR_OUT_OF_DEVICE_MEMORY: () = ();

/// A requested layer is not present on the system.
/// Most commonly returned from `vkCreateInstance` or extension enumeration 
/// when a specified layer name doesn’t match any installed or registered layer.
/// > This global member only exists to document the error code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_ERROR_LAYER_NOT_PRESENT: () = ();

/// A requested extension is not supported or not available.
/// This can happen if the extension was not exposed by the driver, or if 
/// it wasn't enabled properly when creating the instance or device.
/// > This global member only exists to document the error code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_ERROR_EXTENSION_NOT_PRESENT: () = ();

/// The requested Vulkan version or feature is incompatible with the installed driver.
/// Often returned from `vkCreateInstance` if the ICD cannot support the version requested.
/// Indicates a mismatch between app expectations and driver capability.
/// > This global member only exists to document the error code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_ERROR_INCOMPATIBLE_DRIVER: () = ();

/// Command buffer recording was attempted while already in a recording state,
/// or ended without a matching `vkBeginCommandBuffer`. Can also indicate that a 
/// command was used outside a valid recording session.
/// > This global member only exists to document the error code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_ERROR_COMMAND_BUFFER_RECORDING_IN_PROGRESS: () = ();

/// A command buffer is currently submitted and cannot be reset or modified.
/// Usually happens if you try to reset a buffer still in use by the GPU.
/// > This global member only exists to document the error code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_ERROR_COMMAND_BUFFER_SUBMITTED: () = ();

/// A resource or object was used in an invalid state.
/// This is a general-purpose error for misuse or illegal combinations of states, 
/// and usually indicates a serious bug or API mis-sequencing in your code.
/// > This global member only exists to document the error code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_ERROR_INVALID_STATE: () = ();

/// A surface is no longer available, or the swap-chain is out of date.
/// Commonly returned from `vkAcquireNextImageKHR` or `vkQueuePresentKHR` if the 
/// surface was resized, minimized, or became invalid. Requires recreating the swap-chain.
/// > This global member only exists to document the error code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_ERROR_OUT_OF_DATE_KHR: () = ();

/// A presentation surface has become suboptimal.
/// The swap-chain can still be used, but performance or scaling might be degraded. 
/// Typically, a warning returned from `vkQueuePresentKHR`.
/// > This global member only exists to document the status code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_SUBOPTIMAL_KHR: () = ();

/// Synchronization timeout expired before the operation completed.
/// Most commonly returned by `vkWaitForFences`. The fence or semaphore did not 
/// signal within the specified timeout duration.
/// > This global member only exists to document the status code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_TIMEOUT: () = ();

/// Fence or semaphore was already signaled when waiting.
/// Indicates a wait call completed immediately because the sync object was already in the signaled state.
/// > This global member only exists to document the status code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_EVENT_SET: () = ();

/// An operation or wait returned because the object was not yet ready.
/// This is not an error — it means the object is still in-flight or waiting to be signaled.
/// > This global member only exists to document the status code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_NOT_READY: () = ();

/// An unknown or unrecoverable error occurred.
/// This is the Vulkan equivalent of “something went horribly wrong.” Usually means 
/// driver or implementation failure, hardware fault, or critical corruption.
/// It Should never happen in well-behaved applications and drivers.
/// > This global member only exists to document the error code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_ERROR_UNKNOWN: () = ();


/// The logical device was lost, likely due to a GPU crash, hang, or driver reset.
/// This is a catastrophic failure. After this, the device is unusable and must be destroyed.
/// Recovering typically involves recreating the entire Vulkan context from scratch.
/// > This global member only exists to document the error code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_ERROR_DEVICE_LOST: () = ();

/// An object was accessed in a way not allowed by its current usage.
/// Example: attempting to bind a descriptor set that was never updated.
/// Indicates misusage or incorrect API sequencing.
/// > This global member only exists to document the error code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_ERROR_INVALID_USAGE: () = ();

/// The implementation does not support a requested feature.
/// Commonly returned from `vkCreateDevice` if you enable a feature the physical device doesn’t support.
/// Always query device features first to avoid this.
/// > This global member only exists to document the error code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_ERROR_FEATURE_NOT_PRESENT: () = ();

/// The swap-chain creation failed because the native surface is already in use.
/// Usually returned if the windowing system doesn't allow multiple swap-chains on the same surface,
/// or if another process is holding exclusive access.
/// > This global member only exists to document the error code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_ERROR_NATIVE_WINDOW_IN_USE_KHR: () = ();

/// Validation layer error - pipeline creation failed due to an invalid shader.
/// Usually caused by bad SPIR-V, missing entry points, or mismatched stage interfaces.
/// The pipeline is not usable.
/// > This global member only exists to document the error code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_ERROR_INVALID_SHADER_NV: () = ();

/// An external handle passed into Vulkan was not compatible with the implementation.
/// Common in external memory/semaphore/sync interop cases across processes or APIs.
/// Always validate external handle capabilities before using them.
/// > This global member only exists to document the error code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_ERROR_INVALID_EXTERNAL_HANDLE: () = ();

/// A deferred host operation has not yet completed.
/// Used with `VK_KHR_deferred_host_operations`. This is not an error,
/// but indicates that you must poll or wait again later.
/// > This global member only exists to document the status code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_OPERATION_DEFERRED_KHR: () = ();

/// A deferred host operation has completed successfully.
/// Part of `VK_KHR_deferred_host_operations`. Indicates no further polling is needed.
/// > This global member only exists to document the status code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_OPERATION_NOT_DEFERRED_KHR: () = ();

/// A multithreaded deferred operation worker thread has completed its job and is idle.
/// You can reuse or terminate the thread depending on your threading model.
/// > This global member only exists to document the status code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_THREAD_IDLE_KHR: () = ();

/// All worker threads involved in a deferred host operation have completed successfully.
/// You may now finalize the operation or clean up its resources.
/// > This global member only exists to document the status code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_THREAD_DONE_KHR: () = ();

/// The operation was canceled before completion.
/// Often returned from operations involving `VK_KHR_deferred_host_operations` or
/// when driver-side tasks are aborted by application or system interruption.
/// > This global member only exists to document the error code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_ERROR_OPERATION_DEFERRED_KHR: () = ();

/// The operation is not supported by the current implementation or platform.
/// Can occur with extensions involving external memory, surface support, or advanced sync primitives.
/// > This global member only exists to document the error code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_ERROR_OPERATION_NOT_SUPPORTED_KHR: () = ();

/// A memory mapping or sync operation violated memory access rules.
/// May happen if you map memory twice, use stale mappings, or break coherency contracts.
/// > This global member only exists to document the error code.
/// > It is not used in this program.
#[doc(hidden)]
pub const VK_ERROR_MEMORY_MAP_FAILED: () = ();
