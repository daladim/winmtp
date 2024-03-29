//! Access to content of a device

use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};
use windows::core::{PCWSTR, GUID};
use windows::Win32::Devices::PortableDevices::{
    IPortableDeviceContent, IPortableDeviceKeyCollection, PortableDeviceKeyCollection, WPD_OBJECT_NAME, WPD_OBJECT_CONTENT_TYPE, WPD_DEVICE_OBJECT_ID
};
use widestring::{U16CString, U16CStr};

use crate::object::{Object, ObjectType};
use crate::device::device_values::DeviceValues;

#[derive(Debug, Clone)]
/// Abstraction over the content of a device
pub struct Content{
    com_content: IPortableDeviceContent,
    case_sensitive_fs: bool,
}

impl Content {
    pub(crate) fn new(com_content: IPortableDeviceContent, case_sensitive_fs: bool) -> Self {
        Self{ com_content, case_sensitive_fs }
    }

    /// Retrieve the inner COM object, in case one wants to call a method for which there is no wrapper in this crate
    pub fn com_object(&self) -> &IPortableDeviceContent {
        &self.com_content
    }

    pub fn case_sensitive_fs(&self) -> bool {
        self.case_sensitive_fs
    }

    /// Get the root object of the current device
    pub fn root(&self) -> crate::WindowsResult<Object> {
        self.object_by_id(unsafe{ U16CString::from_ptr_str(WPD_DEVICE_OBJECT_ID.as_ptr()) })
    }

    /// List all functional objects for this device
    pub fn functional_objects(&self) -> crate::WindowsResult<Vec<Object>> {
        Ok(self.root()?
            .children()?
            .filter(|child| child.object_type() == ObjectType::FunctionalObject)
            .collect())
    }

    /// Get an MTP object given its MTP object ID
    pub fn object_by_id(&self, object_id: U16CString) -> crate::WindowsResult<Object> {
        // Get object name and type
        let basic_properties = self.properties(&object_id, &[WPD_OBJECT_NAME, WPD_OBJECT_CONTENT_TYPE])?;

        let name = basic_properties.get_string(&WPD_OBJECT_NAME)?;
        let ty_guid = basic_properties.get_guid(&WPD_OBJECT_CONTENT_TYPE)?;
        let object_type = ObjectType::from_guid(ty_guid);

        Ok(Object::new(self.clone(), object_id, name, object_type))
    }

    /// Get a list of requested metadata about an object.
    pub fn properties(&self, object_id: &U16CStr, properties_to_fetch: &[crate::PROPERTYKEY]) -> crate::WindowsResult<DeviceValues> {
        let props_to_read: IPortableDeviceKeyCollection = unsafe {
            CoCreateInstance(
                &PortableDeviceKeyCollection as *const GUID,
                None,
                CLSCTX_ALL
            )
        }?;
        for prop_to_fetch in properties_to_fetch {
            unsafe{ props_to_read.Add(prop_to_fetch as *const _)}?;
        }

        let properties = unsafe{ self.com_content.Properties() }?;
        unsafe{ properties.GetValues(
                PCWSTR::from_raw(object_id.as_ptr()),
                &props_to_read,
            )
        }.map(|dv| DeviceValues::new(dv))
    }
}
