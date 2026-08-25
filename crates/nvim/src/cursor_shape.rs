//! Cursor and mouse-pointer shapes, one entry per editor mode.
//!
//! `'guicursor'` is a comma-separated list of `mode-list:attribute-list`
//! parts. [`parse_shape_opt`] walks the whole value **twice** — once to
//! reject it without touching anything, once to apply it — and the eighteen
//! entries it fills are what every consumer reads afterwards: the
//! `mode_info_set` UI event ([`mode_style_array`]), the TUI's DECSCUSR
//! codes, a terminal buffer's initial cursor, and the redraw's decision to
//! leave the cursor cell uninverted inside a Visual selection
//! ([`cursor_is_block_during_visual`]).
//!
//! The table itself is private. [`shape_entry`] copies one entry out and
//! [`update_shape_entry`] changes one in place; neither hands out a pointer
//! into it, and both are deliberately momentary. That is not tidiness — a
//! `syn_check_group` or `syn_id2attr` call can come back through
//! [`cursor_mode_uses_syn_id`], so a borrow of the table may never span a
//! call out of this module.
//!
//! The same parser serves `'mouseshape'`, which nvim does not implement:
//! `what` is `SHAPE_CURSOR` at every call site today, but the `used_for`
//! filter and the `mouse_shape` key are upstream's and stay.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use core::ptr;

use crate::api::private::helpers::{arena_array, arena_dict, array_add, cstr_as_string, dict_put};
use crate::charset::getdigits_int;
use crate::ex_getln::{cmdline_at_end, cmdline_overstrike};
use crate::global_cell::GlobalCell;
use crate::highlight_group::{syn_check_group, syn_id2attr};
use crate::main::{State, finish_op, p_guicursor, p_sel};
use crate::normal::visual_active;
use crate::state::{
    MODE_CMDLINE, MODE_INSERT, MODE_SHOWMATCH, MODE_TERMINAL, REPLACE_FLAG, VREPLACE_FLAG,
};
use crate::types::builders::static_cstring;
use crate::types::{Arena, Array, CursorShape, Object, cursorentry_T, size_t};
use crate::ui::ui_mode_info_set;

/// Where a mode's cursor shape sits in the shape table.
pub(crate) type ShapeIdx = c_int;

pub(crate) const SHAPE_IDX_N: ShapeIdx = 0;
pub(crate) const SHAPE_IDX_V: ShapeIdx = 1;
pub(crate) const SHAPE_IDX_I: ShapeIdx = 2;
pub(crate) const SHAPE_IDX_R: ShapeIdx = 3;
pub(crate) const SHAPE_IDX_C: ShapeIdx = 4;
pub(crate) const SHAPE_IDX_CI: ShapeIdx = 5;
pub(crate) const SHAPE_IDX_CR: ShapeIdx = 6;
pub(crate) const SHAPE_IDX_O: ShapeIdx = 7;
pub(crate) const SHAPE_IDX_VE: ShapeIdx = 8;
pub(crate) const SHAPE_IDX_SM: ShapeIdx = 16;
pub(crate) const SHAPE_IDX_TERM: ShapeIdx = 17;
pub(crate) const SHAPE_IDX_COUNT: ShapeIdx = 18;

pub(crate) const SHAPE_BLOCK: CursorShape = 0;
pub(crate) const SHAPE_HOR: CursorShape = 1;
pub(crate) const SHAPE_VER: CursorShape = 2;

/// What an entry is consulted for: `'mouseshape'`, `'guicursor'`, or both.
pub(crate) const SHAPE_MOUSE: c_int = 1;
pub(crate) const SHAPE_CURSOR: c_int = 2;

/// `N_("E548: Digit expected")`. The option layer translates what it gets
/// back, as it does the other four messages here.
static E_DIGIT_EXPECTED: &CStr = c"E548: Digit expected";

