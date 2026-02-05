#[macro_export]
macro_rules! enum_impl {
    (
        $(#[$outer:meta])*
        $enum_vis:vis enum $name:ident {
            $(
                $(#[$inner:meta])*
                $variant:ident = $ext:expr,
            )+
        }
    ) => {
        $(#[$outer])*
        $enum_vis enum $name {
            $(
                $(#[$inner])*
                $variant,
            )+
        }

        impl $name {
            /// Returns the Vulkan name buffer with stable storage (good for comparisons).
            #[inline]
            pub fn name_buf(self) -> &'static ExtensionStr {
                match self {
                    $(
                        Self::$variant => {
                            // One static per variant, stable address, not a temporary.
                            static BUF: ExtensionStr = $ext;
                            &BUF
                        }
                    )+
                }
            }

            /// Returns the Vulkan name pointer for FFI (NUL-terminated).
            #[inline]
            pub fn name_ptr(self) -> *const ::std::ffi::c_char {
                self.name_buf().as_ptr() as *const ::std::ffi::c_char
            }

            /// Constructs the enum variant from a raw Vulkan extension/layer name.
            #[inline]
            pub fn try_from_name(name: &ExtensionStr) -> Option<Self> {
                $(
                    if name == Self::$variant.name_buf() {
                        return Some(Self::$variant);
                    }
                )+
                None
            }

            /// Like `try_from_name`, but panics on unknown names.
            #[inline]
            pub fn from_name(name: &ExtensionStr) -> Self {
                Self::try_from_name(name)
                    .unwrap_or_else(|| panic!("Unknown extension name: {:?}", name))
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                use ::std::ffi::CStr;
                let cstr = unsafe { CStr::from_ptr(self.name_ptr()) };
                f.write_str(&cstr.to_string_lossy())
            }
        }
    };
}
