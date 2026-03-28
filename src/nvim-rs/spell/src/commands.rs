//! Spell command utilities for Neovim
//!
//! This module provides support functions for spell-related commands:
//! - Navigation commands (`]s`, `[s`, `]S`, `[S`)
//! - Word modification commands (`zg`, `zw`, `zug`, `zuw`, `zG`, `zW`)
//! - Ex commands (`:spellinfo`, `:spelldump`, `:spellrepall`)
//!
//! The actual command execution remains in C due to deep integration with
//! Neovim's buffer, window, and undo systems. This module provides the
//! supporting logic and data structures.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Navigation Direction and Types
// =============================================================================

/// Direction of spell navigation movement.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpellMoveDirection {
    /// Move forward in the buffer
    #[default]
    Forward = 0,
    /// Move backward in the buffer
    Backward = 1,
}

impl SpellMoveDirection {
    /// Convert from C integer (FORWARD = 1, BACKWARD = -1 in Neovim).
    #[must_use]
    pub const fn from_vim_direction(dir: c_int) -> Self {
        if dir < 0 {
            Self::Backward
        } else {
            Self::Forward
        }
    }

    /// Convert to Vim's direction constant.
    #[must_use]
    pub const fn to_vim_direction(self) -> c_int {
        match self {
            Self::Forward => 1,
            Self::Backward => -1,
        }
    }

    /// Check if this is forward movement.
    #[must_use]
    pub const fn is_forward(self) -> bool {
        matches!(self, Self::Forward)
    }

    /// Check if this is backward movement.
    #[must_use]
    pub const fn is_backward(self) -> bool {
        matches!(self, Self::Backward)
    }
}

/// FFI wrapper to create direction from Vim's direction constant.
#[no_mangle]
pub extern "C" fn rs_spell_direction_from_vim(dir: c_int) -> SpellMoveDirection {
    SpellMoveDirection::from_vim_direction(dir)
}

/// FFI wrapper to convert direction to Vim's constant.
#[no_mangle]
pub extern "C" fn rs_spell_direction_to_vim(dir: SpellMoveDirection) -> c_int {
    dir.to_vim_direction()
}

/// FFI wrapper to check if direction is forward.
#[no_mangle]
pub extern "C" fn rs_spell_direction_is_forward(dir: SpellMoveDirection) -> bool {
    dir.is_forward()
}

// =============================================================================
// Spell Navigation Behavior
// =============================================================================

/// Behavior for spell navigation (what types of errors to find).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpellMoveBehavior {
    /// Find all types of spelling errors
    #[default]
    All = 0,
    /// Find only bad (wrong) words
    Bad = 1,
    /// Find only rare words
    Rare = 2,
}

impl SpellMoveBehavior {
    /// Convert from C integer.
    #[must_use]
    pub const fn from_c_int(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::All),
            1 => Some(Self::Bad),
            2 => Some(Self::Rare),
            _ => None,
        }
    }

    /// Check if a spell result matches this behavior.
    ///
    /// # Arguments
    /// * `hlf` - The highlight group (HLF_SPB = bad, HLF_SPR = rare, HLF_SPC = cap, HLF_SPL = local)
    #[must_use]
    pub const fn matches_highlight(self, hlf: c_int) -> bool {
        // HLF values from highlight.h
        const HLF_SPB: c_int = 37; // SpellBad
        const HLF_SPR: c_int = 39; // SpellRare
        const HLF_SPC: c_int = 38; // SpellCap
        const HLF_SPL: c_int = 40; // SpellLocal

        match self {
            Self::All => hlf == HLF_SPB || hlf == HLF_SPR || hlf == HLF_SPC || hlf == HLF_SPL,
            Self::Bad => hlf == HLF_SPB,
            Self::Rare => hlf == HLF_SPR,
        }
    }
}

/// FFI wrapper to convert integer to SpellMoveBehavior.
#[no_mangle]
pub extern "C" fn rs_spell_move_behavior_from_int(value: c_int) -> c_int {
    SpellMoveBehavior::from_c_int(value).map_or(0, |b| b as c_int)
}

/// FFI wrapper to check if a highlight matches the behavior.
#[no_mangle]
pub extern "C" fn rs_spell_move_behavior_matches(behavior: c_int, hlf: c_int) -> bool {
    SpellMoveBehavior::from_c_int(behavior).is_some_and(|b| b.matches_highlight(hlf))
}

