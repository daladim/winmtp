#[derive(thiserror::Error, Debug)]
pub enum MtpError {
    #[error("Windows API error ({0})")]
    Windows(#[from] crate::WindowsError),
    #[error("Incoherent results from successive calls to Windows API")]
    ChangedConditions,
    #[error("Invalid UTF-16 string")]
    Utf16Error(#[from] std::string::FromUtf16Error),
}

#[derive(thiserror::Error, Debug)]
pub enum ItemByPathError {
    #[error("Windows API error ({0})")]
    Windows(#[from] crate::WindowsError),
    #[error("Path not found")]
    NotFound,
    #[error("Got an absolute path, expected a relative path")]
    AbsolutePath,
}

#[derive(thiserror::Error, Debug)]
pub enum OpenStreamError {
    #[error("Windows API error ({0})")]
    Windows(#[from] crate::WindowsError),
    #[error("MTP API did not return any stream")] // Will probably never happen, as a Windows error would be raised before. But we never know
    UnableToCreate,
}

#[derive(thiserror::Error, Debug)]
pub enum CreateFolderError {
    #[error("Windows API error ({0})")]
    Windows(#[from] crate::WindowsError),
    #[error("There already is an object at this path")]
    AlreadyExists,
    #[error("Path should be relative, without any parent (..) component")]
    NonRelativePath,
}

#[derive(thiserror::Error, Debug)]
pub enum AddFileError {
    #[error("Windows API error ({0})")]
    Windows(#[from] crate::WindowsError),
    #[error("std::io error ({0})")]
    Std(#[from] std::io::Error),
    #[error("Invalid local file")]
    InvalidLocalFile,
    #[error("A file already exists at this path")]
    AlreadyExists,
    #[error("MTP API did not return any stream")] // Will probably never happen, as a Windows error would be raised before. But we never know
    UnableToCreate,
}
