//! FFI bindings to utf8proc for Neovim
//!
//! This crate provides minimal FFI bindings to the utf8proc library,
//! which is already linked into Neovim for Unicode property queries.

#![allow(unsafe_code)]
#![allow(non_camel_case_types)]
#![allow(clippy::used_underscore_binding)]

/// utf8proc property structure containing Unicode character information.
///
/// This struct matches the C `utf8proc_property_t` layout.
/// We only define the fields we need access to.
#[repr(C)]
pub struct Utf8procProperty {
    pub category: i16,
    pub combining_class: i16,
    pub bidi_class: i16,
    pub decomp_type: i16,
    pub decomp_seqindex: u16,
    pub casefold_seqindex: u16,
    pub uppercase_seqindex: u16,
    pub lowercase_seqindex: u16,
    pub titlecase_seqindex: u16,
    // Bitfields packed into u16:
    // comb_index: 10 bits
    // comb_length: 5 bits
    // comb_issecond: 1 bit
    _comb_bits: u16,
    // More bitfields packed:
    // bidi_mirrored: 1
    // comp_exclusion: 1
    // ignorable: 1
    // control_boundary: 1
    // charwidth: 2
    // ambiguous_width: 1
    // pad: 1
    // boundclass: 6
    // indic_conjunct_break: 2
    _flags: u16,
}

/// Unicode General Category values (subset used by Neovim)
pub mod category {
    /// Mark, nonspacing (Mn)
    pub const MN: i16 = 6;
    /// Mark, spacing combining (Mc)
    pub const MC: i16 = 7;
    /// Mark, enclosing (Me)
    pub const ME: i16 = 8;
}

/// Boundclass values for grapheme break rules
pub mod boundclass {
    pub const REGIONAL_INDICATOR: u8 = 11;
    pub const EXTENDED_PICTOGRAPHIC: u8 = 19;
}

impl Utf8procProperty {
    /// Get the character width (0, 1, or 2).
    #[inline]
    #[must_use] 
    pub const fn charwidth(&self) -> u8 {
        // charwidth is bits 4-5 of _flags (after bidi_mirrored, comp_exclusion, ignorable, control_boundary)
        ((self._flags >> 4) & 0x3) as u8
    }

    /// Check if the character has ambiguous East Asian width.
    #[inline]
    #[must_use] 
    pub const fn ambiguous_width(&self) -> bool {
        // ambiguous_width is bit 6 of _flags
        (self._flags >> 6) & 0x1 != 0
    }

    /// Get the boundclass value for grapheme break rules.
    #[inline]
    #[must_use] 
    pub const fn boundclass(&self) -> u8 {
        // boundclass is bits 8-13 of _flags (after pad at bit 7)
        ((self._flags >> 8) & 0x3F) as u8
    }

    /// Check if this character is "emoji-like" (extended pictographic or regional indicator).
    #[inline]
    #[must_use]
    pub const fn is_emojilike(&self) -> bool {
        let bc = self.boundclass();
        bc == boundclass::EXTENDED_PICTOGRAPHIC || bc == boundclass::REGIONAL_INDICATOR
    }

    /// Check if this is a composing character (legacy check).
    ///
    /// Returns true for nonspacing marks (Mn) and enclosing marks (Me).
    /// This is a legacy check - for proper grapheme cluster detection,
    /// use the stateful grapheme algorithm instead.
    #[inline]
    #[must_use]
    pub const fn is_composing_legacy(&self) -> bool {
        self.category == category::MN || self.category == category::ME
    }
}

extern "C" {
    /// Get Unicode properties for a codepoint.
    ///
    /// Returns a pointer to a static property structure for the given codepoint.
    /// The returned pointer is valid for the lifetime of the program.
    pub fn utf8proc_get_property(codepoint: i32) -> *const Utf8procProperty;

    /// Check if there is a grapheme break between two codepoints.
    ///
    /// Returns true if there is a grapheme break between codepoint1 and codepoint2.
    /// The values are UCS-4 codepoints.
    pub fn utf8proc_grapheme_break(codepoint1: i32, codepoint2: i32) -> bool;
}

/// Safe wrapper to get Unicode properties for a codepoint.
///
/// Returns None if the codepoint is invalid (though utf8proc typically
/// returns properties for unassigned codepoints with category CN).
#[inline]
#[must_use]
pub fn get_property(codepoint: i32) -> Option<&'static Utf8procProperty> {
    // SAFETY: utf8proc_get_property always returns a valid pointer to static data
    unsafe {
        let prop = utf8proc_get_property(codepoint);
        if prop.is_null() {
            None
        } else {
            Some(&*prop)
        }
    }
}

/// Check if there is a grapheme break between two codepoints.
///
/// Returns true if there is a grapheme break (i.e., the codepoints are in
/// different grapheme clusters).
#[inline]
#[must_use]
pub fn grapheme_break(codepoint1: i32, codepoint2: i32) -> bool {
    // SAFETY: utf8proc_grapheme_break is a pure function with no side effects
    unsafe { utf8proc_grapheme_break(codepoint1, codepoint2) }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_property_ascii() {
        // ASCII 'A' should have charwidth 1
        let prop = get_property(0x41).expect("should get property for 'A'");
        assert_eq!(prop.charwidth(), 1);
        assert!(!prop.ambiguous_width());
    }

    #[test]
    fn test_get_property_cjk() {
        // CJK character '中' (U+4E2D) should have charwidth 2
        let prop = get_property(0x4E2D).expect("should get property for '中'");
        assert_eq!(prop.charwidth(), 2);
    }

    #[test]
    fn test_get_property_combining() {
        // Combining character should have charwidth 0
        // U+0300 is COMBINING GRAVE ACCENT
        let prop = get_property(0x0300).expect("should get property for combining accent");
        assert_eq!(prop.charwidth(), 0);
    }

    #[test]
    fn test_is_composing_legacy() {
        // U+0300 COMBINING GRAVE ACCENT is a nonspacing mark (Mn)
        let prop = get_property(0x0300).expect("should get property for combining accent");
        assert!(prop.is_composing_legacy());

        // U+20DD COMBINING ENCLOSING CIRCLE is an enclosing mark (Me)
        let prop = get_property(0x20DD).expect("should get property for enclosing mark");
        assert!(prop.is_composing_legacy());

        // ASCII 'A' is not composing
        let prop = get_property(0x41).expect("should get property for 'A'");
        assert!(!prop.is_composing_legacy());
    }

    #[test]
    fn test_grapheme_break() {
        // Space followed by combining accent - no break (combines with space)
        assert!(!grapheme_break(b' ' as i32, 0x0300));

        // Two ASCII letters - break between them
        assert!(grapheme_break(b'a' as i32, b'b' as i32));

        // Letter followed by combining accent - no break
        assert!(!grapheme_break(b'e' as i32, 0x0301)); // e + acute accent
    }
}