// =============================================================================
// Word Add/Remove Commands
// =============================================================================

/// Type of spell word add command.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpellWordCommand {
    /// zg - Add word as good
    #[default]
    AddGood = 0,
    /// zw - Add word as wrong/bad
    AddWrong = 1,
    /// zug - Undo zg (remove good word)
    UndoGood = 2,
    /// zuw - Undo zw (remove wrong word)
    UndoWrong = 3,
    /// zG - Add to internal wordlist as good
    AddGoodInternal = 4,
    /// zW - Add to internal wordlist as wrong
    AddWrongInternal = 5,
    /// zuG - Undo zG
    UndoGoodInternal = 6,
    /// zuW - Undo zW
    UndoWrongInternal = 7,
}

impl SpellWordCommand {
    /// Check if this is an add operation (vs undo/remove).
    #[must_use]
    pub const fn is_add(self) -> bool {
        matches!(
            self,
            Self::AddGood | Self::AddWrong | Self::AddGoodInternal | Self::AddWrongInternal
        )
    }

    /// Check if this is an undo/remove operation.
    #[must_use]
    pub const fn is_undo(self) -> bool {
        matches!(
            self,
            Self::UndoGood | Self::UndoWrong | Self::UndoGoodInternal | Self::UndoWrongInternal
        )
    }

    /// Check if this targets the internal wordlist (zG/zW/zuG/zuW).
    #[must_use]
    pub const fn is_internal(self) -> bool {
        matches!(
            self,
            Self::AddGoodInternal
                | Self::AddWrongInternal
                | Self::UndoGoodInternal
                | Self::UndoWrongInternal
        )
    }

    /// Check if this is a "good word" operation (zg/zG/zug/zuG).
    #[must_use]
    pub const fn is_good(self) -> bool {
        matches!(
            self,
            Self::AddGood | Self::AddGoodInternal | Self::UndoGood | Self::UndoGoodInternal
        )
    }

    /// Check if this is a "wrong word" operation (zw/zW/zuw/zuW).
    #[must_use]
    pub const fn is_wrong(self) -> bool {
        matches!(
            self,
            Self::AddWrong | Self::AddWrongInternal | Self::UndoWrong | Self::UndoWrongInternal
        )
    }

    /// Get the spell add type for this command.
    #[must_use]
    pub const fn add_type(self) -> c_int {
        if self.is_good() {
            0 // SpellAddType::Good
        } else {
            1 // SpellAddType::Bad
        }
    }

    /// Get the spellfile index for this command.
    ///
    /// Returns 0 for internal wordlist commands, -1 to indicate "use default".
    #[must_use]
    pub const fn spellfile_idx(self) -> c_int {
        if self.is_internal() {
            0
        } else {
            -1 // Use default from 'spellfile' option
        }
    }
}

/// FFI wrapper to check if command is an add operation.
#[no_mangle]
pub extern "C" fn rs_spell_word_cmd_is_add(cmd: SpellWordCommand) -> bool {
    cmd.is_add()
}

/// FFI wrapper to check if command is an undo operation.
#[no_mangle]
pub extern "C" fn rs_spell_word_cmd_is_undo(cmd: SpellWordCommand) -> bool {
    cmd.is_undo()
}

/// FFI wrapper to check if command targets internal wordlist.
#[no_mangle]
pub extern "C" fn rs_spell_word_cmd_is_internal(cmd: SpellWordCommand) -> bool {
    cmd.is_internal()
}

/// FFI wrapper to get add type for command.
#[no_mangle]
pub extern "C" fn rs_spell_word_cmd_add_type(cmd: SpellWordCommand) -> c_int {
    cmd.add_type()
}

/// FFI wrapper to get spellfile index for command.
#[no_mangle]
pub extern "C" fn rs_spell_word_cmd_spellfile_idx(cmd: SpellWordCommand) -> c_int {
    cmd.spellfile_idx()
}

// =============================================================================
// Spell Dump Options
// =============================================================================

/// Options for :spelldump command.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpellDumpOptions {
    /// Include only words from this region (empty = all regions)
    pub region: [u8; 3],
    /// Include word counts
    pub include_counts: bool,
    /// Include rare words
    pub include_rare: bool,
    /// Include banned words (preceded by /)
    pub include_banned: bool,
}

