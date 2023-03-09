//! MTP object (can be a folder, a file, etc.)

use std::path::{Path, Components, Component};
use std::iter::Peekable;

use windows::core::{PWSTR, PCWSTR};
use windows::Win32::System::Com::{IStream, STGM, STGM_READ, CoTaskMemFree};
use windows::Win32::Devices::PortableDevices::{WPD_OBJECT_PARENT_ID, WPD_RESOURCE_DEFAULT};
use widestring::{U16CString, U16CStr};

use crate::device::Content;
use crate::device::device_values::make_values_for_create_folder;
use crate::error::{ItemByPathError, OpenStreamError};
use crate::io::ReadStream;

mod object_id;
pub use object_id::ObjectId;

mod object_type;
pub use object_type::ObjectType;

mod object_iterator;
pub use object_iterator::ObjectIterator;


#[derive(Debug, Clone)]
pub struct Object {
    device_content: Content,
    /// The MTP ID of the object (e.g. "o2C")
    id: U16CString,
    /// The object display name (e.g. "PIC_001.jpg")
    name: U16CString,
    ty: ObjectType,
}

impl Object {
    pub fn new(device_content: Content, id: U16CString, name: U16CString, ty: ObjectType) -> Self {
        Self { device_content, id, name, ty }
    }

    pub(crate) fn device_content(&self) -> &Content {
        &self.device_content
    }

    pub fn id(&self) -> &U16CStr {
        &self.id
    }

    pub fn name(&self) -> &U16CStr {
        // TODO: lazy evaluation (of all properties at once to save calls to properties.GetValues) (depends on how much iterating/filtering by folder is baked-in)?
        &self.name
    }

    pub fn object_type(&self) -> ObjectType {
        // TODO: lazy evaluation?
        self.ty
    }

    pub fn parent_id(&self) -> crate::WindowsResult<U16CString> {
        let parent_id_props = self.device_content.get_object_properties(&self.id, &[WPD_OBJECT_PARENT_ID])?;
        let parent_id_pwstr = unsafe{ parent_id_props.GetStringValue(&WPD_OBJECT_PARENT_ID as *const _) }?;
        Ok(U16CString::from_vec_truncate(unsafe{ parent_id_pwstr.as_wide() }))
    }

    /// Returns an iterator to list every children of the current object (including sub-folders)
    pub fn children(&self) -> crate::WindowsResult<ObjectIterator> {
        let com_iter = unsafe{
            self.device_content.com_object().EnumObjects(
                0,
                PCWSTR::from_raw(self.id.as_ptr()),
                None,
            )
        }?;

        Ok(ObjectIterator::new(&self.device_content, com_iter))
    }

