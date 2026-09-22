//! The option table, from `crates/nvim/src/options.lua`.
//!
//! Upstream's `src/gen/gen_options.lua` (tag v0.12.4) turned that file into
//! four generated headers: the `options[]` array itself, the `OptIndex` and
//! per-scope index enums, the name lookup, and the arrays of valid values.
//! This is the same job in Rust, reading the same vendored metadata.
//!
//! Two things upstream deferred to the C compiler have to be settled here:
//!
//! * `enable_if` and a default's `condition` were `#if defined(...)`. The
//!   transpiled tree is the behaviour of record and it was compiled for
//!   Unix, so [`DEFINED`] fixes the same answers. A port to another target
//!   changes that list, and the option table with it.
//! * A default that reads `macros('DFLT_COLS', 'number')` names a C macro.
//!   Those survived transpilation as ordinary `pub const` items, so the
//!   generated table references them by name rather than baking in a value.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use crate::lua::{Table, Value, read_table};
use crate::{Emitted, chunked, rustfmt};

/// The preprocessor symbols the transpiled tree was compiled with, as far as
/// `options.lua` asks. Everything else is undefined.
const DEFINED: &[&str] = &["UNIX"];

/// `scope_idx` is indexed by `OptScope`, which is not the order
/// `valid_scopes` lists the scopes in.
const SCOPE_IDX_ORDER: [&str; 3] = ["global", "win", "buf"];

/// `list` -> the flags it contributes, in upstream's order.
const LIST_FLAGS: &[(&str, &[&str])] = &[
    ("comma", &["kOptFlagComma"]),
    ("onecomma", &["kOptFlagOneComma"]),
    ("commacolon", &["kOptFlagComma", "kOptFlagColon"]),
    ("onecommacolon", &["kOptFlagOneComma", "kOptFlagColon"]),
    ("flags", &["kOptFlagFlagList"]),
    ("flagscomma", &["kOptFlagComma", "kOptFlagFlagList"]),
];

/// A `redraw` entry -> its flag.
const REDRAW_FLAGS: &[(&str, &str)] = &[
    ("ui_option", "kOptFlagUIOption"),
    ("tabline", "kOptFlagRedrTabl"),
    ("statuslines", "kOptFlagRedrStat"),
    ("current_window", "kOptFlagRedrWin"),
    ("current_buffer", "kOptFlagRedrBuf"),
    ("all_windows", "kOptFlagRedrAll"),
    ("curswant", "kOptFlagCurswant"),
    ("highlight_only", "kOptFlagHLOnly"),
];

/// The boolean option attributes that become flags, in the order upstream
/// tests them — the table's `|` chains are in exactly this sequence.
const BOOL_FLAGS: &[(&str, &str)] = &[
    ("nodefault", "kOptFlagNoDefault"),
    ("no_mkrc", "kOptFlagNoMkrc"),
    ("secure", "kOptFlagSecure"),
    ("gettext", "kOptFlagGettext"),
    ("noglob", "kOptFlagNoGlob"),
    ("normal_fname_chars", "kOptFlagNFname"),
    ("normal_dname_chars", "kOptFlagNDname"),
    ("pri_mkrc", "kOptFlagPriMkrc"),
    ("deny_duplicates", "kOptFlagNoDup"),
    ("modelineexpr", "kOptFlagMLE"),
    ("func", "kOptFlagFunc"),
];

/// One entry of an option's `values` list: a name, plus the nested list a
/// `name:` prefix accepts.
struct Choice {
    name: String,
    nested: Vec<Choice>,
}

/// The flag enum an option's `flags` key asks for, in ascending value order.
///
/// Upstream emits one whenever `flags` is truthy and the option either spells
/// the values itself or has a `values` list to take bit positions from — so a
/// bare `flags = true` with no `values` produces nothing. The two spellings
/// are not interchangeable: `flags = { Insert = 6 }` exists precisely because
/// some of these are combinations rather than single bits.
fn flag_enum(o: &Table, values: &[Choice], full_name: &str) -> Result<Vec<(String, i64)>, String> {
    let fail = |what: &str| format!("option '{full_name}': {what}");
    if !o.truthy("flags") {
        return Ok(Vec::new());
    }
    let mut entries: Vec<(String, i64)> = match o.get("flags") {
        Some(Value::Table(t)) => t
            .map
            .iter()
            .map(|(name, value)| match value {
                Value::Int(n) => Ok((name.clone(), *n)),
                other => Err(fail(&format!(
                    "flag `{name}` is {}, not a number",
                    other.describe()
                ))),
            })
            .collect::<Result<_, _>>()?,
        // Otherwise each valid value owns the next bit, in the order the
        // `values` list spells them.
        _ => values
            .iter()
            .enumerate()
            .map(|(i, choice)| match choice.nested.is_empty() {
                true => Ok((choice.name.clone(), 1 << i)),
                false => Err(fail("cannot take flags from a nested `values` list")),
            })
            .collect::<Result<_, _>>()?,
    };
    entries.sort_by_key(|(_, value)| *value);
    Ok(entries
        .into_iter()
        .map(|(name, value)| (titlecase(name.trim_end_matches(':')), value))
        .collect())
}

/// A default value, once the `condition` has been settled.
enum Default {
    Bool(bool),
    Int(i64),
    /// A literal string, as a Rust `c"..."` literal body.
    Str(String),
    /// A C constant that holds the default: its name and its type.
    Macro(String, &'static str),
}

struct Opt {
    index: usize,
    full_name: String,
    abbreviation: Option<String>,
    names: Vec<String>,
    ty: String,
    scope: Vec<String>,
    /// Index within each scope's own enum, for the scopes it has.
    scope_idx: BTreeMap<String, usize>,
    varname: Option<String>,
    flags_varname: Option<String>,
    flags: Vec<String>,
    /// The option's own flag enum: one `(suffix, value)` per bit its parsed
    /// value can carry, in ascending value order. Empty when it has none.
    flag_enum: Vec<(String, i64)>,
    values: Vec<Choice>,
    did_set_cb: Option<String>,
    expand_cb: Option<String>,
    immutable: bool,
    def_val: Default,
}

impl Opt {
    /// The `OptIndex` constant for this option.
    fn index_const(&self) -> String {
        format!("kOpt{}", titlecase(&self.full_name))
    }

    /// The prefix its `values` arrays are named with.
    fn values_prefix(&self) -> String {
        format!(
            "opt_{}",
            self.abbreviation.as_ref().unwrap_or(&self.full_name)
        )
    }

    /// The name its flag enum is spelled with: an abbreviated option is known
    /// by its abbreviation here, not its full name.
    fn flags_name(&self) -> String {
        titlecase(self.abbreviation.as_ref().unwrap_or(&self.full_name))
    }

    /// The type its flag enum's constants carry.
    fn flags_type(&self) -> String {
        format!("Opt{}Flags", self.flags_name())
    }

