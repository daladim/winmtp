#[derive(thiserror::Error, Debug)]
pub enum MtpError {
    #[error("Windows API error")]
    Windows(#[from] crate::WindowsError),
    #[error("Incoherent results from successive calls to Windows API")]
    ChangedConditions,
    #[error("Invalid UTF-16 string")]
    Utf16Error(#[from] std::string::FromUtf16Error),
}

#[derive(thiserror::Error, Debug)]
pub enum ItemByPathError {
    #[error("Windows API error")]
    Windows(#[from] crate::WindowsError),
    #[error("Path not found")]
    NotFound,
    #[error("Got an absolute path, expected a relative path")]
    AbsolutePath,
}
