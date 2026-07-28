//! The Vimscript builtin-function table, from `crates/nvim/src/nvim/eval.lua`.
//!
//! Upstream's `src/gen/gen_eval.lua` (tag v0.12.4) built one `EvalFuncDef[]`
//! out of two sources, and so does this:
//!
//! * `eval.lua`'s `M.funcs` — every builtin written in C, with its arity, the
//!   argument a `base->method()` call fills in, whether it may run during a
//!   fast event, and which function implements it. Entries whose key ends in
//!   `__` or `__<n>` are documentation-only overloads and take no row.
//! * the API spec — every RPC method that is neither Lua-only nor
//!   remote-only also answers to its own name in Vimscript, dispatched
//!   through its row of the handler table. Those row numbers used to be
//!   literals in the transpiled table, cross-checked at generation time;
//!   here both tables come out of [`crate::handler_rows`], so they cannot
//!   drift apart in the first place.
//!
//! The row order is `table_order` over the names *sorted*, the same layout
//! upstream's `hashy` produced and the same idiom the handler table uses. It
//! is not load-bearing — nothing indexes this table by number — but keeping
//! it makes the generated table diffable against the transpiled one.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use crate::lua::{Table, Value, read_table};
use crate::options::Symbols;
use crate::{ApiFn, Emitted, Param, Spec, chunked, handler_rows, rustfmt, table_order};

/// `MAX_FUNC_ARGS` — the ceiling an open-ended `args = { n }` means.
const MAX_FUNC_ARGS: u8 = 20;

/// What a row's `data` union holds.
#[derive(Clone)]
enum Data {
    /// Nothing: the function reads its arguments and nothing else.
    None,
    /// A libm operation on one Float, called through `float_op_wrapper`.
    Float(String),
    /// A row of the RPC handler table, called through `api_wrapper`.
    Handler(usize),
}

/// One row of the builtin table.
#[derive(Clone)]
struct Builtin {
    name: String,
    min_argc: u8,
    max_argc: u8,
    /// The 1-based argument a method call supplies, or 0 for `BASE_NONE`.
    base_arg: u8,
    fast: bool,
    /// The Rust function the row calls.
    func: String,
    data: Data,
}

/// Read `args = 1` / `args = { 1, 3 }` / `args = { 2 }` / absent.
fn arity(entry: &Table, name: &str) -> Result<(u8, u8), String> {
    let small = |n: i64| {
        u8::try_from(n).map_err(|_| format!("eval.lua: {name} has an out-of-range arity {n}"))
    };
    match entry.get("args") {
        None => Ok((0, 0)),
        Some(Value::Int(n)) => Ok((small(*n)?, small(*n)?)),
        Some(Value::Table(t)) => {
            let min = match t.array.first() {
                Some(Value::Int(n)) => small(*n)?,
                None => 0,
                Some(other) => {
                    return Err(format!(
                        "eval.lua: {name}'s min arity is {}",
                        other.describe()
                    ));
                }
            };
            let max = match t.array.get(1) {
                Some(Value::Int(n)) => small(*n)?,
                None => MAX_FUNC_ARGS,
                Some(other) => {
                    return Err(format!(
                        "eval.lua: {name}'s max arity is {}",
                        other.describe()
                    ));
                }
            };
            Ok((min, max))
        }
        Some(other) => Err(format!("eval.lua: {name}'s `args` is {}", other.describe())),
    }
}

/// The builtins `eval.lua` describes, in no particular order.
fn parse_builtins(source: &str) -> Result<Vec<Builtin>, String> {
    let funcs = read_table("eval.lua", source, "M.funcs")?;
    let mut out = Vec::new();
    for (name, value) in &funcs.map {
        // `get__1`, `nvim_api__`: extra documentation entries for an
        // overloaded signature. Upstream dropped them from the table.
        if name
            .rsplit_once("__")
            .is_some_and(|(_, tail)| tail.chars().all(|c| c.is_ascii_digit()))
        {
            continue;
        }
        let Value::Table(entry) = value else {
            return Err(format!("eval.lua: `{name}` is not a table"));
        };
        let (min_argc, max_argc) = arity(entry, name)?;
        let base_arg = match entry.get("base") {
            None => 0,
            Some(Value::Int(n)) => u8::try_from(*n)
                .map_err(|_| format!("eval.lua: {name} has an out-of-range `base` {n}"))?,
            Some(other) => {
                return Err(format!("eval.lua: {name}'s `base` is {}", other.describe()));
            }
        };
        let (func, data) = match entry.str("float_func") {
            // `float_func` names the libm function; the table entry calls the
            // wrapper that unpacks the Float argument and boxes the result.
            Some(op) => ("float_op_wrapper".to_string(), Data::Float(op.to_string())),
            None => (
                entry
                    .str("func")
                    .map_or_else(|| format!("f_{name}"), str::to_string),
                Data::None,
            ),
        };
        out.push(Builtin {
            name: name.clone(),
            min_argc,
            max_argc,
            base_arg,
            fast: entry.truthy("fast"),
            func,
            data,
        });
    }
    Ok(out)
}

