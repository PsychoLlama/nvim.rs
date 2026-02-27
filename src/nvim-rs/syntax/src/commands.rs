//! Ex commands integration for syntax highlighting.
//!
//! This module handles:
//! - Syntax subcommand dispatch
//! - Argument parsing for syntax commands
//! - Pattern option parsing (contained, containedin, etc.)
//! - Command completion support

use std::ffi::{c_char, c_int};

use crate::types::{SynBlockHandle, WinHandle};

// =============================================================================
// FFI declarations for command operations
// =============================================================================

extern "C" {
    // Subcommand execution
    fn nvim_syn_get_cmdlinep() -> *mut *mut c_char;

    // Current window synblock
    fn nvim_syn_get_curwin_synblock() -> SynBlockHandle;
    fn nvim_get_curwin() -> WinHandle;

    // Syntax command state
    fn nvim_syn_get_include_link() -> c_int;
    fn nvim_syn_get_include_default() -> c_int;
    fn nvim_syn_get_include_none() -> c_int;

    // Running inc_tag for :syntax include
    fn nvim_syn_get_running_inc_tag() -> c_int;
    fn nvim_syn_set_running_inc_tag(tag: c_int);

    // -------------------------------------------------------------------------
    // Phase 18a: Synblock getters/setters for simple :syntax commands
    // -------------------------------------------------------------------------

    // Case mode
    fn nvim_synblock_get_syn_ic(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_set_syn_ic(block: SynBlockHandle, ic: c_int);

    // Spell mode
    fn nvim_synblock_get_syn_spell(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_set_syn_spell(block: SynBlockHandle, spell: c_int);

    // Foldlevel mode
    fn nvim_synblock_get_syn_foldlevel(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_set_syn_foldlevel(block: SynBlockHandle, foldlevel: c_int);

    // Conceal mode
    fn nvim_synblock_get_conceal(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_set_conceal(block: SynBlockHandle, conceal: c_int);

    // -------------------------------------------------------------------------
    // Message output functions
    // -------------------------------------------------------------------------

    /// Display a message with highlight
    fn msg_keep(s: *const c_char, hl_id: c_int, keep: c_int, multiline: c_int) -> c_int;

    /// Display an error message with format string
    fn semsg(fmt: *const c_char, ...);

    // -------------------------------------------------------------------------
    // String helpers from C
    // -------------------------------------------------------------------------

    /// Skip whitespace
    fn skipwhite(s: *const c_char) -> *mut c_char;

    /// Skip to whitespace
    fn skiptowhite(s: *const c_char) -> *mut c_char;

}

// =============================================================================
// Syntax subcommand enumeration
// =============================================================================

/// Syntax subcommands for `:syntax` command family.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxSubcommand {
    /// `:syntax case` - Set case sensitivity
    Case,
    /// `:syntax clear` - Clear syntax items
    Clear,
    /// `:syntax cluster` - Define a cluster
    Cluster,
    /// `:syntax conceal` - Set concealing
    Conceal,
    /// `:syntax enable` / `:syntax on` - Enable syntax highlighting
    Enable,
    /// `:syntax foldlevel` - Set fold level mode
    FoldLevel,
    /// `:syntax include` - Include another syntax file
    Include,
    /// `:syntax iskeyword` - Set iskeyword for syntax
    IsKeyword,
    /// `:syntax keyword` - Define a keyword
    Keyword,
    /// `:syntax list` - List syntax items
    List,
    /// `:syntax manual` - Set manual mode
    Manual,
    /// `:syntax match` - Define a match pattern
    Match,
    /// `:syntax off` - Disable syntax highlighting
    Off,
    /// `:syntax region` - Define a region
    Region,
    /// `:syntax reset` - Reset to default colors
    Reset,
    /// `:syntax spell` - Set spell checking mode
    Spell,
    /// `:syntax sync` - Set synchronization
    Sync,
}

impl SyntaxSubcommand {
    /// Parse a subcommand name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "case" => Some(Self::Case),
            "clear" => Some(Self::Clear),
            "cluster" => Some(Self::Cluster),
            "conceal" => Some(Self::Conceal),
            "enable" | "on" => Some(Self::Enable),
            "foldlevel" => Some(Self::FoldLevel),
            "include" => Some(Self::Include),
            "iskeyword" => Some(Self::IsKeyword),
            "keyword" => Some(Self::Keyword),
            "list" | "" => Some(Self::List),
            "manual" => Some(Self::Manual),
            "match" => Some(Self::Match),
            "off" => Some(Self::Off),
            "region" => Some(Self::Region),
            "reset" => Some(Self::Reset),
            "spell" => Some(Self::Spell),
            "sync" => Some(Self::Sync),
            _ => None,
        }
    }

    /// Get the canonical name of this subcommand.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Case => "case",
            Self::Clear => "clear",
            Self::Cluster => "cluster",
            Self::Conceal => "conceal",
            Self::Enable => "enable",
            Self::FoldLevel => "foldlevel",
            Self::Include => "include",
            Self::IsKeyword => "iskeyword",
            Self::Keyword => "keyword",
            Self::List => "list",
            Self::Manual => "manual",
            Self::Match => "match",
            Self::Off => "off",
            Self::Region => "region",
            Self::Reset => "reset",
            Self::Spell => "spell",
            Self::Sync => "sync",
        }
    }

    /// Get all available subcommand names.
    #[must_use]
    pub fn all_names() -> &'static [&'static str] {
        &[
            "case",
            "clear",
            "cluster",
            "conceal",
            "enable",
            "foldlevel",
            "include",
            "iskeyword",
            "keyword",
            "list",
            "manual",
            "match",
            "off",
            "on",
            "region",
            "reset",
            "spell",
            "sync",
        ]
    }
}

// =============================================================================
// Case sensitivity mode
// =============================================================================

/// Case sensitivity mode for syntax matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CaseMode {
    /// Use case-sensitive matching.
    #[default]
    Match,
    /// Use case-insensitive matching.
    Ignore,
}

impl CaseMode {
    /// Parse from command argument.
    #[must_use]
    pub fn from_arg(arg: &str) -> Option<Self> {
        match arg {
            "match" => Some(Self::Match),
            "ignore" => Some(Self::Ignore),
            _ => None,
        }
    }

    /// Get the argument string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Match => "match",
            Self::Ignore => "ignore",
        }
    }
}

// =============================================================================
// Spell check mode
// =============================================================================

