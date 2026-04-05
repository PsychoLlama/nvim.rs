//! Key mapping expansion
//!
//! This module provides Rust implementations for key mapping state
//! and related functions.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::too_many_lines,
    clippy::items_after_statements,
    dead_code, // handle_mapping is used in Phase 2
    unused_assignments // mp_match initial assignment is a pattern from C translation
)]

use std::ffi::{c_char, c_int, c_void};

use nvim_mapping::MapblockHandle;

/// Result of mapping lookup in `handle_mapping`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapResult {
    /// Failed, break loop
    Fail = 0,
    /// Get a character from typeahead
    Get = 1,
    /// Try to map again
    Retry = 2,
    /// No matching mapping, get char
    NoMatch = 3,
}

impl From<c_int> for MapResult {
    fn from(value: c_int) -> Self {
        match value {
            0 => Self::Fail,
            1 => Self::Get,
            2 => Self::Retry,
            _ => Self::NoMatch,
        }
    }
}

/// Key length constants
pub mod keylen {
    use std::ffi::c_int;

    /// Need more characters to match mapping
    pub const PART_KEY: c_int = -1;

    /// Part of matching mapping found
    pub const PART_MAP: c_int = -2;
}

/// Mapping timeout state
#[derive(Debug, Clone, Copy, Default)]
pub struct MappingTimeout {
    /// Waited for more than 'timeoutlen' for mapping to complete
    pub mapping_timedout: bool,
    /// Waited for more than 'ttimeoutlen' for key code
    pub keycode_timedout: bool,
}

impl MappingTimeout {
    /// Create a new timeout state
    #[must_use]
    pub const fn new() -> Self {
        Self {
            mapping_timedout: false,
            keycode_timedout: false,
        }
    }

    /// Check if either timeout has occurred
    #[must_use]
    pub const fn is_timedout(&self) -> bool {
        self.mapping_timedout || self.keycode_timedout
    }

    /// Reset both timeout flags
    pub const fn reset(&mut self) {
        self.mapping_timedout = false;
        self.keycode_timedout = false;
    }
}

/// Mapping depth counter for recursive mapping detection
#[derive(Debug, Clone, Copy, Default)]
pub struct MappingDepth {
    /// Current recursion depth
    depth: c_int,
}

impl MappingDepth {
    /// Maximum allowed mapping depth
    pub const MAX_DEPTH: c_int = 1000;

    /// Create a new depth counter
    #[must_use]
    pub const fn new() -> Self {
        Self { depth: 0 }
    }

    /// Increment depth, returns true if exceeded max
    pub const fn increment(&mut self) -> bool {
        self.depth += 1;
        self.depth > Self::MAX_DEPTH
    }

    /// Decrement depth
    pub const fn decrement(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }
    }

    /// Get current depth
    #[must_use]
    pub const fn get(&self) -> c_int {
        self.depth
    }

    /// Reset to zero
    pub const fn reset(&mut self) {
        self.depth = 0;
    }
}

// =============================================================================
// C FFI Accessor Functions
// =============================================================================

extern "C" {
    /// no_mapping: currently no mapping allowed
    static mut no_mapping: c_int;
    /// allow_keys: allow key codes when no_mapping is set
    static mut allow_keys: c_int;
    /// KeyNoremap: remapping flags (non-static in C after Phase 3)
    static KeyNoremap: c_int;
    /// Set the KeyNoremap global variable
    fn nvim_set_keynoremap(val: c_int);
    /// KeyTyped: true if user typed current char
    static mut KeyTyped: bool;
    /// KeyStuffed: true if current char from stuffbuf
    static mut KeyStuffed: c_int;
    /// vgetc_busy: counter for vgetc recursion
    static mut vgetc_busy: c_int;
    /// ex_normal_busy: recursiveness of ex_normal()
    static mut ex_normal_busy: c_int;
    /// maptick: tick for each non-mapped char
    static mut maptick: c_int;
}

/// Check if key mapping is disabled.
///
/// # Safety
/// Reads C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_no_mapping() -> c_int {
    no_mapping
}

/// Set the no_mapping flag.
///
/// # Safety
/// Writes C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_set_no_mapping(val: c_int) {
    no_mapping = val;
}

