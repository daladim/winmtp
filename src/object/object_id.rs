use windows::core::PCWSTR;

#[derive(Debug)]
pub struct ObjectId(pub PCWSTR);

impl<'a> std::convert::From<&'a ObjectId> for &'a PCWSTR {
    fn from(val: &'a ObjectId) -> Self {
        &val.0
    }
}
