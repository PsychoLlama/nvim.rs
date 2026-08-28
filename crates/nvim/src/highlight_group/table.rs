//! The group table itself: names, ids, and the links between them.
//!
//! Every highlight group is an entry in one `Vec`, and its id is that entry's
//! index plus one. [`syn_check_group`] is the way in — it interns a name,
//! adding a group if it is new — and [`syn_name2id`]/[`syn_id2name`] are the
//! two directions of the lookup. [`syn_ns_get_final_id`] follows
//! `:highlight link` chains (and namespace overrides) to the group that
//! actually carries the attributes, which [`syn_id2attr`] then resolves.
//!
//! Upstream keeps the entries in a `garray_T` and their names in an arena
//! that is never freed, with a `Map(cstr_t, int)` from the uppercased name to
//! the id. Here that is a `Vec` plus a `HashMap`, and the two names of each
//! group are leaked `CStr`s — which is what the arena amounted to, and what
//! lets `HlGroup` stay `Copy` for the scratch entries `highlight_changed`
//! builds.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};
use std::collections::HashMap;
use std::ffi::CString;
use std::hash::{BuildHasherDefault, DefaultHasher};

use crate::ascii::ascii_isdigit;
use crate::charset::vim_isprintc;
use crate::cursor_shape::cursor_mode_uses_syn_id;
use crate::global_cell::GlobalCell;
use crate::highlight::{HLATTRS_INIT, HlAttrFlags, hl_get_syn_attr, ns_get_hl};
use crate::main::{
    curwin, e_highlight_group_name_invalid_char, e_highlight_group_name_too_long, normal_bg,
    normal_fg,
};
use crate::message::{emsg, msg_source};
use crate::os::cshim::gettext;
use crate::types::{HlAttrs, NS, RgbValue, int16_t, sctx_T, size_t};
use crate::ui::ui_mode_info_set;

use super::{HLF_W, MAX_HL_ID, MAX_SYN_NAME, SG_LINK, kColorIdxBg, kColorIdxFg, kColorIdxNone};

/// Information about one highlight group.
///
/// `Copy` on purpose: [`super::highlight_changed`] builds ten scratch entries
/// by cloning real ones, which upstream does with `memmove`.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct HlGroup {
    /// The name as written.
    pub name: &'static CStr,
    /// The same name uppercased — the key groups are looked up by.
    pub name_u: &'static CStr,
    /// `:hi clear` was used, or the entry was created by a lookup and never
    /// given settings.
    pub cleared: bool,
    /// The resolved attribute-table id. @see `syn_attr2entry`
    pub attr: c_int,
    /// The group this one links to, or 0.
    pub link: c_int,
    /// The link `:hi default link` set; [`highlight_clear`] restores it.
    pub deflink: c_int,
    /// Which of `cterm=`/`gui=`/`link` have been set: `SG_*`.
    pub set: c_int,
    /// Where the default link was set.
    pub deflink_sctx: sctx_T,
    /// Where the group was last set.
    pub script_ctx: sctx_T,
    /// `cterm=` attributes.
    pub cterm: HlAttrFlags,
    /// `ctermfg=` colour number plus one, 0 for unset.
    pub cterm_fg: c_int,
    /// `ctermbg=` colour number plus one, 0 for unset.
    pub cterm_bg: c_int,
    /// `bold` was set to reach a light colour on an 8-colour terminal, so a
    /// later `ctermfg=` has to take it away again.
    pub cterm_bold: bool,
    /// `gui=` attributes.
    pub gui: HlAttrFlags,
    /// `guifg=`, `guibg=`, `guisp=`.
    pub rgb_fg: RgbValue,
    pub rgb_bg: RgbValue,
    pub rgb_sp: RgbValue,
    /// Where each RGB colour came from: an index into
    /// [`super::COLOR_NAMES`], or one of the `kColorIdx*` values.
    pub rgb_fg_idx: c_int,
    pub rgb_bg_idx: c_int,
    pub rgb_sp_idx: c_int,
    /// `blend=`, 0..=100, or -1 for unset.
    pub blend: c_int,
    /// For `@nested.group`, the id of `@nested`.
    pub parent: c_int,
}