    /// The constant naming one bit of its flag enum.
    fn flag_const(&self, suffix: &str) -> String {
        format!("kOpt{}Flag{}", self.flags_name(), suffix)
    }
}

/// `lowercase_to_titlecase`: split on `-`/`_`, capitalize, join.
fn titlecase(s: &str) -> String {
    s.split(['-', '_'])
        .map(|word| match word.chars().next() {
            Some(c) => c.to_uppercase().collect::<String>() + &word[c.len_utf8()..],
            None => String::new(),
        })
        .collect()
}

/// The index constant naming an option's row in one scope's enum.
fn scope_const(scope: &str, name: &str) -> String {
    format!("k{}Opt{}", titlecase(scope), titlecase(name))
}

/// Render a string as the body of a Rust `c"..."` literal.
fn c_literal(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '"' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                write!(out, "\\x{:02x}", c as u32).unwrap();
            }
            c => out.push(c),
        }
    }
    out
}

fn choices(values: &[Value]) -> Result<Vec<Choice>, String> {
    values
        .iter()
        .map(|v| match v {
            Value::Str(name) => Ok(Choice {
                name: name.clone(),
                nested: Vec::new(),
            }),
            Value::Table(t) => {
                let (Some(Value::Str(name)), Some(Value::Table(nested))) =
                    (t.array.first(), t.array.get(1))
                else {
                    return Err("a nested `values` entry is not {string, table}".into());
                };
                Ok(Choice {
                    name: name.clone(),
                    nested: choices(&nested.array)?,
                })
            }
            _ => Err("a `values` entry is neither a string nor a table".into()),
        })
        .collect()
}

/// Settle a `defaults` table down to the one value this build takes.
fn default_value(o: &Table, full_name: &str) -> Result<Default, String> {
    let fail = |what: &str| format!("option '{full_name}': {what}");
    // `preprocess` in options.lua wraps a bare default in `{ if_true = .. }`.
    let defaults = match o.get("defaults") {
        Some(Value::Table(t)) if t.get("if_true").is_some() || t.get("condition").is_some() => {
            t.clone()
        }
        Some(value) => {
            let mut t = Table::default();
            t.map.insert("if_true".into(), value.clone());
            t
        }
        None => return Err(fail("should have a default value")),
    };
    let branch = match defaults.str("condition") {
        None => "if_true",
        Some(condition) => {
            let (negated, name) = match condition.strip_prefix('!') {
                Some(name) => (true, name),
                None => (false, condition),
            };
            if DEFINED.contains(&name) != negated {
                "if_true"
            } else {
                "if_false"
            }
        }
    };
    let value = defaults
        .get(branch)
        .ok_or_else(|| fail(&format!("has no `{branch}` default")))?;
    match value {
        Value::Bool(b) => Ok(Default::Bool(*b)),
        Value::Int(n) => Ok(Default::Int(*n)),
        Value::Str(s) => Ok(Default::Str(c_literal(s))),
        // 'statusline' spells its default one piece per line.
        Value::Call(f, args) if f == "table.concat" => {
            let Some(Value::Table(parts)) = args.first() else {
                return Err(fail("`table.concat` of something that is not a table"));
            };
            let sep = match args.get(1) {
                Some(Value::Str(sep)) => sep.as_str(),
                None => "",
                _ => return Err(fail("`table.concat` with a non-string separator")),
            };
            let pieces: Option<Vec<&str>> = parts
                .array
                .iter()
                .map(|v| match v {
                    Value::Str(s) => Some(s.as_str()),
                    _ => None,
                })
                .collect();
            let pieces = pieces.ok_or_else(|| fail("`table.concat` of non-strings"))?;
            Ok(Default::Str(c_literal(&pieces.join(sep))))
        }
        Value::Call(f, args) if f == "macros" => match (args.first(), args.get(1)) {
            (Some(Value::Str(name)), Some(Value::Str(ty))) if ty == "string" => {
                Ok(Default::Macro(name.clone(), "string"))
            }
            (Some(Value::Str(name)), Some(Value::Str(ty))) if ty == "number" => {
                Ok(Default::Macro(name.clone(), "number"))
            }
            _ => Err(fail("unsupported `macros(...)` default")),
        },
        other => Err(fail(&format!(
            "unsupported default value: {}",
            other.describe()
        ))),
    }
}

fn parse_options(source: &str) -> Result<Vec<Opt>, String> {
    let root = read_table("options.lua", source, "local options")?;
    let metas = root
        .table("options")
        .ok_or("options.lua: `options.options` is not a table")?;

    let mut opts = Vec::new();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for (index, meta) in metas.array.iter().enumerate() {
        let Value::Table(o) = meta else {
            return Err("options.lua: an option entry is not a table".into());
        };
        let full_name = o
            .str("full_name")
            .ok_or("options.lua: an option has no `full_name`")?
            .to_string();
        let fail = |what: &str| format!("option '{full_name}': {what}");
        let abbreviation = o.str("abbreviation").map(str::to_string);

        let scope = o.str_list("scope")?.ok_or_else(|| fail("has no `scope`"))?;
        let mut scope_idx = BTreeMap::new();
        for s in &scope {
            if !SCOPE_IDX_ORDER.contains(&s.as_str()) {
                return Err(fail(&format!("unknown scope `{s}`")));
            }
            let n = counts
                .entry(SCOPE_IDX_ORDER.iter().find(|v| *v == s).unwrap())
                .or_default();
            scope_idx.insert(s.clone(), *n);
            *n += 1;
        }

        let mut flags = Vec::new();
        if let Some(list) = o.str("list") {
            let (_, f) = LIST_FLAGS
                .iter()
                .find(|(k, _)| *k == list)
                .ok_or_else(|| fail(&format!("unknown `list` kind `{list}`")))?;
            flags.extend(f.iter().map(|f| (*f).to_string()));
        }
        for r in o.str_list("redraw")?.unwrap_or_default() {
            let (_, f) = REDRAW_FLAGS
                .iter()
                .find(|(k, _)| *k == r)
                .ok_or_else(|| fail(&format!("unknown `redraw` kind `{r}`")))?;
            flags.push((*f).to_string());
        }
        if o.truthy("expand") {
            flags.push("kOptFlagExpand".into());
            if o.str("expand") == Some("nodefault") {
                flags.push("kOptFlagNoDefExp".into());
            }
        }
        for (key, flag) in BOOL_FLAGS {
            if o.truthy(key) {
                flags.push((*flag).to_string());
            }
        }

        let values = match o.table("values") {
            Some(t) => choices(&t.array)?,
            None => Vec::new(),
        };
        // `preprocess`: an option with a `values` list gets the generic
        // string callbacks unless it names its own.
        let generic = !values.is_empty();
        let did_set_cb = o
            .str("cb")
            .map(str::to_string)
            .or_else(|| generic.then(|| "did_set_str_generic".to_string()));
        let expand_cb = o
            .str("expand_cb")
            .map(str::to_string)
            .or_else(|| generic.then(|| "expand_set_str_generic".to_string()));

        let mut names = vec![full_name.clone()];
        names.extend(abbreviation.clone());
        names.extend(o.str_list("alias")?.unwrap_or_default());

        let immutable = o.truthy("immutable");
        let varname = o.str("varname").map(str::to_string);
        // A window-local-only option has no global variable at all; anything
        // else either names one or is immutable and reads its own default.
        if varname.is_none() && !immutable && scope != ["win"] {
            return Err(fail("must be immutable or have a variable"));
        }
        // `enable_if` guards the option out of this build entirely: it keeps
        // its metadata but becomes an immutable read of its own default, and
        // the callbacks are compiled out with the branch that named them.
        let disabled = o.str("enable_if").is_some_and(|c| !DEFINED.contains(&c));

        opts.push(Opt {
            index,
            names,
            ty: o
                .str("type")
                .ok_or_else(|| fail("has no `type`"))?
                .to_string(),
            scope,
            scope_idx,
            varname: varname.filter(|_| !disabled),
            flags_varname: o.str("flags_varname").map(str::to_string),
            flags,
            flag_enum: flag_enum(o, &values, &full_name)?,
            values,
            did_set_cb: did_set_cb.filter(|_| !disabled),
            expand_cb: expand_cb.filter(|_| !disabled),
            immutable: immutable || disabled,
            def_val: default_value(o, &full_name)?,
            abbreviation,
            full_name,
        });
    }
    Ok(opts)
}

