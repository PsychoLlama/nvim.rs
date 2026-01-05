//! Format string parsing for statusline
//!
//! This module provides types and functions for parsing statusline format strings.
//! The format string syntax uses `%` codes to specify dynamic items.

use std::ffi::c_int;

/// Status line item flags matching C's StlFlag enum in statusline_defs.h
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StlFlag {
    /// Path of file in buffer (%f)
    FilePath = b'f',
    /// Full path of file in buffer (%F)
    FullPath = b'F',
    /// Last part (tail) of file path (%t)
    Filename = b't',
    /// Column of cursor (%c)
    Column = b'c',
    /// Virtual column (%v)
    VirtCol = b'v',
    /// Virtual column with 'if different' display (%V)
    VirtColAlt = b'V',
    /// Line number of cursor (%l)
    Line = b'l',
    /// Number of lines in buffer (%L)
    NumLines = b'L',
    /// Current buffer number (%n)
    BufNo = b'n',
    /// Keymap when active (%k)
    Keymap = b'k',
    /// Offset of character under cursor (%o)
    Offset = b'o',
    /// Offset in hexadecimal (%O)
    OffsetX = b'O',
    /// Byte value of character (%b)
    ByteVal = b'b',
    /// Byte value in hexadecimal (%B)
    ByteValX = b'B',
    /// Readonly flag (%r)
    RoFlag = b'r',
    /// Readonly flag - other display (%R)
    RoFlagAlt = b'R',
    /// Window is showing a help file (%h)
    HelpFlag = b'h',
    /// Help flag - other display (%H)
    HelpFlagAlt = b'H',
    /// Filetype (%y)
    Filetype = b'y',
    /// Filetype - other display (%Y)
    FiletypeAlt = b'Y',
    /// Window is showing the preview buf (%w)
    PreviewFlag = b'w',
    /// Preview flag - other display (%W)
    PreviewFlagAlt = b'W',
    /// Modified flag (%m)
    Modified = b'm',
    /// Modified flag - other display (%M)
    ModifiedAlt = b'M',
    /// Quickfix window description (%q)
    Quickfix = b'q',
    /// Percentage through file (%p)
    Percentage = b'p',
    /// Percentage as TOP BOT ALL or NN% (%P)
    AltPercent = b'P',
    /// Argument list status as (x of y) (%a)
    ArgListStat = b'a',
    /// Page number (when printing) (%N)
    PageNum = b'N',
    /// 'showcmd' buffer (%S)
    ShowCmd = b'S',
    /// Fold column for 'statuscolumn' (%C)
    FoldCol = b'C',
    /// Sign column for 'statuscolumn' (%s)
    SignCol = b's',
    /// Start of expression to substitute (%{)
    VimExpr = b'{',
    /// Separation between alignment sections (%=)
    Separate = b'=',
    /// Truncation mark if line is too long (%<)
    TruncMark = b'<',
    /// Highlight from (User)1..9 or 0 (%*)
    UserHl = b'*',
    /// Highlight name (%#)
    Highlight = b'#',
    /// Tab page label nr (%T)
    TabPageNr = b'T',
    /// Tab page close nr (%X)
    TabCloseNr = b'X',
    /// Click region start (%@)
    ClickFunc = b'@',
}

