//! API function support for highlight groups.
//!
//! This module provides types and utilities for the Neovim API functions
//! related to highlight groups, including:
//! - `nvim_get_hl()` / `nvim_set_hl()` support
//! - Highlight group dictionary conversion
//! - Namespace highlight support
//!
//! The actual API implementations are in C/Lua, but this module provides
//! the Rust-side types and logic.

use std::ffi::c_int;

use crate::types::RgbValue;

/// Namespace ID type (0 = global namespace).
pub type Ns = c_int;

/// Global namespace constant.
pub const NS_GLOBAL: Ns = 0;

/// Options for getting highlight group information.
#[derive(Debug, Clone, Default)]
pub struct GetHighlightOpts<'a> {
    /// Highlight group name (optional)
    pub name: Option<&'a str>,
    /// Highlight group ID (optional)
    pub id: Option<c_int>,
    /// Whether to follow links (default: true)
    pub link: bool,
    /// Whether to create the group if it doesn't exist (default: true)
    pub create: bool,
}

impl<'a> GetHighlightOpts<'a> {
    /// Create new options with defaults.
    pub fn new() -> Self {
        GetHighlightOpts {
            name: None,
            id: None,
            link: true,
            create: true,
        }
    }

    /// Set the group name to look up.
    pub fn with_name(mut self, name: &'a str) -> Self {
        self.name = Some(name);
        self
    }

    /// Set the group ID to look up.
    pub fn with_id(mut self, id: c_int) -> Self {
        self.id = Some(id);
        self
    }

    /// Set whether to follow links.
    pub fn follow_link(mut self, follow: bool) -> Self {
        self.link = follow;
        self
    }

    /// Set whether to create the group if missing.
    pub fn create_if_missing(mut self, create: bool) -> Self {
        self.create = create;
        self
    }
}

/// Highlight attribute values for API responses.
#[derive(Debug, Clone, Default)]
pub struct HlAttrValues {
    /// Foreground color (RGB)
    pub fg: Option<RgbValue>,
    /// Background color (RGB)
    pub bg: Option<RgbValue>,
    /// Special color (RGB)
    pub sp: Option<RgbValue>,
    /// Whether text is bold
    pub bold: bool,
    /// Whether text is italic
    pub italic: bool,
    /// Whether text is underlined
    pub underline: bool,
    /// Whether text has undercurl
    pub undercurl: bool,
    /// Whether text has double underline
    pub underdouble: bool,
    /// Whether text has dotted underline
    pub underdotted: bool,
    /// Whether text has dashed underline
    pub underdashed: bool,
    /// Whether text is strikethrough
    pub strikethrough: bool,
    /// Whether text is in reverse video
    pub reverse: bool,
    /// Whether text is standout
    pub standout: bool,
    /// Whether to disable attribute combining
    pub nocombine: bool,
    /// Whether to use alternate font
    pub altfont: bool,
    /// Blend value (0-100, or None if not set)
    pub blend: Option<c_int>,
    /// Link target group name (if linked)
    pub link: Option<String>,
    /// Whether this is a default setting
    pub default: bool,
}

impl HlAttrValues {
    /// Create a new empty attribute set.
    pub fn new() -> Self {
        HlAttrValues::default()
    }

    /// Check if any color is set.
    #[inline]
    pub fn has_colors(&self) -> bool {
        self.fg.is_some() || self.bg.is_some() || self.sp.is_some()
    }

    /// Check if any text attribute is set.
    #[inline]
    pub fn has_attrs(&self) -> bool {
        self.bold
            || self.italic
            || self.underline
            || self.undercurl
            || self.underdouble
            || self.underdotted
            || self.underdashed
            || self.strikethrough
            || self.reverse
            || self.standout
            || self.nocombine
            || self.altfont
    }

    /// Check if this is a link (no direct attributes).
    #[inline]
    pub fn is_link(&self) -> bool {
        self.link.is_some()
    }
}