/// The rows the API contributes: one per method with a Vimscript binding,
/// taking exactly the arguments the method publishes.
fn api_builtins(api: &BTreeMap<String, ApiFn>, specs: &[Spec]) -> Result<Vec<Builtin>, String> {
    let rows = handler_rows(specs);
    let mut out = Vec::new();
    for spec in specs.iter().filter(|s| s.has_eval_binding()) {
        let f = api
            .get(&spec.name)
            .ok_or_else(|| format!("{}: no such API function in the crate", spec.name))?;
        let argc = f
            .params
            .iter()
            .filter(|p| matches!(p, Param::Value { .. }))
            .count();
        let argc = u8::try_from(argc)
            .map_err(|_| format!("{} takes more arguments than a builtin can", spec.name))?;
        let row = rows[spec.name.as_str()];
        out.push(Builtin {
            name: spec.name.clone(),
            min_argc: argc,
            max_argc: argc,
            base_arg: 0,
            fast: false,
            func: "api_wrapper".into(),
            data: Data::Handler(row),
        });
    }
    Ok(out)
}

/// The whole table, in row order.
fn layout(
    api: &BTreeMap<String, ApiFn>,
    specs: &[Spec],
    eval_lua: &str,
) -> Result<Vec<Builtin>, String> {
    let mut all = parse_builtins(eval_lua)?;
    for row in api_builtins(api, specs)? {
        // An API binding wins over an `eval.lua` entry of the same name, as
        // upstream's `funcs[fun.name] = ...` did.
        all.retain(|b| b.name != row.name);
        all.push(row);
    }
    all.sort_by(|a, b| a.name.cmp(&b.name));
    let names: Vec<String> = all.iter().map(|b| b.name.clone()).collect();
    Ok(table_order(&names)
        .into_iter()
        .map(|i| all[i].clone())
        .collect())
}

const HEADER: &str = r#"//! The Vimscript builtin-function table.
//!
//! GENERATED by tools/apigen from `crate::src::nvim::eval.lua` and the API
//! spec — the same two sources upstream's `src/gen/gen_eval.lua` consumed.
//! Do not edit; run `just apigen` (`just apigen --check` fails on drift).
//!
//! A row says what `abs()` or `nvim_get_mode()` *is* to the evaluator: the
//! name it answers to, how many arguments it takes, which of them a
//! `base->method()` call supplies, whether it may run during a fast event,
//! and the function that does the work. The bodies live beside this module,
//! in `crate::src::nvim::eval::funcs` and the modules it shares the work
//! with.
//!
//! The last row is blank on purpose: completion walks the table until it
//! reaches a null name.

#![forbid(unsafe_code)]
"#;

fn child_header(what: &str) -> String {
    format!(
        "//! {what}\n\
         //!\n\
         //! GENERATED by tools/apigen; see the parent module. Do not edit;\n\
         //! run `just apigen`.\n\
         \n\
         #![forbid(unsafe_code)]\n\
         \n\
         // A chunk may hold nothing that needs the parent's support code.\n\
         #[allow(unused_imports)]\n\
         use super::*;\n\
         \n"
    )
}

/// The module's own support code: the blank row every entry builds on and the
/// three shapes a row comes in.
const SUPPORT: &str = r#"
/// `base_arg` for a function that cannot be used as a method.
const BASE_NONE: u8 = 0;

/// A row with every field at rest: nameless, argumentless, not a method, not
/// fast, and calling nothing. It is also the table's terminator.
const BLANK: EvalFuncDef = EvalFuncDef {
    name: ptr::null_mut(),
    min_argc: 0,
    max_argc: 0,
    base_arg: BASE_NONE,
    fast: false,
    func: None,
    data: EvalFuncData { null: ptr::null_mut() },
};

/// A builtin with a function of its own.
const fn builtin(
    name: &'static CStr,
    min_argc: u8,
    max_argc: u8,
    base_arg: u8,
    func: VimLFunc,
) -> EvalFuncDef {
    EvalFuncDef {
        name: name.as_ptr().cast_mut(),
        min_argc,
        max_argc,
        base_arg,
        func,
        ..BLANK
    }
}

