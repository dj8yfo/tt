pub trait AsFromBytes: Sized {
    fn as_bytes(&self) -> &[u8];
    fn read_from(bytes: &[u8]) -> Option<Self>;
}

impl AsFromBytes for String {
    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
    fn read_from(bytes: &[u8]) -> Option<Self> {
        String::from_utf8(bytes.to_vec()).ok()
    }
}
