use windows::core::PWSTR;
use windows::Win32::Devices::PortableDevices::IEnumPortableDeviceObjectIDs;
use widestring::U16CString;

use crate::device::Content;
use crate::object::Object;

pub struct ObjectIterator<'content> {
    device_content: &'content Content,
    com_iter: IEnumPortableDeviceObjectIDs,
}

impl<'content> ObjectIterator<'content> {
    pub(crate) fn new(device_content: &'content Content, com_iter: IEnumPortableDeviceObjectIDs) -> Self {
        Self{ device_content, com_iter }
    }
}

impl<'content> std::iter::Iterator for ObjectIterator<'content> {
    type Item = Object;

    fn next(&mut self) -> Option<Self::Item> {
        let mut out = [PWSTR::null()];
        let mut requested: u32 = 1;
        unsafe{ self.com_iter.Next(&mut out, &mut requested as *mut u32) }.ok().ok()?;
        if requested != 1 {
            return None;
        }

        let single_child = out.get(0)?;  // cannot return None, `out` is a 1-item array
        let child_widestring = unsafe{ U16CString::from_ptr_str(single_child.as_ptr()) };

        self.device_content.object_by_id(child_widestring).ok()
    }
}