// ----------------------------------------------------------- name resolution

/// Where each name the table references is declared, so the generated module
/// can import it. A renamed callback becomes a generation error instead of a
/// table that silently points somewhere else.
/// Where in the crate each `pub` item a generated table can name is defined.
pub struct Symbols(BTreeMap<String, Vec<String>>);

impl Symbols {
    /// Scan the crate for `pub` items a generated table can name. `skip` is
    /// the generated directory itself, whose own definitions must not win.
    ///
    /// `prefer` breaks ties: c2rust copied the enum constants into every
    /// module that included the header, so a name can have dozens of
    /// definitions and only one of them is the canonical home. A name defined
    /// more than once with no preferred home is left ambiguous and reported
    /// by [`Symbols::path`] when something asks for it, rather than resolved
    /// by whichever module the directory walk happened to reach first.
    pub fn collect(root: &Path, skip: &Path, prefer: &[&str]) -> Result<Symbols, String> {
        let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut private: BTreeSet<String> = BTreeSet::new();
        let mut files: Vec<(String, String)> = Vec::new();
        let mut stack = vec![root.join("src")];
        while let Some(dir) = stack.pop() {
            let entries = std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    if path != skip {
                        stack.push(path);
                    }
                    continue;
                }
                if path.extension().is_none_or(|e| e != "rs") {
                    continue;
                }
                let module = module_path(root, &path);
                let text = std::fs::read_to_string(&path)
                    .map_err(|e| format!("{}: {e}", path.display()))?;
                for line in text.lines() {
                    // A module a file declares without `pub` is invisible
                    // from outside; its `pub` items are only reachable
                    // through whatever the parent re-exports.
                    if let Some(child) = line
                        .strip_prefix("mod ")
                        .and_then(|rest| rest.strip_suffix(';'))
                    {
                        private.insert(format!("{module}::{child}"));
                    }
                }
                files.push((module, text));
            }
        }
        for (module, text) in &files {
            // Hoist out of any private module: the parent is where a `pub
            // use` puts the name, and if it does not, the import the caller
            // writes will not compile, which is the intended failure.
            let module = match private
                .iter()
                .find(|p| module == *p || module.starts_with(&format!("{p}::")))
            {
                Some(p) => p.rsplit_once("::").unwrap().0.to_string(),
                None => module.clone(),
            };
            let mut in_extern = false;
            for line in text.lines() {
                if let Some(name) = declared(line, in_extern) {
                    out.entry(name.to_string())
                        .or_default()
                        .insert(module.clone());
                }
                // A c2rust `unsafe extern "C" { .. }` block opens and
                // closes at column 0; its `pub fn` declarations are how
                // libm and the C deps enter the crate.
                if line == "unsafe extern \"C\" {" {
                    in_extern = true;
                } else if line == "}" {
                    in_extern = false;
                }
            }
        }
        Ok(Symbols(
            out.into_iter()
                .map(|(name, modules)| {
                    let winner = prefer.iter().find(|p| modules.contains(**p));
                    let modules = match (modules.len(), winner) {
                        (1, _) => modules.into_iter().collect(),
                        (_, Some(p)) => vec![(*p).to_string()],
                        (_, None) => modules.into_iter().collect(),
                    };
                    (name, modules)
                })
                .collect(),
        ))
    }

    pub fn path(&self, name: &str) -> Result<&str, String> {
        match self.0.get(name).map(Vec::as_slice) {
            Some([one]) => Ok(one),
            Some(many) => Err(format!(
                "`{name}` is defined in {} modules ({}); name the canonical one",
                many.len(),
                many.join(", ")
            )),
            None => Err(format!("no `pub` item named `{name}` in the crate")),
        }
    }
}

/// The visibilities a generated table can name. The generated modules live
/// *inside* the crate, so `pub(crate)` is as good as `pub` to them -- and as
/// the crate's flat `pub mod` root narrows, most callbacks land there. A
/// `pub(super)` item is deliberately not accepted: whether the generated
/// module can see it depends on where that module sits, so it is left to fail
/// as an unresolvable name rather than resolved to a path that may not
/// compile.
const VISIBILITIES: &[&str] = &["pub ", "pub(crate) "];

/// The name an item on this line declares, if it is one a table can
/// reference. `in_extern` allows the indented `pub fn` declarations of a
/// c2rust `unsafe extern "C"` block, which is where `cos` and friends live.
fn declared(line: &str, in_extern: bool) -> Option<&str> {
    let line = if in_extern { line.trim_start() } else { line };
    let rest = VISIBILITIES.iter().find_map(|vis| line.strip_prefix(vis))?;
    let prefixes: &[&str] = if in_extern {
        &["fn "]
    } else {
        &[
            "unsafe extern \"C\" fn ",
            "extern \"C\" fn ",
            "unsafe fn ",
            "fn ",
            "static ",
            "const ",
        ]
    };
    for prefix in prefixes {
        if let Some(rest) = rest.strip_prefix(prefix) {
            let name = rest
                .split(|c: char| !c.is_alphanumeric() && c != '_')
                .next()?;
            return (!name.is_empty()).then_some(name);
        }
    }
    None
}

pub fn module_path(root: &Path, path: &Path) -> String {
    let rel = path
        .strip_prefix(root.join("src"))
        .unwrap()
        .with_extension("");
    let mut parts: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    if parts.last().is_some_and(|p| p == "mod") {
        parts.pop();
    }
    // `loop.rs`, `match.rs`, `move.rs`: lib.rs spells those as raw
    // identifiers, and so must anything importing from them.
    for part in &mut parts {
        if [
            "loop", "match", "move", "type", "ref", "box", "in", "fn", "mod",
        ]
        .contains(&part.as_str())
        {
            *part = format!("r#{part}");
        }
    }
    format!("crate::{}", parts.join("::"))
}

// ------------------------------------------------------------------ emitters