/// Spell checking mode for syntax items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpellMode {
    /// Default: spell check if no @Spell cluster.
    #[default]
    Default,
    /// Spell check top-level text only.
    TopLevel,
    /// No spell checking in syntax items.
    NoTopLevel,
}

/// Spell mode constants.
pub mod spell_mode {
    pub const DEFAULT: i32 = 0;
    pub const TOP: i32 = 1;
    pub const NOTOP: i32 = 2;
}

impl SpellMode {
    /// Parse from command argument.
    #[must_use]
    pub fn from_arg(arg: &str) -> Option<Self> {
        match arg {
            "toplevel" => Some(Self::TopLevel),
            "notoplevel" => Some(Self::NoTopLevel),
            "default" => Some(Self::Default),
            _ => None,
        }
    }

    /// Convert from raw C value.
    #[must_use]
    pub const fn from_raw(val: i32) -> Self {
        match val {
            spell_mode::TOP => Self::TopLevel,
            spell_mode::NOTOP => Self::NoTopLevel,
            _ => Self::Default,
        }
    }

    /// Convert to raw C value.
    #[must_use]
    pub const fn to_raw(self) -> i32 {
        match self {
            Self::Default => spell_mode::DEFAULT,
            Self::TopLevel => spell_mode::TOP,
            Self::NoTopLevel => spell_mode::NOTOP,
        }
    }

    /// Get the argument string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::TopLevel => "toplevel",
            Self::NoTopLevel => "notoplevel",
        }
    }
}

// =============================================================================
// Pattern options
// =============================================================================

/// Options for syntax patterns (keyword, match, region).
#[derive(Debug, Clone, Default)]
pub struct PatternOptions {
    /// Pattern is contained within another.
    pub contained: bool,
    /// Display the pattern without highlighting.
    pub transparent: bool,
    /// Pattern starts at beginning of line.
    pub oneline: bool,
    /// Keep end position even when contained ends earlier.
    pub keepend: bool,
    /// Pattern can extend across lines.
    pub extend: bool,
    /// Exclude pattern from 'include'.
    pub excludenl: bool,
    /// Skip leading whitespace.
    pub skipwhite: bool,
    /// Skip blank lines.
    pub skipnl: bool,
    /// Skip empty lines.
    pub skipempty: bool,
    /// Conceal the matched text.
    pub conceal: bool,
    /// Conceal start and end of region.
    pub concealends: bool,
    /// This is a fold item.
    pub fold: bool,
    /// Display character for concealed text.
    pub cchar: Option<char>,
    /// Highlight group for contained items.
    pub contains: Vec<String>,
    /// Groups this pattern can be contained in.
    pub containedin: Vec<String>,
    /// Groups to look for at start of match.
    pub nextgroup: Vec<String>,
}

impl PatternOptions {
    /// Create new default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any containment is specified.
    #[must_use]
    pub fn has_containment(&self) -> bool {
        !self.contains.is_empty() || !self.containedin.is_empty()
    }
}

// =============================================================================
// Sync mode
// =============================================================================

/// Synchronization mode for syntax highlighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyncMode {
    /// Start from beginning of file.
    #[default]
    FromStart,
    /// Use C-style comments.
    CComment,
    /// Search for match pattern.
    Match,
    /// Use line count for syncing.
    Lines,
    /// Minimum lines to look back.
    MinLines,
    /// Maximum lines to look back.
    MaxLines,
    /// Number of lines for line continuations.
    LineBreaks,
}

impl SyncMode {
    /// Parse from command argument.
    #[must_use]
    pub fn from_arg(arg: &str) -> Option<Self> {
        match arg {
            "fromstart" => Some(Self::FromStart),
            "ccomment" => Some(Self::CComment),
            "match" => Some(Self::Match),
            "lines" => Some(Self::Lines),
            "minlines" => Some(Self::MinLines),
            "maxlines" => Some(Self::MaxLines),
            "linebreaks" => Some(Self::LineBreaks),
            _ => None,
        }
    }
}

// =============================================================================
// Command state accessors
// =============================================================================

/// Get the current command line pointer.
///
/// # Safety
/// Must be called during command execution.
#[must_use]
pub unsafe fn cmdlinep() -> *mut *mut c_char {
    nvim_syn_get_cmdlinep()
}

/// Get the synblock for the current window.
///
/// # Safety
/// Must be called from the main thread.
#[must_use]
pub unsafe fn curwin_synblock() -> SynBlockHandle {
    nvim_syn_get_curwin_synblock()
}

/// Get the current window handle.
///
/// # Safety
/// Must be called from the main thread.
#[must_use]
pub unsafe fn curwin() -> WinHandle {
    nvim_get_curwin()
}

/// Get whether to include linked groups in completion.
#[must_use]
pub fn include_link() -> bool {
    unsafe { nvim_syn_get_include_link() != 0 }
}

/// Get whether to include default groups in completion.
#[must_use]
pub fn include_default() -> bool {
    unsafe { nvim_syn_get_include_default() != 0 }
}

/// Get whether to include "None" in completion.
#[must_use]
pub fn include_none() -> bool {
    unsafe { nvim_syn_get_include_none() != 0 }
}

// =============================================================================
// Include tag management
// =============================================================================

/// Get the current running include tag.
///
/// This is used by `:syntax include` to track nested includes.
#[must_use]
pub fn running_inc_tag() -> i32 {
    unsafe { nvim_syn_get_running_inc_tag() }
}

/// Set the running include tag.
///
/// # Safety
/// Must be called from the main thread.
pub unsafe fn set_running_inc_tag(tag: i32) {
    nvim_syn_set_running_inc_tag(tag);
}

// =============================================================================
// Synblock settings accessors
// =============================================================================

/// Get the conceal setting for current window's synblock.
///
/// # Safety
/// Must be called from the main thread.
#[must_use]
pub unsafe fn synblock_conceal_setting(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    nvim_synblock_get_conceal(block) != 0
}

/// Get the case-insensitive setting for current window's synblock.
///
/// # Safety
/// Must be called from the main thread.
#[must_use]
pub unsafe fn synblock_ic_setting(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    nvim_synblock_get_syn_ic(block) != 0
}

// =============================================================================
// Command expansion helpers
// =============================================================================