impl HlGroup {
    /// The all-zero entry: what upstream's `CLEAR_POINTER` leaves behind, and
    /// not the same thing as a freshly added group (whose colours start at
    /// -1). The scratch entries in `highlight_changed` are these.
    pub(crate) const ZEROED: Self = Self {
        name: c"",
        name_u: c"",
        cleared: false,
        attr: 0,
        link: 0,
        deflink: 0,
        set: 0,
        deflink_sctx: sctx_T::NONE,
        script_ctx: sctx_T::NONE,
        cterm: HlAttrFlags::NONE,
        cterm_fg: 0,
        cterm_bg: 0,
        cterm_bold: false,
        gui: HlAttrFlags::NONE,
        rgb_fg: 0,
        rgb_bg: 0,
        rgb_sp: 0,
        rgb_fg_idx: 0,
        rgb_bg_idx: 0,
        rgb_sp_idx: 0,
        blend: 0,
        parent: 0,
    };

    /// A group that exists but has no settings yet.
    fn new(name: &'static CStr, name_u: &'static CStr, parent: c_int) -> Self {
        Self {
            name,
            name_u,
            cleared: true,
            rgb_fg: -1,
            rgb_bg: -1,
            rgb_sp: -1,
            rgb_fg_idx: kColorIdxNone,
            rgb_bg_idx: kColorIdxNone,
            rgb_sp_idx: kColorIdxNone,
            blend: -1,
            parent,
            ..Self::ZEROED
        }
    }

    /// True if the group carries any highlighting of its own.
    fn has_settings(&self, check_link: bool) -> bool {
        !self.cleared
            && (self.attr != 0
                || self.cterm_fg != 0
                || self.cterm_bg != 0
                || self.rgb_fg_idx != kColorIdxNone
                || self.rgb_bg_idx != kColorIdxNone
                || self.rgb_sp_idx != kColorIdxNone
                || (check_link && self.set & SG_LINK as c_int != 0))
    }
}

/// Small keys, never iterated, so a fixed-seed hasher is enough and is
/// constructible in a `static`.
type Table<K, V> = HashMap<K, V, BuildHasherDefault<DefaultHasher>>;

/// The groups in id order, plus the reverse lookup from uppercased name.
struct GroupTable {
    entries: Vec<HlGroup>,
    ids: Table<&'static CStr, c_int>,
}

impl GroupTable {
    const fn new() -> Self {
        Self {
            entries: Vec::new(),
            ids: HashMap::with_hasher(BuildHasherDefault::new()),
        }
    }
}

static GROUPS: GlobalCell<GroupTable> = GlobalCell::new(GroupTable::new());

/// The number of highlight groups.
pub(crate) fn highlight_num_groups() -> c_int {
    GROUPS.with(|table| table.entries.len() as c_int)
}

/// The name of the group at *index* `id` — note that this one is not an id
/// minus one, which is how its two callers in syntax.rs use it.
pub(crate) fn highlight_group_name(id: c_int) -> *mut c_char {
    GROUPS.with(|table| table.entries[id as usize].name.as_ptr().cast_mut())
}

/// The id the group at *index* `id` links to, 0 for none.
pub(crate) fn highlight_link_id(id: c_int) -> c_int {
    GROUPS.with(|table| table.entries[id as usize].link)
}

/// A copy of the group with id `id`.
pub(crate) fn group(id: c_int) -> HlGroup {
    GROUPS.with(|table| table.entries[id as usize - 1])
}

/// Runs `f` on the group with id `id`.
///
/// The borrow must not outlive `f`, and `f` must not call anything that can
/// reach the table again — a `:highlight` from an autocommand, or the Lua
/// callback a namespace lookup can run.
pub(crate) fn with_group<R>(id: c_int, f: impl FnOnce(&mut HlGroup) -> R) -> R {
    GROUPS.with_mut(|table| f(&mut table.entries[id as usize - 1]))
}

/// Whether the group with id `id` has any highlighting of its own.
///
/// `check_link` also counts an explicit `:highlight link`.
pub(crate) fn hl_has_settings(id: c_int, check_link: bool) -> bool {
    GROUPS.with(|table| table.entries[id as usize - 1].has_settings(check_link))
}

/// Forgets everything the group with id `id` was given, keeping only its
/// default link — and the position that link was set from, so that
/// `:verbose hi` still names it.
pub(crate) fn highlight_clear(id: c_int) {
    with_group(id, |group| {
        let deflink = group.deflink;
        *group = HlGroup {
            cleared: true,
            link: deflink,
            script_ctx: group.deflink_sctx,
            attr: 0,
            cterm: HlAttrFlags::NONE,
            cterm_bold: false,
            cterm_fg: 0,
            cterm_bg: 0,
            gui: HlAttrFlags::NONE,
            rgb_fg: -1,
            rgb_bg: -1,
            rgb_sp: -1,
            rgb_fg_idx: kColorIdxNone,
            rgb_bg_idx: kColorIdxNone,
            rgb_sp_idx: kColorIdxNone,
            blend: -1,
            ..*group
        };
    });
}