const HEADER: &str = r#"//! The option table.
//!
//! GENERATED by tools/apigen from `crate::options.lua` — the same
//! metadata upstream's `src/gen/gen_options.lua` consumed. Do not edit; run
//! `just apigen` (`just apigen --check` fails on drift).
//!
//! Everything an option *is* lives here: its name and abbreviation, its type
//! and scopes, the flags that drive redraws and parsing, the variable holding
//! its value, the callbacks that validate a change, and its default.
//! `crate::option` keeps everything an option *does*.
//!
//! Two orderings are load-bearing. The table is in `:set all` order, and a
//! row's position is its `OptIndex` — the handle every caller passes around.
//! Within each scope, a row's position is its `GlobalOptIndex` /
//! `WinOptIndex` / `BufOptIndex`, which is how a window or buffer finds its
//! own copy of the value.

#![forbid(unsafe_code)]
// The names are upstream's, and each is looked up by that spelling.
#![allow(non_upper_case_globals)]
"#;

fn child_header(what: &str, body: &str) -> String {
    let attr = if body.contains("unsafe ") {
        "#![deny(unsafe_op_in_unsafe_fn)]"
    } else {
        "#![forbid(unsafe_code)]"
    };
    format!(
        "//! {what}\n\
         //!\n\
         //! GENERATED by tools/apigen; see the parent module. Do not edit;\n\
         //! run `just apigen`.\n\
         \n\
         {attr}\n\
         {}\
         \n\
         // A chunk may hold nothing that needs the parent's support code.\n\
         #[allow(unused_imports)]\n\
         use super::*;\n\
         \n",
        crate::upper_case_globals_allow(body)
    )
}

/// The `OptIndex` enum, the per-scope index enums, and the reverse lookup
/// from a buffer-local index back to the option.
fn emit_index(out: &mut String, opts: &[Opt]) {
    writeln!(
        out,
        "/// Every option, in `:set all` order. `kOptInvalid` is \"no such option\"."
    )
    .unwrap();
    writeln!(out, "pub const kOptInvalid: OptIndex = -1;").unwrap();
    for o in opts {
        writeln!(
            out,
            "pub const {}: OptIndex = {};",
            o.index_const(),
            o.index
        )
        .unwrap();
    }
    writeln!(out, "pub const kOptCount: c_int = {};", opts.len()).unwrap();

    for scope in SCOPE_IDX_ORDER {
        let title = titlecase(scope);
        let rows: Vec<&Opt> = opts
            .iter()
            .filter(|o| o.scope_idx.contains_key(scope))
            .collect();
        out.push('\n');
        writeln!(
            out,
            "/// A row of a {scope}-scoped option's value, within whatever holds it."
        )
        .unwrap();
        writeln!(out, "pub type {title}OptIndex = c_int;").unwrap();
        writeln!(
            out,
            "pub const {}: {title}OptIndex = -1;",
            scope_const(scope, "invalid")
        )
        .unwrap();
        for o in &rows {
            writeln!(
                out,
                "pub const {}: {title}OptIndex = {};",
                scope_const(scope, &o.full_name),
                o.scope_idx[scope]
            )
            .unwrap();
        }
    }

    let buf: Vec<&Opt> = opts
        .iter()
        .filter(|o| o.scope_idx.contains_key("buf"))
        .collect();
    out.push('\n');
    writeln!(
        out,
        "/// `BufOptIndex` -> `OptIndex`: which option a buffer-local row belongs to."
    )
    .unwrap();
    writeln!(out, "pub static buf_opt_idx: [OptIndex; {}] = [", buf.len()).unwrap();
    for o in &buf {
        writeln!(out, "    {},", o.index_const()).unwrap();
    }
    writeln!(out, "];").unwrap();
}

/// The per-option flag enums: the bits a string option's parsed value is
/// remembered as, in the `flags_var` its row names.
fn emit_flags(out: &mut String, opts: &[Opt]) {
    let mut first = true;
    for o in opts.iter().filter(|o| !o.flag_enum.is_empty()) {
        if !first {
            out.push('\n');
        }
        first = false;
        let ty = o.flags_type();
        writeln!(out, "/// The bits '{}' parses into.", o.full_name).unwrap();
        writeln!(out, "pub type {ty} = c_uint;").unwrap();
        for (suffix, value) in &o.flag_enum {
            writeln!(
                out,
                "pub const {}: {ty} = {value:#04x};",
                o.flag_const(suffix)
            )
            .unwrap();
        }
    }
}

/// The arrays of valid values, in the preorder upstream walked them.
fn emit_values(out: &mut String, opts: &[Opt]) {
    fn walk(out: &mut String, prefix: &str, values: &[Choice]) {
        writeln!(
            out,
            "pub static {prefix}_values: [&CStr; {}] = [",
            values.len()
        )
        .unwrap();
        for c in values {
            writeln!(out, "    c\"{}\",", c_literal(&c.name)).unwrap();
        }
        writeln!(out, "];").unwrap();
        for c in values {
            if c.nested.is_empty() {
                continue;
            }
            out.push('\n');
            walk(
                out,
                &format!("{prefix}_{}", c.name.trim_end_matches(':')),
                &c.nested,
            );
        }
    }
    let mut first = true;
    for o in opts.iter().filter(|o| !o.values.is_empty()) {
        if !first {
            out.push('\n');
        }
        first = false;
        writeln!(out, "/// Valid values for '{}'.", o.full_name).unwrap();
        walk(out, &o.values_prefix(), &o.values);
    }
}

/// The support code [`emit_vars`] writes above the table of fields.
const VARS_SUPPORT: &str = r##"
use core::ffi::{CStr, c_char};
use core::mem::offset_of;

use crate::global_cell::GlobalCell;
use crate::memory::XString;
use crate::option::vars::option_defaults_set;
use crate::optionstr::empty_option;
use crate::types::OptInt;

/// Which field of [`Options`] one option's global value is.
///
/// A selector is a projection into the record plus the offset it projects
/// to, which is its identity: two selectors are equal when they name the
/// same field. The option table holds one per row, in place of the address
/// its `var` used to be, and [`crate::option::scope`] is what turns it into
/// a read or a write.
macro_rules! selector {
    ($name:ident, $ty:ty, $what:expr) => {
        #[doc = $what]
        ///
        /// A field selector; see the module docs.
        #[derive(Copy, Clone)]
        pub struct $name {
            /// The field's offset in [`Options`] — the selector's identity.
            at: usize,
            /// The field itself, typed, so a selector cannot name a field of
            /// another type.
            project: fn(&mut Options) -> &mut $ty,
        }

        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                self.at == other.at
            }
        }

        impl Eq for $name {}
    };
}

/// A selector over a field small enough to copy in and out: the numbers and
/// the booleans.
macro_rules! copy_selector {
    ($name:ident, $ty:ty, $what:expr) => {
        selector!($name, $ty, $what);

        impl $name {
            /// What the field holds.
            pub fn get(self) -> $ty {
                OPTIONS.get_field(self.project)
            }

            /// Overwrite the field.
            pub fn set(self, value: $ty) {
                OPTIONS.set_field(self.project, value);
            }
        }
    };
}