/// What to expand in syntax commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpandWhat {
    /// Expand `:syn` sub-commands.
    Subcmd,
    /// Expand `:syn case` arguments.
    Case,
    /// Expand `:syn spell` arguments.
    Spell,
    /// Expand `:syn sync` arguments.
    Sync,
    /// Expand `:syn list @cluster` arguments.
    Cluster,
}

impl ExpandWhat {
    /// Get expansion options for a mode.
    #[must_use]
    pub fn options(&self) -> &'static [&'static str] {
        match self {
            Self::Subcmd => SyntaxSubcommand::all_names(),
            Self::Case => &["match", "ignore"],
            Self::Spell => &["toplevel", "notoplevel", "default"],
            Self::Sync => &[
                "ccomment",
                "clear",
                "fromstart",
                "linebreaks=",
                "lines=",
                "linecont",
                "match",
                "maxlines=",
                "minlines=",
                "region",
            ],
            Self::Cluster => &[], // Clusters are dynamically expanded
        }
    }
}

// =============================================================================
// FFI exports for Ex commands (Phase Y6)
// =============================================================================

use std::ffi::c_void;

/// Opaque pointer to synblock for FFI
pub type SynBlockPtr = *const c_void;

/// Subcommand ID constants for FFI
pub mod subcmd_id {
    pub const CASE: i32 = 0;
    pub const CLEAR: i32 = 1;
    pub const CLUSTER: i32 = 2;
    pub const CONCEAL: i32 = 3;
    pub const ENABLE: i32 = 4;
    pub const FOLDLEVEL: i32 = 5;
    pub const INCLUDE: i32 = 6;
    pub const ISKEYWORD: i32 = 7;
    pub const KEYWORD: i32 = 8;
    pub const LIST: i32 = 9;
    pub const MANUAL: i32 = 10;
    pub const MATCH: i32 = 11;
    pub const OFF: i32 = 12;
    pub const REGION: i32 = 13;
    pub const RESET: i32 = 14;
    pub const SPELL: i32 = 15;
    pub const SYNC: i32 = 16;
    pub const INVALID: i32 = -1;
}
/// Get the number of available syntax subcommands.
#[no_mangle]
pub extern "C" fn rs_syn_subcmd_count() -> c_int {
    SyntaxSubcommand::all_names().len() as c_int
}

/// Check if a subcommand ID is valid.
#[no_mangle]
pub const extern "C" fn rs_syn_subcmd_is_valid(id: c_int) -> c_int {
    if id >= 0 && id <= subcmd_id::SYNC {
        1
    } else {
        0
    }
}

/// Case mode constants
#[no_mangle]
pub const extern "C" fn rs_syn_case_match() -> c_int {
    0
}

#[no_mangle]
pub const extern "C" fn rs_syn_case_ignore() -> c_int {
    1
}
/// Spell mode constants
#[no_mangle]
pub const extern "C" fn rs_syn_spell_default() -> c_int {
    spell_mode::DEFAULT
}

#[no_mangle]
pub const extern "C" fn rs_syn_spell_toplevel() -> c_int {
    spell_mode::TOP
}

#[no_mangle]
pub const extern "C" fn rs_syn_spell_notoplevel() -> c_int {
    spell_mode::NOTOP
}
/// Sync mode constants
pub mod sync_mode_id {
    pub const FROM_START: i32 = 0;
    pub const CCOMMENT: i32 = 1;
    pub const MATCH: i32 = 2;
    pub const LINES: i32 = 3;
    pub const MINLINES: i32 = 4;
    pub const MAXLINES: i32 = 5;
    pub const LINEBREAKS: i32 = 6;
    pub const INVALID: i32 = -1;
}
/// Command settings struct
#[repr(C)]
pub struct SynCmdSettings {
    /// Whether to include linked groups
    pub include_link: c_int,
    /// Whether to include default groups
    pub include_default: c_int,
    /// Whether to include "None"
    pub include_none: c_int,
    /// Current running include tag
    pub inc_tag: c_int,
}
/// Synblock settings for commands
#[repr(C)]
pub struct SynblockCmdSettings {
    /// Whether concealing is enabled
    pub conceal: c_int,
    /// Whether case-insensitive matching is enabled
    pub ignorecase: c_int,
}
/// Expansion type constants
pub mod expand_type {
    pub const SUBCMD: i32 = 0;
    pub const CASE: i32 = 1;
    pub const SPELL: i32 = 2;
    pub const SYNC: i32 = 3;
    pub const CLUSTER: i32 = 4;
}

/// Get the number of options for an expansion type.
#[no_mangle]
pub extern "C" fn rs_syn_expand_count(expand_type: c_int) -> c_int {
    match expand_type {
        expand_type::SUBCMD => SyntaxSubcommand::all_names().len() as c_int,
        expand_type::CASE => 2,
        expand_type::SPELL => 3,
        expand_type::SYNC => 10,
        expand_type::CLUSTER => 0, // Dynamic
        _ => 0,
    }
}

// =============================================================================
// Phase 18a: Simple Settings Command Implementations
// =============================================================================

/// Foldlevel mode constants
pub mod foldlevel_mode {
    pub const START: i32 = 0;
    pub const MINIMUM: i32 = 1;
}

// Static strings for messages
static MSG_SYNTAX_CASE_IGNORE: &[u8] = b"syntax case ignore\0";
static MSG_SYNTAX_CASE_MATCH: &[u8] = b"syntax case match\0";
static MSG_SYNTAX_CONCEAL_ON: &[u8] = b"syntax conceal on\0";
static MSG_SYNTAX_CONCEAL_OFF: &[u8] = b"syntax conceal off\0";
static MSG_SYNTAX_SPELL_TOPLEVEL: &[u8] = b"syntax spell toplevel\0";
static MSG_SYNTAX_SPELL_NOTOPLEVEL: &[u8] = b"syntax spell notoplevel\0";
static MSG_SYNTAX_SPELL_DEFAULT: &[u8] = b"syntax spell default\0";
static MSG_SYNTAX_FOLDLEVEL_START: &[u8] = b"syntax foldlevel start\0";
static MSG_SYNTAX_FOLDLEVEL_MINIMUM: &[u8] = b"syntax foldlevel minimum\0";
static MSG_E_ILLEGAL_ARG: &[u8] = b"E474: Invalid argument: %s\0";

/// Helper: display a message
#[inline]
unsafe fn msg_display(s: *const c_char) {
    msg_keep(s, 0, 0, 0);
}