impl SpellDumpOptions {
    /// Create default options (dump all words).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            region: [0, 0, 0],
            include_counts: false,
            include_rare: true,
            include_banned: true,
        }
    }

    /// Set the region filter.
    #[must_use]
    pub fn with_region(mut self, region: &[u8]) -> Self {
        if region.len() >= 2 {
            self.region[0] = region[0];
            self.region[1] = region[1];
            self.region[2] = 0;
        }
        self
    }

    /// Enable word counts.
    #[must_use]
    pub const fn with_counts(mut self) -> Self {
        self.include_counts = true;
        self
    }

    /// Check if a region filter is set.
    #[must_use]
    pub const fn has_region_filter(&self) -> bool {
        self.region[0] != 0
    }
}

/// FFI wrapper to create default SpellDumpOptions.
#[no_mangle]
pub extern "C" fn rs_spell_dump_options_new() -> SpellDumpOptions {
    SpellDumpOptions::new()
}

/// FFI wrapper to check if options have a region filter.
///
/// # Safety
/// `opts` must be a valid pointer to a SpellDumpOptions struct.
#[no_mangle]
pub unsafe extern "C" fn rs_spell_dump_has_region_filter(opts: *const SpellDumpOptions) -> bool {
    if opts.is_null() {
        return false;
    }
    (*opts).has_region_filter()
}

// =============================================================================
// Spell Info Formatting
// =============================================================================

/// Maximum length of a spell language info line.
pub const SPELL_INFO_MAX_LINE: usize = 256;

/// Format flags for spell info output.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpellInfoFlags {
    /// Show file path
    pub show_path: bool,
    /// Show word counts
    pub show_counts: bool,
    /// Show region information
    pub show_regions: bool,
    /// Show compound rules
    pub show_compound: bool,
}

impl SpellInfoFlags {
    /// Create with all flags enabled.
    #[must_use]
    pub const fn all() -> Self {
        Self {
            show_path: true,
            show_counts: true,
            show_regions: true,
            show_compound: true,
        }
    }

    /// Create with minimal info (path only).
    #[must_use]
    pub const fn minimal() -> Self {
        Self {
            show_path: true,
            show_counts: false,
            show_regions: false,
            show_compound: false,
        }
    }
}

/// FFI wrapper to create all SpellInfoFlags.
#[no_mangle]
pub extern "C" fn rs_spell_info_flags_all() -> SpellInfoFlags {
    SpellInfoFlags::all()
}

/// FFI wrapper to create minimal SpellInfoFlags.
#[no_mangle]
pub extern "C" fn rs_spell_info_flags_minimal() -> SpellInfoFlags {
    SpellInfoFlags::minimal()
}

// =============================================================================
// Spellrepall Support
// =============================================================================

/// State for :spellrepall command.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SpellRepallState {
    /// Number of replacements made
    pub count: c_int,
    /// Line number of last replacement
    pub last_lnum: c_int,
    /// Whether any errors occurred
    pub had_error: bool,
}

impl SpellRepallState {
    /// Create new state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            count: 0,
            last_lnum: 0,
            had_error: false,
        }
    }

    /// Record a successful replacement.
    pub fn record_replacement(&mut self, lnum: c_int) {
        self.count += 1;
        self.last_lnum = lnum;
    }

    /// Record an error.
    pub fn record_error(&mut self) {
        self.had_error = true;
    }
}

/// FFI wrapper to create SpellRepallState.
#[no_mangle]
pub extern "C" fn rs_spell_repall_state_new() -> SpellRepallState {
    SpellRepallState::new()
}

/// FFI wrapper to record a replacement.
///
/// # Safety
/// `state` must be a valid pointer to a SpellRepallState struct.
#[no_mangle]
pub unsafe extern "C" fn rs_spell_repall_record(state: *mut SpellRepallState, lnum: c_int) {
    if !state.is_null() {
        (*state).record_replacement(lnum);
    }
}

/// FFI wrapper to record an error.
///
/// # Safety
/// `state` must be a valid pointer to a SpellRepallState struct.
#[no_mangle]
pub unsafe extern "C" fn rs_spell_repall_error(state: *mut SpellRepallState) {
    if !state.is_null() {
        (*state).record_error();
    }
}

