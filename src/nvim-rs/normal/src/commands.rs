//! Normal mode command handlers
//!
//! This module provides implementations of normal mode command handlers,
//! particularly g-commands and z-commands.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(dead_code)]

use std::ffi::c_int;

/// Opaque handle to command arguments (`cmdarg_T*`).
pub type CapHandle = *mut std::ffi::c_void;

/// Opaque handle to operator arguments (`oparg_T*`).
pub type OapHandle = *mut std::ffi::c_void;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    // Cap accessors
    fn nvim_cap_get_oap(cap: CapHandle) -> OapHandle;
    fn nvim_cap_get_cmdchar(cap: CapHandle) -> c_int;
    fn nvim_cap_get_nchar(cap: CapHandle) -> c_int;
    fn nvim_cap_get_count0(cap: CapHandle) -> c_int;
    fn nvim_cap_get_count1(cap: CapHandle) -> c_int;
    fn nvim_cap_get_arg(cap: CapHandle) -> c_int;

    // Oap accessors
    fn nvim_oap_get_op_type_ptr(oap: OapHandle) -> c_int;
    fn nvim_oap_set_op_type(oap: OapHandle, val: c_int);
    fn nvim_oap_set_inclusive(oap: OapHandle, val: bool);
    fn nvim_oap_set_motion_type(oap: OapHandle, val: c_int);

    // Utility functions
    fn beep_flush();

    // Phase 3: do_nv_ident accessor
    fn nvim_create_temp_cap_for_ident(c1: c_int, c2: c_int) -> CapHandle;
}

// =============================================================================
// Command Type Classification
// =============================================================================

/// Classification of normal mode commands
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CommandType {
    /// Unknown command type
    #[default]
    Unknown = 0,
    /// Motion command (moves cursor)
    Motion = 1,
    /// Operator command (d, c, y, etc.)
    Operator = 2,
    /// Insert/Replace command (i, a, R, etc.)
    Insert = 3,
    /// Visual mode command (v, V, Ctrl-V)
    Visual = 4,
    /// Window command (Ctrl-W followed by key)
    Window = 5,
    /// G-command (g followed by key)
    GCommand = 6,
    /// Z-command (z followed by key)
    ZCommand = 7,
    /// Bracket command ([ or ] followed by key)
    Bracket = 8,
    /// Mark command (m, ', `)
    Mark = 9,
    /// Search command (/, ?, n, N, *, #)
    Search = 10,
    /// Scroll command (Ctrl-F, Ctrl-B, etc.)
    Scroll = 11,
    /// Undo/Redo command (u, Ctrl-R)
    UndoRedo = 12,
    /// Miscellaneous command
    Misc = 13,
}

impl CommandType {
    /// Check if this is a motion command
    #[must_use]
    pub const fn is_motion(self) -> bool {
        matches!(self, Self::Motion | Self::Search)
    }

    /// Check if this is an operator command
    #[must_use]
    pub const fn is_operator(self) -> bool {
        matches!(self, Self::Operator)
    }

    /// Check if this requires a follow-up character
    #[must_use]
    pub const fn needs_nchar(self) -> bool {
        matches!(
            self,
            Self::GCommand | Self::ZCommand | Self::Bracket | Self::Mark | Self::Window
        )
    }

    /// Convert to raw integer
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Create from raw integer
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Motion,
            2 => Self::Operator,
            3 => Self::Insert,
            4 => Self::Visual,
            5 => Self::Window,
            6 => Self::GCommand,
            7 => Self::ZCommand,
            8 => Self::Bracket,
            9 => Self::Mark,
            10 => Self::Search,
            11 => Self::Scroll,
            12 => Self::UndoRedo,
            13 => Self::Misc,
            _ => Self::Unknown,
        }
    }
}

