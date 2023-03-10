//! Adapters so that COM streams implement `std::io::Read` and `std::io::Write`

use std::ffi::c_void;
use std::io::{Read, Write};

use windows::Win32::System::Com::{IStream, STGC_DEFAULT};
use windows::Win32::Foundation::{S_OK, S_FALSE};

/// A wrapper around a COM [`IStream`](windows::Win32::System::Com::IStream) that implements `std::io::Read`
pub struct ReadStream {
    stream: IStream,
    optimal_transfer_size: usize,
}

impl ReadStream {
    pub fn new(stream: IStream, optimal_transfer_size: usize) -> Self {
        Self{ stream, optimal_transfer_size }
    }

    pub fn optimal_transfer_size(&self) -> usize {
        self.optimal_transfer_size
    }
}

impl Read for ReadStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let requested_bytes = match buf.len().try_into() {
            Ok(b) => b,
            Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Requested too many bytes to read")),
        };

        let mut bytes_read: u32 = 0;
        let res = unsafe{
            self.stream.Read(
                buf.as_mut_ptr() as *mut u8 as *mut c_void,
                requested_bytes,
                Some(&mut bytes_read as *mut u32),
            )
        };

        match res {
            // regular case
            S_OK => Ok(bytes_read as usize),

            // EOF reached
            S_FALSE => Ok(bytes_read as usize),

            // Other error
            err => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Unexpected error {:?} when reading from a stream", err))),
        }
    }
}

/// A wrapper around a COM [`IStream`](windows::Win32::System::Com::IStream) that implements `std::io::Write`
pub struct WriteStream {
    stream: IStream,
    optimal_transfer_size: usize,
}

impl WriteStream {
    pub fn new(stream: IStream, optimal_transfer_size: usize) -> Self {
        Self{ stream, optimal_transfer_size }
    }

    pub fn optimal_transfer_size(&self) -> usize {
        self.optimal_transfer_size
    }

    /// Call the COM `Commit` API
    pub fn commit(&self) -> crate::WindowsResult<()> {
        unsafe{ self.stream.Commit(STGC_DEFAULT) }
    }
}

impl Write for WriteStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let requested_bytes = match buf.len().try_into() {
            Ok(b) => b,
            Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::Other, "Requested too many bytes to read")),
        };

        let mut bytes_read: u32 = 0;
        let res = unsafe{
            self.stream.Write(
                buf.as_ptr() as *const u8 as *const c_void,
                requested_bytes,
                Some(&mut bytes_read as *mut u32),
            )
        };

        match res {
            // regular case
            S_OK => Ok(bytes_read as usize),

            // Other error
            err => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Unexpected error {:?} when writing into a stream", err))),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.commit().map_err(|err| std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Unexpected error {:?} when flushing a stream", err)))
    }
}