/// Recomputes the attribute-table id of the group with id `id` after one of
/// its settings changed.
///
/// # Safety
/// Reaches the attribute table, which can rebuild itself and re-enter here;
/// main thread only.
pub(crate) unsafe fn set_hl_attr(id: c_int) {
    // The unset value for an RGB colour is -1, but a group is created with
    // zeroes, so the colour index is what says whether one was ever set.
    let at_en = GROUPS.with(|table| {
        let group = &table.entries[id as usize - 1];
        HlAttrs {
            cterm_ae_attr: group.cterm,
            cterm_fg_color: group.cterm_fg as int16_t,
            cterm_bg_color: group.cterm_bg as int16_t,
            rgb_ae_attr: group.gui,
            rgb_fg_color: if group.rgb_fg_idx != kColorIdxNone {
                group.rgb_fg
            } else {
                -1
            },
            rgb_bg_color: if group.rgb_bg_idx != kColorIdxNone {
                group.rgb_bg
            } else {
                -1
            },
            rgb_sp_color: if group.rgb_sp_idx != kColorIdxNone {
                group.rgb_sp
            } else {
                -1
            },
            hl_blend: group.blend,
            ..HLATTRS_INIT
        }
    });

    // Outside any borrow: this can rebuild the attribute table, which comes
    // back through `highlight_attr_set_all` into this very function.
    // SAFETY: the editor's own tables.
    let attr = unsafe { hl_get_syn_attr(0, id, at_en) };
    with_group(id, |group| group.attr = attr);

    // A cursor style may use this group; if so its attribute has changed.
    // SAFETY: main-thread UI call.
    if unsafe { cursor_mode_uses_syn_id(id) } {
        ui_mode_info_set();
    }
}

/// Recomputes every group's attributes, after the `Normal` colours moved:
/// `guibg=fg` and friends are stored as an index and resolved here.
///
/// # Safety
/// See [`set_hl_attr`].
pub(crate) unsafe fn highlight_attr_set_all() {
    let mut id = 1;
    // The count is re-read every round because `set_hl_attr` can add groups.
    while id <= highlight_num_groups() {
        with_group(id, |group| {
            for (idx, color) in [
                (group.rgb_bg_idx, &mut group.rgb_bg),
                (group.rgb_fg_idx, &mut group.rgb_fg),
                (group.rgb_sp_idx, &mut group.rgb_sp),
            ] {
                if idx == kColorIdxFg {
                    *color = normal_fg.get();
                } else if idx == kColorIdxBg {
                    *color = normal_bg.get();
                }
            }
        });
        // SAFETY: the editor's own tables.
        unsafe { set_hl_attr(id) };
        id += 1;
    }
}

/// Leaks `name` as a `&'static CStr`, as upstream's never-freed arena did.
fn intern(name: &[u8]) -> &'static CStr {
    let owned = CString::new(name).expect("group names carry no NUL");
    Box::leak(owned.into_boxed_c_str())
}

/// The key a group is looked up by: its name, ASCII-uppercased.
///
/// A NUL inside `name` truncates, which is what upstream's
/// `vim_memcpy_up` + `map_get(cstr_t, ...)` pair amounts to — the copy keeps
/// every byte but the hash stops at the first NUL. Such a name is rejected
/// by [`syn_add_group`] anyway, so this only affects a failed lookup.
fn upper_key(name: &[u8]) -> CString {
    let mut key = name.to_ascii_uppercase();
    key.truncate(key.iter().position(|&b| b == 0).unwrap_or(key.len()));
    CString::new(key).expect("truncated at the first NUL")
}

/// The id of the group named `name`, or 0.
///
/// A leading `@` goes through [`syn_check_group`] instead, because
/// `@aaa.bbb` has to consider `@aaa` as well.
///
/// # Safety
/// `name` is NUL-terminated; may add a group; main thread only.
pub(crate) unsafe fn syn_name2id(name: *const c_char) -> c_int {
    // SAFETY: the caller's NUL-terminated name.
    let bytes = unsafe { CStr::from_ptr(name) }.to_bytes();
    if bytes.first() == Some(&b'@') {
        // SAFETY: as above.
        return unsafe { syn_check_group(name, bytes.len() as size_t) };
    }
    lookup(bytes)
}