/// Classify a command character
#[must_use]
pub fn classify_command(cmdchar: c_int) -> CommandType {
    match cmdchar as u8 {
        // Operators
        b'd' | b'c' | b'y' | b'!' | b'<' | b'>' => CommandType::Operator,
        // Insert/Replace
        b'i' | b'I' | b'a' | b'A' | b'o' | b'O' | b'R' | b's' | b'S' => CommandType::Insert,
        // Visual
        b'v' | b'V' => CommandType::Visual,
        // G-commands
        b'g' => CommandType::GCommand,
        // Z-commands
        b'z' => CommandType::ZCommand,
        // Bracket commands
        b'[' | b']' => CommandType::Bracket,
        // Mark commands
        b'm' | b'\'' | b'`' => CommandType::Mark,
        // Search
        b'/' | b'?' | b'n' | b'N' | b'*' | b'#' => CommandType::Search,
        // Undo/Redo
        b'u' | b'U' => CommandType::UndoRedo,
        // Motion (sample)
        b'h' | b'j' | b'k' | b'l' | b'w' | b'b' | b'e' | b'0' | b'$' | b'^' | b'%' => {
            CommandType::Motion
        }
        // Window
        23 => CommandType::Window, // Ctrl-W (ASCII 23)
        // Default
        _ => CommandType::Unknown,
    }
}

// =============================================================================
// G-Command Handlers
// =============================================================================

/// G-command sub-commands
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GCommand {
    /// Unknown g-command
    #[default]
    Unknown = 0,
    /// gj - down screen line
    Down = 1,
    /// gk - up screen line
    Up = 2,
    /// g0 - start of screen line
    Home = 3,
    /// g$ - end of screen line
    End = 4,
    /// g^ - first non-blank of screen line
    FirstNonBlank = 5,
    /// gm - middle of screen line
    Middle = 6,
    /// gM - middle of text line
    MiddleText = 7,
    /// gg - go to line
    Goto = 8,
    /// gd - go to local declaration
    GotoDeclaration = 9,
    /// gD - go to global declaration
    GotoGlobalDecl = 10,
    /// gf - go to file
    GotoFile = 11,
    /// gF - go to file and line
    GotoFileLine = 12,
    /// gv - reselect visual
    Reselect = 13,
    /// gn - select next search match
    SelectNext = 14,
    /// gN - select previous search match
    SelectPrev = 15,
    /// g; - go to older change
    OlderChange = 16,
    /// g, - go to newer change
    NewerChange = 17,
    /// gq - format text
    Format = 18,
    /// gw - format text (no cursor move)
    FormatNoCursor = 19,
    /// g~ - toggle case
    ToggleCase = 20,
    /// gu - lowercase
    Lower = 21,
    /// gU - uppercase
    Upper = 22,
    /// g@ - call operatorfunc
    OpFunc = 23,
    /// g? - rot13
    Rot13 = 24,
    /// gi - insert at last insert position
    InsertLast = 25,
    /// ga - show ascii value
    ShowAscii = 26,
    /// g8 - show utf-8 bytes
    ShowUtf8 = 27,
    /// gI - insert at column 1
    InsertCol1 = 28,
    /// ge - end of previous word
    EndPrevWord = 29,
    /// gE - end of previous WORD
    EndPrevWORD = 30,
}

impl GCommand {
    /// Parse nchar to get g-command type
    #[must_use]
    pub fn from_nchar(nchar: c_int) -> Self {
        match nchar as u8 {
            b'j' => Self::Down,
            b'k' => Self::Up,
            b'0' => Self::Home,
            b'$' => Self::End,
            b'^' => Self::FirstNonBlank,
            b'm' => Self::Middle,
            b'M' => Self::MiddleText,
            b'g' => Self::Goto,
            b'd' => Self::GotoDeclaration,
            b'D' => Self::GotoGlobalDecl,
            b'f' => Self::GotoFile,
            b'F' => Self::GotoFileLine,
            b'v' => Self::Reselect,
            b'n' => Self::SelectNext,
            b'N' => Self::SelectPrev,
            b';' => Self::OlderChange,
            b',' => Self::NewerChange,
            b'q' => Self::Format,
            b'w' => Self::FormatNoCursor,
            b'~' => Self::ToggleCase,
            b'u' => Self::Lower,
            b'U' => Self::Upper,
            b'@' => Self::OpFunc,
            b'?' => Self::Rot13,
            b'i' => Self::InsertLast,
            b'a' => Self::ShowAscii,
            b'8' => Self::ShowUtf8,
            b'I' => Self::InsertCol1,
            b'e' => Self::EndPrevWord,
            b'E' => Self::EndPrevWORD,
            _ => Self::Unknown,
        }
    }

    /// Check if this is an operator g-command
    #[must_use]
    pub const fn is_operator(self) -> bool {
        matches!(
            self,
            Self::Format
                | Self::FormatNoCursor
                | Self::ToggleCase
                | Self::Lower
                | Self::Upper
                | Self::OpFunc
                | Self::Rot13
        )
    }