/// FFI wrapper to get replacement count.
///
/// # Safety
/// `state` must be a valid pointer to a SpellRepallState struct.
#[no_mangle]
pub unsafe extern "C" fn rs_spell_repall_count(state: *const SpellRepallState) -> c_int {
    if state.is_null() {
        return 0;
    }
    (*state).count
}

// =============================================================================
// Phase 2: Spell dump helper functions
// =============================================================================

/// DUMPFLAG constants (local to spell.c dump logic).
const DUMPFLAG_KEEPCASE: c_int = 1;
const DUMPFLAG_COUNT: c_int = 2;
const DUMPFLAG_ICASE: c_int = 4;
const DUMPFLAG_ONECAP: c_int = 8;
const DUMPFLAG_ALLCAP: c_int = 16;

/// C OK constant (vim_defs.h: #define OK 1)
const OK: c_int = 1;
/// FORWARD direction constant
const FORWARD: c_int = 1;
/// WF_ word flag constants used in dump logic
const WF_ONECAP: c_int = 0x02;
const WF_ALLCAP: c_int = 0x04;
const WF_FIXCAP: c_int = 0x40;
const WF_KEEPCAP: c_int = 0x80;
const WF_CAPMASK: c_int = WF_ONECAP | WF_ALLCAP | WF_KEEPCAP;
const WF_BANNED: c_int = 0x10;
const WF_RARE: c_int = 0x08;
const WF_REGION: c_int = 0x01;
const WF_RAREPFX: c_int = 0x0020_0000;
const WF_NEEDCOMP: c_int = 0x100;

extern "C" {
    fn ml_append(lnum: i32, line: *const c_char, len: c_int, newfile: bool) -> c_int;
    fn ins_compl_add_infercase(
        str_arg: *mut c_char,
        len: c_int,
        icase: bool,
        fname: *const c_char,
        dir: c_int,
        cont_s_ipos: bool,
        score: c_int,
    ) -> c_int;
    fn line_breakcheck();
    fn mb_strnicmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn vim_snprintf(str_: *mut c_char, str_m: usize, fmt: *const c_char, ...) -> c_int;
    fn hash_find(ht: *const crate::HashtabRaw, key: *const c_char) -> *mut crate::HashitemRaw;
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, dsize: usize) -> usize;
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn strlen(s: *const c_char) -> usize;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn rs_ins_compl_check_keys(frequency: c_int, in_compl_func: c_int);
    fn rs_ins_compl_interrupted() -> c_int;
    fn nvim_win_get_b_langp(wp: *const c_void) -> *const crate::GArrayRaw;

    static hash_removed: c_char;
    static mut got_int: bool;

    #[link_name = "IObuff"]
    static mut IObuff_global: [c_char; 1025];

    #[link_name = "p_ic"]
    static p_ic_global: c_int;

    #[link_name = "curwin"]
    static curwin_global: *mut c_void;
}

/// WC_KEY_OFF: offset of wc_count before the key in wordcount_T
const WC_KEY_OFF: usize = 2;

/// Returns true if the hash item is empty (HASHITEM_EMPTY macro).
#[inline]
unsafe fn hashitem_empty_dump(hi: *const crate::HashitemRaw) -> bool {
    (*hi).hi_key.is_null()
        || std::ptr::eq((*hi).hi_key, std::ptr::addr_of!(hash_removed).cast_mut())
}

/// Get the word count from a hash item (HI2WC macro: key - WC_KEY_OFF gives wc_count u16).
#[inline]
#[allow(clippy::cast_sign_loss)]
unsafe fn hi2wc_count(hi: *const crate::HashitemRaw) -> u32 {
    let wc_base = (*hi).hi_key.cast::<u8>().sub(WC_KEY_OFF);
    let b0 = wc_base.read();
    let b1 = wc_base.add(1).read();
    u32::from(u16::from_ne_bytes([b0, b1]))
}