/// The same, for one that may also run during a fast event.
const fn fast(
    name: &'static CStr,
    min_argc: u8,
    max_argc: u8,
    base_arg: u8,
    func: VimLFunc,
) -> EvalFuncDef {
    EvalFuncDef {
        fast: true,
        ..builtin(name, min_argc, max_argc, base_arg, func)
    }
}

/// A libm operation on one Float argument.
const fn float(name: &'static CStr, op: FloatFunc) -> EvalFuncDef {
    EvalFuncDef {
        data: EvalFuncData { float_func: op },
        ..builtin(name, 1, 1, 1, Some(float_op_wrapper))
    }
}

/// An API method, called through its row of the RPC handler table.
const fn api(name: &'static CStr, argc: u8, row: usize) -> EvalFuncDef {
    EvalFuncDef {
        data: EvalFuncData {
            api_handler: method_handlers
                .as_raw()
                .cast::<MsgpackRpcRequestHandler>()
                .wrapping_add(row)
                .cast_const(),
        },
        ..builtin(name, argc, argc, BASE_NONE, Some(api_wrapper))
    }
}

/// Copy one generated run of rows into place.
const fn fill(table: &mut [EvalFuncDef], base: usize, part: &[EvalFuncDef]) -> usize {
    let mut i = 0;
    while i < part.len() {
        table[base + i] = part[i];
        i += 1;
    }
    base + part.len()
}
"#;

/// One row, as the call that builds it.
fn emit_row(out: &mut String, b: &Builtin) {
    let name = format!("c\"{}\"", b.name);
    match &b.data {
        Data::Float(op) => writeln!(out, "    float({name}, Some({op})),").unwrap(),
        Data::Handler(row) => writeln!(out, "    api({name}, {}, {row}),", b.min_argc).unwrap(),
        Data::None => {
            let base = match b.base_arg {
                0 => "BASE_NONE".to_string(),
                n => n.to_string(),
            };
            let shape = if b.fast { "fast" } else { "builtin" };
            writeln!(
                out,
                "    {shape}({name}, {}, {}, {base}, Some({})),",
                b.min_argc, b.max_argc, b.func
            )
            .unwrap();
        }
    }
}

/// The name lookup. A byte-string pattern lowers to a switch on the length
/// and a decision tree over the bytes, which is what upstream's perfect hash
/// plus its confirming `memcmp` amounted to.
fn emit_lookup(out: &mut String, rows: &[Builtin]) {
    out.push_str(
        "/// The row of [`BUILTINS`] the builtin called `name` sits in.\n\
         pub fn builtin_index(name: &[u8]) -> Option<usize> {\n\
         \x20   Some(match name {\n",
    );
    let mut sorted: Vec<(usize, &Builtin)> = rows.iter().enumerate().collect();
    sorted.sort_by(|a, b| a.1.name.cmp(&b.1.name));
    for (row, b) in sorted {
        writeln!(out, "        b\"{}\" => {row},", b.name).unwrap();
    }
    out.push_str("        _ => return None,\n    })\n}\n");
}

/// The `use` lines: whatever functions the rows name, grouped by the module
/// that defines them, plus the fixed support imports.
fn imports(out: &mut String, rows: &[Builtin], symbols: &Symbols) -> Result<(), String> {
    let mut wanted: BTreeSet<&str> = BTreeSet::new();
    for b in rows {
        wanted.insert(b.func.as_str());
        if let Data::Float(op) = &b.data {
            wanted.insert(op);
        }
    }
    let mut by_module: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for name in wanted {
        by_module
            .entry(symbols.path(name)?)
            .or_default()
            .insert(name);
    }
    out.push_str("use core::ffi::CStr;\n");
    out.push_str("use core::ptr;\n\n");
    out.push_str("use crate::src::nvim::api::private::dispatch::method_handlers;\n");
    out.push_str("use crate::src::nvim::global_cell::GlobalCell;\n");
    for (module, names) in &by_module {
        writeln!(
            out,
            "use {module}::{{{}}};",
            names.iter().copied().collect::<Vec<_>>().join(", ")
        )
        .unwrap();
    }
    out.push_str(
        "use crate::src::nvim::types::{\n\
         \x20   EvalFuncData, EvalFuncDef, FloatFunc, MsgpackRpcRequestHandler, VimLFunc,\n\
         };\n",
    );
    Ok(())
}