    /// Check if this is a motion g-command
    #[must_use]
    pub const fn is_motion(self) -> bool {
        matches!(
            self,
            Self::Down
                | Self::Up
                | Self::Home
                | Self::End
                | Self::FirstNonBlank
                | Self::Middle
                | Self::MiddleText
                | Self::Goto
                | Self::GotoDeclaration
                | Self::GotoGlobalDecl
                | Self::GotoFile
                | Self::GotoFileLine
                | Self::OlderChange
                | Self::NewerChange
                | Self::EndPrevWord
                | Self::EndPrevWORD
        )
    }

    /// Convert to raw integer
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Z-Command Handlers
// =============================================================================

/// Z-command sub-commands
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ZCommand {
    /// Unknown z-command
    #[default]
    Unknown = 0,
    /// zt - scroll cursor to top
    Top = 1,
    /// zz - scroll cursor to center
    Center = 2,
    /// zb - scroll cursor to bottom
    Bottom = 3,
    /// z<CR> - top with cursor at first non-blank
    TopFirstNb = 4,
    /// z. - center with cursor at first non-blank
    CenterFirstNb = 5,
    /// z- - bottom with cursor at first non-blank
    BottomFirstNb = 6,
    /// zh - scroll right
    ScrollRight = 7,
    /// zl - scroll left
    ScrollLeft = 8,
    /// zH - scroll half screen right
    HalfRight = 9,
    /// zL - scroll half screen left
    HalfLeft = 10,
    /// zs - scroll to put cursor at start
    CursorStart = 11,
    /// ze - scroll to put cursor at end
    CursorEnd = 12,
    /// zo - open fold
    OpenFold = 13,
    /// zO - open folds recursively
    OpenFoldRec = 14,
    /// zc - close fold
    CloseFold = 15,
    /// zC - close folds recursively
    CloseFoldRec = 16,
    /// za - toggle fold
    ToggleFold = 17,
    /// zA - toggle folds recursively
    ToggleFoldRec = 18,
    /// zv - view cursor line
    ViewCursor = 19,
    /// zf - create fold
    CreateFold = 20,
    /// zd - delete fold
    DeleteFold = 21,
    /// zD - delete folds recursively
    DeleteFoldRec = 22,
    /// zE - eliminate all folds
    ElimFolds = 23,
    /// zM - close all folds
    CloseAll = 24,
    /// zR - open all folds
    OpenAll = 25,
    /// zm - fold more
    FoldMore = 26,
    /// zr - fold less
    FoldLess = 27,
    /// zn - fold none
    FoldNone = 28,
    /// zN - fold normal
    FoldNormal = 29,
    /// zi - fold invert
    FoldInvert = 30,
}

impl ZCommand {
    /// Parse nchar to get z-command type
    #[must_use]
    pub fn from_nchar(nchar: c_int) -> Self {
        match nchar as u8 {
            b't' => Self::Top,
            b'z' => Self::Center,
            b'b' => Self::Bottom,
            b'\r' => Self::TopFirstNb,
            b'.' => Self::CenterFirstNb,
            b'-' => Self::BottomFirstNb,
            b'h' => Self::ScrollRight,
            b'l' => Self::ScrollLeft,
            b'H' => Self::HalfRight,
            b'L' => Self::HalfLeft,
            b's' => Self::CursorStart,
            b'e' => Self::CursorEnd,
            b'o' => Self::OpenFold,
            b'O' => Self::OpenFoldRec,
            b'c' => Self::CloseFold,
            b'C' => Self::CloseFoldRec,
            b'a' => Self::ToggleFold,
            b'A' => Self::ToggleFoldRec,
            b'v' => Self::ViewCursor,
            b'f' => Self::CreateFold,
            b'd' => Self::DeleteFold,
            b'D' => Self::DeleteFoldRec,
            b'E' => Self::ElimFolds,
            b'M' => Self::CloseAll,
            b'R' => Self::OpenAll,
            b'm' => Self::FoldMore,
            b'r' => Self::FoldLess,
            b'n' => Self::FoldNone,
            b'N' => Self::FoldNormal,
            b'i' => Self::FoldInvert,
            _ => Self::Unknown,
        }
    }

