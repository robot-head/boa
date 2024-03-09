use crate::string::JsString;

/// Helper function to check if a `char` is trimmable.
pub(crate) const fn is_trimmable_whitespace(c: char) -> bool {
    // The rust implementation of `trim` does not regard the same characters whitespace as ecma standard does
    //
    // Rust uses \p{White_Space} by default, which also includes:
    // `\u{0085}' (next line)
    // And does not include:
    // '\u{FEFF}' (zero width non-breaking space)
    // Explicit whitespace: https://tc39.es/ecma262/#sec-white-space
    matches!(
        c,
        '\u{0009}' | '\u{000B}' | '\u{000C}' | '\u{0020}' | '\u{00A0}' | '\u{FEFF}' |
    // Unicode Space_Separator category
    '\u{1680}' | '\u{2000}'
            ..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}' |
    // Line terminators: https://tc39.es/ecma262/#sec-line-terminators
    '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}'
    )
}

use super::{is_ascii, JsStr, JsStrVariant};

#[derive(Debug, Clone, Copy)]
pub enum JsStringSliceVariant<'a> {
    U8Ascii(&'a [u8]),
    U8NonAscii(&'a str, usize),
    U16Ascii(&'a [u16]),
    U16NonAscii(&'a [u16]),
}

#[derive(Debug, Clone, Copy)]
pub struct JsStringSlice<'a> {
    inner: JsStringSliceVariant<'a>,
}

impl<'a> JsStringSlice<'a> {
    pub(crate) unsafe fn u8_ascii_unchecked(value: &'a [u8]) -> Self {
        debug_assert!(value.is_ascii(), "string must be ascii");

        Self {
            inner: JsStringSliceVariant::U8Ascii(value),
        }
    }

    pub(crate) unsafe fn u16_ascii_unchecked(value: &'a [u16]) -> Self {
        debug_assert!(is_ascii(value), "string must be ascii");

        Self {
            inner: JsStringSliceVariant::U16Ascii(value),
        }
    }

    pub(crate) unsafe fn u8_non_ascii_unchecked(value: &'a str) -> Self {
        debug_assert!(!value.is_ascii(), "string must not be ascii");
        let len = value.encode_utf16().count();

        Self {
            inner: JsStringSliceVariant::U8NonAscii(value, len),
        }
    }

    pub(crate) unsafe fn u16_non_ascii_unchecked(value: &'a [u16]) -> Self {
        debug_assert!(!is_ascii(value), "string must not be ascii");

        Self {
            inner: JsStringSliceVariant::U16NonAscii(value),
        }
    }

    pub(crate) fn variant(self) -> JsStringSliceVariant<'a> {
        self.inner
    }

    pub fn len(&self) -> usize {
        match self.variant() {
            JsStringSliceVariant::U8Ascii(s) => s.len(),
            JsStringSliceVariant::U8NonAscii(_, len) => len,
            JsStringSliceVariant::U16NonAscii(s) | JsStringSliceVariant::U16Ascii(s) => s.len(),
        }
    }

    pub fn is_ascii(&self) -> bool {
        matches!(
            self.variant(),
            JsStringSliceVariant::U8Ascii(_) | JsStringSliceVariant::U16Ascii(_)
        )
    }

    /// Trims both leading and trailing space.
    #[inline]
    #[must_use]
    pub fn trim(&self) -> Self {
        self.trim_start().trim_end()
    }

    /// Trims all leading space.
    #[inline]
    #[must_use]
    pub fn trim_start(&self) -> JsStringSlice<'a> {
        match self.variant() {
            JsStringSliceVariant::U8Ascii(s) => {
                // Safety: A JsStringSlice's Ascii field must always contain valid ascii, so this is safe.
                let s = unsafe { std::str::from_utf8_unchecked(s) };

                // SAFETY: Calling `trim_start()` on ASCII string always returns ASCII string, so this is safe.
                unsafe { JsStringSlice::u8_ascii_unchecked(s.trim_start().as_bytes()) }
            }
            JsStringSliceVariant::U8NonAscii(s, _) => JsStringSlice::from(s.trim_start()),
            JsStringSliceVariant::U16Ascii(s) => {
                let value = if let Some(left) = s.iter().copied().position(|r| {
                    !char::from_u32(u32::from(r))
                        .map(is_trimmable_whitespace)
                        .unwrap_or_default()
                }) {
                    &s[left..]
                } else {
                    // SAFETY: An empty string is valid ASCII, so this is safe.
                    return unsafe { JsStringSlice::u8_ascii_unchecked("".as_bytes()) };
                };

                // SAFETY: Calling `trim_start()` on ASCII string always returns ASCII string, so this is safe.
                unsafe { JsStringSlice::u16_ascii_unchecked(value) }
            }
            JsStringSliceVariant::U16NonAscii(s) => {
                let value = if let Some(left) = s.iter().copied().position(|r| {
                    !char::from_u32(u32::from(r))
                        .map(is_trimmable_whitespace)
                        .unwrap_or_default()
                }) {
                    &s[left..]
                } else {
                    // SAFETY: An empty string is valid ASCII, so this is safe.
                    return unsafe { JsStringSlice::u8_ascii_unchecked("".as_bytes()) };
                };

                JsStringSlice::from(value)
            }
        }
    }

    /// Trims all trailing space.
    #[inline]
    #[must_use]
    pub fn trim_end(&self) -> JsStringSlice<'a> {
        match self.variant() {
            JsStringSliceVariant::U8Ascii(s) => {
                // Safety: A JsStringSlice's Ascii field must always contain valid ascii, so this is safe.
                let s = unsafe { std::str::from_utf8_unchecked(s) };

                // SAFETY: Calling `trim_start()` on ASCII string always returns ASCII string, so this is safe.
                unsafe { JsStringSlice::u8_ascii_unchecked(s.trim_end().as_bytes()) }
            }
            JsStringSliceVariant::U8NonAscii(s, _) => JsStringSlice::from(s.trim_end()),
            JsStringSliceVariant::U16Ascii(s) => {
                let value = if let Some(right) = s.iter().copied().rposition(|r| {
                    !char::from_u32(u32::from(r))
                        .map(is_trimmable_whitespace)
                        .unwrap_or_default()
                }) {
                    &s[..=right]
                } else {
                    // SAFETY: An empty string is valid ASCII, so this is safe.
                    return unsafe { JsStringSlice::u8_ascii_unchecked("".as_bytes()) };
                };

                // SAFETY: Calling `trim_start()` on ASCII string always returns ASCII string, so this is safe.
                unsafe { JsStringSlice::u16_ascii_unchecked(value) }
            }
            JsStringSliceVariant::U16NonAscii(s) => {
                let value = if let Some(right) = s.iter().copied().rposition(|r| {
                    !char::from_u32(u32::from(r))
                        .map(is_trimmable_whitespace)
                        .unwrap_or_default()
                }) {
                    &s[..=right]
                } else {
                    // SAFETY: An empty string is valid ASCII, so this is safe.
                    return unsafe { JsStringSlice::u8_ascii_unchecked("".as_bytes()) };
                };

                JsStringSlice::from(value)
            }
        }
    }

    #[must_use]
    pub fn iter(self) -> crate::string::Iter<'a> {
        crate::string::Iter::new(self)
    }
}