/// The table as it ships: display name, the name `'guicursor'` matches,
/// `(blinkwait, blinkon, blinkoff)`, and what the entry is used for.
///
/// Every other field starts at zero — which for `shape` is [`SHAPE_BLOCK`].
/// Order is load-bearing twice over: the `SHAPE_IDX_*` constants index it,
/// and the mode-name search below takes the *first* entry whose name agrees
/// over the length being compared, so `"c"` resolves to `cmdline_normal`
/// rather than `cmdline_insert`.
#[rustfmt::skip]
const DEFAULT_SHAPES: [(&CStr, &CStr, [c_int; 3], c_int); SHAPE_IDX_COUNT as usize] = [
    (c"normal",           c"n",  [700, 400, 250], SHAPE_CURSOR + SHAPE_MOUSE),
    (c"visual",           c"v",  [700, 400, 250], SHAPE_CURSOR + SHAPE_MOUSE),
    (c"insert",           c"i",  [700, 400, 250], SHAPE_CURSOR + SHAPE_MOUSE),
    (c"replace",          c"r",  [700, 400, 250], SHAPE_CURSOR + SHAPE_MOUSE),
    (c"cmdline_normal",   c"c",  [700, 400, 250], SHAPE_CURSOR + SHAPE_MOUSE),
    (c"cmdline_insert",   c"ci", [700, 400, 250], SHAPE_CURSOR + SHAPE_MOUSE),
    (c"cmdline_replace",  c"cr", [700, 400, 250], SHAPE_CURSOR + SHAPE_MOUSE),
    (c"operator",         c"o",  [700, 400, 250], SHAPE_CURSOR + SHAPE_MOUSE),
    (c"visual_select",    c"ve", [700, 400, 250], SHAPE_CURSOR + SHAPE_MOUSE),
    (c"cmdline_hover",    c"e",  [  0,   0,   0], SHAPE_MOUSE),
    (c"statusline_hover", c"s",  [  0,   0,   0], SHAPE_MOUSE),
    (c"statusline_drag",  c"sd", [  0,   0,   0], SHAPE_MOUSE),
    (c"vsep_hover",       c"vs", [  0,   0,   0], SHAPE_MOUSE),
    (c"vsep_drag",        c"vd", [  0,   0,   0], SHAPE_MOUSE),
    (c"more",             c"m",  [  0,   0,   0], SHAPE_MOUSE),
    (c"more_lastline",    c"ml", [  0,   0,   0], SHAPE_MOUSE),
    (c"showmatch",        c"sm", [100, 100, 100], SHAPE_CURSOR),
    (c"terminal",         c"t",  [  0,   0,   0], SHAPE_CURSOR),
];

const fn default_entry(idx: usize) -> cursorentry_T {
    let (full_name, name, blink, used_for) = DEFAULT_SHAPES[idx];
    cursorentry_T {
        full_name: full_name.as_ptr().cast_mut(),
        shape: SHAPE_BLOCK,
        mshape: 0,
        percentage: 0,
        blinkwait: blink[0],
        blinkon: blink[1],
        blinkoff: blink[2],
        id: 0,
        id_lm: 0,
        name: name.as_ptr().cast_mut(),
        used_for: used_for as c_char,
    }
}

const fn default_table() -> [cursorentry_T; SHAPE_IDX_COUNT as usize] {
    let mut table = [default_entry(0); SHAPE_IDX_COUNT as usize];
    let mut idx = 1;
    while idx < SHAPE_IDX_COUNT as usize {
        table[idx] = default_entry(idx);
        idx += 1;
    }
    table
}

/// The shapes in force, as `'guicursor'` last left them.
static SHAPE_TABLE: GlobalCell<[cursorentry_T; SHAPE_IDX_COUNT as usize]> =
    GlobalCell::new(default_table());

/// One mode's entry, copied out of the table.
pub(crate) fn shape_entry(idx: ShapeIdx) -> cursorentry_T {
    SHAPE_TABLE.with(|table| table[idx as usize])
}

/// Change one mode's entry in place.
///
/// `f` runs with the table borrowed, so it must be arithmetic and nothing
/// else: anything that calls out of this module can read the table back
/// (module docs).
pub(crate) fn update_shape_entry(idx: ShapeIdx, f: impl FnOnce(&mut cursorentry_T)) {
    SHAPE_TABLE.with_mut(|table| f(&mut table[idx as usize]));
}

