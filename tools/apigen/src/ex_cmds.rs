//! The Ex command table, from `crates/nvim/src/ex_cmds.lua`.
//!
//! Upstream's `src/gen/gen_ex_cmds.lua` (tag v0.12.4) turned that file into
//! two generated headers: the `CmdIdx` enum, and `cmdnames[]` plus the two
//! precomputed first-letter/second-letter offset tables `find_ex_command`
//! walks. c2rust saw only the *output*, so the port inherited all three as
//! hand-maintained source with a comment saying "Generated; regenerate with
//! the table" and no generator behind it.
//!
//! This is that generator. It emits the same two artifacts as Rust:
//!
//! - `ex_docmd/cmdtable.rs` — `command_count`, `cmdnames`, `cmdidxs1`,
//!   `cmdidxs2`
//! - `types/cmdidx.rs` — every `CMD_*`, in enum order
//!
//! Their contents cannot drift apart afterwards, which is the point: a
//! `CMD_*` is an *index* into `cmdnames`, and `cmdidxs1`/`cmdidxs2` are
//! offsets into it, so a row added in the wrong place silently misdirects
//! every command lookup past it.
//!
//! `ex_cmds.lua` has two other consumers — `test_cmdmods.vim` `loadfile()`s
//! it for the modifier list, and `scripts/gen.sh` feeds it to
//! `gen_vimvim.lua` for the vim syntax file — so it is read here, never
//! rewritten.

use std::collections::BTreeMap;
use std::path::Path;

use crate::lua::{Table, Value, fold_int, read_int_locals, read_table};

/// The bit order flags are spelled in. Upstream's C output wrote the numeric
/// value; the port spells the names, and the only stable order for them is
/// the one the bits are in — the Lua lists each row's flags in whatever order
/// its author found readable.
const LOW_BIT_FIRST: fn(&(String, i64)) -> i64 = |(_, bits)| *bits;

/// One row of `ex_cmds.lua`'s `M.cmds`.
struct Row {
    command: String,
    enum_name: String,
    flags: i64,
    addr_type: String,
    func: String,
    preview_func: Option<String>,
}

fn field<'a>(row: &'a Table, key: &str, command: &str) -> Result<&'a str, String> {
    row.str(key)
        .ok_or_else(|| format!("ex_cmds.lua: `{command}` has no string `{key}`"))
}

fn parse(source: &str) -> Result<Vec<Row>, String> {
    let consts = read_int_locals("ex_cmds.lua", source)?;
    let cmds = read_table("ex_cmds.lua", source, "M.cmds")?;
    let flags = read_table("ex_cmds.lua", source, "M.flags")?;
    let named = |key: &str| -> Result<i64, String> {
        flags
            .get(key)
            .and_then(|v| fold_int(v, &consts))
            .ok_or_else(|| format!("ex_cmds.lua: `M.flags.{key}` is not an integer"))
    };
    // The three flags whose invariants the generator asserts, read from the
    // table the Lua exports them in rather than from the private locals.
    let (range, dflall, preview) = (named("RANGE")?, named("DFLALL")?, named("PREVIEW")?);

    let mut out = Vec::with_capacity(cmds.array.len());
    for item in &cmds.array {
        let Value::Table(row) = item else {
            return Err("ex_cmds.lua: a command entry is not a table".into());
        };
        let command = field(row, "command", "?")?.to_string();
        let flags = row
            .get("flags")
            .and_then(|v| fold_int(v, &consts))
            .ok_or_else(|| format!("ex_cmds.lua: `{command}` has no integer `flags`"))?;
        let addr_type = field(row, "addr_type", &command)?.to_string();
        let preview_func = row.str("preview_func").map(str::to_string);

        // Upstream's four assertions, kept because they are the only thing
        // standing between a typo in the Lua and a command whose range is
        // read against the wrong address space.
        if flags & range == range {
            if addr_type == "ADDR_NONE" {
                return Err(format!(
                    "ex_cmds.lua: `{command}` uses RANGE with ADDR_NONE"
                ));
            }
        } else if addr_type != "ADDR_NONE" {
            return Err(format!("ex_cmds.lua: `{command}` is missing ADDR_NONE"));
        }
        if flags & dflall == dflall && matches!(addr_type.as_str(), "ADDR_OTHER" | "ADDR_NONE") {
            return Err(format!("ex_cmds.lua: `{command}` misplaces DFLALL"));
        }
        if flags & preview == preview && preview_func.is_none() {
            return Err(format!("ex_cmds.lua: `{command}` is missing preview_func"));
        }

        out.push(Row {
            enum_name: row
                .str("enum")
                .map_or_else(|| format!("CMD_{command}"), str::to_string),
            func: field(row, "func", &command)?.to_string(),
            command,
            flags,
            addr_type,
            preview_func,
        });
    }
    Ok(out)
}