    /// Check if this is a scroll command
    #[must_use]
    pub const fn is_scroll(self) -> bool {
        matches!(
            self,
            Self::Top
                | Self::Center
                | Self::Bottom
                | Self::TopFirstNb
                | Self::CenterFirstNb
                | Self::BottomFirstNb
                | Self::ScrollRight
                | Self::ScrollLeft
                | Self::HalfRight
                | Self::HalfLeft
                | Self::CursorStart
                | Self::CursorEnd
        )
    }

    /// Check if this is a fold command
    #[must_use]
    pub const fn is_fold(self) -> bool {
        matches!(
            self,
            Self::OpenFold
                | Self::OpenFoldRec
                | Self::CloseFold
                | Self::CloseFoldRec
                | Self::ToggleFold
                | Self::ToggleFoldRec
                | Self::ViewCursor
                | Self::CreateFold
                | Self::DeleteFold
                | Self::DeleteFoldRec
                | Self::ElimFolds
                | Self::CloseAll
                | Self::OpenAll
                | Self::FoldMore
                | Self::FoldLess
                | Self::FoldNone
                | Self::FoldNormal
                | Self::FoldInvert
        )
    }

    /// Convert to raw integer
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Bracket Command Handlers
// =============================================================================

/// Bracket command sub-commands ([x] or ]x)
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BracketCommand {
    /// Unknown bracket command
    #[default]
    Unknown = 0,
    /// [( / ]) - go to unmatched paren
    UnmatchedParen = 1,
    /// [{ / ]} - go to unmatched brace
    UnmatchedBrace = 2,
    /// [[ / ]] - go to section
    Section = 3,
    /// [] / ][ - go to end of section
    SectionEnd = 4,
    /// [m / ]m - go to method start
    MethodStart = 5,
    /// [M / ]M - go to method end
    MethodEnd = 6,
    /// [# / ]# - go to unmatched #if/#else/#endif
    UnmatchedPreproc = 7,
    /// [* / ]* - go to comment start/end
    Comment = 8,
    /// [/ / ]/ - same as [* / ]*
    CommentAlt = 9,
    /// [c / ]c - go to diff change
    DiffChange = 10,
    /// [z / ]z - go to fold start/end
    FoldBoundary = 11,
    /// [s / ]s - go to spelling error
    SpellError = 12,
}

impl BracketCommand {
    /// Parse nchar to get bracket command type
    #[must_use]
    pub fn from_nchar(nchar: c_int) -> Self {
        match nchar as u8 {
            b'(' | b')' => Self::UnmatchedParen,
            b'{' | b'}' => Self::UnmatchedBrace,
            b'[' | b']' => Self::Section,
            b'm' => Self::MethodStart,
            b'M' => Self::MethodEnd,
            b'#' => Self::UnmatchedPreproc,
            b'*' => Self::Comment,
            b'/' => Self::CommentAlt,
            b'c' => Self::DiffChange,
            b'z' => Self::FoldBoundary,
            b's' => Self::SpellError,
            _ => Self::Unknown,
        }
    }

    /// Check if this is a motion command
    #[must_use]
    pub const fn is_motion(self) -> bool {
        // All bracket commands are motions
        !matches!(self, Self::Unknown)
    }