/// The id of the group named by the first `len` bytes of `name`, or 0.
///
/// # Safety
/// `name` points to at least `len` readable bytes; main thread only.
pub(crate) unsafe fn syn_name2id_len(name: *const c_char, len: size_t) -> c_int {
    // SAFETY: the caller's buffer, `len` bytes of it.
    lookup(unsafe { core::slice::from_raw_parts(name.cast::<u8>(), len) })
}

/// The shared body of the two lookups. An over-long name cannot have been
/// added, so it cannot be found either.
fn lookup(name: &[u8]) -> c_int {
    if name.is_empty() || name.len() > MAX_SYN_NAME as usize {
        return 0;
    }
    let key = upper_key(name);
    GROUPS.with(|table| table.ids.get(key.as_c_str()).copied().unwrap_or(0))
}

/// The attributes of the group named `name`, or 0 if there is no such group.
///
/// # Safety
/// See [`syn_name2id`].
pub(crate) unsafe fn syn_name2attr(name: *const c_char) -> c_int {
    // SAFETY: the caller's NUL-terminated name.
    match unsafe { syn_name2id(name) } {
        0 => 0,
        id => unsafe { syn_id2attr(id) },
    }
}

/// Whether a group named `name` exists.
///
/// # Safety
/// See [`syn_name2id`].
pub(crate) unsafe fn highlight_exists(name: *const c_char) -> c_int {
    // SAFETY: the caller's NUL-terminated name.
    c_int::from(unsafe { syn_name2id(name) } > 0)
}

/// The name of the group with id `id`, or `""` for an id that names none.
pub(crate) fn syn_id2name(id: c_int) -> *mut c_char {
    // The bound has to be tested BEFORE the index is formed: `id` of 0 is the
    // "no group" answer every caller may pass, and `0usize - 1` panics.
    let index = id.checked_sub(1).and_then(|i| usize::try_from(i).ok());
    GROUPS.with(|table| {
        let name = index
            .and_then(|i| table.entries.get(i))
            .map_or(c"", |group| group.name);
        name.as_ptr().cast_mut()
    })
}

/// The id of the group named by the first `len` bytes of `name`, adding it if
/// it does not exist yet. 0 on failure.
///
/// # Safety
/// `name` points to at least `len` readable bytes; may run `emsg`; main
/// thread only.
pub(crate) unsafe fn syn_check_group(name: *const c_char, len: size_t) -> c_int {
    if len > MAX_SYN_NAME as size_t {
        // SAFETY: main-thread message call.
        unsafe { emsg(gettext(e_highlight_group_name_too_long.as_ptr())) };
        return 0;
    }
    // SAFETY: the caller's buffer, `len` bytes of it.
    let bytes = unsafe { core::slice::from_raw_parts(name.cast::<u8>(), len) };
    match lookup(bytes) {
        0 => syn_add_group(bytes),
        id => id,
    }
}

/// Adds a group named `name` and answers its id, or 0 if the name is not a
/// legal one (or the table is full).
///
/// `.` and `@` are allowed because treesitter capture names use them; a
/// `@nested.group` also records `@nested` as its parent, adding that too, so
/// that a cleared child can fall back to it.
fn syn_add_group(name: &[u8]) -> c_int {
    for &byte in name {
        let c = c_int::from(byte);
        // SAFETY: main-thread message calls.
        if !unsafe { vim_isprintc(c) } {
            unsafe {
                emsg(gettext(
                    c"E669: Unprintable character in group name".as_ptr(),
                ))
            };
            return 0;
        }
        if !byte.is_ascii_alphabetic()
            && !ascii_isdigit(c)
            && !matches!(byte, b'_' | b'.' | b'@' | b'-')
        {
            unsafe { msg_source(HLF_W) };
            unsafe { emsg(gettext(e_highlight_group_name_invalid_char.as_ptr())) };
            return 0;
        }
    }

    let mut scoped_parent = 0;
    if name.len() > 1
        && name[0] == b'@'
        && let Some(at) = name.iter().rposition(|&b| b == b'.')
    {
        // Recursive, and it can add a group of its own, so it happens before
        // this one is appended.
        // SAFETY: `name` is a live slice.
        scoped_parent = unsafe { syn_check_group(name.as_ptr().cast(), at as size_t) };
    }

    if highlight_num_groups() >= MAX_HL_ID as c_int {
        // SAFETY: main-thread message call.
        unsafe {
            emsg(gettext(
                c"E849: Too many highlight and syntax groups".as_ptr(),
            ))
        };
        return 0;
    }

    let group = HlGroup::new(
        intern(name),
        intern(&name.to_ascii_uppercase()),
        scoped_parent,
    );
    GROUPS.with_mut(|table| {
        if table.entries.is_empty() {
            // 265 builtin groups, always used, plus some room.
            table.entries.reserve(300);
        }
        table.entries.push(group);
        let id = table.entries.len() as c_int;
        table.ids.insert(group.name_u, id);
        id
    })
}

