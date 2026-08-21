//! The definition table: what `:sign define` records, and the sign groups
//! `:sign place group=` has created.
//!
//! A definition is a name, up to two cells of text and up to four highlight
//! groups. It is *not* what the drawing code reads -- a placement carries its
//! own copy (see [`super::place`]) -- which is why redefining a placed sign
//! has to walk the decoration store and patch every copy.
//!
//! Every read of a definition goes through [`Sign`], whose construction is
//! the promise that the definition is live; the accessors are ordinary safe
//! Rust.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;

/// One `:sign define` entry.
struct SignEntry {
    /// Owns the string `def.sn_name` points at. In the same box as `def`, so
    /// that pointer stays valid for the entry's whole life.
    name: CString,
    def: sign_T,
}

/// A sign definition the caller has promised is live. Definitions are boxed
/// (see [`SIGNS`]), so one stays put until its box is dropped.
#[derive(Clone, Copy)]
pub(crate) struct Sign(*mut sign_T);

impl Sign {
    /// # Safety
    /// `def` must be a live definition -- one [`SIGNS`] still holds.
    unsafe fn new(def: *mut sign_T) -> Self {
        Self(def)
    }

    /// The definition's `SIGN_WIDTH` cells, for the two readers that render
    /// them back into bytes.
    ///
    /// Derived from the wrapped pointer rather than from `deref_mut`, which
    /// is what keeps the answer valid past the end of this call.
    pub(crate) fn cells(self) -> *mut schar_T {
        // SAFETY: the constructor's promise — a live definition, whose
        // `sn_text` is one of its own fields. No read happens here.
        unsafe { (&raw mut (*self.0).sn_text).cast() }
    }
}