/// Check if special keys are allowed.
///
/// # Safety
/// Reads C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_allow_keys() -> c_int {
    allow_keys
}

/// Get the current key noremap value.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_get_keynoremap() -> c_int {
    KeyNoremap
}

/// Set the key noremap value.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_keynoremap(val: c_int) {
    nvim_set_keynoremap(val);
}

/// Check if the key was typed by user.
///
/// # Safety
/// Reads C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_key_typed() -> c_int {
    c_int::from(KeyTyped)
}

/// Set the key typed flag.
///
/// # Safety
/// Writes C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_set_key_typed(val: c_int) {
    KeyTyped = val != 0;
}

/// Check if the key was stuffed (from mapping or script).
///
/// # Safety
/// Reads C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_key_stuffed() -> c_int {
    KeyStuffed
}

/// Set the key stuffed flag.
///
/// # Safety
/// Writes C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_set_key_stuffed(val: c_int) {
    KeyStuffed = val;
}

/// Check if we are busy getting a character (in vgetc).
///
/// # Safety
/// Reads C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_vgetc_busy() -> c_int {
    vgetc_busy
}

/// Increment the vgetc busy counter.
///
/// # Safety
/// Writes C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_inc_vgetc_busy() {
    vgetc_busy += 1;
}

/// Decrement the vgetc busy counter.
///
/// # Safety
/// Writes C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_dec_vgetc_busy() {
    if vgetc_busy > 0 {
        vgetc_busy -= 1;
    }
}

/// Check if :normal command is being executed.
///
/// # Safety
/// Reads C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_normal_busy() -> c_int {
    ex_normal_busy
}

/// Get the mapping tick counter.
///
/// # Safety
/// Reads C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_get_maptick() -> c_int {
    maptick
}

/// Increment the mapping tick counter.
///
/// # Safety
/// Writes C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_inc_maptick() {
    maptick += 1;
}

/// Check if in recursive vgetc call (not safe to get user input).
///
/// # Safety
/// Reads C globals directly.
#[no_mangle]
pub unsafe extern "C" fn rs_vgetc_recursive() -> c_int {
    c_int::from(vgetc_busy > 0 && ex_normal_busy == 0)
}

// =============================================================================
// C FFI for handle_mapping
// =============================================================================

extern "C" {
    /// typebuf: the typeahead buffer (direct C global access)
    static mut typebuf: crate::typebuf::TypebufT;

    /// utf8len_tab: UTF-8 byte-length lookup table
    static utf8len_tab: [u8; 256];

    /// State: current editor state
    static State: c_int;

    /// p_mmd: 'maxmapdepth' option
    static p_mmd: i64;

    /// p_paste: 'paste' option
    static p_paste: c_int;

    /// no_zero_mapping: mapping for "0" disabled
    static mut no_zero_mapping: c_int;

    /// VIsual_active: is Visual mode active
    static mut VIsual_active: bool;

    /// VIsual_select: is Select mode active
    static mut VIsual_select: bool;

    /// cmd_silent: don't echo the command line
    static mut cmd_silent: bool;

    /// did_emsg: incremented by emsg()
    static mut did_emsg: c_int;

    /// msg_row: current row for messages
    static mut msg_row: c_int;

    /// cmdline_row: row where the command line starts
    static cmdline_row: c_int;

    /// msg_didout: msg_outstr() was used in line
    static mut msg_didout: bool;

    /// may_garbage_collect: set after garbagecollect() is called
    static may_garbage_collect: bool;

    /// eval_map_expr: evaluate <expr> mapping
    fn eval_map_expr(mp: MapblockHandle, c: c_int) -> *mut c_char;

    /// get_real_state: get the current mode
    fn get_real_state() -> c_int;

    /// nvim_langmap_adjust: apply 'langmap' to a char
    fn nvim_langmap_adjust(c: c_int, condition: bool) -> c_int;

    /// mb_unescape: unescape a key sequence
    fn mb_unescape(pp: *mut *const c_char) -> *const c_char;

    /// utfc_ptr2len: get byte length of a UTF-8 character
    fn utfc_ptr2len(p: *const c_char) -> c_int;

    /// merge_modifiers: merge modifier bits into key
    fn merge_modifiers(c: c_int, modifiers: *mut c_int) -> c_int;

    /// redrawcmdline: redraw the command line
    fn redrawcmdline();

    /// redrawcmd: redraw (part of) the command line
    fn redrawcmd();

    /// setcursor: set cursor position
    fn setcursor();

    /// emsg: display an error message
    fn emsg(s: *const c_char);

    /// gettext: translate a message string
    fn gettext(msgid: *const c_char) -> *const c_char;

    /// xmemdupz: duplicate memory, zero-terminated
    fn xmemdupz(p: *const c_void, len: usize) -> *mut c_void;

    /// xfree: free memory
    fn xfree(ptr: *mut c_void);

    /// get_maphash_list: get global mapping list for state/char
    fn get_maphash_list(state: c_int, c: c_int) -> MapblockHandle;

    /// get_buf_maphash_list: get buffer-local mapping list for state/char
    fn get_buf_maphash_list(state: c_int, c: c_int) -> MapblockHandle;
}