copy_selector!(BoolOpt, bool, "A boolean option's global value.");
copy_selector!(NumOpt, OptInt, "A number option's global value.");
selector!(
    StrOpt,
    Option<XString>,
    "A string option's global value: the string the option owns, if it owns one."
);

/// A global string option that owns nothing is one of two upstream states,
/// and which one depends on *when* it is asked.
///
/// Upstream's `char *p_xx` is `NULL` until `set_init_1` installs the
/// defaults, and after that it is either a string of its own or the shared
/// `empty_string_option` an explicit clear leaves behind (`substitute()`'s
/// quiet 'cpoptions', `:cc`'s spent 'switchbuf'). Both are `None` here; the
/// table-wide `crate::option::vars::option_defaults_set` tells them apart.
/// [`StrOpt::is_uninit`] is the `p_xx == NULL` test and
/// [`StrOpt::is_unset`] the `p_xx == empty_string_option` one.
///
/// Reading the value of a `NULL` option crashed upstream, so it panics in a
/// debug build here: code that runs that early asks `is_uninit` first.
impl StrOpt {
    /// The value as the `char *` the option protocol still speaks, which is
    /// the shared empty string when the option owns nothing.
    ///
    /// The pointer is the option's own buffer and stays live until the
    /// option is written. `crate::option::scope`'s `StrVar` is the seam
    /// where an owned string meets that protocol; the handful of other
    /// callers are the readers that are asked several times per screen line
    /// and hand the value straight to a C callee (`crate::option::query`'s
    /// `local_or_global_raw`). Deliberately **not** spelled `as_raw`: that
    /// is `GlobalCell`'s escape hatch, and the ratchet counts it by name.
    pub fn value_ptr(self) -> *mut c_char {
        OPTIONS.with_field(self.project, |value| match value {
            Some(s) => s.as_ptr().cast_mut(),
            None => {
                self.read_nothing();
                empty_option()
            }
        })
    }

    /// Whether the option has no value at all yet — upstream's `p_xx ==
    /// NULL`, which is true of every option but 'verbosefile' until
    /// `set_init_1` installs the defaults, and of none after.
    ///
    /// The probe for code that runs early enough to find the table empty
    /// (`buf_copy_options` for the first buffer, `vim_getenv` for
    /// `$VIMRUNTIME`), and the only question such code may ask.
    pub fn is_uninit(self) -> bool {
        OPTIONS.with_field(self.project, Option::is_none) && self.before_defaults()
    }

    /// Whether the option has been left owning no string of its own —
    /// upstream's `p_xx == empty_string_option` (its `is_empty_option` for a
    /// global value).
    ///
    /// It is **not** "the value is empty": an option explicitly set to `""`
    /// owns an empty string and answers false. The save-and-restore idioms
    /// (`substitute()`'s quiet 'cpoptions', `:vimgrep`'s 'switchbuf') lean
    /// on exactly that difference. Nor is it
    /// [`is_uninit`](Self::is_uninit): before the defaults exist an option
    /// is `NULL`, not empty, and this answers false.
    pub fn is_unset(self) -> bool {
        OPTIONS.with_field(self.project, Option::is_none) && !self.before_defaults()
    }

    /// Whether owning nothing means upstream's `NULL`: the defaults are not
    /// in yet, and upstream does not start this option out empty.
    fn before_defaults(self) -> bool {
        !option_defaults_set() && !EMPTY_BEFORE_DEFAULTS.contains(&self)
    }

    /// The `None` arm of every read that hands out a value: `""` stands in
    /// for `empty_string_option`, never for `NULL`. Only that arm checks.
    #[inline]
    fn read_nothing(self) {
        if cfg!(debug_assertions) && self.before_defaults() {
            read_before_defaults(self.at);
        }
    }

    /// The value's first byte, which is its NUL when the option holds
    /// nothing — upstream's `*p_xx`, and a constant-time read.
    ///
    /// This is the accessor a **hot** caller wants for "is this option set
    /// at all": the projecting reader below has to find the value's NUL to
    /// hand out a `&CStr`, which walks the string. Answering `*p_langmap !=
    /// NUL` through the projection cost 616 M instructions on `inbench` --
    /// 1.2 % of the whole benchmark -- because the mapping match asks it
    /// once per typeahead byte per candidate mapping.
    ///
    /// Unlike [`is_unset`](Self::is_unset) this does not distinguish an
    /// option that owns nothing from one explicitly set to `""`; upstream's
    /// `*p_xx == NUL` does not either.
    pub fn first_byte(self) -> u8 {
        OPTIONS.with_field(self.project, |value| match value.as_deref() {
            Some(bytes) => bytes.first().copied().unwrap_or(0),
            None => {
                self.read_nothing();
                0
            }
        })
    }

    /// Whether the value holds `byte` — upstream's
    /// `vim_strchr(p_xx, c) != NULL`, which is how every *letter* option
    /// ('cpoptions', 'shortmess', 'formatoptions', 'backspace') is
    /// queried.
    ///
    /// Like [`first_byte`](Self::first_byte) this exists so a hot caller
    /// does not go through the projecting reader: handing out a `&CStr`
    /// means finding the value's NUL first, and `cpo_has` is asked once
    /// per screen line by `Win::col_off2`. Measured at 45 M instructions
    /// on `scrbench`.
    pub fn has_byte(self, byte: u8) -> bool {
        OPTIONS.with_field(self.project, |value| match value.as_deref() {
            Some(bytes) => bytes.contains(&byte),
            None => {
                self.read_nothing();
                false
            }
        })
    }

    /// A copy of the value, for a caller that needs it to outlive the
    /// borrow.
    pub fn get(self) -> XString {
        OPTIONS.with_field(self.project, |value| match value {
            Some(s) => s.clone(),
            None => {
                self.read_nothing();
                XString::new()
            }
        })
    }

    /// Give the option a new value — or none — and answer what it held.
    /// The move out and the move in are one step, so neither side can free
    /// the other's string.
    pub fn swap(self, value: Option<XString>) -> Option<XString> {
        OPTIONS.replace_field(self.project, value)
    }

    /// Give the option a value of its own, releasing what it held.
    pub fn set(self, value: XString) {
        drop(self.swap(Some(value)));
    }

    /// Leave the option owning nothing, and answer what it held.
    pub fn clear(self) -> Option<XString> {
        self.swap(None)
    }

    /// Put back what [`clear`](Self::clear) answered, releasing whatever is
    /// there now. The other half of the save-and-restore idioms.
    pub fn restore(self, value: Option<XString>) {
        drop(self.swap(value));
    }
}

/// Report a read of an option upstream still had as `NULL`, naming the
/// field. Out of line: it is the panic, and only a debug build reaches it.
#[cold]
#[inline(never)]
fn read_before_defaults(at: usize) -> ! {
    panic!(
        "`{}` read before `set_init_1` installed the option defaults (upstream's NULL); \
         ask `is_uninit()` first",
        field_name(at)
    )
}