impl StlFlag {
    /// Try to create a StlFlag from a byte character.
    #[allow(clippy::too_many_lines)]
    pub const fn from_byte(b: u8) -> Option<Self> {
        match b {
            b'f' => Some(Self::FilePath),
            b'F' => Some(Self::FullPath),
            b't' => Some(Self::Filename),
            b'c' => Some(Self::Column),
            b'v' => Some(Self::VirtCol),
            b'V' => Some(Self::VirtColAlt),
            b'l' => Some(Self::Line),
            b'L' => Some(Self::NumLines),
            b'n' => Some(Self::BufNo),
            b'k' => Some(Self::Keymap),
            b'o' => Some(Self::Offset),
            b'O' => Some(Self::OffsetX),
            b'b' => Some(Self::ByteVal),
            b'B' => Some(Self::ByteValX),
            b'r' => Some(Self::RoFlag),
            b'R' => Some(Self::RoFlagAlt),
            b'h' => Some(Self::HelpFlag),
            b'H' => Some(Self::HelpFlagAlt),
            b'y' => Some(Self::Filetype),
            b'Y' => Some(Self::FiletypeAlt),
            b'w' => Some(Self::PreviewFlag),
            b'W' => Some(Self::PreviewFlagAlt),
            b'm' => Some(Self::Modified),
            b'M' => Some(Self::ModifiedAlt),
            b'q' => Some(Self::Quickfix),
            b'p' => Some(Self::Percentage),
            b'P' => Some(Self::AltPercent),
            b'a' => Some(Self::ArgListStat),
            b'N' => Some(Self::PageNum),
            b'S' => Some(Self::ShowCmd),
            b'C' => Some(Self::FoldCol),
            b's' => Some(Self::SignCol),
            b'{' => Some(Self::VimExpr),
            b'=' => Some(Self::Separate),
            b'<' => Some(Self::TruncMark),
            b'*' => Some(Self::UserHl),
            b'#' => Some(Self::Highlight),
            b'T' => Some(Self::TabPageNr),
            b'X' => Some(Self::TabCloseNr),
            b'@' => Some(Self::ClickFunc),
            _ => None,
        }
    }

    /// Check if this flag produces a numeric value.
    pub const fn is_numeric(&self) -> bool {
        matches!(
            self,
            Self::Column
                | Self::VirtCol
                | Self::VirtColAlt
                | Self::Line
                | Self::NumLines
                | Self::BufNo
                | Self::Offset
                | Self::OffsetX
                | Self::ByteVal
                | Self::ByteValX
                | Self::Percentage
                | Self::PageNum
        )
    }

    /// Check if this flag is a conditional item (empty when condition not met).
    pub const fn is_flag_item(&self) -> bool {
        matches!(
            self,
            Self::RoFlag
                | Self::RoFlagAlt
                | Self::HelpFlag
                | Self::HelpFlagAlt
                | Self::PreviewFlag
                | Self::PreviewFlagAlt
                | Self::Modified
                | Self::ModifiedAlt
                | Self::Filetype
                | Self::FiletypeAlt
        )
    }

    /// Check if spaces should not be replaced with fillchar.
    pub const fn is_fillable(&self) -> bool {
        !matches!(
            self,
            Self::FilePath | Self::FullPath | Self::Filename | Self::ArgListStat | Self::Keymap
        )
    }
}

/// Item types for statusline rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StlItemType {
    /// Normal text item
    Normal = 0,
    /// Empty item (no output)
    Empty = 1,
    /// Group item `%(...%)`
    Group = 2,
    /// Separator mark `%=`
    Separate = 3,
    /// Highlight change
    Highlight = 4,
    /// Sign column highlight
    HighlightSign = 5,
    /// Fold column highlight
    HighlightFold = 6,
    /// Tab page item `%T` or `%X`
    TabPage = 7,
    /// Click function `%@...@`
    ClickFunc = 8,
    /// Truncation mark `%<`
    Trunc = 9,
}

/// A parsed statusline item.
#[derive(Debug, Clone)]
pub struct StlItem {
    /// Start position in output buffer (byte offset)
    pub start: usize,
    /// Minimum width (-ve means left align)
    pub minwid: c_int,
    /// Maximum width
    pub maxwid: c_int,
    /// Item type
    pub item_type: StlItemType,
    /// Command string for ClickFunc items
    pub cmd: Option<String>,
}

impl StlItem {
    /// Create a new item at the given position.
    pub const fn new(start: usize, item_type: StlItemType) -> Self {
        Self {
            start,
            minwid: 0,
            maxwid: 9999,
            item_type,
            cmd: None,
        }
    }

    /// Create a highlight item.
    pub const fn highlight(start: usize, userhl: c_int) -> Self {
        Self {
            start,
            minwid: userhl,
            maxwid: 9999,
            item_type: StlItemType::Highlight,
            cmd: None,
        }
    }

