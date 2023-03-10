//! MTP object (can be a folder, a file, etc.)

use std::path::{Path, Components, Component};
use std::iter::Peekable;

use windows::core::{GUID, PWSTR, PCWSTR};
use windows::Win32::System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_ALL};
use windows::Win32::System::Com::{IStream, STGM, STGM_READ};
use windows::Win32::System::Com::StructuredStorage::PROPVARIANT;
use windows::Win32::Devices::PortableDevices::{PortableDevicePropVariantCollection, IPortableDevicePropVariantCollection, PORTABLE_DEVICE_DELETE_WITH_RECURSION, PORTABLE_DEVICE_DELETE_NO_RECURSION, WPD_OBJECT_PARENT_ID, WPD_RESOURCE_DEFAULT};
use widestring::{U16CString, U16CStr};

use crate::device::Content;
use crate::device::device_values::{make_values_for_create_folder, make_values_for_create_file};
use crate::error::{ItemByPathError, OpenStreamError, CreateFolderError, AddFileError};
use crate::io::{ReadStream, WriteStream};

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

    /// Add a file into the current directory
    pub fn push_file(&self, local_file: &Path) -> Result<(), AddFileError> {
        let file_name = local_file.file_name().ok_or(AddFileError::InvalidLocalFile)?.to_string_lossy().to_string();
        let file_size = local_file.metadata()?.len();

        let mut source_reader = std::fs::File::open(local_file)?;

        let file_properties = make_values_for_create_file(&self.id, &file_name, file_size)?;
        let mut write_stream = None;
        let mut optimal_write_buffer_size = 0;
        unsafe{ self.device_content.com_object().CreateObjectWithPropertiesAndData(
            &file_properties,
            &mut write_stream as *mut _,
            &mut optimal_write_buffer_size,
            &mut PWSTR::null() as *mut PWSTR,
        )}?;

        let write_stream = write_stream.ok_or(AddFileError::UnableToCreate)?;
        let mut dest_writer = WriteStream::new(write_stream, optimal_write_buffer_size as usize);
        std::io::copy(&mut source_reader, &mut dest_writer)?;

        dest_writer.commit()?;

        Ok(())
    }

    /// Delete an object
    ///
    /// If this is a folder, you must set `recursive` to `true`, otherwise this would return an error.
    pub fn delete(mut self, recursive: bool) -> crate::WindowsResult<()> {
        let id_as_propvariant = unsafe{ init_propvariant_from_string(&mut self.id) };

        let objects_to_delete: IPortableDevicePropVariantCollection = unsafe {
            CoCreateInstance(
                &PortableDevicePropVariantCollection as *const GUID,
                None,
                CLSCTX_ALL
            )
        }.unwrap();
        unsafe{ objects_to_delete.Add(&id_as_propvariant as *const _) }.unwrap();

        let options = if recursive { PORTABLE_DEVICE_DELETE_WITH_RECURSION } else { PORTABLE_DEVICE_DELETE_NO_RECURSION };
        let mut result_status = None;
        unsafe{
            self.device_content.com_object().Delete(
                options.0 as u32,
                &objects_to_delete,
                &mut result_status as *mut _,
            )
        }.unwrap();

        Ok(())
    }

    /// Move an object that is already on the device to a new folder
    pub fn move_to(&mut self, new_folder_id: &U16CStr) -> crate::WindowsResult<()>  {
        let id_as_propvariant = unsafe{ init_propvariant_from_string(&mut self.id) };

        let objects_to_move: IPortableDevicePropVariantCollection = unsafe {
            CoCreateInstance(
                &PortableDevicePropVariantCollection as *const GUID,
                None,
                CLSCTX_ALL
            )
        }.unwrap();
        unsafe{ objects_to_move.Add(&id_as_propvariant as *const _) }.unwrap();

        let dest = PCWSTR::from_raw(new_folder_id.as_ptr());
        let mut result_status = None;
        unsafe{
            self.device_content.com_object().Move(
                &objects_to_move,
                dest,
                &mut result_status as *mut _,
            )
        }.unwrap();

        Ok(())
    }
}


/// Re-implementation of `InitPropVariantFromString`, which is missing in windows-rs.
/// See https://github.com/microsoft/windows-rs/issues/976#issuecomment-878697273
///
/// # Safety
///
/// I'm too lazy to wrap to result with a 'a PhantomData, so for now, the result is only valid as long `data` is valid.
unsafe fn init_propvariant_from_string(data: &mut U16CStr) -> PROPVARIANT {
    windows::Win32::System::Com::StructuredStorage::PROPVARIANT{
        Anonymous: windows::Win32::System::Com::StructuredStorage::PROPVARIANT_0 {
            Anonymous: std::mem::ManuallyDrop::new(windows::Win32::System::Com::StructuredStorage::PROPVARIANT_0_0 {
                vt: windows::Win32::System::Com::VT_LPWSTR,
                Anonymous: windows::Win32::System::Com::StructuredStorage::PROPVARIANT_0_0_0 {
                    pwszVal: PWSTR::from_raw(data.as_mut_ptr()),
                },
                ..Default::default()
            })
        },
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