/// Dump one word: apply case modifications and append or add to completion.
///
/// Mirrors C: static void dump_word(slang, word, pat, dir, dumpflags, wordflags, lnum)
///
/// # Safety
/// All pointers must be valid. `dir` may be null when `pat` is null.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
unsafe fn dump_word(
    slang: *mut crate::SlangRaw,
    word: *mut c_char,
    pat: *mut c_char,
    dir: *mut c_int,
    dumpflags: c_int,
    wordflags: c_int,
    lnum: i32,
) {
    let mut flags = wordflags;
    if dumpflags & DUMPFLAG_ONECAP != 0 {
        flags |= WF_ONECAP;
    }
    if dumpflags & DUMPFLAG_ALLCAP != 0 {
        flags |= WF_ALLCAP;
    }

    // Determine the word to display: apply case modification if needed.
    let mut cword = [0u8; crate::MAXWLEN + 10];
    let (p, keepcap) = if (dumpflags & DUMPFLAG_KEEPCASE) == 0 && (flags & WF_CAPMASK) != 0 {
        crate::rs_make_case_word(word, cword.as_mut_ptr().cast::<c_char>(), flags);
        (cword.as_ptr().cast::<c_char>(), false)
    } else {
        let kc = (dumpflags & DUMPFLAG_KEEPCASE) != 0
            && ((crate::rs_captype(word, std::ptr::null()) & WF_KEEPCAP) == 0
                || (flags & WF_FIXCAP) != 0);
        (word.cast_const(), kc)
    };
    let tw: *const c_char = p; // pointer used for word-count lookup

    if pat.is_null() {
        // Build the output string, possibly with flags/regions.
        let mut badword = [0u8; crate::MAXWLEN + 10];

        let out: *const c_char = if (flags & (WF_BANNED | WF_RARE | WF_REGION)) != 0 || keepcap {
            // Copy the word into badword then append flag characters.
            let plen = strlen(p);
            std::ptr::copy_nonoverlapping(p.cast::<u8>(), badword.as_mut_ptr(), plen);
            badword[plen] = b'/';
            let mut blen = plen + 1;
            if keepcap {
                badword[blen] = b'=';
                blen += 1;
            }
            if flags & WF_BANNED != 0 {
                badword[blen] = b'!';
                blen += 1;
            } else if flags & WF_RARE != 0 {
                badword[blen] = b'?';
                blen += 1;
            }
            if flags & WF_REGION != 0 {
                for i in 0..7usize {
                    if flags & (0x1_0000 << i) != 0 {
                        badword[blen] = (i + 1) as u8 + b'0';
                        blen += 1;
                    }
                }
            }
            badword[blen] = 0;
            badword.as_ptr().cast::<c_char>()
        } else {
            p
        };

        if dumpflags & DUMPFLAG_COUNT != 0 {
            // Include word count for ":spelldump!"
            let hi = hash_find(std::ptr::addr_of!((*slang).sl_wordcount), tw);
            if !hashitem_empty_dump(hi) {
                let wc = hi2wc_count(hi);
                let iobuff = std::ptr::addr_of_mut!(IObuff_global).cast::<c_char>();
                vim_snprintf(iobuff, 1025, c"%s\t%d".as_ptr(), tw, wc);
                ml_append(lnum, iobuff, 0, false);
                return;
            }
        }

        ml_append(lnum, out, 0, false);
    } else {
        // Pattern mode: match and add to completion list.
        let patlen = strlen(pat);
        let matches = if dumpflags & DUMPFLAG_ICASE != 0 {
            mb_strnicmp(p, pat, patlen) == 0
        } else {
            strncmp(p, pat, patlen) == 0
        };
        if matches
            && ins_compl_add_infercase(
                p.cast_mut(),
                strlen(p) as c_int,
                p_ic_global != 0,
                std::ptr::null(),
                *dir,
                false,
                0,
            ) == OK
        {
            // If dir was BACKWARD, honor it just once.
            *dir = FORWARD;
        }
    }
}

