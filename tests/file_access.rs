//! These test should succeed when an Android device is connected.

use std::ffi::OsStr;
use std::path::Path;
use std::error::Error;

use windows::core::PCWSTR;
use widestring::{U16CString, U16CStr};

use winmtp::Provider;
use winmtp::device::BasicDevice;
use winmtp::device::device_values::AppIdentifiers;
use winmtp::object::ObjectId;
use winmtp::object::ObjectType;

#[test]
fn file_access() {
    let provider = Provider::new().unwrap();
    let devices = provider.enumerate_devices().unwrap();
    let first_device = devices.get(0).expect("a device to be connected");

    println!("Testing on {}:", first_device.friendly_name());
    access_by_path(first_device);
    access_by_id(first_device);
    push_content(first_device);
    pull_content(first_device);
}

fn access_by_path(basic_device: &BasicDevice) {
    let app_ident = winmtp::make_current_app_identifiers!();

    let device = basic_device.open(&app_ident, true).unwrap();
    let content = device.content().unwrap();

    let root_obj = content.root().unwrap();
    assert_eq!(root_obj.object_type(), ObjectType::FunctionalObject);

    let object_by_path = root_obj.object_by_path(Path::new(r"Internal shared storage\Download\")).unwrap();
    assert_eq!(object_by_path.object_type(), ObjectType::Folder);

    let object_by_path = root_obj.object_by_path(Path::new(r"Internal shared storage\.\.\Download\.\")).unwrap();
    assert_eq!(object_by_path.object_type(), ObjectType::Folder);

    let object_by_path = root_obj.object_by_path(Path::new(r"Internal shared storage\.\.\Download\.\hbl.m3u")).unwrap();
    assert_eq!(object_by_path.object_type(), ObjectType::Playlist);

    let object_by_path = root_obj.object_by_path(Path::new(r"Internal shared storage\\\Download\\\")).unwrap();
    assert_eq!(object_by_path.object_type(), ObjectType::Folder);

    let object_by_path = root_obj.object_by_path(Path::new(r"Internal shared storage\\\Download\\\prout"));
    assert!(object_by_path.is_err());

    let object_by_path = root_obj.object_by_path(Path::new(r"Internal shared storage\\\Download\\\..\Download")).unwrap();
    assert_eq!(object_by_path.object_type(), ObjectType::Folder);

    let object_by_path = root_obj.object_by_path(Path::new(r"Internal shared storage\\\Download\\\CYA\..\..\Download")).unwrap();
    assert_eq!(object_by_path.object_type(), ObjectType::Folder);

    let object_by_path = root_obj.object_by_path(Path::new(r".")).unwrap();
    assert_eq!(object_by_path.object_type(), ObjectType::FunctionalObject);

    let object_by_path = root_obj.object_by_path(Path::new(r"Internal shared storage\..")).unwrap();
    assert_eq!(object_by_path.object_type(), ObjectType::FunctionalObject);

    let object_by_path = root_obj.object_by_path(Path::new(r"Internal shared storage\..\")).unwrap();
    assert_eq!(object_by_path.object_type(), ObjectType::FunctionalObject);

    let object_by_path = root_obj.object_by_path(Path::new(r".."));
    assert!(object_by_path.is_err());
}


fn access_by_id(basic_device: &BasicDevice) {
    let app_ident = winmtp::make_current_app_identifiers!();

    let device = basic_device.open(&app_ident, true).unwrap();
    let content = device.content().unwrap();

    let root_obj = content.root().unwrap();
    let download_folder_by_path = root_obj.object_by_path(Path::new(r"Internal shared storage\Download\")).unwrap();
    let download_folder_by_id = content.object_by_id(download_folder_by_path.id().to_ucstring()).unwrap();
    assert_eq!(download_folder_by_id.name(), &U16CString::from_str_truncate("Download"));
}

fn push_content(basic_device: &BasicDevice) {
    // let provider = winmtp::Provider::new().unwrap();
    // let basic_device = provider.enumerate_devices().unwrap()[0];
    let app_identifiers = winmtp::make_current_app_identifiers!();
    let device = basic_device.open(&app_identifiers, true).unwrap();
    let content = device.content().unwrap();
    let download_folder = content.root().unwrap().object_by_path(Path::new(r"Internal shared storage\Download\")).unwrap();
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
    test_folder.push_file(Path::new(r"tests\assets\Rough Draft (open source mp3 from audiohub.com).mp3"), true).unwrap();
}

fn pull_content(basic_device: &BasicDevice) {
    // let provider = winmtp::Provider::new().unwrap();
    // let basic_device = provider.enumerate_devices().unwrap()[0];
    let app_identifiers = winmtp::make_current_app_identifiers!();
    let device = basic_device.open(&app_identifiers, true).unwrap();
    let object = device.content().unwrap().root().unwrap().object_by_path(Path::new(r"Internal shared storage\Download\winmtp_test\Rough Draft (open source mp3 from audiohub.com).mp3")).unwrap();
    let mut input_stream = object.open_read_stream().unwrap();
    let mut output_file = std::fs::File::create(r"tests\assets\pulled-from-device.dat").unwrap();
    std::io::copy(&mut input_stream, &mut output_file).unwrap();
}