    /// Create a group item.
    pub const fn group(start: usize, minwid: c_int, maxwid: c_int) -> Self {
        Self {
            start,
            minwid,
            maxwid,
            item_type: StlItemType::Group,
            cmd: None,
        }
    }
}

/// Result of parsing a format specifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatSpec {
    /// Literal `%` character
    Percent,
    /// Separator `%=`
    Separator,
    /// Truncation mark `%<`
    TruncMark,
    /// Group start `%(`
    GroupStart { minwid: c_int, maxwid: c_int },
    /// Group end `%)`
    GroupEnd,
    /// User highlight `%1*` through `%9*`
    UserHighlight(u8),
    /// Named highlight `%#Name#`
    NamedHighlight(String),
    /// Tab page number `%nT`
    TabPageNr(c_int),
    /// Tab close `%nX`
    TabCloseNr(c_int),
    /// Click function `%@func@`
    ClickFunc(String),
    /// Expression `%{expr}` or `%{%expr%}`
    Expression { expr: String, reevaluate: bool },
    /// Standard item
    Item {
        flag: StlFlag,
        minwid: c_int,
        maxwid: c_int,
        zeropad: bool,
        left_align: bool,
    },
    /// End of expression block `%}`
    ExprEnd,
}

/// Parser state for format strings.
pub struct FormatParser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> FormatParser<'a> {
    /// Create a new parser for the given format string.
    pub const fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            pos: 0,
        }
    }

    /// Get the current position in the input.
    pub const fn position(&self) -> usize {
        self.pos
    }

    /// Check if we've reached the end of input.
    pub const fn is_empty(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Peek at the current character.
    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    /// Check if current char matches and consume it.
    fn consume(&mut self, c: u8) -> bool {
        if self.peek() == Some(c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    /// Advance and return the current character.
    fn next(&mut self) -> Option<u8> {
        let c = self.peek()?;
        self.pos += 1;
        Some(c)
    }

    /// Skip over literal text, returning bytes consumed.
    pub fn skip_literal(&mut self) -> usize {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c == b'%' {
                break;
            }
            self.pos += 1;
        }
        self.pos - start
    }

    /// Parse a number from the input.
    fn parse_number(&mut self) -> c_int {
        let mut n: c_int = 0;
        while let Some(c) = self.peek() {
            if !c.is_ascii_digit() {
                break;
            }
            n = n.saturating_mul(10).saturating_add(c_int::from(c - b'0'));
            self.pos += 1;
        }
        n
    }

    /// Parse a delimited string (e.g., between @ or #).
    fn parse_delimited(&mut self, delim: u8) -> Option<String> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c == delim {
                let s = String::from_utf8_lossy(&self.input[start..self.pos]).into_owned();
                self.pos += 1;
                return Some(s);
            }
            self.pos += 1;
        }
        None
    }

    /// Parse an expression body.
    fn parse_expression_body(&mut self, reevaluate: bool) -> Option<FormatSpec> {
        let start = self.pos;
        let mut depth = 1;

        while let Some(c) = self.peek() {
            if c == b'{' {
                depth += 1;
            } else if c == b'}' {
                if reevaluate && self.pos > start && self.input[self.pos - 1] == b'%' {
                    let expr =
                        String::from_utf8_lossy(&self.input[start..self.pos - 1]).into_owned();
                    self.pos += 1;
                    return Some(FormatSpec::Expression { expr, reevaluate });
                }
                depth -= 1;
                if depth == 0 {
                    let expr = String::from_utf8_lossy(&self.input[start..self.pos]).into_owned();
                    self.pos += 1;
                    return Some(FormatSpec::Expression { expr, reevaluate });
                }
            }
            self.pos += 1;
        }
        None
    }

    /// Parse simple specifiers that don't need width/flags.
    fn parse_simple_spec(&mut self) -> Option<FormatSpec> {
        if self.consume(b'%') {
            return Some(FormatSpec::Percent);
        }
        if self.consume(b'=') {
            return Some(FormatSpec::Separator);
        }
        if self.consume(b'<') {
            return Some(FormatSpec::TruncMark);
        }
        if self.consume(b')') {
            return Some(FormatSpec::GroupEnd);
        }
        if self.consume(b'}') {
            return Some(FormatSpec::ExprEnd);
        }
        None
    }

    /// Parse width-dependent specifiers.
    fn parse_width_spec(&mut self, minwid: c_int) -> Option<FormatSpec> {
        if self.consume(b'*') {
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            let hl = if minwid > 9 { 1 } else { minwid as u8 };
            return Some(FormatSpec::UserHighlight(hl));
        }
        if self.consume(b'T') {
            return Some(FormatSpec::TabPageNr(minwid));
        }
        if self.consume(b'X') {
            return Some(FormatSpec::TabCloseNr(minwid));
        }
        if self.consume(b'@') {
            return self.parse_delimited(b'@').map(FormatSpec::ClickFunc);
        }
        if self.consume(b'#') {
            return self.parse_delimited(b'#').map(FormatSpec::NamedHighlight);
        }
        None
    }

    /// Parse group or expression start.
    fn parse_complex_spec(&mut self, minwid: c_int, maxwid: c_int) -> Option<FormatSpec> {
        if self.consume(b'(') {
            return Some(FormatSpec::GroupStart { minwid, maxwid });
        }
        if self.consume(b'{') {
            let reevaluate = self.consume(b'%');
            return self.parse_expression_body(reevaluate);
        }
        None
    }

    /// Parse a format specifier after the `%`.
    #[allow(clippy::too_many_lines)]
    pub fn parse_spec(&mut self) -> Option<FormatSpec> {
        // Skip the `%`
        if self.next()? != b'%' {
            return None;
        }

        // Try simple specifiers first
        if let Some(spec) = self.parse_simple_spec() {
            return Some(spec);
        }

        // Parse optional flags
        let zeropad = self.consume(b'0');
        let left_align = self.consume(b'-');

        // Parse minimum width
        let minwid = self.parse_number();

        // Try width-dependent specifiers
        if let Some(spec) = self.parse_width_spec(minwid) {
            return Some(spec);
        }

        // Parse max width
        let maxwid = if self.consume(b'.') {
            let w = self.parse_number();
            if w == 0 {
                50
            } else {
                w
            }
        } else {
            9999
        };

        // Clamp minwid to 50
        let minwid = minwid.min(50) * if left_align { -1 } else { 1 };

        // Try complex specifiers
        if let Some(spec) = self.parse_complex_spec(minwid, maxwid) {
            return Some(spec);
        }

        // Parse standard item
        let flag_byte = self.next()?;
        let flag = StlFlag::from_byte(flag_byte)?;

        Some(FormatSpec::Item {
            flag,
            minwid,
            maxwid,
            zeropad,
            left_align,
        })
    }
}