// Keycode constants for handle_mapping
const K_SPECIAL: u8 = 0x80;
const KS_EXTRA: u8 = 253;
const KE_PLUG: u8 = 83;
const KE_SNR: u8 = 82;
const KE_IGNORE: u8 = 53;
const KS_MODIFIER: u8 = 252;

// Remap flags (must match C RM_* enum)
const RM_NONE: u8 = 1;
const RM_ABBR: u8 = 4;
const RM_SCRIPT: u8 = 2;

// Noremap values (must match C REMAP_* enum)
const REMAP_YES: c_int = 0;
const REMAP_NONE: c_int = -1;
const REMAP_SKIP: c_int = -3;

// KEYLEN constants
const KEYLEN_PART_KEY: c_int = -1;
const KEYLEN_PART_MAP: c_int = -2;

// Mode flags (must match state_defs.h)
const MODE_NORMAL: c_int = 0x01;
const MODE_VISUAL: c_int = 0x02;
const MODE_CMDLINE: c_int = 0x08;
const MODE_INSERT: c_int = 0x10;
const MODE_LANGMAP: c_int = 0x20;
const MODE_SELECT: c_int = 0x40;
const MODE_ASKMORE: c_int = 0x3000;
const MODE_HITRETURN: c_int = 0x2000 | MODE_NORMAL; // = 0x2001

// FAIL = -1 (matches C FAIL macro)
const FAIL: c_int = 0;

// FLUSH_MINIMAL
const FLUSH_MINIMAL: c_int = 0;

// K_SELECT_STRING = "\x80\xf5X" (0x80 = K_SPECIAL, 0xf5 = 245, 'X')
const K_SELECT_STRING: &[u8] = b"\x80\xf5X";

// MapResult values for return (must match C map_result_T)
const MAP_RESULT_FAIL: c_int = 0;
const MAP_RESULT_GET: c_int = 1;
const MAP_RESULT_RETRY: c_int = 2;
const MAP_RESULT_NOMATCH: c_int = 3;

/// Get the byte length for a UTF-8 first byte.
///
/// # Safety
/// Reads from `utf8len_tab` C global.
unsafe fn mb_byte2len(b: u8) -> c_int {
    c_int::from(utf8len_tab[b as usize])
}

/// Handle mappings in the typeahead buffer.
///
/// Returns:
/// - `MAP_RESULT_FAIL` (0): failed, break loop
/// - `MAP_RESULT_GET` (1): get a character from typeahead
/// - `MAP_RESULT_RETRY` (2): try mapping again
/// - `MAP_RESULT_NOMATCH` (3): no matching mapping
///
/// # Safety
/// Accesses C globals directly. Must be called from the main Neovim thread.
#[no_mangle]
pub unsafe extern "C" fn rs_handle_mapping(
    keylenp: *mut c_int,
    timedout: *const bool,
    mapdepth: *mut c_int,
) -> c_int {
    handle_mapping_impl(keylenp, timedout, mapdepth)
}

