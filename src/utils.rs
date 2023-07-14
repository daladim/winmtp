use widestring::{U16CStr, U16CString};
use std::ffi::OsStr;

/// Compare paths, with or without case folding
pub fn are_path_eq(left: &U16CStr, right: &OsStr, case_sensitive: bool) -> bool {
    if case_sensitive {
        left == U16CString::from_os_str_truncate(right)
    } else {
        let l = left.to_string();
        let r = right.to_str();
        match (l,r) {
            (Ok(l), Some(r)) => {
                l.to_lowercase() == r.to_lowercase()
            },
            _ => false
        }
    }
}