/// Context for format string parsing.
#[repr(C)]
pub struct StlFormatContext {
    /// Current group depth
    pub group_depth: c_int,
    /// Expression evaluation depth
    pub eval_depth: c_int,
    /// Whether previous char was a flag item
    pub prevchar_isflag: bool,
    /// Whether previous char was an item
    pub prevchar_isitem: bool,
}

impl Default for StlFormatContext {
    fn default() -> Self {
        Self::new()
    }
}

impl StlFormatContext {
    /// Create a new format context.
    pub const fn new() -> Self {
        Self {
            group_depth: 0,
            eval_depth: 0,
            prevchar_isflag: true,
            prevchar_isitem: false,
        }
    }

    /// Enter a group.
    pub const fn enter_group(&mut self) {
        self.group_depth += 1;
    }

    /// Leave a group.
    pub const fn leave_group(&mut self) -> bool {
        if self.group_depth > 0 {
            self.group_depth -= 1;
            true
        } else {
            false
        }
    }

    /// Check if we're inside a group.
    pub const fn in_group(&self) -> bool {
        self.group_depth > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stl_flag_from_byte() {
        assert_eq!(StlFlag::from_byte(b'f'), Some(StlFlag::FilePath));
        assert_eq!(StlFlag::from_byte(b'F'), Some(StlFlag::FullPath));
        assert_eq!(StlFlag::from_byte(b't'), Some(StlFlag::Filename));
        assert_eq!(StlFlag::from_byte(b'l'), Some(StlFlag::Line));
        assert_eq!(StlFlag::from_byte(b'c'), Some(StlFlag::Column));
        assert_eq!(StlFlag::from_byte(b'%'), None);
        assert_eq!(StlFlag::from_byte(b'x'), None);
    }

    #[test]
    fn test_stl_flag_is_numeric() {
        assert!(StlFlag::Line.is_numeric());
        assert!(StlFlag::Column.is_numeric());
        assert!(StlFlag::Percentage.is_numeric());
        assert!(!StlFlag::FilePath.is_numeric());
        assert!(!StlFlag::Modified.is_numeric());
    }

    #[test]
    fn test_stl_flag_is_flag_item() {
        assert!(StlFlag::RoFlag.is_flag_item());
        assert!(StlFlag::Modified.is_flag_item());
        assert!(!StlFlag::Line.is_flag_item());
        assert!(!StlFlag::FilePath.is_flag_item());
    }

    #[test]
    fn test_parse_literal() {
        let mut parser = FormatParser::new("hello%f");
        let consumed = parser.skip_literal();
        assert_eq!(consumed, 5);
        assert_eq!(parser.position(), 5);
    }

    #[test]
    fn test_parse_percent_escape() {
        let mut parser = FormatParser::new("%%");
        let spec = parser.parse_spec();
        assert_eq!(spec, Some(FormatSpec::Percent));
    }

    #[test]
    fn test_parse_separator() {
        let mut parser = FormatParser::new("%=");
        let spec = parser.parse_spec();
        assert_eq!(spec, Some(FormatSpec::Separator));
    }

    #[test]
    fn test_parse_truncmark() {
        let mut parser = FormatParser::new("%<");
        let spec = parser.parse_spec();
        assert_eq!(spec, Some(FormatSpec::TruncMark));
    }

    #[test]
    fn test_parse_simple_item() {
        let mut parser = FormatParser::new("%f");
        let spec = parser.parse_spec();
        assert_eq!(
            spec,
            Some(FormatSpec::Item {
                flag: StlFlag::FilePath,
                minwid: 0,
                maxwid: 9999,
                zeropad: false,
                left_align: false,
            })
        );
    }

    #[test]
    fn test_parse_item_with_width() {
        let mut parser = FormatParser::new("%10f");
        let spec = parser.parse_spec();
        assert_eq!(
            spec,
            Some(FormatSpec::Item {
                flag: StlFlag::FilePath,
                minwid: 10,
                maxwid: 9999,
                zeropad: false,
                left_align: false,
            })
        );
    }

    #[test]
    fn test_parse_item_left_align() {
        let mut parser = FormatParser::new("%-10f");
        let spec = parser.parse_spec();
        assert_eq!(
            spec,
            Some(FormatSpec::Item {
                flag: StlFlag::FilePath,
                minwid: -10,
                maxwid: 9999,
                zeropad: false,
                left_align: true,
            })
        );
    }

    #[test]
    fn test_parse_item_zeropad() {
        let mut parser = FormatParser::new("%05l");
        let spec = parser.parse_spec();
        assert_eq!(
            spec,
            Some(FormatSpec::Item {
                flag: StlFlag::Line,
                minwid: 5,
                maxwid: 9999,
                zeropad: true,
                left_align: false,
            })
        );
    }

    #[test]
    fn test_parse_item_with_maxwid() {
        let mut parser = FormatParser::new("%10.20f");
        let spec = parser.parse_spec();
        assert_eq!(
            spec,
            Some(FormatSpec::Item {
                flag: StlFlag::FilePath,
                minwid: 10,
                maxwid: 20,
                zeropad: false,
                left_align: false,
            })
        );
    }

    #[test]
    fn test_parse_user_highlight() {
        let mut parser = FormatParser::new("%1*");
        let spec = parser.parse_spec();
        assert_eq!(spec, Some(FormatSpec::UserHighlight(1)));
    }

    #[test]
    fn test_parse_named_highlight() {
        let mut parser = FormatParser::new("%#StatusLine#");
        let spec = parser.parse_spec();
        assert_eq!(
            spec,
            Some(FormatSpec::NamedHighlight("StatusLine".to_string()))
        );
    }

    #[test]
    fn test_parse_group_start() {
        let mut parser = FormatParser::new("%(");
        let spec = parser.parse_spec();
        assert_eq!(
            spec,
            Some(FormatSpec::GroupStart {
                minwid: 0,
                maxwid: 9999
            })
        );
    }

    #[test]
    fn test_parse_group_start_with_width() {
        let mut parser = FormatParser::new("%-10.30(");
        let spec = parser.parse_spec();
        assert_eq!(
            spec,
            Some(FormatSpec::GroupStart {
                minwid: -10,
                maxwid: 30
            })
        );
    }

    #[test]
    fn test_parse_group_end() {
        let mut parser = FormatParser::new("%)");
        let spec = parser.parse_spec();
        assert_eq!(spec, Some(FormatSpec::GroupEnd));
    }

    #[test]
    fn test_parse_tabpage() {
        let mut parser = FormatParser::new("%1T");
        let spec = parser.parse_spec();
        assert_eq!(spec, Some(FormatSpec::TabPageNr(1)));
    }

    #[test]
    fn test_parse_tabclose() {
        let mut parser = FormatParser::new("%1X");
        let spec = parser.parse_spec();
        assert_eq!(spec, Some(FormatSpec::TabCloseNr(1)));
    }

    #[test]
    fn test_parse_click_func() {
        let mut parser = FormatParser::new("%@MyClickFunc@");
        let spec = parser.parse_spec();
        assert_eq!(spec, Some(FormatSpec::ClickFunc("MyClickFunc".to_string())));
    }

    #[test]
    fn test_parse_expression() {
        let mut parser = FormatParser::new("%{expr}");
        let spec = parser.parse_spec();
        assert_eq!(
            spec,
            Some(FormatSpec::Expression {
                expr: "expr".to_string(),
                reevaluate: false,
            })
        );
    }

    #[test]
    fn test_parse_reevaluate_expression() {
        let mut parser = FormatParser::new("%{%expr%}");
        let spec = parser.parse_spec();
        assert_eq!(
            spec,
            Some(FormatSpec::Expression {
                expr: "expr".to_string(),
                reevaluate: true,
            })
        );
    }

    #[test]
    fn test_minwid_clamped_to_50() {
        let mut parser = FormatParser::new("%100f");
        let spec = parser.parse_spec();
        assert_eq!(
            spec,
            Some(FormatSpec::Item {
                flag: StlFlag::FilePath,
                minwid: 50, // Clamped from 100
                maxwid: 9999,
                zeropad: false,
                left_align: false,
            })
        );
    }

    #[test]
    fn test_format_context() {
        let mut ctx = StlFormatContext::new();
        assert!(!ctx.in_group());

        ctx.enter_group();
        assert!(ctx.in_group());
        assert_eq!(ctx.group_depth, 1);

        ctx.enter_group();
        assert_eq!(ctx.group_depth, 2);

        assert!(ctx.leave_group());
        assert_eq!(ctx.group_depth, 1);

        assert!(ctx.leave_group());
        assert!(!ctx.in_group());

        assert!(!ctx.leave_group());
        assert_eq!(ctx.group_depth, 0);
    }

    #[test]
    fn test_stl_item_type_values() {
        // Verify enum values match C definitions
        assert_eq!(StlItemType::Normal as c_int, 0);
        assert_eq!(StlItemType::Empty as c_int, 1);
        assert_eq!(StlItemType::Group as c_int, 2);
        assert_eq!(StlItemType::Separate as c_int, 3);
        assert_eq!(StlItemType::Highlight as c_int, 4);
        assert_eq!(StlItemType::HighlightSign as c_int, 5);
        assert_eq!(StlItemType::HighlightFold as c_int, 6);
        assert_eq!(StlItemType::TabPage as c_int, 7);
        assert_eq!(StlItemType::ClickFunc as c_int, 8);
        assert_eq!(StlItemType::Trunc as c_int, 9);
    }
}