/// `Ex::A.or(Ex::B)...`, low bit first, over the single-bit locals the Lua
/// declares. A flag value with no name behind it is an error, not a number
/// quietly left out of the list.
///
/// `.or` rather than `|`: the table is a `static` built by a `const fn`, and
/// a `const fn` cannot call the `BitOr` impl. `Ex` is the table's own alias
/// for `ExArgt`, which keeps a twelve-flag row about as wide as the `EX_A |
/// EX_B` spelling it replaces.
fn spell_flags(flags: i64, bits: &[(String, i64)]) -> Result<String, String> {
    let mut names = Vec::new();
    let mut seen = 0;
    for (name, bit) in bits {
        if flags & bit != 0 {
            names.push(format!("Ex::{name}"));
            seen |= bit;
        }
    }
    if seen != flags {
        return Err(format!(
            "ex_cmds.lua: flag bits {:#x} have no name in the file",
            flags & !seen
        ));
    }
    match names.split_first() {
        None => Ok("Ex::NONE".to_string()),
        Some((first, rest)) => Ok(rest
            .iter()
            .fold(first.clone(), |acc, n| format!("{acc}.or({n})"))),
    }
}

/// The single-bit `local`s, low bit first. `FILES`/`WORD1`/`FILE1` are
/// combinations and are filtered out here, so a row that used one still comes
/// out spelled as its parts.
fn single_bits(source: &str) -> Result<Vec<(String, i64)>, String> {
    let mut bits: Vec<(String, i64)> = read_int_locals("ex_cmds.lua", source)?
        .into_iter()
        .filter(|(_, v)| v.count_ones() == 1)
        .collect();
    bits.sort_by_key(LOW_BIT_FIRST);
    Ok(bits)
}

/// `ex_cmds.lua`'s `ADDR_*` spelling as the crate's `CmdAddr` variant.
///
/// The table is `#[rustfmt::skip]`, one row per line, so the `Ad::` alias is
/// worth six characters a row against `CmdAddr::`'s eleven.
fn spell_addr(addr_type: &str) -> Result<String, String> {
    let variant = match addr_type {
        "ADDR_LINES" => "Lines",
        "ADDR_WINDOWS" => "Windows",
        "ADDR_ARGUMENTS" => "Arguments",
        "ADDR_LOADED_BUFFERS" => "LoadedBuffers",
        "ADDR_BUFFERS" => "Buffers",
        "ADDR_TABS" => "Tabs",
        "ADDR_TABS_RELATIVE" => "TabsRelative",
        "ADDR_QUICKFIX_VALID" => "QuickfixValid",
        "ADDR_QUICKFIX" => "Quickfix",
        "ADDR_UNSIGNED" => "Unsigned",
        "ADDR_OTHER" => "Other",
        "ADDR_NONE" => "NoRange",
        other => return Err(format!("ex_cmds.lua: unknown addr_type `{other}`")),
    };
    Ok(format!("Ad::{variant}"))
}