/// Every entry back to block, unblinking and uncoloured — the state an
/// unmentioned mode is left in by a `'guicursor'` that names others.
///
/// Note what upstream does *not* reset: `percentage` (so `a:ver30` followed
/// by `a:block` reports a block with `cell_percentage = 30`) and `mshape`.
fn clear_shape_table() {
    SHAPE_TABLE.with_mut(|table| {
        for entry in table {
            entry.shape = SHAPE_BLOCK;
            entry.blinkwait = 0;
            entry.blinkon = 0;
            entry.blinkoff = 0;
            entry.id = 0;
            entry.id_lm = 0;
        }
    });
}

/// The whole table as the `mode_info_set` UI event carries it: one dict per
/// mode, in table order.
///
/// # Safety
/// `arena` must be null or the caller's live arena.
pub(crate) unsafe fn mode_style_array(arena: *mut Arena) -> Array {
    let mut all = arena_array(arena, SHAPE_IDX_COUNT as size_t);
    for idx in 0..SHAPE_IDX_COUNT {
        let cur = shape_entry(idx);
        let for_mouse = c_int::from(cur.used_for) & SHAPE_MOUSE != 0;
        let for_cursor = c_int::from(cur.used_for) & SHAPE_CURSOR != 0;
        // Upstream sizes for three keys plus the nine cursor ones, so a
        // cursor-only entry ("sm", "t") leaves one slot unused.
        let mut dic = arena_dict(arena, if for_cursor { 12 } else { 3 });
        let mut put = |key, value| {
            // SAFETY: `dic` was reserved above for every key added below.
            unsafe { dict_put(&mut dic, key, value) };
        };
        // SAFETY: both are static literals the table ships with; nothing
        // ever writes either field.
        let full_name = unsafe { cstr_as_string(cur.full_name) };
        // SAFETY: as above.
        let short_name = unsafe { cstr_as_string(cur.name) };
        put(c"name", Object::string(full_name));
        put(c"short_name", Object::string(short_name));
        if for_mouse {
            put(c"mouse_shape", Object::integer(cur.mshape.into()));
        }
        if for_cursor {
            let shape = match cur.shape {
                SHAPE_BLOCK => c"block",
                SHAPE_VER => c"vertical",
                SHAPE_HOR => c"horizontal",
                _ => c"unknown",
            };
            put(c"cursor_shape", Object::string(static_cstring(shape)));
            put(c"cell_percentage", Object::integer(cur.percentage.into()));
            put(c"blinkwait", Object::integer(cur.blinkwait.into()));
            put(c"blinkon", Object::integer(cur.blinkon.into()));
            put(c"blinkoff", Object::integer(cur.blinkoff.into()));
            put(c"hl_id", Object::integer(cur.id.into()));
            put(c"id_lm", Object::integer(cur.id_lm.into()));
            // SAFETY: main-thread call. Resolving an id can run a namespace
            // callback, which is why the entry was copied out first.
            let attr = unsafe { attr_of(cur.id) };
            put(c"attr_id", Object::integer(attr.into()));
            // Upstream reads `id_lm` back through a live pointer *after*
            // that call, so a callback that rewrote 'guicursor' would be
            // seen here and not two keys above. Kept.
            let id_lm = shape_entry(idx).id_lm;
            // SAFETY: as above.
            let attr_lm = unsafe { attr_of(id_lm) };
            put(c"attr_id_lm", Object::integer(attr_lm.into()));
        }
        // SAFETY: `all` was reserved for one object per entry.
        unsafe { array_add(&mut all, Object::dict(dic)) };
    }
    all
}

/// The attribute a cursor highlight group resolves to; zero for "no group",
/// which is not a group id.
///
/// # Safety
/// Resolves namespace overrides, which can run a Lua callback.
unsafe fn attr_of(id: c_int) -> c_int {
    if id == 0 {
        return 0;
    }
    // SAFETY: caller contract.
    unsafe { syn_id2attr(id) }
}

