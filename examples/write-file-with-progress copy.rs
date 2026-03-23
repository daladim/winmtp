use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
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

// use this if you want a progress bar while sending the file
fn send_file_to_device(basic_device: &BasicDevice) -> Result<(), Box<dyn Error>> {
  let app_ident = winmtp::make_current_app_identifiers!();

  // file size and name has to be known before sending the file
  let file_path = Path::new(r"C:\Users\Username\Downloads\file_to_upload.txt");
  let file_name = file_path.file_name().ok_or("Path terminates in ..")?;
  let file_size = file_path.metadata()?.len();

  let device = basic_device.open(&app_ident, true)?;
  let content = device.content()?;
  let root_obj = content.root()?;

  let mut output_stream = root_obj.create_write_stream(file_name, file_size)?;
  let mut source_file = File::open(file_path)?;
  let buffer_size = output_stream.capacity().max(64 * 1024);
  let mut buffer = vec![0_u8; buffer_size];
  let mut bytes_sent = 0_u64;
  
  loop {
        let read_bytes = source_file.read(&mut buffer)?;
        if read_bytes == 0 {
            break;
        }

        output_stream.write_all(&buffer[..read_bytes])?;
        bytes_sent += read_bytes as u64;
        print!("\rSent {bytes_sent}/{file_size} bytes");
        std::io::stdout().flush()?;
    }

  output_stream.flush()?;

  println!(
    "Transferred {} bytes to {}",
    file_size,
    root_obj.name().to_string_lossy()
  );
  Ok(())
}