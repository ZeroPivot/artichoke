use alloc::vec::Vec;
use bstr::{BStr};

use crate::encoding::Encoding;
use crate::ascii_string::AsciiString;
use crate::binary_string::BinaryString;
use crate::utf8_string::Utf8String;

pub enum EncodedString {
    Ascii(AsciiString),
    Binary(BinaryString),
    Utf8(Utf8String),
}

// Constructors
impl EncodedString {
    pub fn new(buf: Vec<u8>, encoding: Encoding) -> Self {
        match encoding {
            Encoding::Ascii => Self::Ascii(AsciiString::new(buf)),
            Encoding::Binary => Self::Binary(BinaryString::new(buf)),
            Encoding::Utf8 => Self::Utf8(Utf8String::new(buf)),
        }
    }
}

impl EncodedString {
    pub fn encoding(&self) -> Encoding {
        match self {
            EncodedString::Ascii(_) => Encoding::Ascii,
            EncodedString::Binary(_) => Encoding::Binary,
            EncodedString::Utf8(_) => Encoding::Utf8,
        }
    }
}

// Defer to Encoded Implementation
impl EncodedString {
    pub fn as_bstr(&self) -> &BStr {
        match self {
            EncodedString::Ascii(n) => n.as_bstr(),
            EncodedString::Binary(n) => n.as_bstr(),
            EncodedString::Utf8(n) => n.as_bstr(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            EncodedString::Ascii(n) => n.len(),
            EncodedString::Binary(n) => n.len(),
            EncodedString::Utf8(n) => n.len(),
        }
    }

    pub fn truncate(&mut self, len: usize) {
         match self {
            EncodedString::Ascii(n) => n.truncate(len),
            EncodedString::Binary(n) => n.truncate(len),
            EncodedString::Utf8(n) => n.truncate(len),
        };
    }
}

// Migration functions
// TODO: Remove these. If it compiles, we've migrated successfully
impl EncodedString {
    pub fn buf(&self) -> &Vec<u8> {
        match self {
            EncodedString::Ascii(n) => n.buf(),
            EncodedString::Binary(n) => n.buf(),
            EncodedString::Utf8(n) => n.buf(),
        }
    }

    pub fn buf_mut(&mut self) -> &mut Vec<u8> {
        match self {
            EncodedString::Ascii(n) => n.buf_mut(),
            EncodedString::Binary(n) => n.buf_mut(),
            EncodedString::Utf8(n) => n.buf_mut(),
        }
    }
}