impl<'a> From<&'a JsString> for JsStringSlice<'a> {
    fn from(value: &'a JsString) -> Self {
        Self::from(value.as_str())
    }
}

impl<'a> From<JsStr<'a>> for JsStringSlice<'a> {
    fn from(value: JsStr<'a>) -> Self {
        match value.variant() {
            JsStrVariant::Ascii(s) => {
                // SAFETY: `JsStrVariant::Ascii` always contains ASCII string, so this safe.
                unsafe { Self::u8_ascii_unchecked(s) }
            }
            JsStrVariant::U16(s) => {
                // SAFETY: `JsStrVariant::Ascii` always contains non-ASCII string, so this safe.
                unsafe { Self::u16_non_ascii_unchecked(s) }
            }
        }
    }
}

impl<'a> From<&'a str> for JsStringSlice<'a> {
    fn from(value: &'a str) -> Self {
        if value.is_ascii() {
            // SAFETY: Already checked that it's ASCII, so this is safe.
            return unsafe { Self::u8_ascii_unchecked(value.as_bytes()) };
        }

        // SAFETY: Already checked that it's non-ASCII, so this is safe.
        unsafe { Self::u8_non_ascii_unchecked(value) }
    }
}

impl<'a> From<&'a [u16]> for JsStringSlice<'a> {
    fn from(s: &'a [u16]) -> Self {
        if is_ascii(s) {
            // SAFETY: Already checked that it's ASCII, so this is safe.
            return unsafe { Self::u16_ascii_unchecked(s) };
        }

        // SAFETY: Already checked that it's non-ASCII, so this is safe.
        unsafe { Self::u16_non_ascii_unchecked(s) }
    }
}