/// For `:spelldump`: find matching prefixes for "word", prepend each to "word",
/// then call dump_word for each valid prefix+word combination.
///
/// Mirrors C: static linenr_T dump_prefixes(slang, word, pat, dir, dumpflags, flags, startlnum)
///
/// # Safety
/// All pointers must be valid. `dir` may be null when `pat` is null.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_lossless)]
unsafe fn dump_prefixes(
    slang: *mut crate::SlangRaw,
    word: *mut c_char,
    pat: *mut c_char,
    dir: *mut c_int,
    dumpflags: c_int,
    flags: c_int,
    startlnum: i32,
) -> i32 {
    let mut lnum = startlnum;
    let byts = (*slang).sl_pbyts;
    let idxs = (*slang).sl_pidxs;

    if byts.is_null() {
        return lnum;
    }

    // If word starts with a lower-case letter, make uppercase variant in word_up.
    let c = utf_ptr2char(word);
    let mut word_up = [0u8; crate::MAXWLEN];
    let has_word_up = crate::spell_toupper(c) != c;
    if has_word_up {
        crate::rs_onecap_copy(word, word_up.as_mut_ptr().cast::<c_char>(), true);
    }

    // Depth-first walk over the prefix tree.
    let mut arridx = [0i32; crate::MAXWLEN];
    let mut curi = [0i32; crate::MAXWLEN];
    let mut prefix = [0u8; crate::MAXWLEN];
    let mut depth: i32 = 0;
    arridx[0] = 0;
    curi[0] = 1;

    while depth >= 0 && !got_int {
        let n = arridx[depth as usize] as usize;
        let len = *byts.add(n) as i32;
        if curi[depth as usize] > len {
            // Done all bytes at this node, go up one level.
            depth -= 1;
            line_breakcheck();
        } else {
            let ni = n + curi[depth as usize] as usize;
            curi[depth as usize] += 1;
            let c = *byts.add(ni) as i32;
            if c == 0 {
                // End of prefix: count how many NUL bytes there are.
                let mut i = 1i32;
                while i < len {
                    if *byts.add(n + i as usize) != 0 {
                        break;
                    }
                    i += 1;
                }
                curi[depth as usize] += i - 1;

                let vwp =
                    crate::check::rs_valid_word_prefix(i, ni as i32, flags, word, slang, false);
                if vwp != 0 {
                    xstrlcpy(
                        prefix.as_mut_ptr().cast::<c_char>().add(depth as usize),
                        word,
                        crate::MAXWLEN.saturating_sub(depth as usize),
                    );
                    dump_word(
                        slang,
                        prefix.as_mut_ptr().cast::<c_char>(),
                        pat,
                        dir,
                        dumpflags,
                        if vwp & WF_RAREPFX != 0 {
                            flags | WF_RARE
                        } else {
                            flags
                        },
                        lnum,
                    );
                    if lnum != 0 {
                        lnum += 1;
                    }
                }

                // Also check with the uppercased first letter.
                if has_word_up {
                    let vwp2 = crate::check::rs_valid_word_prefix(
                        i,
                        ni as i32,
                        flags,
                        word_up.as_mut_ptr().cast::<c_char>(),
                        slang,
                        true,
                    );
                    if vwp2 != 0 {
                        xstrlcpy(
                            prefix.as_mut_ptr().cast::<c_char>().add(depth as usize),
                            word_up.as_mut_ptr().cast::<c_char>(),
                            crate::MAXWLEN.saturating_sub(depth as usize),
                        );
                        dump_word(
                            slang,
                            prefix.as_mut_ptr().cast::<c_char>(),
                            pat,
                            dir,
                            dumpflags,
                            if vwp2 & WF_RAREPFX != 0 {
                                flags | WF_RARE
                            } else {
                                flags
                            },
                            lnum,
                        );
                        if lnum != 0 {
                            lnum += 1;
                        }
                    }
                }
            } else {
                // Normal character: go one level deeper.
                prefix[depth as usize] = c as u8;
                depth += 1;
                arridx[depth as usize] = *idxs.add(ni);
                curi[depth as usize] = 1;
            }
        }
    }

    lnum
}

// =============================================================================
// Phase 3: spell_dump_compl traversal engine
// =============================================================================