/// Helper: display an error with argument
#[inline]
unsafe fn error_illegal_arg(arg: *const c_char) {
    semsg(MSG_E_ILLEGAL_ARG.as_ptr().cast(), arg);
}

/// Helper: check if a C string equals another (case-insensitive) with exact length
#[inline]
unsafe fn strnicmp_exact(
    s1: *const c_char,
    s2: &[u8],
    expected_len: usize,
    actual_len: isize,
) -> bool {
    if actual_len as usize != expected_len {
        return false;
    }
    // Case-insensitive comparison
    for (i, &expected_byte) in s2.iter().enumerate().take(expected_len) {
        let c1 = (*s1.add(i) as u8).to_ascii_lowercase();
        let c2 = expected_byte.to_ascii_lowercase();
        if c1 != c2 {
            return false;
        }
    }
    true
}

/// Handle ":syntax case" command from Rust.
///
/// # Arguments
/// * `block` - The synblock to modify
/// * `arg` - Command argument (NUL-terminated)
/// * `arg_end` - Pointer to end of word (result of skiptowhite)
///
/// # Returns
/// 0 on success, -1 on error
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmd_case(
    block: SynBlockHandle,
    arg: *const c_char,
    arg_end: *const c_char,
) -> c_int {
    if block.is_null() || arg.is_null() {
        return -1;
    }

    let arg_len = arg_end.offset_from(arg);

    // No argument: show current setting
    if *arg == 0 {
        if nvim_synblock_get_syn_ic(block) != 0 {
            msg_display(MSG_SYNTAX_CASE_IGNORE.as_ptr().cast());
        } else {
            msg_display(MSG_SYNTAX_CASE_MATCH.as_ptr().cast());
        }
        return 0;
    }

    // Check for "match"
    if strnicmp_exact(arg, b"match", 5, arg_len) {
        nvim_synblock_set_syn_ic(block, 0);
        return 0;
    }

    // Check for "ignore"
    if strnicmp_exact(arg, b"ignore", 6, arg_len) {
        nvim_synblock_set_syn_ic(block, 1);
        return 0;
    }

    // Invalid argument
    error_illegal_arg(arg);
    -1
}

/// Handle ":syntax conceal" command from Rust.
///
/// # Arguments
/// * `block` - The synblock to modify
/// * `arg` - Command argument (NUL-terminated)
/// * `arg_end` - Pointer to end of word (result of skiptowhite)
///
/// # Returns
/// 0 on success, -1 on error
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmd_conceal(
    block: SynBlockHandle,
    arg: *const c_char,
    arg_end: *const c_char,
) -> c_int {
    if block.is_null() || arg.is_null() {
        return -1;
    }

    let arg_len = arg_end.offset_from(arg);

    // No argument: show current setting
    if *arg == 0 {
        if nvim_synblock_get_conceal(block) != 0 {
            msg_display(MSG_SYNTAX_CONCEAL_ON.as_ptr().cast());
        } else {
            msg_display(MSG_SYNTAX_CONCEAL_OFF.as_ptr().cast());
        }
        return 0;
    }

    // Check for "on"
    if strnicmp_exact(arg, b"on", 2, arg_len) {
        nvim_synblock_set_conceal(block, 1);
        return 0;
    }

    // Check for "off"
    if strnicmp_exact(arg, b"off", 3, arg_len) {
        nvim_synblock_set_conceal(block, 0);
        return 0;
    }

    // Invalid argument
    error_illegal_arg(arg);
    -1
}

/// Handle ":syntax spell" command from Rust.
///
/// # Arguments
/// * `block` - The synblock to modify
/// * `arg` - Command argument (NUL-terminated)
/// * `arg_end` - Pointer to end of word (result of skiptowhite)
///
/// # Returns
/// 0 on success, -1 on error
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmd_spell(
    block: SynBlockHandle,
    arg: *const c_char,
    arg_end: *const c_char,
) -> c_int {
    if block.is_null() || arg.is_null() {
        return -1;
    }

    let arg_len = arg_end.offset_from(arg);

    // No argument: show current setting
    if *arg == 0 {
        let spell = nvim_synblock_get_syn_spell(block);
        let msg = match spell {
            spell_mode::TOP => MSG_SYNTAX_SPELL_TOPLEVEL.as_ptr().cast(),
            spell_mode::NOTOP => MSG_SYNTAX_SPELL_NOTOPLEVEL.as_ptr().cast(),
            _ => MSG_SYNTAX_SPELL_DEFAULT.as_ptr().cast(),
        };
        msg_display(msg);
        return 0;
    }

    // Check for "toplevel"
    if strnicmp_exact(arg, b"toplevel", 8, arg_len) {
        nvim_synblock_set_syn_spell(block, spell_mode::TOP);
        return 0;
    }

    // Check for "notoplevel"
    if strnicmp_exact(arg, b"notoplevel", 10, arg_len) {
        nvim_synblock_set_syn_spell(block, spell_mode::NOTOP);
        return 0;
    }

    // Check for "default"
    if strnicmp_exact(arg, b"default", 7, arg_len) {
        nvim_synblock_set_syn_spell(block, spell_mode::DEFAULT);
        return 0;
    }

    // Invalid argument
    error_illegal_arg(arg);
    -1
}

/// Handle ":syntax foldlevel" command from Rust.
///
/// # Arguments
/// * `block` - The synblock to modify
/// * `arg` - Command argument (NUL-terminated)
/// * `arg_end` - Pointer to end of word (result of skiptowhite)
///
/// # Returns
/// 0 on success, -1 on error (also returns -1 if extra args after valid keyword)
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmd_foldlevel(
    block: SynBlockHandle,
    arg: *const c_char,
    arg_end: *const c_char,
) -> c_int {
    if block.is_null() || arg.is_null() {
        return -1;
    }

    let arg_len = arg_end.offset_from(arg);

    // No argument: show current setting
    if *arg == 0 {
        let foldlevel = nvim_synblock_get_syn_foldlevel(block);
        let msg = match foldlevel {
            foldlevel_mode::START => MSG_SYNTAX_FOLDLEVEL_START.as_ptr().cast(),
            foldlevel_mode::MINIMUM => MSG_SYNTAX_FOLDLEVEL_MINIMUM.as_ptr().cast(),
            _ => return 0,
        };
        msg_display(msg);
        return 0;
    }

    // Check for "start"
    if strnicmp_exact(arg, b"start", 5, arg_len) {
        nvim_synblock_set_syn_foldlevel(block, foldlevel_mode::START);
        // Check for extra arguments after the keyword
        let after = skipwhite(arg_end);
        if *after != 0 {
            error_illegal_arg(after);
            return -1;
        }
        return 0;
    }

    // Check for "minimum"
    if strnicmp_exact(arg, b"minimum", 7, arg_len) {
        nvim_synblock_set_syn_foldlevel(block, foldlevel_mode::MINIMUM);
        // Check for extra arguments after the keyword
        let after = skipwhite(arg_end);
        if *after != 0 {
            error_illegal_arg(after);
            return -1;
        }
        return 0;
    }

    // Invalid argument
    error_illegal_arg(arg);
    -1
}