impl Deref for Sign {
    type Target = sign_T;
    fn deref(&self) -> &sign_T {
        // SAFETY: the constructor's promise — a live definition.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Sign {
    fn deref_mut(&mut self) -> &mut sign_T {
        // SAFETY: as above.
        unsafe { &mut *self.0 }
    }
}

/// Every defined sign, in definition order.
///
/// Boxed because a definition's address escapes: `sign_place` hands a
/// definition to `buf_set_sign`, and `sign_list_defined` holds one across
/// `msg_puts`. Deleting swap-removes, which is what the `Map(cstr_t, ptr_t)`
/// upstream uses does to its dense key array — and that order is observable
/// in `:sign list`, `sign_getdefined()` and `:sign` completion.
#[allow(clippy::vec_box)] // the box keeps the address stable; see above
static SIGNS: GlobalCell<Vec<Box<SignEntry>>> = GlobalCell::new(Vec::new());

/// The namespaces `:sign place group=` has created, in creation order — the
/// list `:sign` completion offers as group names.
///
/// Groups are never removed, so a group whose signs have all been unplaced
/// is still offered. That is upstream's behaviour.
static SIGN_GROUPS: GlobalCell<Vec<Integer>> = GlobalCell::new(Vec::new());

/// Runs `f` over the definition named `name`, if there is one.
///
/// # Safety
/// `name` must be a NUL-terminated string.
unsafe fn with_sign<R>(name: *const c_char, f: impl FnOnce(&mut Box<SignEntry>) -> R) -> Option<R> {
    // SAFETY: the caller's name.
    let key = unsafe { CStr::from_ptr(name) };
    SIGNS.with_mut(|signs| signs.iter_mut().find(|e| e.name.as_c_str() == key).map(f))
}

/// The definition `:sign define` recorded under `name`.
///
/// # Safety
/// `name` must be a NUL-terminated string.
pub(crate) unsafe fn sign_find(name: *const c_char) -> Option<Sign> {
    // SAFETY: the caller's name. The answer stays valid because each entry
    // is boxed and only `sign_undefine_by_name` ever drops one.
    unsafe { with_sign(name, |e| Sign::new(&raw mut e.def)) }
}

/// Whether a sign is still defined under `name`.
///
/// # Safety
/// `name` must be a NUL-terminated string.
unsafe fn sign_is_defined(name: *const c_char) -> bool {
    // SAFETY: the caller's name.
    unsafe { with_sign(name, |_| ()).is_some() }
}

/// Every defined sign, in definition order.
///
/// A snapshot rather than an iterator: `:sign list` and `sign_getdefined()`
/// format each entry as they walk, and formatting can re-enter.
pub(crate) fn sign_defs() -> Vec<Sign> {
    // SAFETY: each entry is boxed and lives until its own `swap_remove`.
    SIGNS.with_mut(|signs| {
        signs
            .iter_mut()
            .map(|e| unsafe { Sign::new(&raw mut e.def) })
            .collect()
    })
}

/// The name of the `idx`'th defined sign, or null past the end — the
/// `:sign list` / `:sign undefine` completion source.
pub(crate) fn sign_nth_name(idx: usize) -> *mut c_char {
    SIGNS.with(|signs| {
        signs
            .get(idx)
            .map_or(::core::ptr::null_mut(), |e| e.name.as_ptr().cast_mut())
    })
}

/// The namespace of the `idx`'th sign group, or `None` past the end — the
/// `group=` completion source.
pub(crate) fn sign_nth_group(idx: usize) -> Option<Integer> {
    SIGN_GROUPS.with(|groups| groups.get(idx).copied())
}

/// The namespace id `group` names, or 0 when it names none.
///
/// # Safety
/// `group` must be a NUL-terminated string.
unsafe fn namespace_id(group: *const c_char) -> c_int {
    let map = namespace_ids.ptr();
    // SAFETY: the caller's group name, and the editor's own namespace table.
    let k = unsafe { mh_get_string(&raw mut (*map).set, cstr_as_string(group)) };
    if k == u32::MAX {
        return 0;
    }
    // SAFETY: `k` is an index that table just answered with.
    unsafe { *(*map).values.add(k as usize) }
}

/// The namespace `group` names, creating it — and remembering it for
/// completion — the first time a sign is placed in it.
///
/// # Safety
/// `group` must be NUL-terminated.
pub(super) unsafe fn namespace_of(group: *mut c_char) -> Integer {
    // SAFETY: the caller's group name.
    let known = unsafe { namespace_id(group) } != 0;
    // SAFETY: as above.
    let ns = unsafe { nvim_create_namespace(cstr_as_string(group)) };
    if !known {
        SIGN_GROUPS.with_mut(|groups| groups.push(ns));
    }
    ns
}

/// The namespace filter `group` asks for: 0 for the global group,
/// [`ALL_GROUPS`] for `"*"`, [`NO_SUCH_GROUP`] for a group that does not
/// exist, and otherwise the group's own namespace.
///
/// # Safety
/// `group` must be null or a NUL-terminated string.
pub(crate) unsafe fn group_get_ns(group: *const c_char) -> int64_t {
    if group.is_null() {
        return 0;
    }
    // SAFETY: the caller's group name.
    if unsafe { strcmp(group, c"*".as_ptr()) } == 0 {
        return ALL_GROUPS;
    }
    // SAFETY: as above.
    match unsafe { namespace_id(group) } {
        0 => NO_SUCH_GROUP,
        ns => int64_t::from(ns),
    }
}

/// The name to report for a placed sign: the definition's name while it is
/// still defined, `"[Deleted]"` once it is not, and `""` for a sign placed
/// through `nvim_buf_set_extmark` rather than `:sign`.
///
/// # Safety
/// `sh` must be a live sign decoration.
pub(crate) unsafe fn sign_get_name(sh: *mut DecorSignHighlight) -> *const c_char {
    // SAFETY: the caller's decoration.
    let name = unsafe { Sh::new(sh) }.sign_name;
    if name.is_null() {
        return c"".as_ptr();
    }
    // SAFETY: a sign decoration's name is a NUL-terminated string it owns.
    if unsafe { sign_is_defined(name) } {
        name
    } else {
        c"[Deleted]".as_ptr()
    }
}

/// Defines a sign, or updates the one already defined under `name`.
///
/// Every argument but `name` and `prio` is optional: a null leaves that
/// property alone, which is what makes `:sign define X texthl=Y` an update
/// rather than a redefinition. `prio` is always written, `-1` meaning
/// [`SIGN_DEF_PRIO`].
///
/// # Safety
/// Every non-null argument must be a NUL-terminated string; `text` must
/// additionally be writable ([`init_sign_text`] unescapes it in place).
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn sign_define_by_name(
    name: *mut c_char,
    icon: *mut c_char,
    text: *mut c_char,
    linehl: *mut c_char,
    texthl: *mut c_char,
    culhl: *mut c_char,
    numhl: *mut c_char,
    prio: c_int,
) -> c_int {
    // SAFETY: the caller's name.
    let found = unsafe { sign_find(name) };
    let mut def = match found {
        Some(def) => def,
        None => {
            // SAFETY: as above.
            let owned = unsafe { CStr::from_ptr(name) }.to_owned();
            let mut entry = Box::new(SignEntry {
                def: sign_T {
                    sn_name: owned.as_ptr().cast_mut(),
                    ..Default::default()
                },
                name: owned,
            });
            let def = &raw mut entry.def;
            SIGNS.with_mut(|signs| signs.push(entry));
            // SAFETY: the entry was just pushed, and is boxed.
            unsafe { Sign::new(def) }
        }
    };

    if !icon.is_null() {
        // SAFETY: the old icon is this module's own `xstrdup` and the new
        // one the caller's NUL-terminated path.
        def.sn_icon = unsafe {
            xfree(def.sn_icon.cast());
            let owned = xstrdup(icon);
            backslash_halve(owned);
            owned
        };
    }

    // SAFETY: the caller's text, writable and NUL-terminated, and the
    // definition's own cells.
    if !text.is_null() && unsafe { init_sign_text(text, def.cells(), true) } != OK {
        return FAIL;
    }

    def.sn_priority = prio;

    for (which, arg) in [linehl, texthl, culhl, numhl].into_iter().enumerate() {
        if arg.is_null() {
            continue;
        }
        // SAFETY: the caller's highlight group name, NUL-terminated.
        let hl = unsafe {
            if *arg != 0 {
                syn_check_group(arg, strlen(arg))
            } else {
                0
            }
        };
        match which {
            0 => def.sn_line_hl = hl,
            1 => def.sn_text_hl = hl,
            2 => def.sn_cul_hl = hl,
            _ => def.sn_num_hl = hl,
        }
    }

    if found.is_some() {
        // SAFETY: the caller's name.
        unsafe { update_placements(name, def) };
    }
    OK
}

/// Copies a redefined sign's text and highlights into every placement of it,
/// and redraws the windows showing one.
///
/// Placements carry their own copy of the definition, so this is the only
/// thing that makes a `:sign define` of an already-placed sign visible.
///
/// # Safety
/// `name` must be NUL-terminated.
unsafe fn update_placements(name: *const c_char, def: Sign) {
    // The definition is copied out so the store below cannot move the entry
    // underneath the walk.
    let def = *def;
    let mut did_redraw = false;
    for mut sh in decor_items() {
        // SAFETY: the caller's name, and a store item's own name string.
        if sh.sign_name.is_null() || unsafe { strcmp(sh.sign_name, name) } != 0 {
            continue;
        }
        sh.text = def.sn_text;
        sh.hl_id = def.sn_text_hl;
        sh.line_hl_id = def.sn_line_hl;
        sh.number_hl_id = def.sn_num_hl;
        sh.cursorline_hl_id = def.sn_cul_hl;
        if !did_redraw {
            for wp in windows() {
                let buf = wp.buffer();
                // SAFETY: a live window's buffer is live.
                if unsafe { buf_has_signs(buf.raw()) } {
                    // SAFETY: as above.
                    unsafe { redraw_buf_later(buf.raw(), UPD_NOT_VALID) };
                }
            }
            did_redraw = true;
        }
    }
}

/// Forgets the definition named `name`, or answers `FAIL` with E155.
///
/// Placements survive: they carry their own copy, and [`sign_get_name`]
/// starts reporting them as `[Deleted]`.
///
/// # Safety
/// `name` must be a NUL-terminated string.
pub(crate) unsafe fn sign_undefine_by_name(name: *const c_char) -> c_int {
    // SAFETY: the caller's name.
    let key = unsafe { CStr::from_ptr(name) };
    let entry = SIGNS.with_mut(|signs| {
        signs
            .iter()
            .position(|e| e.name.as_c_str() == key)
            // Swap-remove, which is what the map upstream uses does to its
            // dense key array; the resulting order is observable.
            .map(|i| signs.swap_remove(i))
    });
    let Some(entry) = entry else {
        // SAFETY: the caller's name, and a format the message takes.
        unsafe { semsg_c!(gettext(c"E155: Unknown sign: %s".as_ptr()), name) };
        return FAIL;
    };
    // SAFETY: the icon is this module's own `xstrdup`.
    unsafe { xfree(entry.def.sn_icon.cast()) };
    OK
}

/// Forgets every definition — `sign_undefine()` with no argument.
pub fn free_signs() {
    for entry in SIGNS.with_mut(::core::mem::take) {
        // SAFETY: the icon is this module's own `xstrdup` and nothing else
        // holds it.
        unsafe { xfree(entry.def.sn_icon.cast()) };
    }
}
