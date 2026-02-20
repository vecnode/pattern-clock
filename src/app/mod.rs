// Platform-specific app components

#[cfg(feature = "desktop")]
pub mod desktop;

#[cfg(any(feature = "web", feature = "server"))]
pub mod web;
