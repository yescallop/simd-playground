//! Percent-encoding utilities.

pub mod table_bitset;
pub mod table_bool_array;

use std::borrow::Cow;

const fn gen_octet_table(hi: bool) -> [u8; 256] {
    let mut out = [0xff; 256];
    let shift = if hi { 4 } else { 0 };

    let mut i = 0;
    while i < 10 {
        out[(i + b'0') as usize] = i << shift;
        i += 1;
    }
    while i < 16 {
        out[(i - 10 + b'A') as usize] = i << shift;
        out[(i - 10 + b'a') as usize] = i << shift;
        i += 1;
    }
    out
}

const OCTET_TABLE_HI: &[u8; 256] = &gen_octet_table(true);
const OCTET_TABLE_LO: &[u8; 256] = &gen_octet_table(false);

/// Decodes a percent-encoded octet, assuming that the bytes are hexadecimal.
fn decode_octet(hi: u8, lo: u8) -> u8 {
    debug_assert!(hi.is_ascii_hexdigit() && lo.is_ascii_hexdigit());
    OCTET_TABLE_HI[hi as usize] | OCTET_TABLE_LO[lo as usize]
}

pub(crate) const fn is_valid_octet(hi: u8, lo: u8) -> bool {
    OCTET_TABLE_LO[hi as usize] | OCTET_TABLE_LO[lo as usize] < 128
}

/// An iterator used to decode an [`EStr`] slice.
///
/// This struct is created by [`EStr::decode`]. Normally you'll use the methods below
/// instead of iterating over a `Decode` manually, unless you need precise control
/// over allocation.
///
/// See the [`DecodedChunk`] type for documentation of the items yielded by this iterator.
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Decode<'a> {
    source: &'a [u8],
}

/// An item returned by the [`Decode`] iterator.
#[derive(Clone, Copy, Debug)]
pub enum DecodedChunk<'a> {
    /// An unencoded subslice.
    Unencoded(&'a [u8]),
    /// A percent-encoded octet, decoded (for example, `"%20"` decoded as `0x20`).
    PctDecoded(u8),
}

impl<'a> Decode<'a> {
    pub fn new(source: &'a [u8]) -> Self {
        Self { source }
    }

    fn next_if_unencoded(&mut self) -> Option<&'a [u8]> {
        let i = self
            .source
            .iter()
            .position(|&x| x == b'%')
            .unwrap_or(self.source.len());

        if i == 0 {
            None
        } else {
            let (s, rem) = self.source.split_at(i);
            self.source = rem;
            Some(s)
        }
    }
}

impl<'a> Iterator for Decode<'a> {
    type Item = DecodedChunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.source.is_empty() {
            None
        } else if let Some(s) = self.next_if_unencoded() {
            Some(DecodedChunk::Unencoded(s))
        } else {
            let (s, rem) = self.source.split_at(3);
            self.source = rem;

            let x = decode_octet(s[1], s[2]);
            Some(DecodedChunk::PctDecoded(x))
        }
    }
}

impl<'a> Decode<'a> {
    fn decoded_len(&self) -> usize {
        self.source.len() - self.source.iter().filter(|&&x| x == b'%').count() * 2
    }

    fn borrow_all_or_prep_buf(&mut self) -> Result<&'a [u8], Vec<u8>> {
        if let Some(s) = self.next_if_unencoded() {
            if self.source.is_empty() {
                return Ok(s);
            }
            let mut buf = Vec::with_capacity(s.len() + self.decoded_len());
            buf.extend_from_slice(s);
            Err(buf)
        } else {
            Err(Vec::with_capacity(self.decoded_len()))
        }
    }

    /// Decodes the slice to bytes.
    ///
    /// This method allocates only when the slice contains any percent-encoded octet.
    #[must_use]
    pub fn to_bytes(mut self) -> Cow<'a, [u8]> {
        if self.source.is_empty() {
            return Cow::Borrowed(&[]);
        }

        let mut buf = match self.borrow_all_or_prep_buf() {
            Ok(s) => return Cow::Borrowed(s),
            Err(buf) => buf,
        };

        for chunk in self {
            match chunk {
                DecodedChunk::Unencoded(s) => buf.extend_from_slice(s),
                DecodedChunk::PctDecoded(s) => buf.push(s),
            }
        }
        Cow::Owned(buf)
    }
}

pub(crate) fn encode_byte(x: u8) -> &'static [u8] {
    const TABLE: &[u8; 256 * 3] = &{
        const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

        let mut i = 0;
        let mut table = [0; 256 * 3];
        while i < 256 {
            table[i * 3] = b'%';
            table[i * 3 + 1] = HEX_DIGITS[i >> 4];
            table[i * 3 + 2] = HEX_DIGITS[i & 0b1111];
            i += 1;
        }
        table
    };

    &TABLE[x as usize * 3..x as usize * 3 + 3]
}

/// An iterator used to percent-encode a string slice.
///
/// This struct is created by [`Table::encode`]. Normally you'll use [`EString::encode`]
/// instead, unless you need precise control over allocation.
///
/// See the [`EncodedChunk`] type for documentation of the items yielded by this iterator.
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Encode<'s> {
    table: table_bitset::Table,
    source: &'s [u8],
    to_enc: &'s [u8],
}

impl<'s> Encode<'s> {
    pub fn new(table: table_bitset::Table, source: &'s [u8]) -> Self {
        Self {
            table,
            source,
            to_enc: &[],
        }
    }
}

/// An item returned by the [`Encode`] iterator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncodedChunk<'a> {
    /// An unencoded subslice.
    Unencoded(&'a [u8]),
    /// A byte, percent-encoded (for example, `0x20` encoded as `"%20"`).
    PctEncoded(&'static [u8]),
}

impl<'a> EncodedChunk<'a> {
    #[must_use]
    pub fn as_bytes(self) -> &'a [u8] {
        match self {
            Self::Unencoded(s) | Self::PctEncoded(s) => s,
        }
    }
}

impl<'a> Iterator for Encode<'a> {
    type Item = EncodedChunk<'a>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if let [x, rem @ ..] = self.to_enc {
            self.to_enc = rem;
            return Some(EncodedChunk::PctEncoded(encode_byte(*x)));
        }

        if self.source.is_empty() {
            return None;
        }

        let mut iter = self.source.iter().copied().enumerate();

        let first_disallowed_idx = iter
            .find_map(|(i, x)| (!self.table.allows_ascii(x)).then_some(i))
            .unwrap_or(self.source.len());

        let next_allowed_idx = iter
            .find_map(|(i, x)| self.table.allows_ascii(x).then_some(i))
            .unwrap_or(self.source.len());

        if first_disallowed_idx == 0 {
            let (disallowed, rem) = self.source.split_at(next_allowed_idx);
            self.source = rem;

            let (x, rem) = disallowed.split_first().unwrap();
            self.to_enc = rem;

            Some(EncodedChunk::PctEncoded(encode_byte(*x)))
        } else {
            let allowed = &self.source[..first_disallowed_idx];
            self.to_enc = &self.source[first_disallowed_idx..next_allowed_idx];
            self.source = &self.source[next_allowed_idx..];

            Some(EncodedChunk::Unencoded(allowed))
        }
    }
}
