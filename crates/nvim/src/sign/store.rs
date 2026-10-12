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
use std::rc::Rc;

/// Every defined sign, in definition order.
///
/// Shared, so a reader's copy is a count (`:sign place` takes one per
/// placement, `:sign list` one per definition) and an update clones only a
/// definition some snapshot still holds. Deleting swap-removes, which is what the `Map(cstr_t, ptr_t)` upstream
/// uses does to its dense key array -- and that order is observable in
/// `:sign list`, `sign_getdefined()` and `:sign` completion.
static SIGNS: GlobalCell<Vec<Rc<Sign>>> = GlobalCell::new(Vec::new());

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
fn with_sign<R>(name: &[u8], f: impl FnOnce(&mut Rc<Sign>) -> R) -> Option<R> {
    SIGNS.with_mut(|signs| signs.iter_mut().find(|s| s.name.as_bytes() == name).map(f))
}

/// The definition `:sign define` recorded under `name`, as it is now.
pub(crate) fn sign_find(name: &[u8]) -> Option<Rc<Sign>> {
    SIGNS.with(|signs| signs.iter().find(|s| s.name.as_bytes() == name).cloned())
}

/// Whether a sign is still defined under `name`.
fn sign_is_defined(name: &[u8]) -> bool {
    SIGNS.with(|signs| signs.iter().any(|s| s.name.as_bytes() == name))
}

/// Copies of every defined sign, in definition order: `:sign list` and
/// `sign_getdefined()` format each entry as they walk, and formatting can
/// re-enter.
pub(crate) fn sign_defs() -> Vec<Rc<Sign>> {
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
    // Everything up to the highlight groups is this module's own work, done
    // in one visit of the table; the groups go through `syn_check_group`
    // outside it and land in a second.
    let icon = attrs.icon.map(|icon| backslash_halved(&icon));
    let text = attrs.text.map(|text| unescape(&text));
    let mut new_sign = false;
    let parsed = SIGNS.with_mut(|signs| {
        let at = match signs.iter().position(|s| s.name.as_bytes() == name) {
            Some(at) => at,
            None => {
                new_sign = true;
                let name = CString::new(name).expect("a sign name is a C string's bytes");
                signs.push(Rc::new(Sign {
                    name,
                    ..Default::default()
                }));
                signs.len() - 1
            }
        };
        let s = Rc::make_mut(&mut signs[at]);
        if let Some(icon) = icon {
            s.icon = Some(icon);
        }
        if let Some(text) = &text
            && let Err(failed) = init_sign_text(text, &mut s.text)
        {
            return Err(failed);
        }
        s.priority = attrs.prio;
        Ok(())
    });
    if let Err(failed) = parsed {
        semsg!(
            "E239: Invalid sign text: {}",
            msg_bytes(text.as_deref().unwrap_or_default())
        );
        return Err(failed);
    }

    let groups = [attrs.linehl, attrs.texthl, attrs.culhl, attrs.numhl];
    let ids = groups.map(|group| {
        group.map(|group| {
            if group.is_empty() {
                0
            } else {
                syn_check_group(&group)
            }
        })
    });
    let def = with_sign(name, |s| {
        let def = Rc::make_mut(s);
        let [line, text, cul, num] = ids;
        def.line_hl = line.unwrap_or(def.line_hl);
        def.text_hl = text.unwrap_or(def.text_hl);
        def.cul_hl = cul.unwrap_or(def.cul_hl);
        def.num_hl = num.unwrap_or(def.num_hl);
        Rc::clone(s)
    });

    if !new_sign && let Some(def) = def {
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