// =============================================================================
// Phase 4: Simple subcommands and syn_maybe_enable
// =============================================================================

extern "C" {
    // EAP accessors
    fn nvim_syn_get_eap_arg(eap: *const c_void) -> *mut c_char;
    fn nvim_syn_get_eap_skip(eap: *const c_void) -> c_int;

    // check_nextcmd accessor (sets eap->nextcmd = check_nextcmd(arg))
    fn nvim_syn_set_nextcmd(eap: *mut c_void, arg: *mut c_char);

    // Reset: calls init_highlight(true, true)
    fn nvim_syn_init_highlight(reset: c_int, init: c_int);

    // did_syntax_onoff flag getter/setter (Phase 11)
    fn nvim_syn_get_did_syntax_onoff() -> c_int;
    fn nvim_syn_set_did_syntax_onoff(v: c_int);

    // Run a cmdline command string (Phase 11)
    fn nvim_syn_do_cmdline_cmd(cmd: *const c_char);
}

/// SYNTAX_FNAME format string for sourcing syntax files.
const SYNTAX_FNAME: &str = "$VIMRUNTIME/syntax/%s.vim";

/// Handle `:syntax reset` command: resets highlighting.
///
/// # Safety
/// Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmd_reset(eap: *mut c_void, _syncing: c_int) {
    let arg = nvim_syn_get_eap_arg(eap);
    nvim_syn_set_nextcmd(eap, arg);
    if nvim_syn_get_eap_skip(eap) == 0 {
        nvim_syn_init_highlight(1, 1);
    }
}

/// Core do_onoff logic: sets eap->nextcmd, sets did_syntax_onoff, builds and runs command.
///
/// # Safety
/// Must be called from main thread. `name` must be a valid NUL-terminated C string.
unsafe fn do_onoff_impl(eap: *mut c_void, name: *const c_char) {
    let arg = nvim_syn_get_eap_arg(eap);
    nvim_syn_set_nextcmd(eap, arg);
    if nvim_syn_get_eap_skip(eap) == 0 {
        nvim_syn_set_did_syntax_onoff(1);
        // Build "so $VIMRUNTIME/syntax/<name>.vim"
        let name_str = std::ffi::CStr::from_ptr(name).to_string_lossy();
        let cmd = format!("so {}", SYNTAX_FNAME.replace("%s", &name_str));
        let cmd_cstr = std::ffi::CString::new(cmd).unwrap_or_default();
        nvim_syn_do_cmdline_cmd(cmd_cstr.as_ptr());
    }
}

/// Handle `:syntax on`, `:syntax off`, `:syntax manual` commands.
///
/// `name` must be a NUL-terminated C string: "syntax", "nosyntax", or "manual".
///
/// # Safety
/// Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmd_onoff(eap: *mut c_void, name: *const c_char, _syncing: c_int) {
    do_onoff_impl(eap, name);
}

/// FFI entry point called from C nvim_syn_do_onoff thin wrapper.
///
/// # Safety
/// Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_do_onoff_impl(eap: *mut c_void, name: *const c_char) {
    do_onoff_impl(eap, name);
}

/// Enable syntax if not already done (`syn_maybe_enable`).
///
/// # Safety
/// Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_maybe_enable() {
    if nvim_syn_get_did_syntax_onoff() == 0 {
        rs_syn_do_maybe_enable_impl();
    }
}

/// Core logic for syn_maybe_enable: create minimal dispatch call for syntax on.
///
/// # Safety
/// Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_do_maybe_enable_impl() {
    // Build the "so $VIMRUNTIME/syntax/syntax.vim" command directly
    nvim_syn_set_did_syntax_onoff(1);
    let cmd = format!("so {}", SYNTAX_FNAME.replace("%s", "syntax"));
    let cmd_cstr = std::ffi::CString::new(cmd).unwrap_or_default();
    nvim_syn_do_cmdline_cmd(cmd_cstr.as_ptr());
}

// =============================================================================
// Phase 1: Dispatch wrappers for syn_cmd_case/conceal/foldlevel/spell/on/off/manual
// =============================================================================

extern "C" {
    /// Set eap->nextcmd = find_nextcmd(arg).
    fn nvim_syn_find_nextcmd(eap: *mut c_void, arg: *mut c_char);

    /// Redraw curwin (UPD_NOT_VALID) -- used after :syntax spell.
    fn nvim_syn_redraw_later_curwin();
}

/// Dispatch for `:syntax case` -- signature matches subcommands[] table.
///
/// # Safety
/// Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmd_case_dispatch(eap: *mut c_void, _syncing: c_int) {
    let arg = nvim_syn_get_eap_arg(eap);
    nvim_syn_find_nextcmd(eap, arg);
    if nvim_syn_get_eap_skip(eap) != 0 {
        return;
    }
    let block = nvim_syn_get_curwin_synblock();
    let next = skiptowhite(arg);
    rs_syn_cmd_case(block, arg, next);
}

/// Dispatch for `:syntax conceal` -- signature matches subcommands[] table.
///
/// # Safety
/// Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmd_conceal_dispatch(eap: *mut c_void, _syncing: c_int) {
    let arg = nvim_syn_get_eap_arg(eap);
    nvim_syn_find_nextcmd(eap, arg);
    if nvim_syn_get_eap_skip(eap) != 0 {
        return;
    }
    let block = nvim_syn_get_curwin_synblock();
    let next = skiptowhite(arg);
    rs_syn_cmd_conceal(block, arg, next);
}

