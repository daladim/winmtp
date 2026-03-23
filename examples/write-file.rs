use std::error::Error;
use std::path::Path;

use winmtp::Provider;
use winmtp::device::BasicDevice;

fn main() {
    let provider = Provider::new().unwrap();
    let devices = provider.enumerate_devices().unwrap();
    println!("Found {} devices", devices.len());

    for device in &devices {
        println!("  * {}", device.friendly_name());
    }

    if let Some(first_device) = devices.get(0) {
        println!("Content of {}:", first_device.friendly_name());
        send_file_to_device(first_device).unwrap();
    }
}

// use this if you want to write a file to the device
fn send_file_to_device(basic_device: &BasicDevice) -> Result<(), Box<dyn Error>> {
  let app_ident = winmtp::make_current_app_identifiers!();

  let file_path = Path::new(r"C:\Users\Username\Downloads\file_to_upload.txt");

  let device = basic_device.open(&app_ident, true)?;
  let content = device.content()?;
  let root_obj = content.root()?;

  root_obj.push_file(file_path, false)?;
  Ok(())
}