/// `vim_strchr(s + from, c)` as an offset into `s`. The parser only looks
/// for `:`, `,`, `-` and `/`, all below 0x80, where `vim_strchr` is a plain
/// byte search.
fn find(s: &[u8], from: usize, c: u8) -> Option<usize> {
    s[from..].iter().position(|&b| b == c).map(|i| from + i)
}

/// `STRNICMP(text, word, word.len()) == 0`: a case-insensitive prefix. A
/// `text` shorter than `word` cannot match, because the comparison reaches
/// its NUL first.
fn starts_with_ci(text: &[u8], word: &[u8]) -> bool {
    text.len() >= word.len() && text[..word.len()].eq_ignore_ascii_case(word)
}

/// `STRNICMP(text, shape_table[idx].name, len) == 0`.
///
/// A name *shorter* than `len` cannot match (the comparison reaches its
/// NUL); a longer one can, which is how `"c"` finds `cmdline_normal`.
fn mode_name_matches(idx: usize, text: &[u8], len: usize) -> bool {
    let name = DEFAULT_SHAPES[idx].1.to_bytes();
    name.len() >= len && text.len() >= len && text[..len].eq_ignore_ascii_case(&name[..len])
}

/// `getdigits_int(&p, false, 0)` at offset `at`: the number, and the offset
/// just past what it consumed.
///
/// # Safety
/// `opt` must be NUL-terminated and `at` an offset inside it.
unsafe fn digits_at(opt: *mut c_char, at: usize) -> (c_int, usize) {
    // SAFETY: caller contract; `getdigits_int` advances the pointer over the
    // digits and does not write through it.
    unsafe {
        let mut p = opt.add(at);
        let n = getdigits_int(&raw mut p, false, 0);
        (n, p.offset_from(opt) as usize)
    }
}

/// Parses `'guicursor'` into the shape table, and clears it when the option
/// is empty.
///
/// Two passes: the first rejects the value as a whole without writing
/// anything, the second applies it. `what` is [`SHAPE_CURSOR`] or
/// [`SHAPE_MOUSE`] and decides which modes are legal to name.
///
/// # Safety
/// Reads the option's own value, which must be a live NUL-terminated
/// string, and defines highlight groups; main thread only.
///
/// @returns an error message for an illegal option, null otherwise.
pub(crate) unsafe fn parse_shape_opt(what: c_int) -> *const c_char {
    // Set by a `ve` in the mode list, in either round.
    let mut found_ve = false;

    for round in 1..=2 {
        let opt = p_guicursor.get();
        // SAFETY: an option value is a NUL-terminated string. Upstream
        // re-reads the global at the top of each round; so does this.
        let bytes = unsafe { CStr::from_ptr(opt) }.to_bytes();
        if round == 2 || bytes.is_empty() {
            // Everything not mentioned goes back to the default.
            clear_shape_table();
            if bytes.is_empty() {
                ui_mode_info_set();
                return ptr::null();
            }
        }
        // SAFETY: `bytes` is the string `opt` points at.
        if let Err(msg) = unsafe { parse_parts(opt, bytes, what, round == 2, &mut found_ve) } {
            // The option layer takes the message as a bare pointer; that is
            // the only reason one is handed back rather than reported here.
            return msg.as_ptr();
        }
    }

    // Without an explicit "ve", Select mode copies Visual mode — but only
    // these seven fields: the names, `mshape` and `used_for` stay.
    if !found_ve {
        let v = shape_entry(SHAPE_IDX_V);
        update_shape_entry(SHAPE_IDX_VE, |e| {
            e.shape = v.shape;
            e.percentage = v.percentage;
            e.blinkwait = v.blinkwait;
            e.blinkon = v.blinkon;
            e.blinkoff = v.blinkoff;
            e.id = v.id;
            e.id_lm = v.id_lm;
        });
    }
    ui_mode_info_set();
    ptr::null()
}