/// Dispatch for `:syntax foldlevel` -- signature matches subcommands[] table.
///
/// # Safety
/// Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmd_foldlevel_dispatch(eap: *mut c_void, _syncing: c_int) {
    let arg = nvim_syn_get_eap_arg(eap);
    nvim_syn_find_nextcmd(eap, arg);
    if nvim_syn_get_eap_skip(eap) != 0 {
        return;
    }
    let block = nvim_syn_get_curwin_synblock();
    let arg_end = skiptowhite(arg);
    rs_syn_cmd_foldlevel(block, arg, arg_end);
}

/// Dispatch for `:syntax spell` -- signature matches subcommands[] table.
///
/// # Safety
/// Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmd_spell_dispatch(eap: *mut c_void, _syncing: c_int) {
    let arg = nvim_syn_get_eap_arg(eap);
    nvim_syn_find_nextcmd(eap, arg);
    if nvim_syn_get_eap_skip(eap) != 0 {
        return;
    }
    let block = nvim_syn_get_curwin_synblock();
    let next = skiptowhite(arg);
    rs_syn_cmd_spell(block, arg, next);
    // assume spell checking changed, force a redraw
    nvim_syn_redraw_later_curwin();
}

/// Dispatch for `:syntax on` -- signature matches subcommands[] table.
///
/// # Safety
/// Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmd_on_dispatch(eap: *mut c_void, syncing: c_int) {
    rs_syn_cmd_onoff(eap, c"syntax".as_ptr(), syncing);
}

/// Dispatch for `:syntax manual` -- signature matches subcommands[] table.
///
/// # Safety
/// Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmd_manual_dispatch(eap: *mut c_void, syncing: c_int) {
    rs_syn_cmd_onoff(eap, c"manual".as_ptr(), syncing);
}

/// Dispatch for `:syntax off` -- signature matches subcommands[] table.
///
/// # Safety
/// Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmd_off_dispatch(eap: *mut c_void, syncing: c_int) {
    rs_syn_cmd_onoff(eap, c"nosyntax".as_ptr(), syncing);
}

// =============================================================================
// Phase 2: ex_syntax dispatcher in Rust
// =============================================================================

extern "C" {
    /// Set syn_cmdlinep from eap->cmdlinep.
    fn nvim_syn_set_cmdlinep_from_eap(eap: *mut c_void);

    /// Set eap->arg.
    fn nvim_syn_set_eap_arg(eap: *mut c_void, arg: *mut c_char);

    /// Allocate a copy of `s[0..len]`.
    fn nvim_syn_xstrnsave(s: *const c_char, len: c_int) -> *mut c_char;

    /// Free memory allocated with xmalloc/xstrdup/etc.
    fn xfree(ptr: *mut c_void);

    /// Increment emsg_skip (suppress errors for this subcommand).
    fn nvim_syn_emsg_skip_inc();

    /// Decrement emsg_skip.
    fn nvim_syn_emsg_skip_dec();

    // Subcommand handlers that are still C functions (static wrappers)
    fn rs_syn_cmd_list(eap: *mut c_void, syncing: c_int);
}

/// Type alias for subcommand handler function pointers.
type SynCmdFn = unsafe extern "C" fn(*mut c_void, c_int);

/// Static dispatch table: (name, handler) pairs.
///
/// This mirrors the C `subcommands[]` table, now maintained in Rust.
static SUBCOMMANDS: &[(&str, SynCmdFn)] = &[
    ("case", rs_syn_cmd_case_dispatch),
    ("clear", crate::cmd_clear::rs_syn_cmd_clear),
    ("cluster", crate::cluster::rs_syn_cmd_cluster),
    ("conceal", rs_syn_cmd_conceal_dispatch),
    ("enable", rs_syn_cmd_on_dispatch),
    ("foldlevel", rs_syn_cmd_foldlevel_dispatch),
    ("include", crate::cmd_include::rs_syn_cmd_include),
    ("iskeyword", rs_syn_cmd_iskeyword),
    ("keyword", crate::cmd_keyword::rs_syn_cmd_keyword),
    ("list", rs_syn_cmd_list),
    ("manual", rs_syn_cmd_manual_dispatch),
    ("match", crate::cmd_match::rs_syn_cmd_match),
    ("on", rs_syn_cmd_on_dispatch),
    ("off", rs_syn_cmd_off_dispatch),
    ("region", crate::cmd_region::rs_syn_cmd_region),
    ("reset", rs_syn_cmd_reset),
    ("spell", rs_syn_cmd_spell_dispatch),
    ("sync", crate::cmd_sync::rs_syn_cmd_sync),
    ("", rs_syn_cmd_list),
];

/// Error message for invalid :syntax subcommand.
static E410_FMT: &[u8] = b"E410: Invalid :syntax subcommand: %s\0";

/// `:syntax` command dispatcher -- Rust implementation.
///
/// This replaces the C `ex_syntax` function and the `subcommands[]` table.
///
/// # Safety
/// Must be called from the main thread during command execution.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_syntax(eap: *mut c_void) {
    // Set syn_cmdlinep for error messages
    nvim_syn_set_cmdlinep_from_eap(eap);

    let arg = nvim_syn_get_eap_arg(eap);

    // Isolate subcommand name (skip alpha chars)
    let mut subcmd_end = arg;
    while *subcmd_end != 0 && (*subcmd_end as u8).is_ascii_alphabetic() {
        subcmd_end = subcmd_end.add(1);
    }

    let name_len = subcmd_end.offset_from(arg) as c_int;
    let subcmd_name = nvim_syn_xstrnsave(arg, name_len);

    if nvim_syn_get_eap_skip(eap) != 0 {
        nvim_syn_emsg_skip_inc();
    }

    // Build a Rust string slice for comparison (no allocation)
    let name_bytes = std::slice::from_raw_parts(subcmd_name as *const u8, name_len as usize);
    let name_str = std::str::from_utf8_unchecked(name_bytes);

    let mut found = false;
    for &(entry_name, handler) in SUBCOMMANDS {
        if entry_name == name_str {
            // Advance eap->arg past the subcommand name + whitespace
            nvim_syn_set_eap_arg(eap, skipwhite(subcmd_end));
            handler(eap, 0);
            found = true;
            break;
        }
    }

    if !found {
        semsg(E410_FMT.as_ptr().cast(), subcmd_name);
    }

    xfree(subcmd_name as *mut c_void);

    if nvim_syn_get_eap_skip(eap) != 0 {
        nvim_syn_emsg_skip_dec();
    }
}

// =============================================================================
// Phase 4: syn_cmd_iskeyword and ex_ownsyntax in Rust
// =============================================================================