    /// Convert to raw integer
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Classify a command character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_classify_command(cmdchar: c_int) -> c_int {
    classify_command(cmdchar).to_raw()
}

/// Parse g-command nchar.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_g_command(nchar: c_int) -> c_int {
    GCommand::from_nchar(nchar).to_raw()
}

/// Check if g-command is operator.
#[unsafe(no_mangle)]
pub extern "C" fn rs_g_command_is_operator(gcmd: c_int) -> c_int {
    let gcmd = GCommand::from_nchar(gcmd);
    c_int::from(gcmd.is_operator())
}

/// Check if g-command is motion.
#[unsafe(no_mangle)]
pub extern "C" fn rs_g_command_is_motion(gcmd: c_int) -> c_int {
    let gcmd = GCommand::from_nchar(gcmd);
    c_int::from(gcmd.is_motion())
}

/// Parse z-command nchar.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_z_command(nchar: c_int) -> c_int {
    ZCommand::from_nchar(nchar).to_raw()
}

/// Check if z-command is scroll.
#[unsafe(no_mangle)]
pub extern "C" fn rs_z_command_is_scroll(zcmd: c_int) -> c_int {
    let zcmd = ZCommand::from_nchar(zcmd);
    c_int::from(zcmd.is_scroll())
}

/// Check if z-command is fold.
#[unsafe(no_mangle)]
pub extern "C" fn rs_z_command_is_fold(zcmd: c_int) -> c_int {
    let zcmd = ZCommand::from_nchar(zcmd);
    c_int::from(zcmd.is_fold())
}

/// Parse bracket command nchar.
#[unsafe(no_mangle)]
pub extern "C" fn rs_parse_bracket_command(nchar: c_int) -> c_int {
    BracketCommand::from_nchar(nchar).to_raw()
}

// =============================================================================
// Phase 3: do_nv_ident migration
// =============================================================================

/// Call nv_ident() as if `c1` was the command character, with `c2` as the
/// next character.
///
/// Rust replacement for the C `do_nv_ident`. Creates a temporary
/// `oparg_T`/`cmdarg_T` pair via C accessor and invokes `rs_nv_ident`.
///
/// # Safety
/// Calls C accessors and passes the returned pointer to rs_nv_ident.
/// The returned cap pointer is backed by function-static storage (safe
/// for single-threaded nvim).
#[export_name = "do_nv_ident"]
pub unsafe extern "C" fn rs_do_nv_ident(c1: c_int, c2: c_int) {
    let cap = nvim_create_temp_cap_for_ident(c1, c2);
    crate::rs_nv_ident(cap);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_type_classification() {
        assert_eq!(classify_command(b'd' as c_int), CommandType::Operator);
        assert_eq!(classify_command(b'c' as c_int), CommandType::Operator);
        assert_eq!(classify_command(b'y' as c_int), CommandType::Operator);
        assert_eq!(classify_command(b'i' as c_int), CommandType::Insert);
        assert_eq!(classify_command(b'v' as c_int), CommandType::Visual);
        assert_eq!(classify_command(b'g' as c_int), CommandType::GCommand);
        assert_eq!(classify_command(b'z' as c_int), CommandType::ZCommand);
    }

    #[test]
    fn test_g_command_parsing() {
        assert_eq!(GCommand::from_nchar(b'j' as c_int), GCommand::Down);
        assert_eq!(GCommand::from_nchar(b'k' as c_int), GCommand::Up);
        assert_eq!(GCommand::from_nchar(b'g' as c_int), GCommand::Goto);
        assert_eq!(
            GCommand::from_nchar(b'd' as c_int),
            GCommand::GotoDeclaration
        );
        assert_eq!(GCommand::from_nchar(b'q' as c_int), GCommand::Format);
    }

    #[test]
    fn test_g_command_is_operator() {
        assert!(GCommand::Format.is_operator());
        assert!(GCommand::ToggleCase.is_operator());
        assert!(GCommand::Lower.is_operator());
        assert!(GCommand::Upper.is_operator());
        assert!(!GCommand::Down.is_operator());
        assert!(!GCommand::Goto.is_operator());
    }

    #[test]
    fn test_z_command_parsing() {
        assert_eq!(ZCommand::from_nchar(b't' as c_int), ZCommand::Top);
        assert_eq!(ZCommand::from_nchar(b'z' as c_int), ZCommand::Center);
        assert_eq!(ZCommand::from_nchar(b'b' as c_int), ZCommand::Bottom);
        assert_eq!(ZCommand::from_nchar(b'o' as c_int), ZCommand::OpenFold);
        assert_eq!(ZCommand::from_nchar(b'c' as c_int), ZCommand::CloseFold);
    }

    #[test]
    fn test_z_command_categories() {
        assert!(ZCommand::Top.is_scroll());
        assert!(ZCommand::Center.is_scroll());
        assert!(ZCommand::OpenFold.is_fold());
        assert!(ZCommand::CloseFold.is_fold());
        assert!(!ZCommand::Top.is_fold());
        assert!(!ZCommand::OpenFold.is_scroll());
    }

    #[test]
    fn test_bracket_command_parsing() {
        assert_eq!(
            BracketCommand::from_nchar(b'(' as c_int),
            BracketCommand::UnmatchedParen
        );
        assert_eq!(
            BracketCommand::from_nchar(b'{' as c_int),
            BracketCommand::UnmatchedBrace
        );
        assert_eq!(
            BracketCommand::from_nchar(b'c' as c_int),
            BracketCommand::DiffChange
        );
    }
}