/// The attribute-table id for the group with id `hl_id`, following links.
///
/// # Safety
/// Resolves namespace overrides, which can run a Lua callback; main thread
/// only.
pub(crate) unsafe fn syn_id2attr(hl_id: c_int) -> c_int {
    let mut optional = false;
    // SAFETY: the editor's own tables.
    unsafe { syn_ns_id2attr(-1, hl_id, &mut optional) }
}

/// [`syn_id2attr`] against a particular namespace.
///
/// `optional` says the caller will accept "this namespace defines nothing",
/// and is cleared if the namespace turns out to define the group as empty on
/// purpose.
///
/// # Safety
/// See [`syn_id2attr`].
pub(crate) unsafe fn syn_ns_id2attr(mut ns_id: NS, mut hl_id: c_int, optional: &mut bool) -> c_int {
    // SAFETY: the editor's own tables.
    if unsafe { syn_ns_get_final_id(&mut ns_id, &mut hl_id) } {
        // The namespace defines the group to be empty; that is not optional.
        *optional = false;
    }
    if hl_id < 1 {
        // The id named no group, and upstream read `hl_table[-1]` here and
        // answered whatever `sg_attr` that landed on. Latent: instrumenting
        // this branch and running the ui/ functional specs and the highlight
        // oldtests never reached it, so the read is unreachable in practice
        // as well as undefined.
        return 0;
    }
    let group = group(hl_id);
    // SAFETY: the editor's own tables.
    let attr = unsafe { ns_get_hl(&mut ns_id, hl_id, false, group.set != 0) };
    // An optional group falls through to nothing rather than to the global.
    if attr >= 0 || (*optional && ns_id > 0) {
        return attr;
    }
    group.attr
}

/// Follows `*hl_idp`'s links to the group that carries the attributes.
///
/// Answers whether a namespace had something to say. `*hl_idp` is set to 0
/// for an id outside the table — this is reachable from `eval`.
///
/// # Safety
/// See [`syn_id2attr`].
pub(crate) unsafe fn syn_ns_get_final_id(ns_id: &mut NS, hl_idp: &mut c_int) -> bool {
    let mut hl_id = *hl_idp;
    let mut used = false;

    if hl_id > highlight_num_groups() || hl_id < 1 {
        *hl_idp = 0;
        return false;
    }

    // Look out for loops: give up after 100 links.
    for _ in 0..100 {
        let group = group(hl_id);
        // TODO(bfredl): when using "tmp" attribute (no link) the function
        // might be called twice. it needs be smart enough to remember attr
        // only to syn_id2attr time
        // SAFETY: the editor's own tables.
        let check = unsafe { ns_get_hl(ns_id, hl_id, true, group.set != 0) };
        if check == 0 {
            // How dare! It broke the link.
            *hl_idp = hl_id;
            return true;
        } else if check > 0 {
            used = true;
            hl_id = check;
        } else if group.link > 0 && group.link <= highlight_num_groups() {
            hl_id = group.link;
        } else if group.cleared && group.parent > 0 {
            hl_id = group.parent;
        } else {
            break;
        }
    }

    *hl_idp = hl_id;
    used
}

/// The group id every `:highlight link` chain from `hl_id` ends at, in the
/// namespace the current window has active.
///
/// # Safety
/// See [`syn_id2attr`].
pub(crate) unsafe fn syn_get_final_id(hl_id: c_int) -> c_int {
    // SAFETY: the editor's own state.
    let mut ns_id = unsafe { (*curwin.get()).w_ns_hl_active };
    let mut hl_id = hl_id;
    unsafe { syn_ns_get_final_id(&mut ns_id, &mut hl_id) };
    hl_id
}

/// Replaces the entries from `keep` on with `count` all-zero scratch ones.
///
/// [`super::highlight_changed`] borrows the end of the table for the
/// `User1..9`-over-`StatusLineNC` combinations, which have to be real
/// entries because `syn_id2attr` is asked for them.
pub(crate) fn open_scratch(keep: c_int, count: usize) {
    GROUPS.with_mut(|table| {
        table.entries.truncate(keep as usize);
        table.entries.resize(keep as usize + count, HlGroup::ZEROED);
    });
}

/// Gives the scratch entries back.
pub(crate) fn close_scratch(keep: c_int) {
    GROUPS.with_mut(|table| table.entries.truncate(keep as usize));
}