extern "C" {
    /// Return 1 if block->b_syn_isk is set (not empty_string_option).
    fn nvim_syn_iskeyword_is_set(block: SynBlockHandle) -> c_int;

    /// Return block->b_syn_isk string.
    fn nvim_syn_iskeyword_get(block: SynBlockHandle) -> *mut c_char;

    /// Clear iskeyword: copy curbuf chartab to synblock chartab, clear b_syn_isk.
    fn nvim_syn_iskeyword_clear(block: SynBlockHandle);

    /// Set iskeyword: save/restore curbuf, set b_p_isk=arg, run buf_init_chartab,
    /// copy result to synblock, transfer to b_syn_isk.
    fn nvim_syn_iskeyword_set(block: SynBlockHandle, arg: *const c_char);

    /// msg_outtrans wrapper.
    fn nvim_syn_msg_outtrans(s: *const c_char);

    /// Phase 11: Check if curwin shares buffer's synblock.
    fn nvim_curwin_shares_buf_synblock() -> c_int;
    /// Phase 11: Allocate and init a new synblock (not yet assigned to curwin).
    fn nvim_syn_alloc_new_synblock() -> SynBlockHandle;
    /// Phase 11: Assign new synblock to curwin->w_s, clear w_p_spell.
    fn nvim_curwin_set_synblock(block: SynBlockHandle);

    /// Get a Vim variable value.
    fn nvim_syn_get_var_value(name: *const c_char) -> *mut c_char;

    /// Duplicate a C string.
    fn nvim_syn_xstrdup(s: *const c_char) -> *mut c_char;

    /// Apply EVENT_SYNTAX autocmds.
    fn nvim_syn_apply_autocmds_syntax(arg: *const c_char);

    /// set_internal_string_var wrapper.
    fn nvim_syn_set_internal_string_var(name: *const c_char, val: *const c_char);

    /// Unlet b:current_syntax.
    fn nvim_syn_do_unlet_b_current_syntax();
}

/// Static message strings for iskeyword display
static MSG_ISK_NEWLINE: &[u8] = b"\n\0";
static MSG_ISK_PREFIX: &[u8] = b"syntax iskeyword \0";
static MSG_ISK_NOT_SET: &[u8] = b"syntax iskeyword not set\0";

/// `:syntax iskeyword` command -- Rust implementation.
///
/// # Safety
/// Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_cmd_iskeyword(eap: *mut c_void, _syncing: c_int) {
    use std::ffi::CStr;

    if nvim_syn_get_eap_skip(eap) != 0 {
        return;
    }

    let arg = skipwhite(nvim_syn_get_eap_arg(eap));
    let block = nvim_syn_get_curwin_synblock();

    if *arg == 0 {
        // No argument: display current setting
        // msg_puts("\n")
        extern "C" {
            fn msg_puts(s: *const c_char);
        }
        msg_puts(MSG_ISK_NEWLINE.as_ptr().cast());
        if nvim_syn_iskeyword_is_set(block) != 0 {
            msg_puts(MSG_ISK_PREFIX.as_ptr().cast());
            let isk = nvim_syn_iskeyword_get(block);
            nvim_syn_msg_outtrans(isk);
        } else {
            nvim_syn_msg_outtrans(MSG_ISK_NOT_SET.as_ptr().cast());
        }
    } else {
        // Check for "clear"
        let arg_str = CStr::from_ptr(arg).to_bytes();
        let is_clear = arg_str.len() >= 5 && arg_str[..5].eq_ignore_ascii_case(b"clear");

        if is_clear {
            nvim_syn_iskeyword_clear(block);
        } else {
            nvim_syn_iskeyword_set(block, arg);
        }
    }
    nvim_syn_redraw_later_curwin();
}

/// `:ownsyntax` command -- Rust implementation.
///
/// # Safety
/// Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_ownsyntax(eap: *mut c_void) {
    rs_syn_ownsyntax_init();

    let arg = nvim_syn_get_eap_arg(eap);

    // Save b:current_syntax
    let b_current_syntax_key = c"b:current_syntax".as_ptr();
    let w_current_syntax_key = c"w:current_syntax".as_ptr();

    let old_raw = nvim_syn_get_var_value(b_current_syntax_key);
    let old_value: *mut c_char = if old_raw.is_null() {
        std::ptr::null_mut()
    } else {
        nvim_syn_xstrdup(old_raw)
    };

    // Apply "syntax" autocommand event
    nvim_syn_apply_autocmds_syntax(arg);

    // Move b:current_syntax to w:current_syntax
    let new_value = nvim_syn_get_var_value(b_current_syntax_key);
    if !new_value.is_null() {
        nvim_syn_set_internal_string_var(w_current_syntax_key, new_value);
    }

    // Restore b:current_syntax
    if old_value.is_null() {
        nvim_syn_do_unlet_b_current_syntax();
    } else {
        nvim_syn_set_internal_string_var(b_current_syntax_key, old_value);
        xfree(old_value as *mut c_void);
    }
}