/// One pass over the option value. `apply` is upstream's `round == 2`: the
/// first round validates and writes nothing, so an error leaves the table
/// exactly as it was.
///
/// `found_ve` accumulates across both rounds, as upstream's local does.
///
/// # Safety
/// `bytes` must be the string `opt` points at; defines highlight groups.
unsafe fn parse_parts(
    opt: *mut c_char,
    bytes: &[u8],
    what: c_int,
    apply: bool,
    found_ve: &mut bool,
) -> Result<(), &'static CStr> {
    let e_illegal_mode: &CStr = c"E546: Illegal mode";
    // The mode currently being described, and the cursor into the attribute
    // list; both outlive one iteration, exactly as upstream's do.
    let mut idx = 0usize;
    let mut p = 0usize;

    // Repeat for all comma separated parts.
    let mut modep = 0usize;
    while modep < bytes.len() {
        let Some(colonp) = find(bytes, modep, b':') else {
            return Err(c"E545: Missing colon");
        };
        let commap = find(bytes, modep, b',');
        if commap.is_some_and(|comma| comma < colonp) {
            return Err(c"E545: Missing colon");
        }
        if colonp == modep {
            return Err(e_illegal_mode);
        }

        // Repeat for all modes before the colon; "a" loops over all of them,
        // counting `all_idx` down.
        let mut all_idx: c_int = -1;
        while modep < colonp || all_idx >= 0 {
            if all_idx < 0 {
                // One-letter names are the ones followed by a separator.
                let len = if matches!(bytes[modep + 1], b'-' | b':') {
                    1
                } else {
                    2
                };
                if len == 1 && bytes[modep].eq_ignore_ascii_case(&b'a') {
                    all_idx = SHAPE_IDX_COUNT - 1;
                } else {
                    idx = 0;
                    while idx < SHAPE_IDX_COUNT as usize {
                        if mode_name_matches(idx, &bytes[modep..], len) {
                            break;
                        }
                        idx += 1;
                    }
                    if idx == SHAPE_IDX_COUNT as usize
                        || c_int::from(shape_entry(idx as ShapeIdx).used_for) & what == 0
                    {
                        return Err(e_illegal_mode);
                    }
                    // The name is matched case-insensitively but this test
                    // is not, so `VE:block` still takes the fallback below.
                    if len == 2 && bytes[modep] == b'v' && bytes[modep + 1] == b'e' {
                        *found_ve = true;
                    }
                }
                // Past the name *and* its separator.
                modep += len + 1;
            }
            if all_idx >= 0 {
                // "a" bypasses the `used_for` filter above: it sets every
                // entry, including the ones this `what` does not own.
                idx = all_idx as usize;
                all_idx -= 1;
            }

            // Parse the part after the colon, once per mode named before it.
            p = colonp + 1;
            while p < bytes.len() && bytes[p] != b',' {
                // The first byte of the item, which "ver"/"hor" tell apart
                // and the highlight-group arm reuses as the langmap group.
                let mut first = c_int::from(bytes[p]);
                let rest = &bytes[p..];
                let len = if starts_with_ci(rest, b"ver") || starts_with_ci(rest, b"hor") {
                    3
                } else if starts_with_ci(rest, b"blinkwait") {
                    9
                } else if starts_with_ci(rest, b"blinkon") {
                    7
                } else if starts_with_ci(rest, b"blinkoff") {
                    8
                } else {
                    0
                };
                if len != 0 {
                    // The ones with a number argument.
                    p += len;
                    if !bytes.get(p).is_some_and(u8::is_ascii_digit) {
                        return Err(E_DIGIT_EXPECTED);
                    }
                    // SAFETY: `p` indexes inside the string `opt` points at.
                    let (n, next) = unsafe { digits_at(opt, p) };
                    p = next;
                    if len == 3 {
                        // "ver"/"hor". Note there is no upper bound: hor101
                        // and ver999 are accepted as written.
                        if n == 0 {
                            return Err(c"E549: Illegal percentage");
                        }
                        if apply {
                            let vertical = (first as u8).eq_ignore_ascii_case(&b'v');
                            update_shape_entry(idx as ShapeIdx, |e| {
                                e.shape = if vertical { SHAPE_VER } else { SHAPE_HOR };
                                e.percentage = n;
                            });
                        }
                    } else if apply {
                        update_shape_entry(idx as ShapeIdx, |e| match len {
                            9 => e.blinkwait = n,
                            7 => e.blinkon = n,
                            _ => e.blinkoff = n,
                        });
                    }
                } else if starts_with_ci(rest, b"block") {
                    if apply {
                        update_shape_entry(idx as ShapeIdx, |e| e.shape = SHAPE_BLOCK);
                    }
                    p += 5;
                } else {
                    // Must be a highlight group name then, ending at the
                    // next "-", at this part's comma, or at the end.
                    let dash = find(bytes, p, b'-');
                    let endp = match commap {
                        None => dash.unwrap_or(bytes.len()),
                        Some(comma) => match dash {
                            Some(dash) if dash <= comma => dash,
                            _ => comma,
                        },
                    };
                    let slashp = find(bytes, p, b'/').filter(|&slash| slash < endp);
                    if let Some(slash) = slashp {
                        // "group/langmap_group": the first name is the
                        // langmap one, and note it is looked up in *both*
                        // rounds, so validation defines it too.
                        let len = (slash - p) as size_t;
                        // SAFETY: `p` indexes inside `opt`'s string.
                        first = unsafe { syn_check_group(opt.add(p), len) };
                        p = slash + 1;
                    }
                    if apply {
                        let len = (endp - p) as size_t;
                        // SAFETY: as above. An empty name is upstream's too
                        // — "block-Cursor/" looks one up with length zero.
                        let id = unsafe { syn_check_group(opt.add(p), len) };
                        update_shape_entry(idx as ShapeIdx, |e| {
                            e.id = if slashp.is_some() { first } else { id };
                            e.id_lm = id;
                        });
                    }
                    p = endp;
                }
                if bytes.get(p) == Some(&b'-') {
                    p += 1;
                }
            }
        }
        modep = p;
        if bytes.get(modep) == Some(&b',') {
            modep += 1;
        }
    }
    Ok(())
}

