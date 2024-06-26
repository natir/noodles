use std::fmt;

use bstr::ByteSlice;
use noodles_sam as sam;

/// A BAM record name.
#[derive(Eq, PartialEq)]
pub struct Name<'a>(&'a [u8]);

impl<'a> Name<'a> {
    pub(super) fn new(src: &'a [u8]) -> Self {
        Self(src)
    }

    /// Returns the name as a byte slice.
    ///
    /// The returned slice will _not_ have the trailing `NUL` terminator.
    pub fn as_bytes(&self) -> &[u8] {
        const NUL: u8 = 0x00;
        self.as_ref().strip_suffix(&[NUL]).unwrap_or(self.as_ref())
    }
}

impl<'a> sam::alignment::record::Name for Name<'a> {
    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<'a> AsRef<[u8]> for Name<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

impl<'a> fmt::Debug for Name<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Name")
            .field(&self.as_bytes().as_bstr())
            .finish()
    }
}

impl<'a> From<Name<'a>> for sam::alignment::record_buf::Name {
    fn from(name: Name<'a>) -> Self {
        Self::from(name.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_bytes() {
        let name = Name::new(b"r0\x00");
        assert_eq!(name.as_bytes(), b"r0");

        let name = Name::new(b"r0");
        assert_eq!(name.as_bytes(), b"r0");
    }

    #[test]
    fn test_from_name_for_sam_alignment_record_buf_name() {
        use noodles_sam::alignment::record_buf::Name as NameBuf;

        let expected = NameBuf::from(b"r0");

        let name = Name::new(b"r0\x00");
        let actual = NameBuf::from(name);
        assert_eq!(actual, expected);

        let name = Name::new(b"r0");
        let actual = NameBuf::from(name);
        assert_eq!(actual, expected);
    }
}