/// Initialize ownsyntax: allocate a new synblock for curwin if it currently
/// shares the buffer's synblock.
///
/// Replaces C `nvim_syn_ownsyntax_init`.
///
/// Returns 1 if a new block was created, 0 if curwin already has its own block.
///
/// # Safety
/// Accesses C global state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_ownsyntax_init() -> c_int {
    if nvim_curwin_shares_buf_synblock() == 0 {
        return 0;
    }
    let new_block = nvim_syn_alloc_new_synblock();
    nvim_curwin_set_synblock(new_block);
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_subcommand() {
        assert_eq!(
            SyntaxSubcommand::from_name("case"),
            Some(SyntaxSubcommand::Case)
        );
        assert_eq!(
            SyntaxSubcommand::from_name("clear"),
            Some(SyntaxSubcommand::Clear)
        );
        assert_eq!(
            SyntaxSubcommand::from_name("on"),
            Some(SyntaxSubcommand::Enable)
        );
        assert_eq!(
            SyntaxSubcommand::from_name("enable"),
            Some(SyntaxSubcommand::Enable)
        );
        assert_eq!(
            SyntaxSubcommand::from_name(""),
            Some(SyntaxSubcommand::List)
        );
        assert_eq!(SyntaxSubcommand::from_name("invalid"), None);

        assert_eq!(SyntaxSubcommand::Case.name(), "case");
        assert_eq!(SyntaxSubcommand::Enable.name(), "enable");
    }

    #[test]
    fn test_case_mode() {
        assert_eq!(CaseMode::from_arg("match"), Some(CaseMode::Match));
        assert_eq!(CaseMode::from_arg("ignore"), Some(CaseMode::Ignore));
        assert_eq!(CaseMode::from_arg("invalid"), None);

        assert_eq!(CaseMode::Match.as_str(), "match");
        assert_eq!(CaseMode::Ignore.as_str(), "ignore");
    }

    #[test]
    fn test_spell_mode() {
        assert_eq!(SpellMode::from_arg("toplevel"), Some(SpellMode::TopLevel));
        assert_eq!(
            SpellMode::from_arg("notoplevel"),
            Some(SpellMode::NoTopLevel)
        );
        assert_eq!(SpellMode::from_arg("default"), Some(SpellMode::Default));
        assert_eq!(SpellMode::from_arg("invalid"), None);

        assert_eq!(SpellMode::from_raw(spell_mode::DEFAULT), SpellMode::Default);
        assert_eq!(SpellMode::from_raw(spell_mode::TOP), SpellMode::TopLevel);
        assert_eq!(
            SpellMode::from_raw(spell_mode::NOTOP),
            SpellMode::NoTopLevel
        );

        assert_eq!(SpellMode::Default.to_raw(), spell_mode::DEFAULT);
        assert_eq!(SpellMode::TopLevel.to_raw(), spell_mode::TOP);
        assert_eq!(SpellMode::NoTopLevel.to_raw(), spell_mode::NOTOP);
    }

    #[test]
    fn test_pattern_options() {
        let opts = PatternOptions::new();
        assert!(!opts.contained);
        assert!(!opts.transparent);
        assert!(!opts.has_containment());

        let opts = PatternOptions {
            contained: true,
            contains: vec!["Comment".to_string()],
            ..Default::default()
        };
        assert!(opts.contained);
        assert!(opts.has_containment());
    }

    #[test]
    fn test_sync_mode() {
        assert_eq!(SyncMode::from_arg("fromstart"), Some(SyncMode::FromStart));
        assert_eq!(SyncMode::from_arg("ccomment"), Some(SyncMode::CComment));
        assert_eq!(SyncMode::from_arg("match"), Some(SyncMode::Match));
        assert_eq!(SyncMode::from_arg("invalid"), None);
    }

    #[test]
    fn test_expand_what() {
        assert!(!ExpandWhat::Subcmd.options().is_empty());
        assert!(ExpandWhat::Subcmd.options().contains(&"case"));
        assert!(ExpandWhat::Subcmd.options().contains(&"clear"));

        assert_eq!(ExpandWhat::Case.options(), &["match", "ignore"]);
        assert_eq!(
            ExpandWhat::Spell.options(),
            &["toplevel", "notoplevel", "default"]
        );
    }

    #[test]
    fn test_all_subcommand_names() {
        let names = SyntaxSubcommand::all_names();
        // Should have at least the basic commands
        assert!(names.len() >= 15);
        assert!(names.contains(&"case"));
        assert!(names.contains(&"clear"));
        assert!(names.contains(&"keyword"));
        assert!(names.contains(&"match"));
        assert!(names.contains(&"region"));
        // "on" is an alias for "enable"
        assert!(names.contains(&"on"));
        assert!(names.contains(&"enable"));
    }

    #[test]
    fn test_subcommand_roundtrip() {
        // All subcommands should roundtrip through name()
        let cmds = [
            SyntaxSubcommand::Case,
            SyntaxSubcommand::Clear,
            SyntaxSubcommand::Cluster,
            SyntaxSubcommand::Conceal,
            SyntaxSubcommand::Enable,
            SyntaxSubcommand::FoldLevel,
            SyntaxSubcommand::Include,
            SyntaxSubcommand::IsKeyword,
            SyntaxSubcommand::Keyword,
            SyntaxSubcommand::List,
            SyntaxSubcommand::Manual,
            SyntaxSubcommand::Match,
            SyntaxSubcommand::Off,
            SyntaxSubcommand::Region,
            SyntaxSubcommand::Reset,
            SyntaxSubcommand::Spell,
            SyntaxSubcommand::Sync,
        ];

        for cmd in cmds {
            let name = cmd.name();
            let parsed = SyntaxSubcommand::from_name(name);
            assert_eq!(parsed, Some(cmd), "Failed roundtrip for {:?}", cmd);
        }
    }

    #[test]
    fn test_pattern_options_default_values() {
        let opts = PatternOptions::default();
        assert!(!opts.contained);
        assert!(!opts.transparent);
        assert!(!opts.oneline);
        assert!(!opts.keepend);
        assert!(!opts.extend);
        assert!(!opts.excludenl);
        assert!(!opts.skipwhite);
        assert!(!opts.skipnl);
        assert!(!opts.skipempty);
        assert!(!opts.conceal);
        assert!(!opts.concealends);
        assert!(!opts.fold);
        assert!(opts.cchar.is_none());
        assert!(opts.contains.is_empty());
        assert!(opts.containedin.is_empty());
        assert!(opts.nextgroup.is_empty());
    }

    #[test]
    fn test_sync_mode_all_variants() {
        // Test all sync mode variants can be parsed
        assert_eq!(SyncMode::from_arg("fromstart"), Some(SyncMode::FromStart));
        assert_eq!(SyncMode::from_arg("ccomment"), Some(SyncMode::CComment));
        assert_eq!(SyncMode::from_arg("match"), Some(SyncMode::Match));
        assert_eq!(SyncMode::from_arg("lines"), Some(SyncMode::Lines));
        assert_eq!(SyncMode::from_arg("minlines"), Some(SyncMode::MinLines));
        assert_eq!(SyncMode::from_arg("maxlines"), Some(SyncMode::MaxLines));
        assert_eq!(SyncMode::from_arg("linebreaks"), Some(SyncMode::LineBreaks));
    }

    #[test]
    fn test_case_mode_default() {
        // Default should be Match (case-sensitive)
        let default = CaseMode::default();
        assert_eq!(default, CaseMode::Match);
    }

    #[test]
    fn test_spell_mode_default() {
        let default = SpellMode::default();
        assert_eq!(default, SpellMode::Default);
    }

    #[test]
    fn test_sync_mode_default() {
        let default = SyncMode::default();
        assert_eq!(default, SyncMode::FromStart);
    }
}
