#[macro_export]
macro_rules! enum_impl {
    (
        $(#[$outer:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$inner:meta])*
                $variant:ident = $ext:expr,
            )+
        }
    ) => {
        $(#[$outer])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis enum $name {
            $(
                $(#[$inner])*
                $variant,
            )+
        }

        impl $name {
            $(
                $vis const $variant: $crate::ExtensionStr = $ext;
            )+

            /// Returns the Vulkan `ExtensionName` associated with this extension enum.
            pub fn name(&self) -> &'static $crate::ExtensionStr {
                match self {
                    $(
                        Self::$variant => &Self::$variant,
                    )+
                }
            }

            /// Constructs the enum variant from a raw Vulkan extension name.
            pub fn from(name: &$crate::ExtensionStr) -> Self {
                $(
                    if name == &Self::$variant {
                        return Self::$variant;
                    }
                )+
                panic!("Unknown extension name: {:?}", name);
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                // Convert to &CStr and then to String
                use std::ffi::CStr;
                let cstr = unsafe { CStr::from_ptr(self.name().as_ptr()) };
                write!(f, "{}", cstr.to_string_lossy())
            }
        }
    };
}