const TABLE_DOC: &str = r#"//! GENERATED by tools/apigen from `crate::ex_cmds.lua` -- the same
//! metadata upstream's `src/gen/gen_ex_cmds.lua` consumed. Do not edit; run
//! `just apigen` (`just apigen --check` fails on drift).
//!
//! `cmdnames` is the whole of `:`. Each row names the command, the handler,
//! the `'inccommand'` preview handler if it has one, the [`ExArgt`] flags
//! saying what syntax it accepts (aliased to `Ex` here, and `|`'d with
//! `.or()` because a `const fn` cannot call `BitOr`), and the [`CmdAddr`]
//! address space its range is counted in (aliased to `Ad`).
//!
//! **The order is load-bearing.** A `CMD_*` (see
//! [`crate::types::cmdidx`]) is an index into this array, and
//! `cmdidxs1`/`cmdidxs2` are precomputed offsets into it that
//! `find_ex_command` uses to skip straight to the first command with a given
//! first, or first two, letters. All three come out of the same pass over
//! `ex_cmds.lua`, so they cannot disagree.
"#;

const HELPERS: &str = r#"
/// One row of the Ex command table, spelled the way `ex_cmds.lua` spells it.
///
/// c2rust wrote each of these as a twelve-line struct literal whose
/// `cmd_func` went through a transmute from `ex_func_T` to `ex_func_T` --
/// a no-op that cost three `unsafe ` tokens a row.
const fn cmd<const N: usize>(
    name: &'static [u8; N],
    func: unsafe fn(*mut exarg_T),
    argt: ExArgt,
    addr: CmdAddr,
) -> CommandDefinition {
    CommandDefinition {
        cmd_name: name.as_ptr() as *mut c_char,
        cmd_func: Some(func),
        cmd_preview_func: None,
        cmd_argt: argt,
        cmd_addr_type: addr,
    }
}

/// A row whose command also has a 'inccommand' preview implementation.
const fn cmd_pv<const N: usize>(
    name: &'static [u8; N],
    func: unsafe fn(*mut exarg_T),
    preview: unsafe fn(*mut exarg_T, c_int, handle_T) -> c_int,
    argt: ExArgt,
    addr: CmdAddr,
) -> CommandDefinition {
    CommandDefinition {
        cmd_name: name.as_ptr() as *mut c_char,
        cmd_func: Some(func),
        cmd_preview_func: Some(preview),
        cmd_argt: argt,
        cmd_addr_type: addr,
    }
}
"#;

/// Rust keywords that an Ex command is also named. A variant keeps the
/// command's own spelling (see [`ENUM_DOC`]), so these twelve need the raw
/// form. None of them is one of the four names (`crate`, `self`, `Self`,
/// `super`) a raw identifier may not be.
const KEYWORDS: &[&str] = &[
    "break", "const", "continue", "else", "for", "if", "let", "match", "move", "return", "try",
    "while",
];

/// `CmdIdx::r#if`, `CmdIdx::append`.
fn variant(enum_name: &str) -> String {
    let tail = enum_name.strip_prefix("CMD_").unwrap_or(enum_name);
    if KEYWORDS.contains(&tail) {
        format!("r#{tail}")
    } else {
        tail.to_string()
    }
}

const ENUM_DOC: &str = r#"//! GENERATED by tools/apigen from `crate::ex_cmds.lua` -- the same
//! metadata upstream's `src/gen/gen_ex_cmds.lua` consumed. Do not edit; run
//! `just apigen` (`just apigen --check` fails on drift).
//!
//! Which Ex command a `:` line names, in enum order: each built-in variant's
//! discriminant is its row in the `cmdnames` table generated from the same
//! file, which is why the two are emitted together. c2rust rendered the C
//! enumeration as a `typedef`ed `int` plus 560 `const`s, rendered each
//! `switch (cmdidx)` arm as a bare integer comparison, and thirty-one modules
//! then re-declared whichever names they happened to need.
//!
//! **A variant is spelled the way the command is**, `append` and `Next` both,
//! rather than converted to Rust's convention: `:next` and `:Next` are
//! different commands, twelve pairs differ only in case, and capitalising
//! would collide them. Twelve more are Rust keywords and take the raw form
//! (`CmdIdx::r#if`).
//!
//! `USER`/`USER_BUF` are the two negative members (a user command, global or
//! buffer-local); `SIZE` is the count of built-ins, and doubles as the "not a
//! command" marker.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use ::core::ffi::c_int;
"#;