/// Options for setting highlight group attributes.
#[derive(Debug, Clone, Default)]
pub struct SetHighlightOpts {
    /// Foreground color
    pub fg: Option<RgbValue>,
    /// Background color
    pub bg: Option<RgbValue>,
    /// Special color
    pub sp: Option<RgbValue>,
    /// Bold attribute
    pub bold: Option<bool>,
    /// Italic attribute
    pub italic: Option<bool>,
    /// Underline attribute
    pub underline: Option<bool>,
    /// Undercurl attribute
    pub undercurl: Option<bool>,
    /// Underdouble attribute
    pub underdouble: Option<bool>,
    /// Underdotted attribute
    pub underdotted: Option<bool>,
    /// Underdashed attribute
    pub underdashed: Option<bool>,
    /// Strikethrough attribute
    pub strikethrough: Option<bool>,
    /// Reverse attribute
    pub reverse: Option<bool>,
    /// Standout attribute
    pub standout: Option<bool>,
    /// Nocombine attribute
    pub nocombine: Option<bool>,
    /// Altfont attribute
    pub altfont: Option<bool>,
    /// Blend value
    pub blend: Option<c_int>,
    /// Link to another group
    pub link: Option<String>,
    /// Whether this is a default setting
    pub default: bool,
    /// Cterm foreground color
    pub cterm_fg: Option<c_int>,
    /// Cterm background color
    pub cterm_bg: Option<c_int>,
    /// Cterm attributes
    pub cterm: Option<c_int>,
}

impl SetHighlightOpts {
    /// Create new options.
    pub fn new() -> Self {
        SetHighlightOpts::default()
    }

    /// Set foreground color.
    pub fn with_fg(mut self, color: RgbValue) -> Self {
        self.fg = Some(color);
        self
    }

    /// Set background color.
    pub fn with_bg(mut self, color: RgbValue) -> Self {
        self.bg = Some(color);
        self
    }

    /// Set special color.
    pub fn with_sp(mut self, color: RgbValue) -> Self {
        self.sp = Some(color);
        self
    }

    /// Set bold attribute.
    pub fn with_bold(mut self, bold: bool) -> Self {
        self.bold = Some(bold);
        self
    }

    /// Set italic attribute.
    pub fn with_italic(mut self, italic: bool) -> Self {
        self.italic = Some(italic);
        self
    }

    /// Set link target.
    pub fn with_link(mut self, target: impl Into<String>) -> Self {
        self.link = Some(target.into());
        self
    }

    /// Set as default (can be overridden).
    pub fn as_default(mut self) -> Self {
        self.default = true;
        self
    }

    /// Check if this sets a link.
    #[inline]
    pub fn is_link(&self) -> bool {
        self.link.is_some()
    }

    /// Check if any GUI attributes are set.
    pub fn has_gui_attrs(&self) -> bool {
        self.fg.is_some()
            || self.bg.is_some()
            || self.sp.is_some()
            || self.bold.is_some()
            || self.italic.is_some()
            || self.underline.is_some()
            || self.undercurl.is_some()
            || self.underdouble.is_some()
            || self.underdotted.is_some()
            || self.underdashed.is_some()
            || self.strikethrough.is_some()
            || self.reverse.is_some()
            || self.standout.is_some()
            || self.nocombine.is_some()
            || self.altfont.is_some()
            || self.blend.is_some()
    }

    /// Check if any cterm attributes are set.
    pub fn has_cterm_attrs(&self) -> bool {
        self.cterm_fg.is_some() || self.cterm_bg.is_some() || self.cterm.is_some()
    }
}

/// Result of a get_hl operation for a single group.
#[derive(Debug, Clone)]
pub enum GetHlResult {
    /// Group found with attributes
    Found(HlAttrValues),
    /// Group not found
    NotFound,
    /// Group was created (when create=true)
    Created(c_int),
    /// Invalid ID
    InvalidId(c_int),
}

/// Parse a color string for the API.
///
/// Accepts:
/// - Hex colors: "#RRGGBB" or "#RGB"
/// - Color names: "Red", "Blue", etc.
/// - Special values: "NONE", "fg", "bg"
///
/// # Arguments
/// * `s` - The color string
///
/// # Returns
/// The RGB color value or None for NONE/invalid
pub fn parse_api_color(s: &str) -> Option<RgbValue> {
    let s = s.trim();

    // Check for NONE
    if s.eq_ignore_ascii_case("NONE") {
        return None;
    }

    // Check for hex color (returns -1 if invalid)
    let hex = crate::color::parse_hex_color(s);
    if hex >= 0 {
        return Some(hex);
    }

    // Check for color name
    lookup_color_name_rgb(s)
}

