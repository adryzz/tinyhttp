#[cfg(feature = "defmt")]
#[doc(hidden)]
pub use defmt;

#[cfg(feature = "log")]
#[doc(hidden)]
pub use log;

#[macro_export]
#[macro_use]
#[doc(hidden)]
macro_rules! log {
    ($level:ident, $($args:tt)*) => {
        $crate::defmt_with!($level, $($args)*);

        $crate::log_with!($level, $($args)*);
    }
}

#[macro_export]
#[doc(hidden)]
#[cfg(feature = "defmt")]
macro_rules! defmt_with {
    ($level:ident, $($args:tt)*) => {
        $crate::logging::defmt::$level!($($args)*);
    }
}

#[macro_export]
#[doc(hidden)]
#[cfg(feature = "log")]
macro_rules! log_with {
    ($level:ident, $($args:tt)*) => {
        $crate::logging::log::$level!($($args)*);
    }
}

#[macro_export]
#[doc(hidden)]
#[cfg(not(feature = "defmt"))]
macro_rules! defmt_with {
    ($level:ident, $($args:tt)*) => {};
}

#[macro_export]
#[doc(hidden)]
#[cfg(not(feature = "log"))]
macro_rules! log_with {
    ($level:ident, $($args:tt)*) => {};
}