const ENUM_HEAD: &str = r#"
/// The command a parsed `:` line names.
///
/// `#[allow(non_camel_case_types)]` because a variant is the command, spelled
/// as the user types it -- see the module doc.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
#[repr(i32)]
pub enum CmdIdx {
"#;

const ENUM_IMPL: &str = r#"
/// Hand-written rather than derived, and `#[inline(always)]`, because the
/// derived `eq` is an ordinary call at `-O0` -- which is what both test
/// suites build -- and roughly a thousand sites compare a `cmdidx` against a
/// command, several of them per Ex command executed.
impl PartialEq for CmdIdx {
    #[inline(always)]
    fn eq(&self, other: &CmdIdx) -> bool {
        self.code() == other.code()
    }
}

impl Eq for CmdIdx {}

impl CmdIdx {
    /// The `int` upstream stores in `exarg_T.cmdidx`.
    ///
    /// A `const fn` so that a bound reaches a `const` item rather than being
    /// recomputed per iteration: an enum-to-integer conversion is a call at
    /// `-O0`, which is what both test suites build.
    #[inline(always)]
    pub const fn code(self) -> c_int {
        self as c_int
    }

    /// This command's row in `cmdnames`.
    ///
    /// # Panics
    ///
    /// If asked of `USER`, `USER_BUF` or `SIZE`, none of which names a row.
    /// Every caller has already ruled those out -- `is_user_cmd` for the
    /// first two, a comparison against `SIZE` for the third -- so the panic
    /// is a bug in the caller, not a case to handle.
    #[inline(always)]
    pub fn index(self) -> usize {
        assert!(self.code() < CmdIdx::SIZE.code(), "{self:?} names no command");
        usize::try_from(self.code()).expect("a built-in command's index is not negative")
    }