/// pub(crate) version for direct Rust use in vgetorpeek (Phase 2)
pub(crate) unsafe fn handle_mapping(
    keylenp: *mut c_int,
    timedout: *const bool,
    mapdepth: *mut c_int,
) -> c_int {
    handle_mapping_impl(keylenp, timedout, mapdepth)
}

unsafe fn handle_mapping_impl(
    keylenp: *mut c_int,
    timedout: *const bool,
    mapdepth: *mut c_int,
) -> c_int {
    let tb = &raw mut typebuf;
    let tb_len = (*tb).tb_len;
    let tb_off = (*tb).tb_off;
    let tb_buf = (*tb).tb_buf;
    let tb_noremap = (*tb).tb_noremap;
    let tb_maplen = (*tb).tb_maplen;

    let mut mp: MapblockHandle = std::ptr::null_mut();
    let mut mp2: MapblockHandle;
    let mut mp_match: MapblockHandle = std::ptr::null_mut();
    let mut mp_match_len: c_int = 0;
    let mut max_mlen: c_int = 0;
    let mut keylen = *keylenp;
    let local_state = get_real_state();

    // If typeahead starts with <Plug> then remap, even for a "noremap" mapping.
    let is_plug_map = tb_len >= 3
        && *tb_buf.add(tb_off as usize) == K_SPECIAL
        && *tb_buf.add(tb_off as usize + 1) == KS_EXTRA
        && *tb_buf.add(tb_off as usize + 2) == KE_PLUG;

    let tb_c1 = c_int::from(*tb_buf.add(tb_off as usize));
    let mut tb_c1_mapped = tb_c1; // possibly langmap-adjusted

    if no_mapping == 0
        && (no_zero_mapping == 0 || tb_c1 != c_int::from(b'0'))
        && (tb_maplen == 0
            || is_plug_map
            || (*tb_noremap.add(tb_off as usize) & (RM_NONE | RM_ABBR)) == 0)
        && !(p_paste != 0 && (State & (MODE_INSERT | MODE_CMDLINE)) != 0)
        && !(State == MODE_HITRETURN && (tb_c1 == c_int::from(b'\r') || tb_c1 == c_int::from(b' ')))
        && State != MODE_ASKMORE
        && !crate::typebuf::at_ins_compl_key_export()
    {
        let nolmaplen: c_int = if tb_c1 == c_int::from(K_SPECIAL) {
            2
        } else {
            let cond =
                (State & (MODE_CMDLINE | MODE_INSERT)) == 0 && get_real_state() != MODE_SELECT;
            tb_c1_mapped = nvim_langmap_adjust(tb_c1, cond);
            0
        };

        // First try buffer-local mappings.
        mp = get_buf_maphash_list(local_state, tb_c1_mapped);
        mp2 = get_maphash_list(local_state, tb_c1_mapped);
        if mp.is_null() {
            mp = mp2;
            mp2 = std::ptr::null_mut();
        }

        mp_match = std::ptr::null_mut();
        mp_match_len = 0;

        // Walk all (local) mappings for this hash bucket.
        loop {
            if mp.is_null() {
                break;
            }

            // Check first byte and mode.
            let mp_keys_0 = *(*mp).m_keys as u8;
            let mp_mode = (*mp).m_mode;

            if mp_keys_0 == tb_c1_mapped as u8
                && (mp_mode & local_state) != 0
                && ((mp_mode & MODE_LANGMAP) == 0 || tb_maplen == 0)
            {
                let mut nomap = nolmaplen;
                let mut modifiers: c_int = 0;
                let mut mlen: c_int = 1;

                // find the match length of this mapping
                while mlen < tb_len {
                    let mut c2 = c_int::from(*tb_buf.add(tb_off as usize + mlen as usize));
                    if nomap > 0 {
                        if nomap == 2 && c2 == c_int::from(KS_MODIFIER) {
                            modifiers = 1;
                        } else if nomap == 1 && modifiers == 1 {
                            modifiers = c2;
                        }
                        nomap -= 1;
                    } else {
                        if c2 == c_int::from(K_SPECIAL) {
                            nomap = 2;
                        } else if merge_modifiers(c2, std::ptr::addr_of_mut!(modifiers)) == c2 {
                            // Only apply 'langmap' if merging modifiers into the key
                            // won't result in a different character.
                            c2 = nvim_langmap_adjust(c2, true);
                        }
                        modifiers = 0;
                    }
                    if *(*mp).m_keys.add(mlen as usize) as u8 != c2 as u8 {
                        break;
                    }
                    mlen += 1;
                }

                // Don't allow mapping the first byte(s) of a multi-byte char.
                let p1: *const c_char = (*mp).m_keys;
                let mut p1_ptr: *const c_char = p1;
                let p2 = mb_unescape(std::ptr::addr_of_mut!(p1_ptr));

                if !p2.is_null() && mb_byte2len(tb_c1_mapped as u8) > utfc_ptr2len(p2) {
                    mlen = 0;
                }

                // Check entry for full/partial match.
                keylen = (*mp).m_keylen;
                if mlen == keylen || (mlen == tb_len && tb_len < keylen) {
                    // Check for script-local restriction.
                    let s: *const u8 = tb_noremap.add(tb_off as usize);
                    if *s == RM_SCRIPT
                        && !(*(*mp).m_keys as u8 == K_SPECIAL
                            && *(*mp).m_keys.add(1) as u8 == KS_EXTRA
                            && *(*mp).m_keys.add(2) as u8 == KE_SNR)
                    {
                        // skip to next mapping
                    } else {
                        // If one of the typed keys cannot be remapped, skip.
                        let mut s2: *const u8 = s;
                        let mut n = mlen;
                        let mut skip = false;
                        loop {
                            n -= 1;
                            if n < 0 {
                                break;
                            }
                            if *s2 & (RM_NONE | RM_ABBR) != 0 {
                                skip = true;
                                break;
                            }
                            s2 = s2.add(1);
                        }
                        if !is_plug_map && skip {
                            // skip this entry
                        } else if keylen > tb_len {
                            let has_nowait_match = mp_match_len > 0
                                && !mp_match.is_null()
                                && (*mp_match).m_nowait != 0;
                            if !*timedout && !has_nowait_match {
                                // break at a partly match
                                keylen = KEYLEN_PART_MAP;
                                mp = std::ptr::null_mut(); // sentinel to break outer loop
                                break;
                            }
                        } else if keylen > mp_match_len
                            || (keylen == mp_match_len
                                && !mp_match.is_null()
                                && ((*mp_match).m_mode & MODE_LANGMAP) == 0
                                && ((*mp).m_mode & MODE_LANGMAP) != 0)
                        {
                            // found a longer match
                            mp_match = mp;
                            mp_match_len = keylen;
                        }
                    }
                } else {
                    // No match; may have to check for termcode at next character.
                    if mlen > max_mlen {
                        max_mlen = mlen;
                    }
                }
            }

            // Advance to next entry (may fall through to global list)
            if (*mp).m_next.is_null() && !mp2.is_null() {
                mp = mp2;
                mp2 = std::ptr::null_mut();
            } else {
                mp = (*mp).m_next;
            }
        }

        // If no partly match found, use the longest full match.
        if keylen != KEYLEN_PART_MAP && !mp_match.is_null() {
            mp = mp_match;
            keylen = mp_match_len;
        }
    }

    if (mp.is_null() || max_mlen > mp_match_len) && keylen != KEYLEN_PART_MAP {
        // When no matching mapping found or found a non-matching mapping that
        // matches at least what the matching mapping matched:
        // Try to include the modifier into the key when mapping is allowed.
        if no_mapping == 0 || allow_keys != 0 {
            let tb_off_usize = tb_off as usize;
            if tb_c1 == c_int::from(K_SPECIAL)
                && (tb_len < 2 || (*tb_buf.add(tb_off_usize + 1) == KS_MODIFIER && tb_len < 4))
            {
                // Incomplete modifier sequence: cannot decide yet.
                keylen = KEYLEN_PART_KEY;
            } else {
                // Try to include the modifier into the key.
                keylen = crate::typebuf::rs_check_simplify_modifier(max_mlen + 1);
                if keylen < 0 {
                    // ins_typebuf() failed
                    *keylenp = keylen;
                    return MAP_RESULT_FAIL;
                }
            }
        } else {
            keylen = 0;
        }

        if keylen == 0 {
            // If there was no mapping at all use the character from the
            // typeahead buffer right here.
            if mp.is_null() {
                *keylenp = keylen;
                return MAP_RESULT_GET;
            }
        }

        if keylen > 0 {
            // keys have been simplified
            *keylenp = keylen;
            return MAP_RESULT_RETRY;
        }

        if keylen < 0 {
            // Incomplete key sequence.
            assert!(keylen == KEYLEN_PART_KEY);
        } else {
            assert!(!mp.is_null());
            // When a matching mapping was found use that one.
            keylen = mp_match_len;
        }
    }

    // complete match
    if keylen >= 0 && keylen <= tb_len {
        // Write chars to script file(s).
        // Note: :lmap mappings are written *after* being applied. #5658
        if keylen > tb_maplen && ((*mp).m_mode & MODE_LANGMAP) == 0 {
            crate::macro_recording::rs_gotchars(
                (*tb)
                    .tb_buf
                    .add((*tb).tb_off as usize + (*tb).tb_maplen as usize),
                (keylen - (*tb).tb_maplen) as usize,
            );
        }

        cmd_silent = (*tb).tb_silent > 0;
        crate::typebuf::rs_del_typebuf(keylen, 0); // remove the mapped keys

        // Check mapping depth.
        *mapdepth += 1;
        if *mapdepth >= p_mmd as c_int {
            emsg(gettext(c"E223: Recursive mapping".as_ptr()));
            if State & MODE_CMDLINE != 0 {
                redrawcmdline();
            } else {
                setcursor();
            }
            crate::typebuf::flush_buffers_export(FLUSH_MINIMAL);
            *mapdepth = 0;
            *keylenp = keylen;
            return MAP_RESULT_FAIL;
        }

        // In Select mode and a Visual mode mapping is used: Switch to Visual
        // mode temporarily. Append K_SELECT to switch back to Select mode.
        if VIsual_active && VIsual_select && ((*mp).m_mode & MODE_VISUAL) != 0 {
            VIsual_select = false;
            crate::typebuf::rs_ins_typebuf(K_SELECT_STRING.as_ptr(), REMAP_NONE, 0, 1, 0);
        }

        // Copy the values from *mp that are used, because evaluating the
        // expression may invoke a function that redefines the mapping, thereby
        // making *mp invalid.
        let save_m_expr = (*mp).m_expr != 0;
        let save_m_noremap = (*mp).m_noremap;
        let save_m_silent = (*mp).m_silent != 0;
        let mut save_m_keys: *mut c_char = std::ptr::null_mut();
        let mut save_alt_m_keys: *mut c_char = std::ptr::null_mut();
        let save_alt_m_keylen: c_int = if (*mp).m_alt.is_null() {
            0
        } else {
            (*(*mp).m_alt).m_keylen
        };

        let map_str: *mut c_char = if save_m_expr {
            let save_vgetc_busy = vgetc_busy;
            // may_garbage_collect write via raw pointer (it's an extern const bool in C)
            let mgc_ptr = (&raw const may_garbage_collect).cast_mut();
            let save_may_garbage_collect = *mgc_ptr;
            let prev_did_emsg = did_emsg;

            vgetc_busy = 0;
            *mgc_ptr = false;

            save_m_keys =
                xmemdupz((*mp).m_keys.cast::<c_void>(), (*mp).m_keylen as usize).cast::<c_char>();
            save_alt_m_keys = if (*mp).m_alt.is_null() {
                std::ptr::null_mut()
            } else {
                xmemdupz(
                    (*(*mp).m_alt).m_keys.cast::<c_void>(),
                    save_alt_m_keylen as usize,
                )
                .cast::<c_char>()
            };
            let eval_str = eval_map_expr(mp, 0 /* NUL */);

            let result = if eval_str.is_null() || *eval_str == 0 {
                // Error or empty expression
                if prev_did_emsg == did_emsg {
                    if State & (MODE_NORMAL | MODE_INSERT) != 0 {
                        setcursor();
                    }
                    eval_str
                } else {
                    // Generate a <Nop> to allow for a redraw.
                    xfree(eval_str.cast::<c_void>());
                    let nop_buf = [K_SPECIAL, KS_EXTRA, KE_IGNORE];
                    let dup = xmemdupz(nop_buf.as_ptr().cast::<c_void>(), 3).cast::<c_char>();
                    if State & MODE_CMDLINE != 0 {
                        msg_didout = true;
                        if msg_row < cmdline_row {
                            msg_row = cmdline_row;
                        }
                        redrawcmd();
                    } else if State & (MODE_NORMAL | MODE_INSERT) != 0 {
                        setcursor();
                    }
                    dup
                }
            } else {
                eval_str
            };

            vgetc_busy = save_vgetc_busy;
            *mgc_ptr = save_may_garbage_collect;

            result
        } else {
            (*mp).m_str
        };

        // Insert the 'to' part in the typebuf.
        let ins_result: c_int = if map_str.is_null() {
            FAIL
        } else {
            // If this is a LANGMAP mapping, record the replacement now.
            if keylen > (*tb).tb_maplen && ((*mp).m_mode & MODE_LANGMAP) != 0 {
                crate::macro_recording::rs_gotchars(map_str.cast::<u8>(), libc::strlen(map_str));
            }

            let noremap: c_int = if save_m_noremap != REMAP_YES {
                save_m_noremap
            } else if save_m_expr {
                // check if map_str matches save_m_keys or save_alt_m_keys
                let keys_match = !save_m_keys.is_null()
                    && libc::strncmp(map_str, save_m_keys, keylen as usize) == 0;
                let alt_match = !save_alt_m_keys.is_null()
                    && libc::strncmp(map_str, save_alt_m_keys, save_alt_m_keylen as usize) == 0;
                if keys_match || alt_match {
                    REMAP_SKIP
                } else {
                    REMAP_YES
                }
            } else {
                // check if map_str matches mp->m_keys or mp->m_alt->m_keys
                let keys_match = libc::strncmp(map_str, (*mp).m_keys, keylen as usize) == 0;
                let alt_match = !(*mp).m_alt.is_null()
                    && libc::strncmp(
                        map_str,
                        (*(*mp).m_alt).m_keys,
                        (*(*mp).m_alt).m_keylen as usize,
                    ) == 0;
                if keys_match || alt_match {
                    REMAP_SKIP
                } else {
                    REMAP_YES
                }
            };

            let result = crate::typebuf::rs_ins_typebuf(
                map_str.cast::<u8>(),
                noremap,
                0,
                1,
                c_int::from(cmd_silent || save_m_silent),
            );
            if save_m_expr {
                xfree(map_str.cast::<c_void>());
            }
            result
        };

        xfree(save_m_keys.cast::<c_void>());
        xfree(save_alt_m_keys.cast::<c_void>());
        *keylenp = keylen;
        if ins_result == FAIL {
            return MAP_RESULT_FAIL;
        }
        return MAP_RESULT_RETRY;
    }

    *keylenp = keylen;
    MAP_RESULT_NOMATCH
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_result_from() {
        assert_eq!(MapResult::from(0), MapResult::Fail);
        assert_eq!(MapResult::from(1), MapResult::Get);
        assert_eq!(MapResult::from(2), MapResult::Retry);
        assert_eq!(MapResult::from(3), MapResult::NoMatch);
        assert_eq!(MapResult::from(99), MapResult::NoMatch);
    }

    #[test]
    fn test_mapping_timeout() {
        let mut timeout = MappingTimeout::new();
        assert!(!timeout.is_timedout());

        timeout.mapping_timedout = true;
        assert!(timeout.is_timedout());

        timeout.reset();
        assert!(!timeout.is_timedout());
    }

    #[test]
    fn test_mapping_depth() {
        let mut depth = MappingDepth::new();
        assert_eq!(depth.get(), 0);

        for _ in 0..1000 {
            assert!(!depth.increment());
        }
        assert!(depth.increment()); // 1001 exceeds max

        depth.reset();
        assert_eq!(depth.get(), 0);
    }
}
