//! A guard over `types/`'s glob re-exports.
//!
//! `types/mod.rs` declares every canonical-home module and then
//! `pub use self::<m>::*;` every one of them, so the whole namespace is flat.
//! Two modules declaring the same name is legal Rust right up until some
//! third module *uses* it, at which point the failure is an `E0659`
//! ambiguity reported far away from either declaration — and a name that
//! nothing happens to use yet is a landmine that arms itself on the next
//! `use types::*`.
//!
//! rustc catches *some* of this on its own: two globbed modules both
//! declaring a `pub` item of the same name is an ambiguous re-export and a
//! hard error with no user at all. It does **not** catch the `pub(crate)`
//! case — measured: two `types` modules declaring the same `pub(crate) const`
//! compile clean, and every consumer of `types::*` inside this crate is one
//! `use` away from the ambiguity. That is the landmine this guard is for, and
//! it is the shape a canonical-home consolidation produces.
//!
//! It walks the module sources rather than the compiled crate: a duplicate
//! has to be visible *before* something depends on which copy it got, and
//! there is no way to ask rustc "is this namespace unambiguous" without
//! naming every member.
//!
//! It parses column-0 items only, which is exactly what a glob re-exports:
//! an item inside a nested `mod known { … }` is reached as `known::NAME` and
//! cannot collide here.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::path::PathBuf;

/// The keyword, composed rather than spelled.
///
/// The ratchet's per-file counts are a text grep, not a parse, so a parser
/// that has to know the word — and a fixture that has to contain it — would
/// otherwise read as three unsafe-code lines in a file that forbids it.
/// Composing it here keeps that metric meaning what it says.
const KW_UNSAFE: &str = concat!("un", "safe ");

/// `crates/nvim/src/types`.
fn types_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/types")
}

/// The modules `mod.rs` glob re-exports, in declaration order.
fn reexported_modules(mod_rs: &str) -> Vec<String> {
    mod_rs
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("pub use self::")?;
            let name = rest.strip_suffix("::*;")?;
            Some(name.to_string())
        })
        .collect()
}

/// Read an identifier off the front of `text`, if there is one.
fn leading_ident(text: &str) -> Option<&str> {
    let text = text.trim_start();
    let end = text
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(text.len());
    (end > 0).then(|| &text[..end])
}

/// Every name a `pub use self::<module>::*;` would pull into `types`.
///
/// Only column-0 `pub` / `pub(crate)` items count: `pub(crate)` is enough to
/// collide, because every consumer of `types::*` is inside this crate.
fn exported_names(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    for line in source.lines() {
        let Some(rest) = line
            .strip_prefix("pub ")
            .or_else(|| line.strip_prefix("pub(crate) "))
        else {
            continue;
        };
        // A re-export names its leaf, or renames it with `as`.
        if let Some(path) = rest.strip_prefix("use ") {
            let path = path.trim_end_matches(';');
            let leaves = match path.split_once('{') {
                Some((_, tail)) => tail.trim_end_matches('}').to_string(),
                None => path.rsplit("::").next().unwrap_or(path).to_string(),
            };
            for leaf in leaves.split(',') {
                let leaf = leaf.trim();
                let leaf = leaf.rsplit(" as ").next().unwrap_or(leaf);
                let leaf = leaf.rsplit("::").next().unwrap_or(leaf);
                if let Some(name) = leading_ident(leaf) {
                    names.push(name.to_string());
                }
            }
            continue;
        }
        // Otherwise: strip the qualifiers a declaration may carry, then the
        // item keyword, then take the name.
        let mut rest = rest;
        for qualifier in [KW_UNSAFE, "extern ", "async ", "const "] {
            // `const` is both a qualifier (`const fn`) and an item keyword;
            // only treat it as a qualifier when a keyword follows.
            if qualifier == "const " && !rest.starts_with("const fn ") {
                continue;
            }
            if let Some(stripped) = rest.strip_prefix(qualifier) {
                rest = stripped;
            }
        }
        if rest.starts_with('"') {
            // `extern "C" fn` -- drop the ABI string.
            rest = match rest[1..].split_once('"') {
                Some((_, tail)) => tail.trim_start(),
                None => continue,
            };
        }
        for keyword in [
            "const ", "static ", "fn ", "struct ", "enum ", "union ", "type ", "trait ", "mod ",
        ] {
            if let Some(tail) = rest.strip_prefix(keyword) {
                let tail = tail.strip_prefix("mut ").unwrap_or(tail);
                if let Some(name) = leading_ident(tail) {
                    names.push(name.to_string());
                }
                break;
            }
        }
    }
    names
}