/// The table of fields: one line per option that has a global value.
///
/// The record itself: each line declares one field, the reader everything
/// outside this module reads it with, and the selector — which is both
/// what the option table's row holds and how the option is *written*
/// (`P_XX.set(value)`). The reader projects the field directly rather than
/// through the selector, because reads are the hot half: `win_line` asks
/// several options per screen line, and a write is a `:set`.
macro_rules! options {
    (struct $record:ident {
        $(
            $(#[$doc:meta])*
            $field:ident: $ty:ty = $init:expr, $sel:ident: $kind:ident;
        )*
    }) => {
        /// The editor's global option values, one field per option.
        ///
        /// Upstream declares these in `option_vars.h` as one `EXTERN` per
        /// option, and the transpile gave each its own cell: three hundred
        /// globals whose only shared fact — that they are the option table's
        /// storage — nothing stated, and which a caller could take the
        /// address of. They are one record here, behind one cell.
        ///
        /// The fields are private and nothing hands out a reference to the
        /// record. A number or a boolean is copied in and out; a string is
        /// *projected*, so the borrow ends inside the reader's closure and a
        /// value that must outlive it is cloned. Either way the record is
        /// not open across a call, so a `did_set_*` callback that re-enters
        /// the editor cannot be holding the option table while it does.
        pub struct $record {
            $($(#[$doc])* $field: $ty,)*
        }

        /// The one cell. Seeded with each field's zero — a string option
        /// owns nothing until startup installs the table's defaults through
        /// `crate::option::set_init_1`, and may not be read until then (see
        /// [`StrOpt`]).
        static OPTIONS: GlobalCell<$record> = GlobalCell::new($record {
            $($field: $init,)*
        });

        /// The field at offset `at`, by name, for [`read_before_defaults`].
        #[cold]
        fn field_name(at: usize) -> &'static str {
            $(
                if at == offset_of!($record, $field) {
                    return stringify!($field);
                }
            )*
            "?"
        }

        $(
            $(#[$doc])*
            ///
            /// The selector: what the option table's row holds, and how
            /// the option is written (`.set(value)`). `pub` because the
            /// unit suite sets a handful of options directly.
            pub const $sel: $kind = $kind {
                at: offset_of!($record, $field),
                project: |o| &mut o.$field,
            };

            reader! {
                $(#[$doc])*
                $kind, $sel, $field, $ty
            }
        )*
    };
}

/// The reader for one field, whose shape follows the field's type: a number
/// or a boolean is answered, a string is projected into a closure.
macro_rules! reader {
    ($(#[$doc:meta])* StrOpt, $sel:ident, $field:ident, $ty:ty) => {
        $(#[$doc])*
        ///
        /// The projecting accessor: `f` sees the value and the borrow ends
        /// when it returns. [`StrOpt::get`] on the selector is the copy, for
        /// a value that has to outlive the call.
        ///
        /// **It is not free.** Handing out a `&CStr` means finding the
        /// value's NUL, so this walks the string; upstream's `char *` was
        /// already measured. A caller in a loop that only wants the first
        /// byte, or "does this option hold anything", wants
        /// [`StrOpt::first_byte`] or [`StrOpt::is_unset`] on the selector
        /// instead -- both constant time.
        #[inline(always)]
        pub fn $field<R>(f: impl FnOnce(&CStr) -> R) -> R {
            OPTIONS.with_field(|o| &mut o.$field, |value| {
                f(match value {
                    Some(s) => s.as_cstr(),
                    None => {
                        $sel.read_nothing();
                        c""
                    }
                })
            })
        }
    };
    ($(#[$doc:meta])* $kind:ident, $sel:ident, $field:ident, $ty:ty) => {
        $(#[$doc])*
        ///
        /// The cheap accessor: one load, no selector in the way. `pub`
        /// for the same reason the selector is, and because a table
        /// declares a reader per row whether or not the tree has a
        /// caller for that row today.
        #[inline(always)]
        pub fn $field() -> $ty {
            OPTIONS.get_field(|o| &mut o.$field)
        }
    };
}

"##;

/// The record of global option values: one field, one selector and one pair
/// of accessors per option whose row names a variable.
///
/// The whole thing is one `options!` invocation, so it is emitted as one
/// file rather than chunked — a macro call cannot be cut in half.
fn emit_vars(out: &mut String, opts: &[Opt]) {
    out.push_str(VARS_SUPPORT.trim_start_matches('\n'));
    out.push_str("options! {\n    struct Options {\n");
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for o in opts {
        let Some(var) = o.varname.as_deref() else {
            continue;
        };
        if !seen.insert(var) {
            continue;
        }
        // A string option owns nothing until startup gives it a value: it is
        // upstream's NULL until then, and `StrOpt` panics on a read of it
        // (see `EMPTY_BEFORE_DEFAULTS` for the exception).
        let (kind, ty, init) = match o.ty.as_str() {
            "boolean" => ("BoolOpt", "bool", "false"),
            "number" => ("NumOpt", "OptInt", "0"),
            _ => ("StrOpt", "Option<XString>", "None"),
        };
        writeln!(out, "        /// `'{}'`.", o.full_name).unwrap();
        writeln!(
            out,
            "        {var}: {ty} = {init}, {}: {kind};",
            var.to_uppercase()
        )
        .unwrap();
    }
    out.push_str("    }\n}\n");
    emit_empty_before_defaults(out, opts);
}

/// The options upstream declares as `empty_string_option` rather than
/// leaving `NULL` until `set_init_1`, because something reads them first.
/// 'verbosefile' is the one: its `option_vars.h` declaration says it is
/// "used before options are initialized".
const EMPTY_BEFORE_DEFAULTS: &[&str] = &["verbosefile"];

/// The generated half of [`EMPTY_BEFORE_DEFAULTS`]: the same list as
/// selectors, which `StrOpt` exempts from its read-before-defaults check.
fn emit_empty_before_defaults(out: &mut String, opts: &[Opt]) {
    let sels: Vec<String> = EMPTY_BEFORE_DEFAULTS
        .iter()
        .map(|name| {
            let o = opts
                .iter()
                .find(|o| o.full_name == *name)
                .unwrap_or_else(|| panic!("no option '{name}'"));
            o.varname
                .as_deref()
                .unwrap_or_else(|| panic!("'{name}' has no global variable"))
                .to_uppercase()
        })
        .collect();
    out.push_str(
        "\n/// The string options upstream starts out empty rather than `NULL`,\n\
         /// because something reads them before the defaults are in.\n",
    );
    writeln!(
        out,
        "const EMPTY_BEFORE_DEFAULTS: [StrOpt; {}] = [{}];",
        sels.len(),
        sels.join(", ")
    )
    .unwrap();
}

/// One row of the table, as a struct literal that fills in only what differs
/// from `BLANK`.
fn emit_row(out: &mut String, o: &Opt) {
    writeln!(out, "    // '{}'", o.full_name).unwrap();
    writeln!(out, "    VimOption {{").unwrap();
    writeln!(
        out,
        "        fullname: name(c\"{}\"),",
        c_literal(&o.full_name)
    )
    .unwrap();
    if let Some(abbr) = &o.abbreviation {
        writeln!(out, "        shortname: name(c\"{}\"),", c_literal(abbr)).unwrap();
    }
    if !o.flags.is_empty() {
        writeln!(out, "        flags: {},", o.flags.join(" | ")).unwrap();
    }
    if o.ty != "boolean" {
        writeln!(out, "        type_0: kOptValType{},", titlecase(&o.ty)).unwrap();
    }
    let scopes: Vec<String> = o.scope.iter().map(|s| s.to_uppercase()).collect();
    writeln!(out, "        scope_flags: {},", scopes.join(" | ")).unwrap();
    let idx: Vec<String> = SCOPE_IDX_ORDER
        .iter()
        .map(|scope| match o.scope_idx.contains_key(*scope) {
            true => scope_const(scope, &o.full_name),
            false => scope_const(scope, "invalid"),
        })
        .collect();
    writeln!(out, "        scope_idx: scope_idx({}),", idx.join(", ")).unwrap();
    match &o.varname {
        Some(var) => writeln!(
            out,
            "        var: OptVar::{}({}),",
            titlecase(&o.ty),
            var.to_uppercase()
        )
        .unwrap(),
        None if o.immutable => writeln!(out, "        var: OptVar::OwnDefault,").unwrap(),
        None => {}
    }
    if let Some(flags_var) = &o.flags_varname {
        writeln!(out, "        flags_var: Some(&{flags_var}),").unwrap();
    }
    if o.immutable {
        writeln!(out, "        immutable: true,").unwrap();
    }
    if !o.values.is_empty() {
        writeln!(out, "        values: &{}_values,", o.values_prefix()).unwrap();
    }
    if let Some(cb) = &o.did_set_cb {
        writeln!(out, "        opt_did_set_cb: Some({cb}),").unwrap();
    }
    if let Some(cb) = &o.expand_cb {
        writeln!(out, "        opt_expand_cb: Some({cb}),").unwrap();
    }
    match &o.def_val {
        Default::Bool(false) if o.ty == "boolean" => {}
        Default::Bool(b) => writeln!(out, "        def_val: boolean({b}),").unwrap(),
        Default::Int(n) => writeln!(out, "        def_val: number({n}),").unwrap(),
        Default::Str(s) => writeln!(out, "        def_val: string(c\"{s}\"),").unwrap(),
        Default::Macro(name, "number") => {
            writeln!(out, "        def_val: number({name} as OptInt),").unwrap()
        }
        Default::Macro(name, _) => writeln!(out, "        def_val: string({name}),").unwrap(),
    }
    writeln!(out, "        ..BLANK").unwrap();
    writeln!(out, "    }},").unwrap();
}

/// Every name an option answers to -> its index.
fn emit_lookup(out: &mut String, opts: &[Opt]) -> Result<(), String> {
    let mut names: BTreeMap<&str, &Opt> = BTreeMap::new();
    for o in opts {
        for name in &o.names {
            if let Some(other) = names.insert(name, o) {
                return Err(format!(
                    "'{}' and '{}' both answer to `{name}`",
                    other.full_name, o.full_name
                ));
            }
        }
    }
    out.push_str(
        "/// The index of the option `name` spells — its full name, its\n\
         /// abbreviation, or one of its aliases.\n\
         pub fn find_option_index(name: &[u8]) -> OptIndex {\n\
         \x20   match name {\n",
    );
    for (name, o) in &names {
        writeln!(out, "        b\"{name}\" => {},", o.index_const()).unwrap();
    }
    out.push_str("        _ => kOptInvalid,\n    }\n}\n");
    Ok(())
}

/// The module's own support code: the blank row every entry builds on and
/// the constructors that keep a row to a readable handful of lines.
const SUPPORT: &str = r#"
/// `scope_flags` bits.
const GLOBAL: OptScopeFlags = 1 << kOptScopeGlobal;
const WIN: OptScopeFlags = 1 << kOptScopeWin;
const BUF: OptScopeFlags = 1 << kOptScopeBuf;

/// A row with every field at rest: a global-scoped, mutable, flagless
/// boolean that defaults to false and belongs to no scope. Each row below
/// fills in what it needs and takes the rest from here.
const BLANK: VimOption = VimOption {
    fullname: ptr::null_mut(),
    shortname: ptr::null_mut(),
    flags: 0,
    type_0: kOptValTypeBoolean,
    scope_flags: 0,
    var: OptVar::NoGlobal,
    flags_var: None,
    scope_idx: scope_idx(kGlobalOptInvalid, kWinOptInvalid, kBufOptInvalid),
    immutable: false,
    values: &[],
    opt_did_set_cb: None,
    opt_expand_cb: None,
    def_val: boolean(false),
};

const fn name(s: &'static CStr) -> *mut c_char {
    s.as_ptr().cast_mut()
}

const fn scope_idx(
    global: GlobalOptIndex,
    win: WinOptIndex,
    buf: BufOptIndex,
) -> [ssize_t; 3] {
    [global as ssize_t, win as ssize_t, buf as ssize_t]
}

const fn boolean(value: bool) -> OptVal {
    // A default is never the third state -- an option's own default is
    // always a value, never "not set in this scope".
    OptVal::Boolean(Some(value))
}

const fn number(value: OptInt) -> OptVal {
    OptVal::Number(value)
}

const fn string(value: &'static CStr) -> OptVal {
    OptVal::static_string(value)
}

/// Copy one generated part into the table under construction.
const fn fill(table: &mut [VimOption], base: usize, part: &[VimOption]) -> usize {
    let mut i = 0;
    while i < part.len() {
        table[base + i] = part[i];
        i += 1;
    }
    base + part.len()
}
"#;

/// Everything the generated module needs from the rest of the crate.
fn imports(out: &mut String, opts: &[Opt], symbols: &Symbols) -> Result<(), String> {
    let mut wanted: BTreeSet<&str> = [
        "kOptScopeGlobal",
        "kOptScopeWin",
        "kOptScopeBuf",
        "kOptValTypeBoolean",
        "kOptValTypeNumber",
        "kOptValTypeString",
    ]
    .into_iter()
    .collect();
    for o in opts {
        wanted.extend(o.flags.iter().map(String::as_str));
        wanted.extend(o.flags_varname.as_deref());
        wanted.extend(o.did_set_cb.as_deref());
        wanted.extend(o.expand_cb.as_deref());
        if let Default::Macro(name, _) = &o.def_val {
            wanted.insert(name);
        }
    }
    let mut by_module: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for name in wanted {
        by_module
            .entry(symbols.path(name)?)
            .or_default()
            .insert(name);
    }

    let mut ffi = vec!["CStr", "c_char", "c_int"];
    if opts.iter().any(|o| !o.flag_enum.is_empty()) {
        ffi.push("c_uint");
    }
    ffi.sort_by_key(|name| (name.starts_with("c_"), *name));
    writeln!(out, "use core::ffi::{{{}}};", ffi.join(", ")).unwrap();
    out.push_str("use core::ptr;\n\n");
    out.push_str("use crate::global_cell::ConstTable;\n");
    for (module, names) in &by_module {
        writeln!(
            out,
            "use {module}::{{{}}};",
            names.iter().copied().collect::<Vec<_>>().join(", ")
        )
        .unwrap();
    }
    let types = [
        "OptIndex",
        "OptInt",
        "OptScopeFlags",
        "OptVal",
        "OptVar",
        "ssize_t",
        "VimOption",
    ];
    writeln!(out, "use crate::types::{{{}}};", types.join(", ")).unwrap();
    Ok(())
}

pub fn generate(
    root: &Path,
    lua_path: &Path,
    out_dir: &Path,
    config: &Path,
) -> Result<Vec<Emitted>, String> {
    let source =
        std::fs::read_to_string(lua_path).map_err(|e| format!("{}: {e}", lua_path.display()))?;
    let opts = parse_options(&source)?;
    // c2rust copied the option-index constants into every module that
    // included the generated header; `option.rs` is their canonical home.
    let symbols = Symbols::collect(root, out_dir, &["crate::option"])?;

    // Each section is formatted as one batch so the chunker counts the lines
    // the file will actually have.
    let mut sections: Vec<(&str, String)> = Vec::new();

    let mut text = String::new();
    emit_index(&mut text, &opts);
    for chunk in chunked(&rustfmt(config, &text)?) {
        sections.push(("index", chunk));
    }

    // Upstream's `gen_vars` wrote the flag enums ahead of the value arrays;
    // both describe what a string option's value may say.
    let mut text = String::new();
    emit_flags(&mut text, &opts);
    for chunk in chunked(&rustfmt(config, &text)?) {
        sections.push(("flags", chunk));
    }

    let mut text = String::new();
    emit_values(&mut text, &opts);
    for chunk in chunked(&rustfmt(config, &text)?) {
        sections.push(("values", chunk));
    }

    let mut text = String::new();
    emit_lookup(&mut text, &opts)?;
    for chunk in chunked(&rustfmt(config, &text)?) {
        sections.push(("lookup", chunk));
    }

    // The record of option values is one `options!` invocation, so it is a
    // section of its own: the chunker splits at item boundaries and a macro
    // call has none inside it. The driver formats every file it is handed.
    let mut text = String::new();
    emit_vars(&mut text, &opts);
    sections.push(("vars", text));

    // The table itself is one array literal, so it is formatted whole and
    // then cut at row boundaries into `PART` constants the parent splices.
    let mut text = format!("const ALL: [VimOption; {}] = [\n", opts.len());
    for o in &opts {
        emit_row(&mut text, o);
    }
    text.push_str("];\n");
    let rows = split_rows(&rustfmt(config, &text)?);
    if rows.len() != opts.len() {
        return Err(format!(
            "the formatted table has {} rows, expected {}",
            rows.len(),
            opts.len()
        ));
    }
    let mut parts: Vec<Vec<&String>> = vec![Vec::new()];
    let mut lines = 0;
    for row in &rows {
        let n = row.lines().count();
        if lines + n > crate::CHUNK_BUDGET - 30 {
            parts.push(Vec::new());
            lines = 0;
        }
        lines += n;
        parts.last_mut().unwrap().push(row);
    }

    let mut files: Vec<Emitted> = Vec::new();
    let mut part = 0;
    for (i, (stem, chunk)) in sections.iter().enumerate() {
        part = if i > 0 && sections[i - 1].0 == *stem {
            part + 1
        } else {
            1
        };
        let name = if part == 1 {
            format!("{stem}.rs")
        } else {
            format!("{stem}_{part}.rs")
        };
        let what = match *stem {
            "index" => "The option indices: which row is which option.",
            "flags" => "The flag enums a string option's value parses into.",
            "values" => "The values each string option accepts.",
            "vars" => "Where every option's global value lives.",
            _ => "The name lookup: which option a spelling means.",
        };
        files.push(Emitted {
            name,
            text: format!("{}{chunk}", child_header(what, chunk)),
        });
    }
    for (n, rows) in parts.iter().enumerate() {
        let mut body = format!(
            "/// This file's run of the option table, spliced in by the parent.\n\
             pub(super) const PART: [VimOption; {}] = [\n",
            rows.len()
        );
        for row in rows {
            body.push_str(row);
        }
        body.push_str("];\n");
        files.push(Emitted {
            name: format!("table_{}.rs", n + 1),
            text: format!(
                "{}{body}",
                child_header("A run of the option table's rows.", &body)
            ),
        });
    }

    let mut out = String::from(HEADER);
    out.push('\n');
    for file in &files {
        let stem = file.name.strip_suffix(".rs").unwrap();
        // `vars` is named from outside (`crate::options::vars`); the rest are
        // chunks this module splices back together.
        let vis = if stem == "vars" { "pub " } else { "" };
        writeln!(out, "{vis}mod {stem};").unwrap();
    }
    out.push('\n');
    for file in &files {
        let stem = file.name.strip_suffix(".rs").unwrap();
        if !stem.starts_with("table_") {
            writeln!(out, "pub use self::{stem}::*;").unwrap();
        }
    }
    out.push('\n');
    imports(&mut out, &opts, &symbols)?;
    out.push_str(SUPPORT);
    out.push_str(
        "\n/// Every option there is. A row's position is its `OptIndex`.\n\
         ///\n\
         /// Immutable, and in `.rodata`: what changes about an option while\n\
         /// the editor runs lives in `crate::option::state` instead.\n\
         pub static options: ConstTable<[VimOption; kOptCount as usize]> =\n\
         \x20   ConstTable::new(table());\n\
         \n\
         /// The table, spliced together from the generated parts. Public to\n\
         /// the crate because `crate::option::state` seeds each option's\n\
         /// starting default from the row that declares it.\n\
         pub(crate) const fn table() -> [VimOption; kOptCount as usize] {\n\
         \x20   let mut table = [BLANK; kOptCount as usize];\n\
         \x20   let mut base = 0;\n",
    );
    for n in 0..parts.len() {
        writeln!(
            out,
            "    base = fill(&mut table, base, &table_{}::PART);",
            n + 1
        )
        .unwrap();
    }
    out.push_str(
        "    assert!(base == kOptCount as usize);\n\
         \x20   let mut i = 0;\n\
         \x20   while i < table.len() {\n\
         \x20       assert!(\n\
         \x20           table[i].var.agrees_with(table[i].type_0),\n\
         \x20           \"an option's variable is not the type its row declares\"\n\
         \x20       );\n\
         \x20       i += 1;\n\
         \x20   }\n\
         \x20   table\n}\n",
    );

    files.insert(
        0,
        Emitted {
            name: "mod.rs".into(),
            text: out,
        },
    );
    Ok(files)
}

/// Cut a formatted `[ .. ]` array literal into its elements. A row opens with
/// its own comment at the array's indentation.
fn split_rows(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut started = false;
    for line in text.lines() {
        if line.starts_with("    // '") {
            out.push(format!("{line}\n"));
            started = true;
            continue;
        }
        if !started || line == "];" {
            continue;
        }
        let row = out.last_mut().unwrap();
        row.push_str(line);
        row.push('\n');
    }
    out
}
