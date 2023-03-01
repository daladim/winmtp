//! Access to content of a device

use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};
use windows::core::{PCWSTR, GUID};
use windows::Win32::Devices::PortableDevices::{
    IPortableDeviceContent, IPortableDeviceKeyCollection, PortableDeviceKeyCollection, IPortableDeviceValues, WPD_OBJECT_NAME, WPD_OBJECT_CONTENT_TYPE, WPD_DEVICE_OBJECT_ID
};
use windows::Win32::UI::Shell::PropertiesSystem::PROPERTYKEY;
use widestring::{U16CString, U16CStr};

use crate::object::{Object, ObjectType};

#[derive(Debug, Clone)]
/// Abstraction over the content of a device
pub struct Content{
    com_content: IPortableDeviceContent,
}

impl Content {
    pub(crate) fn new(com_content: IPortableDeviceContent) -> Self {
        Self{ com_content }
    }

    /// Retrieve the inner COM object, in case one wants to call a method for which there is no wrapper in this crate
    pub fn com_object(&self) -> &IPortableDeviceContent {
        &self.com_content
    }

    /// Get the root object of the current device
    pub fn root(&self) -> crate::WindowsResult<Object> {
        self.object_by_id(unsafe{ U16CString::from_ptr_str(WPD_DEVICE_OBJECT_ID.as_ptr()) })
    }

    /// Get an MPT object given its MTP object ID
    pub fn object_by_id(&self, object_id: U16CString) -> crate::WindowsResult<Object> {
        // Get object name and type
        let basic_properties = self.get_object_properties(&object_id, &[WPD_OBJECT_NAME, WPD_OBJECT_CONTENT_TYPE])?;

        let name_pwstr = unsafe{ basic_properties.GetStringValue(&WPD_OBJECT_NAME as *const _) }?;
        let name = U16CString::from_vec_truncate(unsafe{ name_pwstr.as_wide() });
        let ty_guid = unsafe{ basic_properties.GetGuidValue(&WPD_OBJECT_CONTENT_TYPE as *const _) }?;
        let object_type = ObjectType::from_guid(ty_guid);

        Ok(Object::new(self.clone(), object_id, name, object_type))
    }

    pub fn get_object_properties(&self, object_id: &U16CStr, properties_to_fetch: &[PROPERTYKEY]) -> crate::WindowsResult<IPortableDeviceValues> {
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
        }
    }
}