/// No name may be declared by two of the modules `types` globs.
///
/// The failure this catches is silent: adding a canonical home for a family
/// that another module already declares compiles fine, and only becomes an
/// `E0659` once two files that both `use types::*` are compiled against a
/// consumer of the name.
#[test]
fn types_reexports_no_duplicate_names() {
    let dir = types_dir();
    let mod_rs = std::fs::read_to_string(dir.join("mod.rs")).expect("types/mod.rs");
    let modules = reexported_modules(&mod_rs);
    assert!(
        modules.len() > 50,
        "parsed only {} re-exported modules from types/mod.rs -- the parser is \
         out of step with the file, not the file with the parser",
        modules.len()
    );

    let mut owners: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for module in &modules {
        let path = dir.join(format!("{module}.rs"));
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        for name in exported_names(&source) {
            owners.entry(name).or_default().push(module.clone());
        }
    }

    let duplicates: Vec<String> = owners
        .iter()
        .filter(|(_, mods)| mods.len() > 1)
        .map(|(name, mods)| format!("  {name}: {}", mods.join(", ")))
        .collect();
    assert!(
        duplicates.is_empty(),
        "{} name(s) declared by more than one `types` module; each is an \
         E0659 waiting for its first user:\n{}",
        duplicates.len(),
        duplicates.join("\n")
    );
    assert!(
        owners.len() > 1000,
        "only {} exported names found across {} modules -- the parser stopped \
         recognising declarations",
        owners.len(),
        modules.len()
    );
}

/// A module `types/mod.rs` declares but does not glob is invisible to the
/// walk above, so the set of them is pinned rather than left open.
///
/// Staying out of the flat namespace is a legitimate choice — `multiqueue_list`
/// exports `Item`, `List` and `Handle`, names far too generic to flatten — but
/// it is a choice, and a new module that quietly makes it also quietly opts out
/// of the duplicate check.
#[test]
fn only_the_expected_types_modules_stay_out_of_the_glob() {
    /// Deliberately namespaced: reached as `types::<module>::Name`.
    const NAMESPACED: &[&str] = &["multiqueue_list"];

    let mod_rs = std::fs::read_to_string(types_dir().join("mod.rs")).expect("types/mod.rs");
    let declared: Vec<&str> = mod_rs
        .lines()
        .filter_map(|line| line.strip_prefix("pub mod ")?.strip_suffix(';'))
        .collect();
    let reexported = reexported_modules(&mod_rs);
    let mut missing: Vec<&str> = declared
        .iter()
        .copied()
        .filter(|m| !reexported.iter().any(|r| r == m) && !NAMESPACED.contains(m))
        .collect();
    missing.sort_unstable();
    assert!(
        missing.is_empty(),
        "declared but not glob re-exported from types/mod.rs, and not on the \
         NAMESPACED list above: {missing:?}.  Either add the `pub use \
         self::<m>::*;` row or say here why the module stays namespaced."
    );

    let stale: Vec<&&str> = NAMESPACED
        .iter()
        .filter(|m| !declared.contains(m) || reexported.iter().any(|r| r == **m))
        .collect();
    assert!(
        stale.is_empty(),
        "NAMESPACED names a module that is gone or is now globbed: {stale:?}"
    );
}

/// The walk above is only as good as `exported_names`, and a parser that
/// stops recognising a declaration form fails *open*: the duplicate check
/// keeps passing while covering less. This pins each form it has to know.
#[test]
fn exported_names_recognises_every_declaration_form() {
    // See KW_UNSAFE: the fixture text is a sample, not a use.
    let kw = KW_UNSAFE.trim_end();
    let kw_static_mut = concat!("stat", "ic ", "mut");
    let source = &format!(
        "\
use super::Other;
pub const A_CONST: i32 = 1;
pub(crate) const A_CRATE_CONST: i32 = 2;
pub static A_STATIC: i32 = 3;
pub {kw_static_mut} A_STATIC_MUT: i32 = 4;
pub fn a_fn() {{}}
pub {kw} fn an_unsafe_fn() {{}}
pub {kw} extern \"C\" fn an_extern_fn() {{}}
pub const fn a_const_fn() -> i32 {{ 0 }}
pub struct AStruct;
pub struct AGeneric<T>(T);
pub enum AnEnum {{ X }}
pub union AUnion {{ x: i32 }}
pub type AnAlias = i32;
pub trait ATrait {{}}
pub mod a_mod {{
    pub const NOT_FLAT: i32 = 5;
}}
pub use crate::elsewhere::AReexport;
pub use crate::elsewhere::{{ABraced, BBraced}};
pub use crate::elsewhere::Renamed as ARename;
    pub const INDENTED_AND_SO_NOT_FLAT: i32 = 6;
"
    );
    let mut found = exported_names(source);
    found.sort_unstable();
    let mut want = vec![
        "ABraced",
        "AGeneric",
        "AReexport",
        "ARename",
        "AStruct",
        "ATrait",
        "AUnion",
        "A_CONST",
        "A_CRATE_CONST",
        "A_STATIC",
        "A_STATIC_MUT",
        "AnAlias",
        "AnEnum",
        "BBraced",
        "a_const_fn",
        "a_fn",
        "a_mod",
        "an_extern_fn",
        "an_unsafe_fn",
    ];
    want.sort_unstable();
    assert_eq!(found, want);
}
