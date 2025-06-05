pub(crate) const VALIDATION_ENABLED: bool = cfg!(debug_assertions);
pub(crate) const RENDERDOC_ENABLED: bool = cfg!(feature = "renderdoc_enabled");
pub(crate) const API_DUMP_ENABLED: bool = cfg!(feature = "api_dump_enabled");
