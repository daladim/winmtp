use std::path::Path;
use std::error::Error;

use windows::core::PCWSTR;
use widestring::{U16CString, U16CStr};

use winmtp::Provider;
use winmtp::device::BasicDevice;
use winmtp::device::AppIdentifiers;
use winmtp::object::ObjectId;



fn main() {
    let provider = Provider::new().unwrap();
    let devices = provider.enumerate_devices().unwrap();
    println!("Found {} devices", devices.len());

    for device in &devices {
        println!("  * {}", device.friendly_name());
    }

    if let Some(first_device) = devices.get(0) {
        println!("Content of {}:", first_device.friendly_name());
        show_content(first_device).unwrap();
    }
}

fn show_content(basic_device: &BasicDevice) -> Result<(), Box<dyn Error>> {
    let app_ident = winmtp::make_current_app_identifiers("list_devices")!;

    let device = basic_device.open(&app_ident)?;
    let content = device.content()?;

    let root_obj = content.root()?;
    println!("root: {:?}", root_obj);

    for child in root_obj.children()? {
        println!("  * {:?}", child);
    }
    Ok(())
}
