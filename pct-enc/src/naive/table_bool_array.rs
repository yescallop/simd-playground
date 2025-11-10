//! Byte pattern tables from RFC 3986 and RFC 3987.
//!
//! The predefined table constants in this module are documented with
//! the ABNF notation of [RFC 5234].
//!
//! [RFC 5234]: https://datatracker.ietf.org/doc/html/rfc5234

const TABLE_LEN: usize = 257;
const INDEX_PCT_ENCODED: usize = 256;

/// A table specifying the byte patterns allowed in a string.
#[derive(Clone, Copy, Debug)]
pub struct Table([bool; 257]);

impl Table {
    /// Creates a table that only allows the given unencoded bytes.
    ///
    /// # Panics
    ///
    /// Panics if any of the bytes is not ASCII or equals `0`, `1`, `2`, or `b'%'`.
    #[must_use]
    pub const fn new(mut bytes: &[u8]) -> Self {
        let mut table = [false; TABLE_LEN];
        while let [cur, rem @ ..] = bytes {
            assert!(
                !matches!(cur, b'%' | 128..),
                "cannot allow non-ASCII byte or %"
            );
            table[*cur as usize] = true;
            bytes = rem;
        }
        Self(table)
    }

    /// Combines two tables into one.
    ///
    /// Returns a new table that allows all the byte patterns allowed
    /// by `self` or by `other`.
    #[must_use]
    pub const fn or(mut self, other: Self) -> Self {
        let mut i = 0;
        while i < TABLE_LEN {
            self.0[i] |= other.0[i];
            i += 1;
        }
        self
    }

    /// Marks this table as allowing percent-encoded octets.
    #[must_use]
    pub const fn or_pct_encoded(mut self) -> Self {
        self.0[INDEX_PCT_ENCODED] = true;
        self
    }

    #[inline]
    pub(crate) const fn allows_ascii(self, x: u8) -> bool {
        self.0[x as usize]
    }

    /// Checks whether percent-encoded octets are allowed by the table.
    #[inline]
    #[must_use]
    pub const fn allows_pct_encoded(self) -> bool {
        self.0[INDEX_PCT_ENCODED]
    }

    /// Validates the given string with the table.
    pub fn validate(&self, s: &[u8]) -> bool {
        let mut i = 0;

        macro_rules! do_loop {
            ($allow_pct_encoded:expr) => {
                while i < s.len() {
                    let x = s[i];
                    if $allow_pct_encoded && x == b'%' {
                        let [hi, lo, ..] = s[i + 1..] else {
                            return false;
                        };
                        if !super::is_valid_octet(hi, lo) {
                            return false;
                        }
                        i += 3;
                    } else {
                        if !self.allows_ascii(x) {
                            return false;
                        }
                        i += 1;
                    }
                }
            };
        }

        // This expansion alone doesn't help much, but combined with
        // `#[inline(always)]` on `utf8::next_code_point`,
        // it improves performance significantly for non-ASCII case.
        if self.allows_pct_encoded() {
            do_loop!(true);
        } else {
            do_loop!(false);
        }

        true
    }
}

const fn new(bytes: &[u8]) -> Table {
    Table::new(bytes)
}

// Rules from RFC 3986:

/// `ALPHA = %x41-5A / %x61-7A`
pub const ALPHA: Table = new(b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz");

/// `DIGIT = %x30-39`
pub const DIGIT: Table = new(b"0123456789");

/// `path = *( pchar / "/" )`
pub const PATH: Table = PCHAR.or(new(b"/"));

/// `pchar = unreserved / pct-encoded / sub-delims / ":" / "@"`
pub const PCHAR: Table = UNRESERVED.or(SUB_DELIMS).or(new(b":@")).or_pct_encoded();

/// `unreserved = ALPHA / DIGIT / "-" / "." / "_" / "~"`
pub const UNRESERVED: Table = ALPHA.or(DIGIT).or(new(b"-._~"));

/// `sub-delims = "!" / "$" / "&" / "'" / "(" / ")"
///             / "*" / "+" / "," / ";" / "="`
pub const SUB_DELIMS: Table = new(b"!$&'()*+,;=");