    /// The command occupying row `index` of `cmdnames`, or `SIZE` past the
    /// end -- which is what a scan that found nothing wants to store.
    ///
    /// Not `at`: `:@` is a command, so `CmdIdx::at` is a variant.
    #[inline(always)]
    pub fn at_row(index: usize) -> CmdIdx {
        BUILTINS.get(index).copied().unwrap_or(CmdIdx::SIZE)
    }
}
"#;

/// `(cmdtable.rs, cmdidx.rs)`.
pub fn generate(lua_path: &Path) -> Result<(String, String), String> {
    let source =
        std::fs::read_to_string(lua_path).map_err(|e| format!("{}: {e}", lua_path.display()))?;
    let rows = parse(&source)?;
    let bits = single_bits(&source)?;
    let n = rows.len();

    let mut table = String::from(TABLE_DOC);
    table.push_str(
        "\n#![forbid(unsafe_code)]\n\n#[allow(unused_imports)]\nuse super::*;\n\
         use crate::global_cell::ConstTable;\n\
         use crate::types::CmdAddr as Ad;\n\
         use crate::types::ExArgt as Ex;\n",
    );
    table.push_str(HELPERS);
    table.push_str(&format!("\npub(crate) const command_count: c_int = {n};\n"));
    table.push_str(&format!(
        "\n#[rustfmt::skip]\npub(crate) static cmdnames: ConstTable<[CommandDefinition; {n}]> = ConstTable::new([\n"
    ));
    for row in &rows {
        let flags = spell_flags(row.flags, &bits)?;
        let (name, func) = (&row.command, &row.func);
        let addr = spell_addr(&row.addr_type)?;
        table.push_str(&match &row.preview_func {
            Some(preview) => {
                format!("    cmd_pv(b\"{name}\\0\", {func}, {preview}, {flags}, {addr}),\n")
            }
            None => format!("    cmd(b\"{name}\\0\", {func}, {flags}, {addr}),\n"),
        });
    }
    table.push_str("]);\n");

    // Where the linear scan over `cmdnames` starts, by first letter and then
    // by the first two. Only the a-z rows take part: `:Next` and the
    // punctuation commands are found by the fallback scan.
    let alpha: Vec<char> = ('a'..='z').collect();
    let mut idx1: BTreeMap<char, usize> = BTreeMap::new();
    let mut idx2: BTreeMap<char, BTreeMap<char, usize>> = BTreeMap::new();
    let lower: Vec<(usize, &str)> = rows
        .iter()
        .map(|r| r.command.as_str())
        .enumerate()
        .filter(|(_, c)| c.starts_with(|c: char| c.is_ascii_lowercase()))
        // The rows are contiguous, so re-index against that run rather than
        // against `cmdnames` -- upstream builds `cmds` the same way.
        .enumerate()
        .map(|(i, (_, c))| (i, c))
        .collect();
    for (i, command) in lower.iter().rev() {
        let mut chars = command.chars();
        let first = chars.next().expect("a command name is never empty");
        idx1.insert(first, *i);
        match chars.next() {
            Some(second) if second.is_ascii_lowercase() => {
                idx2.entry(first).or_default().insert(second, *i);
            }
            _ => {}
        }
    }

    table.push_str(
        "\n/// For each letter a-z, the index of the first command in `cmdnames`\n\
         /// that starts with it.\n\
         pub(crate) static cmdidxs1: [uint16_t; 26] = [\n",
    );
    for c in &alpha {
        table.push_str(&format!("    {},\n", idx1[c]));
    }
    table.push_str("];\n");

    table.push_str(
        "\n/// For each pair of letters, the offset from `cmdidxs1` of the first\n\
         /// command starting with them. Zero means there is none, which the\n\
         /// caller reads as \"start where the first letter says\".\n\
         pub(crate) static cmdidxs2: [[uint8_t; 26]; 26] = [\n",
    );
    for c in &alpha {
        let row: Vec<String> = alpha
            .iter()
            .map(|d| {
                idx2.get(c)
                    .and_then(|m| m.get(d))
                    .map_or(0, |at| at - idx1[c])
                    .to_string()
            })
            .collect();
        table.push_str(&format!("    [{}],\n", row.join(", ")));
    }
    table.push_str("];\n");

    let mut enums = String::from(ENUM_DOC);
    enums.push_str(ENUM_HEAD);
    enums.push_str("    USER_BUF = -2,\n    USER = -1,\n");
    for (i, row) in rows.iter().enumerate() {
        enums.push_str(&format!("    {} = {i},\n", variant(&row.enum_name)));
    }
    enums.push_str(&format!("    SIZE = {n},\n}}\n"));
    enums.push_str(&ENUM_IMPL.replace("{n}", &n.to_string()));

    // The rows in table order, so that a scan over `cmdnames` can name what
    // it stopped at without a 560-arm `match`. Six to a line: one per line
    // would put the file over the ratchet's cap on its own.
    enums.push_str(&format!(
        "\n/// `cmdnames` in table order, indexed by [`CmdIdx::at_row`].\n\
         #[rustfmt::skip]\n\
         static BUILTINS: [CmdIdx; {n}] = [\n"
    ));
    for chunk in rows.chunks(6) {
        let names: Vec<String> = chunk
            .iter()
            .map(|row| format!("CmdIdx::{}", variant(&row.enum_name)))
            .collect();
        enums.push_str(&format!("    {},\n", names.join(", ")));
    }
    enums.push_str("];\n");

    Ok((table, enums))
}
