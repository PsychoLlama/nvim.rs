//! The definition table: what `:sign define` records, and the sign groups
//! `:sign place group=` has created.
//!
//! A definition is a name, up to two cells of text and up to four highlight
//! groups. It is *not* what the drawing code reads -- a placement carries its
//! own copy (see [`super::place`]) -- which is why redefining a placed sign
//! has to walk the decoration store and patch every copy.
//!
//! The table is reached by name and lent for one closure ([`with_sign`]);
//! nothing outside it holds a definition, only copies of one, so a lookup
//! after anything that could have run user code finds what is there now.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::charset::backslash_halved;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::types::Failed;

/// Every defined sign, in definition order.
///
/// Deleting swap-removes, which is what the `Map(cstr_t, ptr_t)` upstream
/// uses does to its dense key array -- and that order is observable in
/// `:sign list`, `sign_getdefined()` and `:sign` completion.
static SIGNS: GlobalCell<Vec<Sign>> = GlobalCell::new(Vec::new());

/// The namespaces `:sign place group=` has created, in creation order — the
/// list `:sign` completion offers as group names.
///
/// Groups are never removed, so a group whose signs have all been unplaced
/// is still offered. That is upstream's behaviour.
static SIGN_GROUPS: GlobalCell<Vec<Integer>> = GlobalCell::new(Vec::new());

/// Runs `f` over the definition named `name`, if there is one.
///
/// `f` must not reach the table again; nothing in this module that runs
/// user code, or reports, does so inside it.
fn with_sign<R>(name: &[u8], f: impl FnOnce(&mut Sign) -> R) -> Option<R> {
    SIGNS.with_mut(|signs| signs.iter_mut().find(|s| s.name.as_bytes() == name).map(f))
}

/// A copy of the definition `:sign define` recorded under `name`.
pub(crate) fn sign_find(name: &[u8]) -> Option<Sign> {
    with_sign(name, |s| s.clone())
}

/// Whether a sign is still defined under `name`.
fn sign_is_defined(name: &[u8]) -> bool {
    with_sign(name, |_| ()).is_some()
}

/// Copies of every defined sign, in definition order: `:sign list` and
/// `sign_getdefined()` format each entry as they walk, and formatting can
/// re-enter.
pub(crate) fn sign_defs() -> Vec<Sign> {
    SIGNS.with(Clone::clone)
}

/// The name of the `idx`'th defined sign, or `None` past the end — the
/// `:sign list` / `:sign undefine` completion source.
pub(crate) fn sign_nth_name(idx: usize) -> Option<CString> {
    SIGNS.with(|signs| signs.get(idx).map(|s| s.name.clone()))
}

/// The namespace of the `idx`'th sign group, or `None` past the end — the
/// `group=` completion source.
pub(crate) fn sign_nth_group(idx: usize) -> Option<Integer> {
    SIGN_GROUPS.with(|groups| groups.get(idx).copied())
}

/// The namespace `group` names, creating it — and remembering it for
/// completion — the first time a sign is placed in it.
pub(super) fn namespace_of(group: &[u8]) -> Integer {
    let known = namespace_id_for(group).unwrap_or(0) != 0;
    let ns = nvim_create_namespace(String_0::from_bytes(group));
    if !known {
        SIGN_GROUPS.with_mut(|groups| groups.push(ns));
    }
    ns
}

/// The namespace filter `group` asks for: 0 for the global group (`None`),
/// [`ALL_GROUPS`] for `"*"`, [`NO_SUCH_GROUP`] for a group that does not
/// exist, and otherwise the group's own namespace.
pub(crate) fn group_get_ns(group: Option<&[u8]>) -> int64_t {
    let Some(group) = group else {
        return 0;
    };
    if group == b"*" {
        return ALL_GROUPS;
    }
    match namespace_id_for(group).unwrap_or(0) {
        0 => NO_SUCH_GROUP,
        ns => int64_t::from(ns),
    }
}

/// The name to report for a placed sign: the definition's name while it is
/// still defined, `"[Deleted]"` once it is not, and `""` for a sign placed
/// through `nvim_buf_set_extmark` rather than `:sign`.
pub(crate) fn placed_sign_name(sh: &DecorSignHighlight) -> &[u8] {
    match sh.sign_name_cstr() {
        None => b"",
        Some(name) if sign_is_defined(name.to_bytes()) => name.to_bytes(),
        Some(_) => b"[Deleted]",
    }
}