/// Iterate all loaded spell languages, traverse word trees, apply pattern
/// matching, and call dump_word/dump_prefixes for each valid entry.
///
/// Signature matches C: void spell_dump_compl(char *pat, int ic, Direction *dir, int dumpflags_arg)
///
/// # Safety
/// Must be called from the main thread with a valid current window.
/// `pat` may be null (dump all words); `dir` may be null when `pat` is null.
#[no_mangle]
#[allow(clippy::too_many_lines)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
#[allow(clippy::cast_lossless)]
pub unsafe extern "C" fn spell_dump_compl(
    pat: *mut c_char,
    ic: c_int,
    dir: *mut c_int,
    dumpflags_arg: c_int,
) {
    let mut dumpflags = dumpflags_arg;
    let mut lnum: i32 = 0;
    let mut region_names: *const c_char = std::ptr::null();
    let mut do_region = true;

    // When ignoring case or when the pattern starts with capital, pass this on
    // to dump_word().
    if !pat.is_null() {
        if ic != 0 {
            dumpflags |= DUMPFLAG_ICASE;
        } else {
            let n = crate::rs_captype(pat, std::ptr::null());
            if n == WF_ONECAP {
                dumpflags |= DUMPFLAG_ONECAP;
            } else if n == WF_ALLCAP && strlen(pat) as c_int > utfc_ptr2len(pat) {
                dumpflags |= DUMPFLAG_ALLCAP;
            }
        }
    }

    // Find out if we can support regions: all languages must have the same
    // region names or none at all.
    let langp_ga = nvim_win_get_b_langp(curwin_global);
    let ga_len = (*langp_ga).ga_len;
    for lpi in 0..ga_len {
        let lp = crate::langp_entry(langp_ga, lpi);
        let p = (*(*lp).lp_slang).sl_regions.as_ptr();
        if *p != 0 {
            if region_names.is_null() {
                region_names = p;
            } else if strcmp(region_names, p) != 0 {
                do_region = false;
                break;
            }
        }
    }

    if do_region && !region_names.is_null() && pat.is_null() {
        let iobuff = std::ptr::addr_of_mut!(IObuff_global).cast::<c_char>();
        vim_snprintf(iobuff, 1025, c"/regions=%s".as_ptr(), region_names);
        ml_append(lnum, iobuff, 0, false);
        lnum += 1;
    } else {
        do_region = false;
    }

    // Loop over all spell files loaded for 'spelllang'.
    for lpi in 0..ga_len {
        let lp = crate::langp_entry(langp_ga, lpi);
        let slang = (*lp).lp_slang;
        if (*slang).sl_fbyts.is_null() {
            continue; // reloading failed
        }

        if pat.is_null() {
            let iobuff = std::ptr::addr_of_mut!(IObuff_global).cast::<c_char>();
            vim_snprintf(iobuff, 1025, c"# file: %s".as_ptr(), (*slang).sl_fname);
            ml_append(lnum, iobuff, 0, false);
            lnum += 1;
        }

        // When matching with a pattern and there are no prefixes, only use
        // parts of the tree that match "pat".
        let patlen: i32 = if !pat.is_null() && (*slang).sl_pbyts.is_null() {
            strlen(pat) as i32
        } else {
            -1
        };

        // Round 1: case-folded tree; round 2: keep-case tree.
        for round in 1..=2i32 {
            let (byts, idxs) = if round == 1 {
                dumpflags &= !DUMPFLAG_KEEPCASE;
                ((*slang).sl_fbyts, (*slang).sl_fidxs)
            } else {
                dumpflags |= DUMPFLAG_KEEPCASE;
                ((*slang).sl_kbyts, (*slang).sl_kidxs)
            };
            if byts.is_null() {
                continue;
            }

            let mut arridx = [0i32; crate::MAXWLEN];
            let mut curi = [0i32; crate::MAXWLEN];
            let mut word = [0u8; crate::MAXWLEN];
            let mut depth: i32 = 0;
            arridx[0] = 0;
            curi[0] = 1;

            while depth >= 0 && !got_int && (pat.is_null() || rs_ins_compl_interrupted() == 0) {
                let ad = arridx[depth as usize] as usize;
                if curi[depth as usize] > *byts.add(ad) as i32 {
                    // Done all bytes at this node, go up one level.
                    depth -= 1;
                    line_breakcheck();
                    rs_ins_compl_check_keys(50, 0);
                } else {
                    let n = ad + curi[depth as usize] as usize;
                    curi[depth as usize] += 1;
                    let c = *byts.add(n) as i32;

                    if c == 0 || depth >= crate::MAXWLEN as i32 - 1 {
                        // End of word or max depth: process the word.
                        let flags = *idxs.add(n);
                        let lp_ref = crate::langp_entry(langp_ga, lpi);
                        if (round == 2 || (flags & WF_KEEPCAP) == 0)
                            && (flags & WF_NEEDCOMP) == 0
                            && (do_region
                                || (flags & WF_REGION) == 0
                                || (((flags as u32) >> 16) & ((*lp_ref).lp_region as u32)) != 0)
                        {
                            word[depth as usize] = 0;
                            let wflags = if do_region { flags } else { flags & !WF_REGION };

                            // Dump basic word if no prefix or it's the first.
                            let pfx_id = (flags as u32) >> 24;
                            if pfx_id == 0 || curi[depth as usize] == 2 {
                                dump_word(
                                    slang,
                                    word.as_mut_ptr().cast::<c_char>(),
                                    pat,
                                    dir,
                                    dumpflags,
                                    wflags,
                                    lnum,
                                );
                                if pat.is_null() {
                                    lnum += 1;
                                }
                            }

                            // Apply prefix if there is one.
                            if pfx_id != 0 {
                                lnum = dump_prefixes(
                                    slang,
                                    word.as_mut_ptr().cast::<c_char>(),
                                    pat,
                                    dir,
                                    dumpflags,
                                    wflags,
                                    lnum,
                                );
                            }
                        }
                    } else {
                        // Normal char: go one level deeper.
                        word[depth as usize] = c as u8;
                        depth += 1;
                        arridx[depth as usize] = *idxs.add(n);
                        curi[depth as usize] = 1;

                        // Check if this character matches the pattern.
                        // If not, skip the whole tree below it.
                        if depth <= patlen
                            && mb_strnicmp(word.as_ptr().cast::<c_char>(), pat, depth as usize) != 0
                        {
                            depth -= 1;
                        }
                    }
                }
            }
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spell_direction() {
        assert!(SpellMoveDirection::Forward.is_forward());
        assert!(SpellMoveDirection::Backward.is_backward());
        assert_eq!(
            SpellMoveDirection::from_vim_direction(1),
            SpellMoveDirection::Forward
        );
        assert_eq!(
            SpellMoveDirection::from_vim_direction(-1),
            SpellMoveDirection::Backward
        );
        assert_eq!(SpellMoveDirection::Forward.to_vim_direction(), 1);
        assert_eq!(SpellMoveDirection::Backward.to_vim_direction(), -1);
    }

    #[test]
    fn test_spell_move_behavior() {
        assert_eq!(
            SpellMoveBehavior::from_c_int(0),
            Some(SpellMoveBehavior::All)
        );
        assert_eq!(
            SpellMoveBehavior::from_c_int(1),
            Some(SpellMoveBehavior::Bad)
        );
        assert_eq!(
            SpellMoveBehavior::from_c_int(2),
            Some(SpellMoveBehavior::Rare)
        );
        assert_eq!(SpellMoveBehavior::from_c_int(99), None);
    }

    #[test]
    fn test_spell_word_command() {
        assert!(SpellWordCommand::AddGood.is_add());
        assert!(SpellWordCommand::AddGood.is_good());
        assert!(!SpellWordCommand::AddGood.is_undo());
        assert!(!SpellWordCommand::AddGood.is_internal());

        assert!(SpellWordCommand::UndoGood.is_undo());
        assert!(SpellWordCommand::UndoGood.is_good());
        assert!(!SpellWordCommand::UndoGood.is_add());

        assert!(SpellWordCommand::AddGoodInternal.is_internal());
        assert!(SpellWordCommand::AddWrongInternal.is_internal());

        assert!(SpellWordCommand::AddWrong.is_wrong());
        assert!(SpellWordCommand::UndoWrong.is_wrong());
    }

    #[test]
    fn test_spell_dump_options() {
        let opts = SpellDumpOptions::new();
        assert!(!opts.has_region_filter());
        assert!(opts.include_rare);
        assert!(opts.include_banned);

        let opts = opts.with_region(b"us").with_counts();
        assert!(opts.has_region_filter());
        assert!(opts.include_counts);
        assert_eq!(opts.region[0], b'u');
        assert_eq!(opts.region[1], b's');
    }

    #[test]
    fn test_spell_info_flags() {
        let all = SpellInfoFlags::all();
        assert!(all.show_path);
        assert!(all.show_counts);
        assert!(all.show_regions);
        assert!(all.show_compound);

        let minimal = SpellInfoFlags::minimal();
        assert!(minimal.show_path);
        assert!(!minimal.show_counts);
    }

    #[test]
    fn test_spell_repall_state() {
        let mut state = SpellRepallState::new();
        assert_eq!(state.count, 0);
        assert!(!state.had_error);

        state.record_replacement(10);
        assert_eq!(state.count, 1);
        assert_eq!(state.last_lnum, 10);

        state.record_replacement(20);
        assert_eq!(state.count, 2);
        assert_eq!(state.last_lnum, 20);

        state.record_error();
        assert!(state.had_error);
    }
}
