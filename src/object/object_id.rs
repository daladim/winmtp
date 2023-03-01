use windows::core::PCWSTR;

#[derive(Debug)]
pub struct ObjectId(pub PCWSTR);

impl<'a> std::convert::Into<&'a PCWSTR> for &'a ObjectId {
    fn into(self) -> &'a PCWSTR {
        &self.0
    }
}
