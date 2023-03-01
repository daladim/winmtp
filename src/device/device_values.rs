use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};
use windows::core::{GUID, PCWSTR};

use windows::Win32::Storage::FileSystem::SECURITY_IMPERSONATION;
use windows::Win32::Devices::PortableDevices::PortableDeviceValues;
use windows::Win32::Devices::PortableDevices::IPortableDeviceValues;
use windows::Win32::Devices::PortableDevices::WPD_CLIENT_NAME;
use windows::Win32::Devices::PortableDevices::WPD_CLIENT_MAJOR_VERSION;
use windows::Win32::Devices::PortableDevices::WPD_CLIENT_MINOR_VERSION;
use windows::Win32::Devices::PortableDevices::WPD_CLIENT_REVISION;
use windows::Win32::Devices::PortableDevices::WPD_CLIENT_SECURITY_QUALITY_OF_SERVICE;

/// Identifies the current application
///
/// This is required by the Windows drivers
#[derive(Debug, Clone)]
pub struct AppIdentifiers {
    pub app_name: String,
    pub app_major: u32,
    pub app_minor: u32,
    pub app_patch: u32,
}

/// Create an instance of `AppIdentifiers` with the values of the current app.
///
/// Because this macro fetches values from Cargo environment variables, this must be
/// called by a **binary** app code, and not from any lib. Otherwise, this will build
/// an instance that contain the lib name and version, which is probably not what you
/// want.
#[macro_export]
macro_rules! make_current_app_identifiers {
    () => {
        AppIdentifiers{
            app_name: env!("CARGO_PKG_NAME").to_string(),
            app_major: env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap_or(0),
            app_minor: env!("CARGO_PKG_VERSION_MINOR").parse().unwrap_or(0),
            app_patch: env!("CARGO_PKG_VERSION_PATCH").parse().unwrap_or(0),
        }
    };
}

/// Create a IPortableDeviceValues instance with suggestions from https://learn.microsoft.com/en-us/windows/win32/wpd_sdk/specifying-client-information
///
pub fn make_device_values(current_app_identifiers: &AppIdentifiers) -> crate::WindowsResult<IPortableDeviceValues> {
    let device_values: IPortableDeviceValues = unsafe {
        CoCreateInstance(
            &PortableDeviceValues as *const GUID,
            None,
            CLSCTX_ALL
        )
    }?;

    // At a minimum, your application should provide a string containing its name, a major version number, a minor version number, and a revision number. These are the fields supplied by the sample application.
    // See https://learn.microsoft.com/en-us/windows/win32/wpd_sdk/specifying-client-information

    let current_app_name_utf16: Vec<u16> = current_app_identifiers.app_name.encode_utf16().collect();
    let pcwstr_current_app_name = PCWSTR::from_raw(current_app_name_utf16.as_ptr());

    unsafe{ device_values.SetStringValue(&WPD_CLIENT_NAME as *const _, pcwstr_current_app_name) }?;
    unsafe{ device_values.SetUnsignedIntegerValue(&WPD_CLIENT_MAJOR_VERSION as *const _, current_app_identifiers.app_major) }?;
    unsafe{ device_values.SetUnsignedIntegerValue(&WPD_CLIENT_MINOR_VERSION as *const _, current_app_identifiers.app_minor) }?;
    unsafe{ device_values.SetUnsignedIntegerValue(&WPD_CLIENT_REVISION as *const _, current_app_identifiers.app_patch) }?;
    // Some device drivers need to impersonate the caller in order to function correctly.  Since our application does not
    // need to restrict its identity, specify SECURITY_IMPERSONATION so that we work with all devices.
    // See https://learn.microsoft.com/en-us/windows/win32/wpd_sdk/specifying-client-information
    unsafe{ device_values.SetUnsignedIntegerValue(&WPD_CLIENT_SECURITY_QUALITY_OF_SERVICE as *const _, SECURITY_IMPERSONATION.0) }?;

    Ok(device_values)
}