    /// Returns an iterator that only lists folders within this object
    pub fn sub_folders(&self) -> crate::WindowsResult<impl Iterator<Item = Object> + '_> {
        self.children().map(|children| children.filter(|obj| obj.object_type() == ObjectType::Folder))
    }

    /// Retrieve an item by its path
    ///
    /// This function looks for a sub-item with the right name, then iteratively does so for the matching child.<br/>
    /// This is quite expensive. Depending on your use-cases, you may want to cache some "parent folders" that you will want to access often.<br/>
    /// Note that caching however defeats the purpose of MTP, which is supposed to _not_ use any cache, so that it guarantees there is no race between concurrent accesses to the same medium.
    pub fn object_by_path(&self, relative_path: &Path) -> Result<Object, ItemByPathError> {
        let mut comps = relative_path.components().peekable();
        self.object_by_components(&mut comps)
    }

    fn object_by_components(&self, comps: &mut Peekable<Components>) -> Result<Object, ItemByPathError> {
        match comps.next() {
            Some(Component::Normal(name)) => {
                let haystack = U16CString::from_os_str_truncate(name);
                let candidate = self
                    .children()?
                    .find(|obj| obj.name() == haystack)
                    .ok_or(ItemByPathError::NotFound)?;

                object_by_components_last_stage(candidate, comps)
            },

            Some(Component::CurDir) => {
                object_by_components_last_stage(self.clone(), comps)
            },

            Some(Component::ParentDir) => {
                let candidate = self
                    .device_content
                    .object_by_id(self.parent_id()?)?;

                object_by_components_last_stage(candidate, comps)
            }

            Some(Component::Prefix(_)) |
            Some(Component::RootDir) =>
                Err(ItemByPathError::AbsolutePath),

            None => Err(ItemByPathError::NotFound)
        }
    }

    /// Opens a COM [`IStream`](windows::Win32::System::Com::IStream) to this object.
    ///
    /// An error will be returned if the required operation does not make sense (e.g. get a stream to a folder).
    ///
    /// Also returns the optimal transfer buffer size (in bytes) for this transfer, as stated by the Microsoft API.
    ///
    /// See also [`Self::open_read_stream`] and [`Self::open_write_stream`].
    pub fn open_raw_stream(&self, stream_mode: STGM) -> Result<(IStream, u32), OpenStreamError> {
        let resources = unsafe{ self.device_content.com_object().Transfer()? };

        let mut stream = None;
        let mut optimal_transfer_size_bytes: u32 = 0;
        unsafe{ resources.GetStream(
            PCWSTR::from_raw(self.id.as_ptr()),
            &WPD_RESOURCE_DEFAULT as *const _,  // We are transferring the default resource (which is the entire object's data)
            stream_mode.0,
            &mut optimal_transfer_size_bytes as *mut u32,
            &mut stream as *mut Option<IStream>,
        )}?;

        match stream {
            None => Err(OpenStreamError::UnableToCreate),
            Some(s) => Ok((s, optimal_transfer_size_bytes)),
        }
    }

    /// The same as [`Self::open_raw_stream`], but wrapped into a [`crate::io::ReadStream`] for more added Rust magic.
    ///
    /// # Example
    /// ```
    /// # let provider = winmtp::Provider::new().unwrap();
    /// # let basic_device = provider.enumerate_devices().unwrap()[0];
    /// # let app_identifiers = winmtp::make_current_app_identifiers!();
    /// # let device = basic_device.open(&app_identifiers).unwrap();
    /// let object = device.content().unwrap().object_by_id(some_id).unwrap();
    /// let mut input_stream = object.open_read_stream().unwrap();
    /// let mut output_file = std::fs::File::create("pulled-from-device.dat").unwrap();
    /// std::io::copy(&mut input_stream, &mut output_file).unwrap();
    /// ```
    pub fn open_read_stream(&self) -> Result<ReadStream, OpenStreamError> {
        let (stream, optimal_transfer_size) = self.open_raw_stream(STGM_READ)?;
        Ok(ReadStream::new(stream, optimal_transfer_size as usize))
    }

    /// Create a subfolder, and return its object ID
    pub fn create_subfolder(&self, folder_name: &str) -> Result<U16CString, CreateFolderError> {
        // Check if such an item already exist (otherwise, `CreateObjectWithPropertiesOnly` would return an unhelpful "Unspecified error ")
        if let Ok(_existing_item) = self.object_by_path(Path::new(folder_name)) {
            return Err(CreateFolderError::AlreadyExists)
        }

        let folder_properties = make_values_for_create_folder(&self.id, folder_name)?;
        let mut created_object_id = PWSTR::null();
        unsafe{ self.device_content.com_object().CreateObjectWithPropertiesOnly(
            &folder_properties,
            &mut created_object_id as *mut _,
        )}?;

        let owned_id = unsafe{ U16CString::from_ptr_str(created_object_id.as_ptr()) };
        unsafe{
            CoTaskMemFree(Some(created_object_id.as_ptr() as *const _))
        };

        Ok(owned_id)
    }
}


fn object_by_components_last_stage(candidate: Object, next_components: &mut Peekable<Components>) -> Result<Object, ItemByPathError> {
    match next_components.peek() {
        None => {
            // We've reached the end of the required path
            // This means the candidate is the object we wanted
            Ok(candidate)
        },
        Some(_) => {
            candidate.object_by_components(next_components)
        }
    }
}