/// What `:sign define` and `sign_define()` set: each `None` leaves that
/// property alone, which is what makes `:sign define X texthl=Y` an update
/// rather than a redefinition.
#[derive(Default)]
pub(crate) struct SignAttrs {
    pub(crate) icon: Option<Vec<u8>>,
    /// The value as given; [`sign_define_by_name`] unescapes it.
    pub(crate) text: Option<Vec<u8>>,
    pub(crate) linehl: Option<Vec<u8>>,
    pub(crate) texthl: Option<Vec<u8>>,
    pub(crate) culhl: Option<Vec<u8>>,
    pub(crate) numhl: Option<Vec<u8>>,
    /// Always written, `-1` meaning [`SIGN_DEF_PRIO`] at placement.
    pub(crate) prio: c_int,
}

/// Defines a sign, or updates the one already defined under `name`.
///
/// A new sign is in the table before its text is parsed, so a `text=` that
/// fails (E239) still leaves it defined -- with the cells it got through and
/// nothing after them, as upstream's does.
pub(crate) fn sign_define_by_name(name: &[u8], attrs: SignAttrs) -> Result<(), Failed> {
    let new_sign = !sign_is_defined(name);
    if new_sign {
        let name = CString::new(name).expect("a sign name is a C string's bytes");
        SIGNS.with_mut(|signs| {
            signs.push(Sign {
                name,
                ..Default::default()
            });
        });
    }

    if let Some(icon) = attrs.icon {
        let icon = backslash_halved(&icon);
        with_sign(name, |s| s.icon = Some(icon));
    }

    if let Some(text) = attrs.text {
        let text = unescape(&text);
        // The sign was defined above, so the lookup finds it.
        if let Some(Err(failed)) = with_sign(name, |s| init_sign_text(&text, &mut s.text)) {
            semsg!("E239: Invalid sign text: {}", msg_bytes(&text));
            return Err(failed);
        }
    }

    with_sign(name, |s| s.priority = attrs.prio);

    let groups = [attrs.linehl, attrs.texthl, attrs.culhl, attrs.numhl];
    for (which, group) in groups.into_iter().enumerate() {
        let Some(group) = group else {
            continue;
        };
        let hl = if group.is_empty() {
            0
        } else {
            syn_check_group(&group)
        };
        with_sign(name, |s| match which {
            0 => s.line_hl = hl,
            1 => s.text_hl = hl,
            2 => s.cul_hl = hl,
            _ => s.num_hl = hl,
        });
    }

    if !new_sign && let Some(def) = sign_find(name) {
        update_placements(&def);
    }
    Ok(())
}

/// Copies a redefined sign's text and highlights into every placement of it,
/// and redraws the windows showing one.
///
/// Placements carry their own copy of the definition, so this is the only
/// thing that makes a `:sign define` of an already-placed sign visible.
fn update_placements(def: &Sign) {
    let name = def.name.as_bytes();
    let mut did_redraw = false;
    for mut sh in decor_items() {
        if sh.sign_name_cstr().is_none_or(|own| own.to_bytes() != name) {
            continue;
        }
        sh.text = def.text;
        sh.hl_id = def.text_hl;
        sh.line_hl_id = def.line_hl;
        sh.number_hl_id = def.num_hl;
        sh.cursorline_hl_id = def.cul_hl;
        if !did_redraw {
            for wp in windows() {
                let buf = wp.buffer();
                if buf_has_signs(buf) {
                    redraw_buf_later(buf, UPD_NOT_VALID);
                }
            }
            did_redraw = true;
        }
    }
}

/// Forgets the definition named `name`, or answers `Err` with E155.
///
/// Placements survive: they carry their own copy, and [`placed_sign_name`]
/// starts reporting them as `[Deleted]`.
pub(crate) fn sign_undefine_by_name(name: &[u8]) -> Result<(), Failed> {
    let removed = SIGNS.with_mut(|signs| {
        signs
            .iter()
            .position(|s| s.name.as_bytes() == name)
            // Swap-remove, which is what the map upstream uses does to its
            // dense key array; the resulting order is observable.
            .map(|i| signs.swap_remove(i))
    });
    if removed.is_none() {
        semsg!("E155: Unknown sign: {}", msg_bytes(name));
        return Err(Failed);
    }
    Ok(())
}

/// Forgets every definition — `sign_undefine()` with no argument.
pub(crate) fn free_signs() {
    drop(SIGNS.with_mut(::core::mem::take));
}
