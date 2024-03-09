use std::slice::SliceIndex;

use boa_interner::JStrRef;

use crate::string::{is_ascii, Iter};

use super::JsStringSlice;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JsStrVariant<'a> {
    Ascii(&'a [u8]),
    U16(&'a [u16]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JsStr<'a> {
    inner: JsStrVariant<'a>,
}

impl<'a> JsStr<'a> {
    /// Creates a [`JsStr`] from an ascii string.
    ///
    /// # Safety
    ///
    /// The caller must insure that the string is an ascii string,
    #[inline]
    #[must_use]
    pub const unsafe fn ascii_unchecked(value: &'a [u8]) -> Self {
        debug_assert!(value.is_ascii());

        Self {
            inner: JsStrVariant::Ascii(value),
        }
    }

    /// Creates a [`JsStr`] from an non-ascii u16 string.
    ///
    /// # Safety
    ///
    /// The caller must insure that the string is non-ascii u16,
    #[inline]
    #[must_use]
    pub const unsafe fn u16_unchecked(value: &'a [u16]) -> Self {
        debug_assert!(!is_ascii(value));

        Self {
            inner: JsStrVariant::U16(value),
        }
    }

    /// Get the length of the [`JsStr`].
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        match self.inner {
            JsStrVariant::Ascii(v) => v.len(),
            JsStrVariant::U16(v) => v.len(),
        }
    }

    #[inline]
    #[must_use]
    pub fn variant(self) -> JsStrVariant<'a> {
        self.inner
    }

    /// Check if the [`JsStr`] is all ascii.
    #[inline]
    #[must_use]
    pub fn is_ascii(&self) -> bool {
        matches!(self.inner, JsStrVariant::Ascii(_))
    }

    #[inline]
    #[must_use]
    pub fn as_ascii(&self) -> Option<&[u8]> {
        if let JsStrVariant::Ascii(slice) = self.inner {
            return Some(slice);
        }

        None
    }

    /// Iterate over the codepoints of the string.
    #[inline]
    #[must_use]
    pub fn iter(self) -> Iter<'a> {
        Iter::new(self.into())
    }

    /// Check if the [`JsStr`] is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Trims both leading and trailing space.
    #[inline]
    #[must_use]
    pub fn trim(self) -> JsStringSlice<'a> {
        self.trim_start().trim_end()
    }

    /// Trims all leading space.
    #[inline]
    #[must_use]
    pub fn trim_start(self) -> JsStringSlice<'a> {
        JsStringSlice::from(self).trim_start()
    }

    /// Trims all trailing space.
    #[inline]
    #[must_use]
    pub fn trim_end(self) -> JsStringSlice<'a> {
        JsStringSlice::from(self).trim_end()
    }

    pub fn get<I>(&'a self, index: I) -> Option<I::Value>
    where
        I: JsSliceIndex<'a>,
    {
        I::get(*self, index)
    }
}

pub trait JsSliceIndex<'a>: SliceIndex<[u8]> + SliceIndex<[u16]> {
    type Value;

    fn get(_: JsStr<'a>, index: Self) -> Option<Self::Value>;
}

impl<'a> JsSliceIndex<'a> for usize {
    type Value = u16;

    fn get(value: JsStr<'a>, index: Self) -> Option<Self::Value> {
        match value.variant() {
            JsStrVariant::Ascii(v) => v.get(index).copied().map(u16::from),
            JsStrVariant::U16(v) => v.get(index).copied(),
        }
    }
}

impl<'a> JsSliceIndex<'a> for std::ops::Range<usize> {
    type Value = JsStr<'a>;

    fn get(value: JsStr<'a>, index: Self) -> Option<Self::Value> {
        match value.variant() {
            JsStrVariant::Ascii(v) => {
                let slice = v.get(index)?;

                // SAFETY: `from_utf8_unchecked` does not alter the string, so this is safe.
                Some(unsafe { JsStr::ascii_unchecked(slice) })
            }
            JsStrVariant::U16(v) => {
                let slice = v.get(index)?;

                // TODO: If we sub-slice an utf16 array, and the sub-slice has only ASCII characters then we need,
                //       account for that.
                //
                // SAFETY:
                Some(unsafe { JsStr::u16_unchecked(slice) })
            }
        }
    }
}

impl<'a> JsSliceIndex<'a> for std::ops::RangeFull {
    type Value = JsStr<'a>;

    fn get(value: JsStr<'a>, _index: Self) -> Option<Self::Value> {
        Some(value)
    }
}

impl<'a> From<JsStr<'a>> for JStrRef<'a> {
    fn from(value: JsStr<'a>) -> Self {
        match value.variant() {
            JsStrVariant::Ascii(str) => {
                debug_assert!(str.is_ascii());

                // Safety: A JsStr's Ascii field must always contain valid ascii, so this is safe.
                let str = unsafe { std::str::from_utf8_unchecked(str) };
                Self::from(str)
            }
            JsStrVariant::U16(str) => Self::from(str),
        }
    }
}
