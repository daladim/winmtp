//! These test should succeed when an Android device is connected.

use std::ffi::OsStr;
use std::io::Write;
use std::path::{Path, PathBuf};

use widestring::U16CString;

use winmtp::PortableDevices::WPD_OBJECT_SIZE;
use winmtp::Provider;
use winmtp::device::BasicDevice;
use winmtp::object::ObjectType;
use winmtp::object::Object;

const EXAMPLE_SONG: &str = r"tests\assets\Rough Draft (open source mp3 from audiohub.com).mp3";
const PLAYLIST_CONTENT: &str = "This is not a valid M3U file, but ideally it should";

#[derive(Debug, Clone, Copy)]
enum DeviceKind {
    GenericAndroid,
    Kindle
}

impl DeviceKind {
    fn storage_root_name(&self) -> &'static str {
        match self {
            DeviceKind::GenericAndroid => "Internal shared storage",
            DeviceKind::Kindle => "Internal Storage",
        }
    }

    fn downloads_dir_name(&self) -> &'static str {
        match self {
            DeviceKind::GenericAndroid => "Download",
            DeviceKind::Kindle => "downloads",
        }
    }

    fn downloads_dir_path(&self) -> PathBuf {
        // r"Internal shared storage\Download\"
        PathBuf::from(format!(r"{}\{}\", self.storage_root_name(), self.downloads_dir_name()))
    }
    fn downloads_dir_path_with_dot_segments(&self) -> PathBuf {
        // r"Internal shared storage\.\.\Download\.\"
        PathBuf::from(format!(r"{}\.\.\{}\.\", self.storage_root_name(), self.downloads_dir_name()))
    }
    fn download_path_playlist_file(&self) -> PathBuf {
        // r"Internal shared storage\.\.\Download\winmtp_test\.\some_playlist.m3u"
        PathBuf::from(format!(r"{}\.\.\{}\winmtp_test\.\some_playlist.m3u", self.storage_root_name(), self.downloads_dir_name()))
    }
    fn downloads_dir_path_with_redundant_separators(&self) -> PathBuf {
        // r"Internal shared storage\\\Download\\\"
        PathBuf::from(format!(r"{}\\\{}\\\", self.storage_root_name(), self.downloads_dir_name()))
    }
    fn nonexistent_path_under_downloads(&self) -> PathBuf {
        // r"Internal shared storage\\\Download\\\this_does_not_exist"
        PathBuf::from(format!(r"{}\\\{}\\\this_does_not_exist", self.storage_root_name(), self.downloads_dir_name()))
    }
    fn downloads_dir_path_with_parent_roundtrip(&self) -> PathBuf {
        // r"Internal shared storage\\\Download\\\..\Download"
        PathBuf::from(format!(r"{}\\\{}\\\..\{}", self.storage_root_name(), self.downloads_dir_name(), self.downloads_dir_name()))
    }
    fn downloads_dir_path_with_nested_parent_segments(&self) -> PathBuf {
        // r"Internal shared storage\\\Download\\\winmtp_test\..\..\Download"
        PathBuf::from(format!(r"{}\\\{}\\\winmtp_test\..\..\{}", self.storage_root_name(), self.downloads_dir_name(), self.downloads_dir_name()))
    }
    fn storage_root_parent_path(&self) -> PathBuf {
        // r"Internal shared storage\.."
        PathBuf::from(format!(r"{}\..", self.storage_root_name()))
    }
    fn storage_root_parent_path_with_trailing_slash(&self) -> PathBuf {
        // r"Internal shared storage\..\"
        PathBuf::from(format!(r"{}\..\", self.storage_root_name()))
    }
    fn uploaded_mp3_path(&self) -> PathBuf {
        // r"Internal shared storage\Download\winmtp_test\Rough Draft (open source mp3 from audiohub.com).mp3"
        PathBuf::from(format!(r"{}\{}\winmtp_test\Rough Draft (open source mp3 from audiohub.com).mp3", self.storage_root_name(), self.downloads_dir_name()))
    }
    fn write_stream_file_path(&self) -> PathBuf {
        PathBuf::from(format!(r"{}\{}\winmtp_test\file_pushed_via_create_write_stream.mp3", self.storage_root_name(), self.downloads_dir_name()))
    }
}

fn get_device_kind(basic_device: &BasicDevice) -> DeviceKind {
    match basic_device.friendly_name().to_lowercase() {
        s if s.contains("kindle") => DeviceKind::Kindle,
        s if s.contains("android") || s.contains("moto") => DeviceKind::GenericAndroid,
        s => panic!("No testing paths for friendly name {}", s)
    }
}

#[test]
fn file_access() {
    // This is a manual smoke test rather than a proper automated test, as this requires a device to be connected, with some assumptions about its content

    let provider = Provider::new().unwrap();
    let devices = provider.enumerate_devices().unwrap();
    let first_device = devices.get(0).expect("a device to be connected");
    let device_kind = get_device_kind(first_device);

    println!("Testing on {}:", first_device.friendly_name());
    push_content(first_device, device_kind);
    access_by_path(first_device, device_kind);
    access_by_id(first_device, device_kind);
    pull_content(first_device, device_kind);
    write_file_via_create_write_stream(first_device, device_kind);
    verify_file_written_via_create_write_stream(first_device, device_kind);
}

fn access_by_path(basic_device: &BasicDevice, device_kind: DeviceKind) {
    let app_ident = winmtp::make_current_app_identifiers!();

    let device = basic_device.open(&app_ident, true).unwrap();
    let content = device.content().unwrap();

    let root_obj = content.root().unwrap();
    assert_eq!(root_obj.object_type(), ObjectType::FunctionalObject);

    let object_by_path = root_obj.object_by_path(Path::new(&device_kind.downloads_dir_path())).unwrap();
    assert_eq!(object_by_path.object_type(), ObjectType::Folder);

    let object_by_path = root_obj.object_by_path(Path::new(&device_kind.downloads_dir_path_with_dot_segments())).unwrap();
    assert_eq!(object_by_path.object_type(), ObjectType::Folder);

    let object_by_path = root_obj.object_by_path(Path::new(&device_kind.download_path_playlist_file())).unwrap();
    match device_kind {
        // kindles seem to report the object_type of .m3u files as unspecified
        DeviceKind::Kindle => assert_eq!(object_by_path.object_type(), ObjectType::Unspecified),
        _ => assert_eq!(object_by_path.object_type(), ObjectType::Playlist)
    }   

    let object_by_path = root_obj.object_by_path(&device_kind.downloads_dir_path_with_redundant_separators()).unwrap();
    assert_eq!(object_by_path.object_type(), ObjectType::Folder);

    let object_by_path = root_obj.object_by_path(&device_kind.nonexistent_path_under_downloads());
    assert!(object_by_path.is_err());

    let object_by_path = root_obj.object_by_path(Path::new(&device_kind.downloads_dir_path_with_parent_roundtrip())).unwrap();
    assert_eq!(object_by_path.object_type(), ObjectType::Folder);

    let object_by_path = root_obj.object_by_path(Path::new(&device_kind.downloads_dir_path_with_nested_parent_segments())).unwrap();
    assert_eq!(object_by_path.object_type(), ObjectType::Folder);

    let object_by_path = root_obj.object_by_path(Path::new(r".")).unwrap();
    assert_eq!(object_by_path.object_type(), ObjectType::FunctionalObject);

    let object_by_path = root_obj.object_by_path(Path::new(&device_kind.storage_root_parent_path())).unwrap();
    assert_eq!(object_by_path.object_type(), ObjectType::FunctionalObject);

    let object_by_path = root_obj.object_by_path(Path::new(&device_kind.storage_root_parent_path_with_trailing_slash())).unwrap();
    assert_eq!(object_by_path.object_type(), ObjectType::FunctionalObject);

    let object_by_path = root_obj.object_by_path(Path::new(r".."));
    assert!(object_by_path.is_err());
}


fn access_by_id(basic_device: &BasicDevice, device_kind: DeviceKind) {
    let app_ident = winmtp::make_current_app_identifiers!();

    let device = basic_device.open(&app_ident, true).unwrap();
    let content = device.content().unwrap();

    let root_obj = content.root().unwrap();
    let download_folder_by_path = root_obj.object_by_path(&device_kind.downloads_dir_path()).unwrap();
    let download_folder_by_id = content.object_by_id(download_folder_by_path.id().to_ucstring()).unwrap();
    assert_eq!(download_folder_by_id.name(), &U16CString::from_str_truncate(device_kind.downloads_dir_name()));
}

fn prepare_upload_folder(basic_device: &BasicDevice, device_kind: DeviceKind) -> Object {
    let app_identifiers = winmtp::make_current_app_identifiers!();
    let device = basic_device.open(&app_identifiers, true).unwrap();
    let content = device.content().unwrap();
    let download_folder = content.root().unwrap().object_by_path(&device_kind.downloads_dir_path()).unwrap();
    let test_folder_id = match download_folder.create_subfolder(OsStr::new("winmtp_test")) {
        Ok(id) => id,
        Err(winmtp::error::CreateFolderError::AlreadyExists) => {
            let mut existing_folder = download_folder.object_by_path(Path::new("winmtp_test")).unwrap();
            existing_folder.delete(true).unwrap();
            // and try again
            download_folder.create_subfolder(OsStr::new("winmtp_test")).unwrap()
        }
        Err(err) => panic!("{}", err),
    };

    let test_folder = content.object_by_id(test_folder_id).unwrap();
    test_folder
}

/// Write some files, that will also be used for reading tests
fn push_content(basic_device: &BasicDevice, device_kind: DeviceKind) {
    let test_folder = prepare_upload_folder(basic_device, device_kind);
    test_folder.push_file(Path::new(EXAMPLE_SONG), true).unwrap();
    
    test_folder.push_data(OsStr::new("some_playlist.m3u"), PLAYLIST_CONTENT.as_bytes(), true).unwrap();
}

fn write_file_via_create_write_stream(basic_device: &BasicDevice, device_kind: DeviceKind) {
    let test_folder = prepare_upload_folder(basic_device, device_kind);

    let file_size = std::fs::metadata(Path::new(EXAMPLE_SONG)).unwrap().len();
    let file_path = device_kind.write_stream_file_path();
    let file_name = file_path.file_name().unwrap();
    let mut source_file = std::fs::File::open(Path::new(EXAMPLE_SONG)).unwrap();
    
    // Write the file
    let mut output_stream = test_folder
        .create_write_stream(file_name, file_size, true)
        .unwrap();
    std::io::copy(&mut source_file, &mut output_stream).unwrap();
    output_stream.flush().unwrap();


    // Check overwriting a file is refused when allow_overwrite is false
    let overwriting_output_stream = test_folder
        .create_write_stream(file_name, file_size, false);
    assert!(overwriting_output_stream.is_err());
}

fn pull_content(basic_device: &BasicDevice, device_kind: DeviceKind) {
    let app_identifiers = winmtp::make_current_app_identifiers!();
    let device = basic_device.open(&app_identifiers, true).unwrap();
    let object = device.content().unwrap().root().unwrap().object_by_path(&device_kind.uploaded_mp3_path()).unwrap();

    // Check the file size
    let metadata = object.properties(&[WPD_OBJECT_SIZE]).unwrap();
    let retrieved_size = metadata.get_u32(&WPD_OBJECT_SIZE).unwrap();
    let original_size = std::fs::metadata(Path::new(EXAMPLE_SONG)).unwrap().len();
    assert_eq!(retrieved_size as u64, original_size);

    // Download the file
    let mut input_stream = object.open_read_stream().unwrap();
    let mut output_file = std::fs::File::create(r"tests\assets\pulled-from-device.dat").unwrap();
    std::io::copy(&mut input_stream, &mut output_file).unwrap();
}

fn verify_file_written_via_create_write_stream(basic_device: &BasicDevice, device_kind: DeviceKind) {
    let app_identifiers = winmtp::make_current_app_identifiers!();
    let device = basic_device.open(&app_identifiers, true).unwrap();
    let object = device.content().unwrap().root().unwrap().object_by_path(&device_kind.write_stream_file_path()).unwrap();

    // Check the file size
    let metadata = object.properties(&[WPD_OBJECT_SIZE]).unwrap();
    let retrieved_size = metadata.get_u32(&WPD_OBJECT_SIZE).unwrap();
    let original_size = std::fs::metadata(Path::new(EXAMPLE_SONG)).unwrap().len();
    assert_eq!(retrieved_size as u64, original_size);

    // Download the file
    let mut input_stream = object.open_read_stream().unwrap();
    let mut output_file = std::fs::File::create(r"tests\assets\created-via-write-stream.dat").unwrap();
    std::io::copy(&mut input_stream, &mut output_file).unwrap();
}