pub fn generate(
    root: &Path,
    api: &BTreeMap<String, ApiFn>,
    specs: &[Spec],
    lua_path: &Path,
    out_dir: &Path,
    config: &Path,
) -> Result<Vec<Emitted>, String> {
    let source =
        std::fs::read_to_string(lua_path).map_err(|e| format!("{}: {e}", lua_path.display()))?;
    let rows = layout(api, specs, &source)?;
    // The wrappers and the float operations live where the rest of the
    // transpiled crate put them; `f_*` bodies are spread over the eval
    // modules. Nothing here is copied into more than one module, so no
    // preference is needed to resolve a name.
    let symbols = Symbols::collect(root, out_dir, &[])?;

    let mut files: Vec<Emitted> = Vec::new();

    let mut text = String::new();
    emit_lookup(&mut text, &rows);
    for (i, chunk) in chunked(&rustfmt(config, &text)?).into_iter().enumerate() {
        let name = match i {
            0 => "lookup.rs".to_string(),
            n => format!("lookup_{}.rs", n + 1),
        };
        files.push(Emitted {
            name,
            text: format!(
                "{}{chunk}",
                child_header("The name lookup: which row a spelling means.")
            ),
        });
    }

    // The table is one array literal, so it is formatted whole and then cut
    // at row boundaries into `PART` constants the parent splices.
    let mut text = format!("const ALL: [EvalFuncDef; {}] = [\n", rows.len());
    for b in &rows {
        emit_row(&mut text, b);
    }
    text.push_str("];\n");
    let formatted = rustfmt(config, &text)?;
    let items = split_rows(&formatted);
    if items.len() != rows.len() {
        return Err(format!(
            "the formatted table has {} rows, expected {}",
            items.len(),
            rows.len()
        ));
    }
    let mut parts: Vec<Vec<&String>> = vec![Vec::new()];
    let mut lines = 0;
    for row in &items {
        let n = row.lines().count();
        if lines + n > crate::CHUNK_BUDGET - 30 {
            parts.push(Vec::new());
            lines = 0;
        }
        lines += n;
        parts.last_mut().unwrap().push(row);
    }
    for (n, part) in parts.iter().enumerate() {
        // A slice rather than an array: 643 rows of a 32-byte struct is over
        // clippy's `large_const_arrays` threshold, and a `static` could not be
        // read back by the `const fn` that splices the parts together.
        let mut body = format!(
            "/// This file's run of the builtin table ({} rows), spliced in by\n\
             /// the parent.\n\
             pub(super) const PART: &[EvalFuncDef] = &[\n",
            part.len()
        );
        for row in part {
            body.push_str(row);
        }
        body.push_str("];\n");
        files.push(Emitted {
            name: format!("table_{}.rs", n + 1),
            text: format!(
                "{}{body}",
                child_header("A run of the builtin table's rows.")
            ),
        });
    }

    let mut out = String::from(HEADER);
    out.push('\n');
    for file in &files {
        writeln!(out, "mod {};", file.name.strip_suffix(".rs").unwrap()).unwrap();
    }
    out.push('\n');
    for file in &files {
        let stem = file.name.strip_suffix(".rs").unwrap();
        if !stem.starts_with("table_") {
            writeln!(out, "pub use self::{stem}::*;").unwrap();
        }
    }
    out.push('\n');
    imports(&mut out, &rows, &symbols)?;
    out.push_str(SUPPORT);
    writeln!(
        out,
        "\n/// Every builtin Vimscript function there is, plus the blank row\n\
         /// that ends the table.\n\
         pub static BUILTINS: GlobalCell<[EvalFuncDef; {}]> = GlobalCell::new(table());\n\
         \n\
         /// The table, spliced together from the generated parts.\n\
         const fn table() -> [EvalFuncDef; {}] {{\n\
         \x20   let mut table = [BLANK; {}];\n\
         \x20   let mut base = 0;",
        rows.len() + 1,
        rows.len() + 1,
        rows.len() + 1
    )
    .unwrap();
    for n in 0..parts.len() {
        writeln!(
            out,
            "    base = fill(&mut table, base, table_{}::PART);",
            n + 1
        )
        .unwrap();
    }
    writeln!(out, "    assert!(base == {});\n    table\n}}", rows.len()).unwrap();

    files.insert(
        0,
        Emitted {
            name: "mod.rs".into(),
            text: out,
        },
    );
    Ok(files)
}

/// Cut a formatted `[ .. ]` array literal into its elements. Every row opens
/// with one of the constructor names at the array's indentation.
fn split_rows(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        let opens = ["    builtin(", "    fast(", "    float(", "    api("]
            .iter()
            .any(|p| line.starts_with(p));
        if opens {
            out.push(format!("{line}\n"));
            continue;
        }
        if out.is_empty() || line == "];" {
            continue;
        }
        let row = out.last_mut().unwrap();
        row.push_str(line);
        row.push('\n');
    }
    out
}