/// Look up a color name and return its RGB value.
///
/// This is a stub - the actual color lookup uses the color_name_table
/// which is defined in C.
fn lookup_color_name_rgb(name: &str) -> Option<RgbValue> {
    // Basic colors (matching what's in types.rs COLOR_NAMES)
    let basic_colors = [
        ("Black", 0x000000),
        ("DarkBlue", 0x00008B),
        ("DarkGreen", 0x006400),
        ("DarkCyan", 0x008B8B),
        ("DarkRed", 0x8B0000),
        ("DarkMagenta", 0x8B008B),
        ("Brown", 0xA52A2A),
        ("DarkYellow", 0xBBBB00),
        ("Gray", 0xBEBEBE),
        ("Grey", 0xBEBEBE),
        ("LightGray", 0xD3D3D3),
        ("LightGrey", 0xD3D3D3),
        ("DarkGray", 0xA9A9A9),
        ("DarkGrey", 0xA9A9A9),
        ("Blue", 0x0000FF),
        ("LightBlue", 0xADD8E6),
        ("Green", 0x00FF00),
        ("LightGreen", 0x90EE90),
        ("Cyan", 0x00FFFF),
        ("LightCyan", 0xE0FFFF),
        ("Red", 0xFF0000),
        ("LightRed", 0xFFA07A),
        ("Magenta", 0xFF00FF),
        ("LightMagenta", 0xFFBBFF),
        ("Yellow", 0xFFFF00),
        ("LightYellow", 0xFFFFE0),
        ("White", 0xFFFFFF),
    ];

    for (color_name, rgb) in &basic_colors {
        if name.eq_ignore_ascii_case(color_name) {
            return Some(*rgb);
        }
    }

    None
}

/// Format an RGB color for API output.
///
/// # Arguments
/// * `rgb` - The RGB color value
///
/// # Returns
/// The color as "#RRGGBB" string
pub fn format_api_color(rgb: RgbValue) -> String {
    let mut buffer = [0u8; 8];
    crate::color::format_hex_color(rgb, &mut buffer).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_highlight_opts() {
        let opts = GetHighlightOpts::new()
            .with_name("Normal")
            .follow_link(false);
        assert_eq!(opts.name, Some("Normal"));
        assert!(!opts.link);
        assert!(opts.create);
    }

    #[test]
    fn test_hl_attr_values() {
        let mut attrs = HlAttrValues::new();
        assert!(!attrs.has_colors());
        assert!(!attrs.has_attrs());

        attrs.fg = Some(0xFF0000);
        assert!(attrs.has_colors());

        attrs.bold = true;
        assert!(attrs.has_attrs());
    }

    #[test]
    fn test_set_highlight_opts() {
        let opts = SetHighlightOpts::new()
            .with_fg(0xFF0000)
            .with_bold(true)
            .as_default();

        assert_eq!(opts.fg, Some(0xFF0000));
        assert_eq!(opts.bold, Some(true));
        assert!(opts.default);
        assert!(opts.has_gui_attrs());
        assert!(!opts.has_cterm_attrs());
    }

    #[test]
    fn test_set_highlight_opts_link() {
        let opts = SetHighlightOpts::new().with_link("Normal");
        assert!(opts.is_link());
        assert_eq!(opts.link, Some("Normal".to_string()));
    }

    #[test]
    fn test_parse_api_color() {
        // Hex colors
        assert_eq!(parse_api_color("#FF0000"), Some(0xFF0000));
        assert_eq!(parse_api_color("#F00"), Some(0xFF0000));

        // Color names
        assert_eq!(parse_api_color("Red"), Some(0xFF0000));
        assert_eq!(parse_api_color("Blue"), Some(0x0000FF));
        assert_eq!(parse_api_color("WHITE"), Some(0xFFFFFF));

        // NONE
        assert_eq!(parse_api_color("NONE"), None);
        assert_eq!(parse_api_color("none"), None);

        // Invalid
        assert_eq!(parse_api_color("InvalidColor"), None);
    }

    #[test]
    fn test_format_api_color() {
        assert_eq!(format_api_color(0xFF0000), "#ff0000");
        assert_eq!(format_api_color(0x00FF00), "#00ff00");
        assert_eq!(format_api_color(0x0000FF), "#0000ff");
        assert_eq!(format_api_color(0xFFFFFF), "#ffffff");
        assert_eq!(format_api_color(0x000000), "#000000");
    }
}
