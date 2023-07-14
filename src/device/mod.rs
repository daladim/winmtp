use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};
use windows::core::{GUID, PCWSTR} ;
use windows::Win32::Devices::PortableDevices::PortableDeviceFTM;
use widestring::U16CString;

use crate::IPortableDevice;
use crate::device::device_values::AppIdentifiers;

pub mod device_values;

mod content;
pub use content::Content;

/// Basic info about an MTP device
///
/// To access its content, you must call [`BasicDevice::open`]
#[derive(Clone)]
pub struct BasicDevice {
    device_id: U16CString,
    friendly_name: String,
}

impl BasicDevice {
    pub(crate) fn new(device_id: U16CString, friendly_name: String) -> Self {
        Self{ device_id, friendly_name }
    }

    pub fn device_id(&self) -> String {
        self.device_id.to_string_lossy() // We trust Windows for not providing invalid UTF-16 characters
    }

    pub fn friendly_name(&self) -> &str {
        &self.friendly_name
    }

    /// Turns this device into an "opened" device, that has more features
    /// (e.g. being able to browse its content)
    ///
    /// Some devices (e.g. ones that are backed by a FAT filesystem) use case-insensitive paths. In this case, you want to set `case_sensitive` to false.
    /// Otherwise, you would often get `Err`s, e.g. when you try to create or replace a file (or folder) with a similar name but different casing.<br/>
    /// Unfortunately, the Windows API does not look to be able to give this info.
    pub fn open(&self, app_identifiers: &AppIdentifiers, case_sensitive_fs: bool) -> crate::WindowsResult<Device> {
        // Fill out information about your application, so the device knows
        // who they are speaking to.
        let device_values = device_values::make_values_for_open_device(app_identifiers)?;

        let com_device: IPortableDevice = unsafe {
            CoCreateInstance(
                &PortableDeviceFTM as *const GUID,
                None,
                CLSCTX_ALL
            )
        }?;

        unsafe { com_device.Open(PCWSTR::from_raw(self.device_id.as_ptr()), &device_values) }.unwrap();

        Ok(Device{
            basic_device: self.clone(),
            com_device,
            case_sensitive_fs,
        })
    }
}

/// An MTP device that as been opened
pub struct Device {
    basic_device: BasicDevice,
    com_device: IPortableDevice,
    case_sensitive_fs: bool,
}

impl Device {
    /// Returns the underlying COM object
    ///
    /// This is useful in case you want to call a function that has no Rust wrapper (yet?) in this crate
    pub fn raw_device(&self) -> &IPortableDevice {
        &self.com_device
    }

    pub fn content(&self) -> crate::WindowsResult<Content> {
        let com_content = unsafe { self.com_device.Content() }?;
        Ok(Content::new(com_content, self.case_sensitive_fs))
    }
}
