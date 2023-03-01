//! Safe Rust abstraction over Windows MTP API
//!
//! Microsoft provides a [COM API for WPD (Windows Portable Devices)](https://learn.microsoft.com/en-us/windows/win32/wpd_sdk/programming-guide).<br/>
//! It also provides [raw bindings over these COM APIs](https://microsoft.github.io/windows-docs-rs/doc/windows/Win32/Devices/PortableDevices/index.html).
//!
//! This create provides a safe Rust abstraction over this API.
//!
//! # Important note
//!
//! The WPD API is fairly large. This crate only provides a small fraction of what is possible.<br/>
//! Basically, only device enumeration, simple content enumeration, and content transfer are implemented.<br/>
//! Much can be added in the future. Contributions are welcome!
//!
//! # Usage
//!
//! The entry point of this library is to create a `Provider`, e.g. by [`Provider::new`]. Other structs can be created from its various methods.

mod provider;
pub use provider::Provider;

pub mod device;
pub mod object;

pub mod error;

/// Re-exported from the windows-rs crate, because it is used in our public API.<br/>
pub use windows::core::Result as WindowsResult;
/// Re-exported from the windows-rs crate, because it is used in our public API.<br/>
pub use windows::core::Error as WindowsError;
/// Re-exported from the windows-rs crate, because it is used in our public API.<br/>
use windows::Win32::Devices::PortableDevices::IPortableDevice;