/// Whether the cursor is a non-blinking block during a Visual selection —
/// in which case the selection highlight may cover the cursor cell, because
/// the cursor is drawn over it anyway.
///
/// `exclusive` says `'selection'` is "exclusive", which selects the `ve`
/// entry over the `v` one.
pub(crate) fn cursor_is_block_during_visual(exclusive: bool) -> bool {
    let entry = shape_entry(if exclusive { SHAPE_IDX_VE } else { SHAPE_IDX_V });
    entry.shape == SHAPE_BLOCK && entry.blinkon == 0
}

/// Whether any mode's cursor is coloured by highlight group `syn_id`, and
/// so needs the UI told when that group changes.
///
/// # Safety
/// Reads the option's own value; main thread only.
pub(crate) unsafe fn cursor_mode_uses_syn_id(syn_id: c_int) -> bool {
    // SAFETY: an option value is a NUL-terminated string.
    if unsafe { *p_guicursor.get() } == 0 {
        return false;
    }
    (0..SHAPE_IDX_COUNT).any(|idx| {
        let entry = shape_entry(idx);
        entry.id == syn_id || entry.id_lm == syn_id
    })
}

/// The entry describing the cursor for the mode the editor is in.
///
/// # Safety
/// Reads the command line and the `'selection'` option; main thread only.
pub(crate) unsafe fn cursor_get_mode_idx() -> ShapeIdx {
    let state = State.get();
    if state == MODE_SHOWMATCH {
        SHAPE_IDX_SM
    } else if state == MODE_TERMINAL {
        SHAPE_IDX_TERM
    } else if state & (VREPLACE_FLAG | REPLACE_FLAG) != 0 {
        SHAPE_IDX_R
    } else if state & MODE_INSERT != 0 {
        SHAPE_IDX_I
    } else if state & MODE_CMDLINE != 0 {
        if cmdline_at_end() {
            SHAPE_IDX_C
        } else if cmdline_overstrike() {
            SHAPE_IDX_CR
        } else {
            SHAPE_IDX_CI
        }
    } else if finish_op.get() {
        SHAPE_IDX_O
    } else if visual_active() {
        // SAFETY: an option value is a NUL-terminated string.
        if unsafe { *p_sel.get() } == b'e' as c_char {
            SHAPE_IDX_VE
        } else {
            SHAPE_IDX_V
        }
    } else {
        SHAPE_IDX_N
    }
}
