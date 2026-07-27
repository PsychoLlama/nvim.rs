// apigen: generate the msgpack-RPC dispatch layer from the Rust sources
// themselves.
//
// Upstream generated `dispatch_wrappers.generated.h` and
// `keysets_defs.generated.h` by parsing the C headers under src/nvim/api/
// (src/gen/gen_api_dispatch.lua at tag v0.12.4). The transpile froze that
// output into one 27k-line Rust module. This tool takes the job back, from
// the two real sources of truth in the crate:
//
//   --out-dir     one wrapper per `pub unsafe extern "C" fn nvim_*` in
//                 <root>/src/nvim/api/*.rs: it validates an `Array` of
//                 msgpack arguments, converts them, calls the API function
//                 and boxes the result back into an `Object`.
//   --tables-dir  the keyset tables and their key lookups, read off the
//                 `KeyDict_*` structs in <root>/src/nvim/types/keysets.rs,
//                 plus the handler table and its method lookup.
//   --lua-dir     the `vim.api` Lua binding: the same conversion job again,
//                 against the Lua stack rather than an argument `Array`, plus
//                 the table that hangs the bindings off their names.
//
// A signature alone does not say everything upstream's `FUNC_API_*` markers
// said: whether a call is refused under textlock, which declared C type name
// appears in a "Wrong type for argument" message ("ArrayOf(Integer, 2)" is a
// plain `Array` in Rust), whether a handler may run on the fast path, whether
// its result is heap-allocated, which of the two callers a method is reachable
// from, and which methods are deprecated spellings of which. That lives in the
// spec file (`--spec`), one line per method. The
// inputs cross-check: a spec entry naming a function that no longer exists,
// or whose declared parameter count disagrees with the Rust signature, is a
// hard error, and so is a handler-table layout that has drifted out from
// under the row numbers eval/funcs.rs has baked in.
//
// Output is committed, rustfmt'd Rust: module directories whose roots hold
// the shared support code and whose children hold the bulk, split when a file
// would pass the tree's 1,000-line cap. `--check` re-generates in memory and
// diffs against the committed files, so drift is a build failure rather than
// a silent hazard (scripts/gen-api-dispatch.sh, `just apigen`).
//
// Build with the repo dev shell; syn is pinned exactly, as in tools/ffigen.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------- model

/// What an API parameter is, once the special ones the dispatcher supplies
/// itself have been recognised.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Param {
    /// The calling channel's id. Supplied by the dispatcher, never by the
    /// client; recognised by name because its type (`uint64_t`) is not
    /// distinctive.
    ChannelId,
    /// The request arena. Supplied by the dispatcher.
    Arena,
    /// The out-parameter the API function reports failure through.
    Error,
    /// A Lua state. Only the Lua binding has one; the RPC wrapper passes null.
    LuaState,
    /// A value the client sends, at 0-based position `index` of the argument
    /// array.
    Value {
        index: usize,
        ty: ApiType,
        /// What to call the parameter when a conversion fails. The Lua
        /// binding names it in the message; the RPC wrapper numbers the
        /// argument instead.
        name: String,
    },
}

/// The msgpack-visible type of a client-supplied parameter.
#[derive(Clone, Debug, PartialEq, Eq)]
enum ApiType {
    Boolean,
    Integer,
    Float,
    String,
    Array,
    Dict,
    Object,
    LuaRef,
    /// `Buffer`, `Window` or `Tabpage`: an `Object` tag of its own, carried in
    /// the integer slot of the union.
    Handle(&'static str),
    /// A `*mut KeyDict_<name>` options struct, filled from a Dict argument.
    KeyDict(String),
}

impl ApiType {
    /// The name that appears in a "Wrong type for argument N" message when the
    /// spec does not override it.
    fn declared(&self) -> String {
        match self {
            ApiType::Boolean => "Boolean".into(),
            ApiType::Integer => "Integer".into(),
            ApiType::Float => "Float".into(),
            ApiType::String => "String".into(),
            ApiType::Array => "Array".into(),
            ApiType::Dict => "Dict".into(),
            ApiType::Object => "Object".into(),
            ApiType::LuaRef => "LuaRef".into(),
            ApiType::Handle(name) => (*name).into(),
            ApiType::KeyDict(name) => format!("Dict({name}) *"),
        }
    }
}

/// What an API function hands back, as far as boxing it into an `Object` goes.
#[derive(Clone, Debug, PartialEq, Eq)]
enum RetType {
    Void,
    Boolean,
    Integer,
    Float,
    String,
    Array,
    Dict,
    Object,
    Handle(&'static str),
    KeyDict(String),
}

/// One `pub unsafe extern "C" fn nvim_*` as parsed out of the crate.
struct ApiFn {
    name: String,
    /// Module path segment under `crate::src::nvim::api::` (the file stem).
    module: String,
    params: Vec<Param>,
    ret: RetType,
}

/// One line of the spec file: everything the signature cannot tell us.
#[derive(Default)]
struct Spec {
    /// The name the method answers to over RPC. Usually also the name of the
    /// API function that implements it.
    name: String,
    /// Refuse the call while the text is locked (upstream `FUNC_API_TEXTLOCK`).
    textlock: bool,
    /// Refuse under textlock, but tolerate the command-line window
    /// (`FUNC_API_TEXTLOCK_ALLOW_CMDWIN`).
    textlock_allow_cmdwin: bool,
    /// Declared C type names for arguments whose Rust type has lost the
    /// decoration, keyed by 1-based argument position: `arg1=ArrayOf(Integer, 2)`.
    declared: BTreeMap<usize, String>,
    /// The handler may run straight from the RPC read callback instead of
    /// being deferred to the main loop (`FUNC_API_FAST`).
    fast: bool,
    /// The result is heap-allocated and the caller frees it, rather than
    /// belonging to the request arena.
    ret_alloc: bool,
    /// This method is a deprecated spelling of another one. It has no wrapper
    /// of its own: the handler table points it at that method's, and it
    /// inherits its `fast` and `ret_alloc`.
    alias: Option<String>,
    /// A hand-written handler elsewhere in the crate, named by its full path.
    /// Mutually exclusive with `alias`; either means no wrapper is generated.
    handler: Option<String>,
    /// RPC clients only: the method gets no Lua binding.
    remote_only: bool,
    /// Lua only: the method gets no RPC wrapper and no handler-table row.
    lua_only: bool,
    /// The API level the method appeared at, or -1 for the internal `nvim__*`
    /// ones. Only the Lua binding reads it, and only to decide how to convert
    /// the result — see [`Spec::push_special`].
    since: Option<i32>,
}

impl Spec {
    /// Whether this entry gets an RPC wrapper of its own.
    fn is_wrapper(&self) -> bool {
        self.alias.is_none() && self.handler.is_none() && !self.lua_only
    }

    /// Whether this entry is one of the methods the RPC dispatcher answers
    /// to, and so takes a row in the handler table.
    fn is_method(&self) -> bool {
        !self.lua_only
    }

    /// Whether this entry gets a `vim.api.<name>` Lua binding. A deprecated
    /// spelling does not: upstream exposed the old names over RPC only.
    fn has_lua_binding(&self) -> bool {
        self.alias.is_none() && self.handler.is_none() && !self.remote_only
    }

    /// Whether the Lua binding converts this method's result the pre-0.11 way
    /// — `nil` and the other special values keep their old spelling. Upstream
    /// froze that for everything that predates API level 11 because clients
    /// depend on it, and used the modern conversion for newer methods.
    fn push_special(&self) -> bool {
        self.since.is_some_and(|since| since < 11)
    }
}

/// One key of a keyset, as the tables see it.
struct Key {
    /// The name clients spell it with.
    wire: String,
    /// The Rust field it fills in.
    field: String,
    /// `KeySetLink::type_0`, as one of the `TAG_*` constants the generated
    /// module defines.
    tag: &'static str,
    /// A highlight-group name, which the converter resolves to an id.
    is_hlgroup: bool,
}

/// One `KeyDict_*` struct: an options dict the API takes by name.
struct Keyset {
    name: String,
    /// In table order, which is not declaration order — see [`table_order`].
    keys: Vec<Key>,
    /// The keyset leads with an `is_set__<name>_` mask, so each key gets an
    /// `opt_index` naming its bit.
    has_optional: bool,
}

impl Keyset {
    /// Elements of the emitted table: one per key plus the null terminator.
    fn len(&self) -> usize {
        self.keys.len() + 1
    }
}

// ---------------------------------------------------------------- parsing

/// Last path segment of a type, e.g. `Array` for `crate::…::Array`.
fn type_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(p) => Some(p.path.segments.last()?.ident.to_string()),
        _ => None,
    }
}

/// For `*mut T` / `*const T`, the name of `T`.
fn pointee_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Ptr(p) => type_name(&p.elem),
        _ => None,
    }
}

fn value_type(ty: &syn::Type) -> Option<ApiType> {
    if let Some(pointee) = pointee_name(ty) {
        return pointee
            .strip_prefix("KeyDict_")
            .map(|k| ApiType::KeyDict(k.to_string()));
    }
    Some(match type_name(ty)?.as_str() {
        "Boolean" => ApiType::Boolean,
        "Integer" => ApiType::Integer,
        "Float" => ApiType::Float,
        "String_0" => ApiType::String,
        "Array" => ApiType::Array,
        "Dict" => ApiType::Dict,
        "Object" => ApiType::Object,
        "LuaRef" => ApiType::LuaRef,
        "Buffer" => ApiType::Handle("Buffer"),
        "Window" => ApiType::Handle("Window"),
        "Tabpage" => ApiType::Handle("Tabpage"),
        _ => return None,
    })
}

fn ret_type(ret: &syn::ReturnType) -> Option<RetType> {
    let ty = match ret {
        syn::ReturnType::Default => return Some(RetType::Void),
        syn::ReturnType::Type(_, ty) => ty,
    };
    let name = type_name(ty)?;
    if let Some(keyset) = name.strip_prefix("KeyDict_") {
        return Some(RetType::KeyDict(keyset.to_string()));
    }
    Some(match name.as_str() {
        "Boolean" => RetType::Boolean,
        "Integer" => RetType::Integer,
        "Float" => RetType::Float,
        "String_0" => RetType::String,
        "Array" => RetType::Array,
        "Dict" => RetType::Dict,
        "Object" => RetType::Object,
        "Buffer" => RetType::Handle("Buffer"),
        "Window" => RetType::Handle("Window"),
        "Tabpage" => RetType::Handle("Tabpage"),
        _ => return None,
    })
}

/// Classify a signature's parameters, in declaration order. The dispatcher
/// supplies `channel_id`, the arena, the error slot and (for functions with a
/// Lua implementation) a null `lua_State`; everything else comes off the wire.
fn classify(sig: &syn::Signature) -> Result<Vec<Param>, String> {
    let mut params = Vec::new();
    let mut index = 0;
    for arg in &sig.inputs {
        let syn::FnArg::Typed(arg) = arg else {
            return Err("method receiver in an API function".into());
        };
        let name = match &*arg.pat {
            syn::Pat::Ident(id) => id.ident.to_string(),
            _ => String::new(),
        };
        let param = match pointee_name(&arg.ty).as_deref() {
            Some("Arena") => Param::Arena,
            Some("Error") => Param::Error,
            Some("lua_State") => Param::LuaState,
            // Some channel-id parameters are unused by their function and
            // carry the leading underscore that says so.
            _ if name == "channel_id" || name == "_channel_id" => Param::ChannelId,
            _ => {
                let ty = value_type(&arg.ty)
                    .ok_or_else(|| format!("parameter `{name}` has an unmapped type"))?;
                let param = Param::Value {
                    index,
                    ty,
                    // c2rust suffixed any C name that is a Rust keyword or
                    // collides with something in scope (`type` -> `type_0`,
                    // `fn` -> `fn_0`, `msg` -> `msg_0`); no API parameter is
                    // spelled with that suffix, so undoing it recovers the
                    // name the API documents.
                    name: name.trim_start_matches('_').trim_end_matches("_0").into(),
                };
                index += 1;
                param
            }
        };
        params.push(param);
    }
    Ok(params)
}

/// Collect every `pub unsafe extern "C" fn` in `<root>/src/nvim/api/*.rs`.
fn collect_api_fns(root: &Path) -> Result<BTreeMap<String, ApiFn>, String> {
    let dir = root.join("src/nvim/api");
    let mut out = BTreeMap::new();
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "rs"))
        .collect();
    entries.sort();
    for path in entries {
        let module = path.file_stem().unwrap().to_string_lossy().into_owned();
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let file = syn::parse_file(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        for item in &file.items {
            let syn::Item::Fn(f) = item else { continue };
            if !matches!(f.vis, syn::Visibility::Public(_)) || f.sig.unsafety.is_none() {
                continue;
            }
            if !matches!(&f.sig.abi, Some(abi) if abi.name.as_ref().is_none_or(|n| n.value() == "C"))
            {
                continue;
            }
            let name = f.sig.ident.to_string();
            let Ok(params) = classify(&f.sig) else {
                continue;
            };
            let Some(ret) = ret_type(&f.sig.output) else {
                continue;
            };
            out.insert(
                name.clone(),
                ApiFn {
                    name,
                    module: module.clone(),
                    params,
                    ret,
                },
            );
        }
    }
    Ok(out)
}

/// The `TAG_*` constant naming the `ObjectType` a value of this Rust type
/// must arrive as, and whether the type marks a highlight group.
fn key_tag(ty: &str) -> Option<(&'static str, bool)> {
    Some(match ty {
        // A highlight group travels as a name but is stored as the id the
        // converter resolves it to, so its slot is an Integer.
        "HLGroupID" => ("TAG_INTEGER", true),
        "Boolean" => ("TAG_BOOLEAN", false),
        "Integer" => ("TAG_INTEGER", false),
        "Float" => ("TAG_FLOAT", false),
        "String_0" => ("TAG_STRING", false),
        "Array" => ("TAG_ARRAY", false),
        "Dict" => ("TAG_DICT", false),
        "LuaRef" => ("TAG_LUAREF", false),
        "Buffer" => ("TAG_BUFFER", false),
        "Window" => ("TAG_WINDOW", false),
        "Tabpage" => ("TAG_TABPAGE", false),
        // Anything goes.
        "Object" => ("TAG_NIL", false),
        // ShaDa's own unpacked-in-place array of strings.
        "StringArray" => ("TAG_STRING_ARRAY", false),
        _ => return None,
    })
}

/// The wire name a field carries, when a `Wire key: \`name\`.` doc comment
/// says it differs from the Rust one.
fn wire_key_override(attrs: &[syn::Attribute]) -> Option<String> {
    for attr in attrs {
        let syn::Meta::NameValue(nv) = &attr.meta else {
            continue;
        };
        if !nv.path.is_ident("doc") {
            continue;
        }
        let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(s),
            ..
        }) = &nv.value
        else {
            continue;
        };
        let text = s.value();
        let Some(rest) = text.trim().strip_prefix("Wire key:") else {
            continue;
        };
        let name = rest.trim().trim_end_matches('.').trim_matches('`');
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    None
}

/// Read the keysets out of their canonical module. The struct definitions are
/// the source of truth: declaration order fixes the table order and hence
/// every key's `opt_index`, the field type fixes the tag, and a wire name that
/// differs from the Rust field name is recorded in a doc comment there.
fn collect_keysets(root: &Path) -> Result<Vec<Keyset>, String> {
    let path = root.join("src/nvim/types/keysets.rs");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let file = syn::parse_file(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut out = Vec::new();
    for item in &file.items {
        let syn::Item::Struct(s) = item else { continue };
        let Some(name) = s
            .ident
            .to_string()
            .strip_prefix("KeyDict_")
            .map(str::to_string)
        else {
            continue;
        };
        let mut keys = Vec::new();
        let mut has_optional = false;
        for (i, field) in s.fields.iter().enumerate() {
            let field_name = field
                .ident
                .as_ref()
                .ok_or_else(|| format!("KeyDict_{name}: tuple struct"))?
                .to_string();
            let ty = type_name(&field.ty)
                .ok_or_else(|| format!("KeyDict_{name}.{field_name}: unreadable type"))?;
            if field_name == format!("is_set__{name}_") {
                if i != 0 || ty != "OptionalKeys" {
                    return Err(format!(
                        "KeyDict_{name}: is_set__{name}_ must come first and be an OptionalKeys"
                    ));
                }
                has_optional = true;
                continue;
            }
            let (tag, is_hlgroup) = key_tag(&ty)
                .ok_or_else(|| format!("KeyDict_{name}.{field_name}: unmapped type `{ty}`"))?;
            keys.push(Key {
                wire: wire_key_override(&field.attrs).unwrap_or_else(|| field_name.clone()),
                field: field_name,
                tag,
                is_hlgroup,
            });
        }
        let order = table_order(&keys.iter().map(|k| k.wire.clone()).collect::<Vec<_>>());
        let mut ordered: Vec<Option<Key>> = keys.into_iter().map(Some).collect();
        let keys = order
            .into_iter()
            .map(|i| ordered[i].take().expect("each key placed once"))
            .collect();
        out.push(Keyset {
            name,
            keys,
            has_optional,
        });
    }
    Ok(out)
}

/// The order a table's entries go in, which is *not* declaration order.
///
/// Upstream's `src/gen/hashy.lua` grouped the keys by length, picked for each
/// length the character position that splits it most evenly, and laid the
/// table out bucket by bucket so a lookup could jump straight to a short run.
/// The lookup here is a plain `match` on the key bytes instead (see the
/// generated module's header), but the layout has to survive: a key's position
/// is its `opt_index`, the bit it owns in the keyset's `is_set__*_` mask, and
/// call sites all over the crate test those bits by number. The same is true
/// of the handler table, whose indices `eval/funcs.rs` has baked in.
///
/// Returns the permutation: `result[i]` is the index in `names` of the key
/// that belongs at position `i`.
fn table_order(names: &[String]) -> Vec<usize> {
    let max_len = names.iter().map(|s| s.len()).max().unwrap_or(0);
    let mut out = Vec::with_capacity(names.len());
    for len in 1..=max_len {
        let bucket: Vec<usize> = (0..names.len())
            .filter(|&i| names[i].len() == len)
            .collect();
        if bucket.is_empty() {
            continue;
        }
        // The position whose character splits this length's keys into the
        // smallest worst-case group; ties go to the leftmost.
        let mut best: Option<(usize, BTreeMap<u8, Vec<usize>>)> = None;
        let mut best_size = bucket.len() * 2;
        for pos in 0..len {
            let mut split: BTreeMap<u8, Vec<usize>> = BTreeMap::new();
            for &i in &bucket {
                split.entry(names[i].as_bytes()[pos]).or_default().push(i);
            }
            let worst = split.values().map(Vec::len).max().unwrap_or(1).max(1);
            if worst < best_size {
                best_size = worst;
                best = Some((pos, split));
            }
        }
        let (_, split) = best.expect("a non-empty length bucket has a best position");
        if split.len() > 1 {
            // Sorted by the discriminating character, as `hashy.switcher`
            // emitted its `case` labels.
            for group in split.values() {
                out.extend(group);
            }
        } else {
            // One group: upstream took only its first member, which is sound
            // because a single group of one key is all a distinct length can
            // hold once duplicates are ruled out.
            out.push(split.values().next().unwrap()[0]);
        }
    }
    assert_eq!(out.len(), names.len(), "every key lands in the table");
    out
}

fn parse_spec(path: &Path) -> Result<Vec<Spec>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut out = Vec::new();
    for (lineno, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        let at = |msg: String| format!("{}:{}: {msg}", path.display(), lineno + 1);
        let mut words = line.split_whitespace();
        let mut spec = Spec {
            name: words.next().unwrap().to_string(),
            ..Spec::default()
        };
        for word in words {
            match word.split_once('=') {
                None if word == "textlock" => spec.textlock = true,
                None if word == "textlock_allow_cmdwin" => spec.textlock_allow_cmdwin = true,
                None if word == "fast" => spec.fast = true,
                None if word == "ret_alloc" => spec.ret_alloc = true,
                None if word == "remote_only" => spec.remote_only = true,
                None if word == "lua_only" => spec.lua_only = true,
                Some(("alias", value)) => spec.alias = Some(value.to_string()),
                Some(("handler", value)) => spec.handler = Some(value.to_string()),
                Some(("since", value)) => {
                    spec.since = Some(
                        value
                            .parse::<i32>()
                            .map_err(|_| at(format!("bad API level in `{word}`")))?,
                    )
                }
                Some((key, value)) => match key.strip_prefix("arg") {
                    Some(n) => {
                        let n = n
                            .parse::<usize>()
                            .map_err(|_| at(format!("bad argument index in `{word}`")))?;
                        if n == 0 {
                            return Err(at("argument indices are 1-based".into()));
                        }
                        spec.declared.insert(n, value.replace('_', " "));
                    }
                    None => return Err(at(format!("unknown key `{key}`"))),
                },
                None => return Err(at(format!("unknown flag `{word}`"))),
            }
        }
        if spec.textlock && spec.textlock_allow_cmdwin {
            return Err(at("textlock and textlock_allow_cmdwin are exclusive".into()));
        }
        if spec.alias.is_some() && spec.handler.is_some() {
            return Err(at("alias and handler are exclusive".into()));
        }
        if !spec.is_wrapper() && (spec.textlock || spec.textlock_allow_cmdwin) {
            return Err(at("an entry without a wrapper has nothing to lock".into()));
        }
        if spec.alias.is_some() && (spec.fast || spec.ret_alloc) {
            return Err(at(
                "an alias inherits fast/ret_alloc from what it aliases".into()
            ));
        }
        if spec.remote_only && spec.lua_only {
            return Err(at("remote_only and lua_only are exclusive".into()));
        }
        if spec.lua_only && (spec.alias.is_some() || spec.handler.is_some()) {
            return Err(at(
                "a lua_only method is not dispatched, so it has no alias or handler".into(),
            ));
        }
        if spec.has_lua_binding() != spec.since.is_some() {
            return Err(at(
                "since= is required on everything with a Lua binding, and meaningless elsewhere"
                    .into(),
            ));
        }
        out.push(spec);
    }
    let names: BTreeSet<&str> = out.iter().map(|s| s.name.as_str()).collect();
    for spec in &out {
        if let Some(target) = &spec.alias {
            if !names.contains(target.as_str()) {
                return Err(format!(
                    "{}: {} aliases {target}, which is not in the spec",
                    path.display(),
                    spec.name
                ));
            }
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------- codegen

/// The `kObjectType*` constant an `ApiType` arrives tagged with.
fn object_tag(ty: &ApiType) -> &'static str {
    match ty {
        ApiType::Boolean => "kObjectTypeBoolean",
        ApiType::Integer => "kObjectTypeInteger",
        ApiType::Float => "kObjectTypeFloat",
        ApiType::String => "kObjectTypeString",
        ApiType::Array => "kObjectTypeArray",
        ApiType::Dict | ApiType::KeyDict(_) => "kObjectTypeDict",
        ApiType::Object => "kObjectTypeNil",
        ApiType::LuaRef => "kObjectTypeLuaRef",
        ApiType::Handle("Buffer") => "kObjectTypeBuffer",
        ApiType::Handle("Window") => "kObjectTypeWindow",
        ApiType::Handle(_) => "kObjectTypeTabpage",
    }
}

/// The `as_*` reader that turns an `Object` into a parameter, or `None` if the
/// tag does not match.
fn reader(ty: &ApiType) -> String {
    match ty {
        ApiType::Boolean => "as_boolean(item)".into(),
        ApiType::Integer => "as_integer(item)".into(),
        ApiType::Float => "as_float(item)".into(),
        ApiType::String => "as_string(item)".into(),
        ApiType::Array => "as_array(item)".into(),
        ApiType::Dict => "as_dict(item)".into(),
        ApiType::LuaRef => "as_luaref(item)".into(),
        ApiType::Handle(_) => format!("as_handle(item, {})", object_tag(ty)),
        ApiType::Object | ApiType::KeyDict(_) => unreachable!("handled inline"),
    }
}

fn emit_fn(
    out: &mut String,
    f: &ApiFn,
    spec: &Spec,
    tables: &BTreeMap<String, usize>,
) -> Result<(), String> {
    let name = &f.name;
    let handler = format!("handle_{name}");
    let values: Vec<(usize, &ApiType)> = f
        .params
        .iter()
        .filter_map(|p| match p {
            Param::Value { index, ty, .. } => Some((*index, ty)),
            _ => None,
        })
        .collect();
    let arity = values.len();
    let takes_arena = f.params.contains(&Param::Arena);
    let can_fail = f.params.contains(&Param::Error);

    for n in spec.declared.keys() {
        if *n > arity {
            return Err(format!(
                "{name}: spec overrides arg{n} but it takes {arity}"
            ));
        }
    }

    writeln!(out, "pub unsafe extern \"C\" fn {handler}(").unwrap();
    writeln!(out, "    channel_id: uint64_t,").unwrap();
    writeln!(out, "    args: Array,").unwrap();
    writeln!(
        out,
        "    {}arena: *mut Arena,",
        if takes_arena { "" } else { "_" }
    )
    .unwrap();
    writeln!(out, "    error: *mut Error,").unwrap();
    writeln!(out, ") -> Object {{").unwrap();
    writeln!(out, "    unsafe {{").unwrap();
    writeln!(out, "        log_invoke(c\"{handler}\", c\"RPC: ch %lu: invoke {name}\", line!() as c_int, channel_id);").unwrap();
    writeln!(out, "        if args.size != {arity} as size_t {{").unwrap();
    writeln!(out, "            wrong_arity(error, {arity}, args.size);").unwrap();
    writeln!(out, "            return NIL;").unwrap();
    writeln!(out, "        }}").unwrap();

    for (index, ty) in &values {
        let slot = index + 1;
        let declared = spec
            .declared
            .get(&slot)
            .cloned()
            .unwrap_or_else(|| ty.declared());
        let bad = format!(
            "wrong_type(error, {slot}, c\"{name}\", c\"{}\");",
            declared.replace('"', "\\\"")
        );
        match ty {
            // Any Object is acceptable, so there is nothing to check.
            ApiType::Object => {
                writeln!(out, "        let arg_{slot} = *args.items.add({index});").unwrap();
            }
            ApiType::KeyDict(keyset) => {
                let get_field = format!("KeyDict_{keyset}_get_field");
                writeln!(out, "        let item = *args.items.add({index});").unwrap();
                writeln!(
                    out,
                    "        let mut arg_{slot}: KeyDict_{keyset} = core::mem::zeroed();"
                )
                .unwrap();
                writeln!(out, "        if item.type_0 == kObjectTypeDict {{").unwrap();
                writeln!(out, "            if !api_dict_to_keydict(").unwrap();
                writeln!(out, "                (&raw mut arg_{slot}).cast(),").unwrap();
                writeln!(out, "                Some({get_field}),").unwrap();
                writeln!(out, "                item.data.dict,").unwrap();
                writeln!(out, "                error,").unwrap();
                writeln!(out, "            ) {{").unwrap();
                writeln!(out, "                return NIL;").unwrap();
                writeln!(out, "            }}").unwrap();
                writeln!(out, "        }} else if !is_empty_array(item) {{").unwrap();
                writeln!(out, "            {bad}").unwrap();
                writeln!(out, "            return NIL;").unwrap();
                writeln!(out, "        }}").unwrap();
            }
            _ => {
                writeln!(out, "        let item = *args.items.add({index});").unwrap();
                writeln!(out, "        let Some(arg_{slot}) = {} else {{", reader(ty)).unwrap();
                writeln!(out, "            {bad}").unwrap();
                writeln!(out, "            return NIL;").unwrap();
                writeln!(out, "        }};").unwrap();
            }
        }
    }

    if spec.textlock {
        writeln!(out, "        if text_locked() {{").unwrap();
        writeln!(
            out,
            "            api_set_error(error, kErrorTypeException, c\"%s\".as_ptr(), get_text_locked_msg());"
        )
        .unwrap();
        writeln!(out, "            return NIL;").unwrap();
        writeln!(out, "        }}").unwrap();
    } else if spec.textlock_allow_cmdwin {
        writeln!(
            out,
            "        if textlock.get() != 0 || expr_map_locked() {{"
        )
        .unwrap();
        writeln!(
            out,
            "            api_set_error(error, kErrorTypeException, c\"%s\".as_ptr(), &raw const e_textlock);"
        )
        .unwrap();
        writeln!(out, "            return NIL;").unwrap();
        writeln!(out, "        }}").unwrap();
    }

    // The call, with the dispatcher's own values threaded back into the
    // positions the signature puts them in.
    let call_args: Vec<String> = f
        .params
        .iter()
        .map(|p| match p {
            Param::ChannelId => "channel_id".into(),
            Param::Arena => "arena".into(),
            Param::Error => "error".into(),
            Param::LuaState => "core::ptr::null_mut()".into(),
            Param::Value {
                index,
                ty: ApiType::KeyDict(_),
                ..
            } => format!("&raw mut arg_{}", index + 1),
            Param::Value { index, .. } => format!("arg_{}", index + 1),
        })
        .collect();
    let call = format!("{name}({})", call_args.join(", "));
    // An `Object` result needs no boxing, so when nothing follows the call it
    // is the wrapper's tail expression rather than a binding.
    if f.ret == RetType::Object && !can_fail {
        writeln!(out, "        {call}").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(out, "}}").unwrap();
        return Ok(());
    }
    let bind = match &f.ret {
        RetType::Void => "",
        // Converting a keyset result to a Dict takes it by pointer.
        RetType::KeyDict(_) => "let mut rv = ",
        _ => "let rv = ",
    };
    writeln!(out, "        {bind}{call};").unwrap();
    if can_fail {
        writeln!(out, "        if (*error).type_0 != kErrorTypeNone {{").unwrap();
        writeln!(out, "            return NIL;").unwrap();
        writeln!(out, "        }}").unwrap();
    }

    let boxed = match &f.ret {
        RetType::Void => "NIL".into(),
        RetType::Object => "rv".into(),
        RetType::Boolean => "obj(kObjectTypeBoolean, object_data { boolean: rv })".into(),
        RetType::Integer => "obj(kObjectTypeInteger, object_data { integer: rv })".into(),
        RetType::Float => "obj(kObjectTypeFloat, object_data { floating: rv })".into(),
        RetType::String => "obj(kObjectTypeString, object_data { string: rv })".into(),
        RetType::Array => "obj(kObjectTypeArray, object_data { array: rv })".into(),
        RetType::Dict => "obj(kObjectTypeDict, object_data { dict: rv })".into(),
        RetType::Handle(tag) => {
            format!("obj(kObjectType{tag}, object_data {{ integer: rv as Integer }})")
        }
        RetType::KeyDict(keyset) => {
            let size = tables
                .get(keyset.as_str())
                .ok_or_else(|| format!("{name}: no {keyset}_table to bound the conversion"))?;
            format!(
                "obj(kObjectTypeDict, object_data {{ dict: api_keydict_to_dict((&raw mut rv).cast(), {keyset}_table.ptr().cast(), {size} as size_t, arena) }})"
            )
        }
    };
    writeln!(out, "        {boxed}").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}").unwrap();
    Ok(())
}

const HEADER: &str = r#"//! Dispatch wrappers for the msgpack-RPC API.
//!
//! GENERATED by tools/apigen from the `nvim_*` signatures under
//! `crate::src::nvim::api` plus `tools/apigen/functions.txt`. Do not edit;
//! run `just apigen` (`just apigen --check` fails on drift).
//!
//! Each wrapper takes the raw argument `Array` a client sent, checks the
//! arity, converts each element to the parameter type the API function
//! declares — refusing with an `Error` if a tag does not match — calls the
//! function, and boxes whatever came back into an `Object`. A wrapper that
//! refuses returns nil and leaves the reason in `*error`.
//!
//! This module holds the shared support code and the imports; the wrappers
//! themselves live in one child module per API source file, re-exported here
//! so callers see one flat namespace. A source file whose wrappers would
//! overflow the tree's 1,000-line file cap is split into numbered parts.

#![deny(unsafe_op_in_unsafe_fn)]
"#;

/// Header for a child module. Children pull the support code in wholesale:
/// a glob import cannot go stale as the batch changes, and rustc does not
/// lint an unused one.
fn child_header(module: &str, part: usize, parts: usize) -> String {
    let of = if parts > 1 {
        format!(", part {part} of {parts}")
    } else {
        String::new()
    };
    format!(
        "//! Dispatch wrappers for `crate::src::nvim::api::{module}`{of}.\n\
         //!\n\
         //! GENERATED by tools/apigen; see the parent module. Do not edit;\n\
         //! run `just apigen`.\n\
         \n\
         #![deny(unsafe_op_in_unsafe_fn)]\n\
         \n\
         use super::*;\n\
         \n"
    )
}

/// Wrapper lines one child module may hold, leaving room under the ratchet's
/// 1,000-line file cap for the nine-line header. Counted on formatted text,
/// so `run`'s recheck is a backstop rather than the real guard.
const CHUNK_BUDGET: usize = 985;

/// One file of the generated module directory.
struct Emitted {
    /// File name within the output directory.
    name: String,
    text: String,
}

/// The fixed part of the support code: every wrapper needs it.
const SUPPORT: &str = r#"
/// What a wrapper returns when it refused the call, and what a `void` API
/// function's result boxes to.
const NIL: Object = Object {
    type_0: kObjectTypeNil,
    data: object_data { boolean: false },
};

const fn obj(type_0: ObjectType, data: object_data) -> Object {
    Object { type_0, data }
}

/// One "RPC: ch N: invoke nvim_foo" debug line. Below the configured log
/// level — the default — this is a load and a compare.
///
/// # Safety
/// Both strings outlive the call; `logmsg` is variadic C.
unsafe fn log_invoke(handler: &CStr, fmt: &CStr, line: c_int, channel_id: uint64_t) {
    unsafe {
        logmsg(
            LOGLVL_DBG,
            core::ptr::null(),
            handler.as_ptr(),
            line,
            true,
            fmt.as_ptr(),
            channel_id,
        );
    }
}

/// # Safety
/// `error` points at a live `Error`.
unsafe fn wrong_arity(error: *mut Error, expected: usize, got: size_t) {
    unsafe {
        api_set_error(
            error,
            kErrorTypeException,
            c"Wrong number of arguments: expecting %zu but got %zu".as_ptr(),
            expected as size_t,
            got,
        );
    }
}

/// # Safety
/// `error` points at a live `Error`; the names outlive the call.
unsafe fn wrong_type(error: *mut Error, slot: usize, func: &CStr, expected: &CStr) {
    unsafe {
        api_set_error(
            error,
            kErrorTypeException,
            c"Wrong type for argument %zu when calling %s, expecting %s".as_ptr(),
            slot as size_t,
            func.as_ptr(),
            expected.as_ptr(),
        );
    }
}
"#;

/// The argument readers, emitted only when a wrapper uses one. Each turns an
/// `Object` off the wire into a parameter, or `None` if the tag does not
/// match anything the parameter accepts.
const READERS: &[(&str, &str)] = &[
    (
        "as_boolean",
        r#"
/// A nonnegative integer is accepted as a boolean, truncated to C `int`
/// first as the C dispatcher did.
fn as_boolean(o: Object) -> Option<Boolean> {
    match o.type_0 {
        // SAFETY: the tag says which union arm is live.
        kObjectTypeBoolean => Some(unsafe { o.data.boolean }),
        kObjectTypeInteger => match unsafe { o.data.integer } {
            n if n >= 0 => Some(n as handle_T != 0),
            _ => None,
        },
        _ => None,
    }
}
"#,
    ),
    (
        "as_integer",
        r#"
fn as_integer(o: Object) -> Option<Integer> {
    // SAFETY: the tag says which union arm is live.
    (o.type_0 == kObjectTypeInteger).then(|| unsafe { o.data.integer })
}
"#,
    ),
    (
        "as_float",
        r#"
/// Integers widen to floats, as they do in Lua.
fn as_float(o: Object) -> Option<Float> {
    match o.type_0 {
        // SAFETY: the tag says which union arm is live.
        kObjectTypeFloat => Some(unsafe { o.data.floating }),
        kObjectTypeInteger => Some(unsafe { o.data.integer } as Float),
        _ => None,
    }
}
"#,
    ),
    (
        "as_string",
        r#"
fn as_string(o: Object) -> Option<String_0> {
    // SAFETY: the tag says which union arm is live.
    (o.type_0 == kObjectTypeString).then(|| unsafe { o.data.string })
}
"#,
    ),
    (
        "as_array",
        r#"
fn as_array(o: Object) -> Option<Array> {
    // SAFETY: the tag says which union arm is live.
    (o.type_0 == kObjectTypeArray).then(|| unsafe { o.data.array })
}
"#,
    ),
    (
        "as_dict",
        r#"
/// An empty Lua table is indistinguishable from an empty list on the wire, so
/// a Dict parameter accepts one.
fn as_dict(o: Object) -> Option<Dict> {
    match o.type_0 {
        // SAFETY: the tag says which union arm is live.
        kObjectTypeDict => Some(unsafe { o.data.dict }),
        _ if is_empty_array(o) => Some(Dict {
            size: 0,
            capacity: 0,
            items: core::ptr::null_mut(),
        }),
        _ => None,
    }
}
"#,
    ),
    (
        "as_luaref",
        r#"
fn as_luaref(o: Object) -> Option<LuaRef> {
    // SAFETY: the tag says which union arm is live.
    (o.type_0 == kObjectTypeLuaRef).then(|| unsafe { o.data.luaref })
}
"#,
    ),
    (
        "as_handle",
        r#"
/// Buffer, Window and Tabpage each have a tag of their own but travel in the
/// integer arm, and a bare nonnegative integer is accepted as any of them.
fn as_handle(o: Object, tag: ObjectType) -> Option<handle_T> {
    if o.type_0 != tag && o.type_0 != kObjectTypeInteger {
        return None;
    }
    // SAFETY: every accepted tag carries an integer.
    match unsafe { o.data.integer } {
        n if n >= 0 => Some(n as handle_T),
        _ => None,
    }
}
"#,
    ),
    (
        "is_empty_array",
        r#"
fn is_empty_array(o: Object) -> bool {
    // SAFETY: the tag says which union arm is live.
    o.type_0 == kObjectTypeArray && unsafe { o.data.array }.size == 0
}
"#,
    ),
];

/// Constants that belong to other modules. Only the referenced ones are
/// emitted: an unused one is a `dead_code` warning, and the dev shell builds
/// with `-D warnings`.
const KNOWN: &[(&str, &str, &str)] = &[
    ("kErrorTypeNone", "ErrorType", "-1"),
    ("kErrorTypeException", "ErrorType", "0"),
    ("kObjectTypeNil", "ObjectType", "0"),
    ("kObjectTypeBoolean", "ObjectType", "1"),
    ("kObjectTypeInteger", "ObjectType", "2"),
    ("kObjectTypeFloat", "ObjectType", "3"),
    ("kObjectTypeString", "ObjectType", "4"),
    ("kObjectTypeArray", "ObjectType", "5"),
    ("kObjectTypeDict", "ObjectType", "6"),
    ("kObjectTypeLuaRef", "ObjectType", "7"),
    ("kObjectTypeBuffer", "ObjectType", "8"),
    ("kObjectTypeWindow", "ObjectType", "9"),
    ("kObjectTypeTabpage", "ObjectType", "10"),
    ("LOGLVL_DBG", "c_int", "1"),
];

/// Names re-exported by `crate::src::nvim::types`, emitted when referenced.
const TYPE_NAMES: &[&str] = &[
    "Arena",
    "Array",
    "Boolean",
    "Dict",
    "Error",
    "ErrorType",
    "Float",
    "Integer",
    "LuaRef",
    "Object",
    "ObjectType",
    "String_0",
    "handle_T",
    "object_data",
    "size_t",
    "uint64_t",
];

/// Every identifier-shaped token in `text`, so "is this referenced?" is an
/// exact-name question rather than a substring one (`Integer` must not match
/// inside `kObjectTypeInteger`).
fn idents(text: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut cur = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            cur.push(ch);
        } else if !cur.is_empty() {
            out.insert(std::mem::take(&mut cur));
        }
    }
    if !cur.is_empty() {
        out.insert(cur);
    }
    out
}

/// Cut formatted top-level `fn` items apart, each keeping the blank line that
/// follows it.
fn split_items(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        if line.starts_with("pub ") || out.is_empty() {
            out.push(String::new());
        }
        out.last_mut().unwrap().push_str(line);
        out.last_mut().unwrap().push('\n');
    }
    out
}

fn generate(
    api: &BTreeMap<String, ApiFn>,
    specs: &[Spec],
    tables: &BTreeMap<String, usize>,
    config: &Path,
) -> Result<Vec<Emitted>, String> {
    // API source file stem -> the wrappers it gets, in name order so the
    // output does not depend on how the spec file happens to be arranged.
    let mut by_module: BTreeMap<&str, Vec<(&ApiFn, &Spec)>> = BTreeMap::new();
    for spec in specs.iter().filter(|s| s.is_wrapper()) {
        let f = api
            .get(&spec.name)
            .ok_or_else(|| format!("{}: no such API function in the crate", spec.name))?;
        by_module
            .entry(f.module.as_str())
            .or_default()
            .push((f, spec));
    }
    for fns in by_module.values_mut() {
        fns.sort_by(|a, b| a.0.name.cmp(&b.0.name));
    }

    // Wrappers, split into files no wider than the budget. Splitting only
    // ever happens within one API source file, so a module's wrappers stay
    // together and in order. The split is decided on formatted text — rustfmt
    // both joins and breaks lines, so the unformatted count is not even an
    // upper bound — which is why a module's wrappers go through rustfmt as one
    // batch here and are re-split on their `pub unsafe extern` openers.
    let mut children: Vec<(&str, String)> = Vec::new();
    for (module, fns) in &by_module {
        let mut all = String::new();
        for (f, spec) in fns {
            emit_fn(&mut all, f, spec, tables)?;
            all.push('\n');
        }
        let mut chunk = String::new();
        for one in split_items(&rustfmt(config, &all)?) {
            if !chunk.is_empty() && chunk.lines().count() + one.lines().count() > CHUNK_BUDGET {
                children.push((module, std::mem::take(&mut chunk)));
            }
            chunk.push_str(&one);
        }
        children.push((module, chunk));
    }
    // Number the parts now that the split is known.
    let mut files: Vec<Emitted> = Vec::new();
    let mut body = String::new();
    let mut part = 0;
    for (i, (module, chunk)) in children.iter().enumerate() {
        let parts = children.iter().filter(|(m, _)| m == module).count();
        part = if i > 0 && children[i - 1].0 == *module {
            part + 1
        } else {
            1
        };
        let name = if part == 1 {
            format!("{module}.rs")
        } else {
            format!("{module}_{part}.rs")
        };
        body.push_str(chunk);
        files.push(Emitted {
            name,
            text: format!("{}{chunk}", child_header(module, part, parts)),
        });
    }

    // module path segment -> API functions to import from it
    let mut api_imports: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for spec in specs.iter().filter(|s| s.is_wrapper()) {
        let f = &api[&spec.name];
        api_imports
            .entry(f.module.clone())
            .or_default()
            .insert(f.name.clone());
    }

    // Readers are needed transitively (as_dict calls is_empty_array), so
    // sweep until the referenced set stops growing.
    let mut support = String::from(SUPPORT);
    loop {
        let referenced = idents(&format!("{support}{body}"));
        let Some((_, code)) = READERS
            .iter()
            .find(|(name, code)| referenced.contains(*name) && !support.contains(*code))
        else {
            break;
        };
        support.push_str(code);
    }

    let referenced = idents(&format!("{support}{body}"));
    let mut known = String::new();
    let mut known_types: BTreeSet<&str> = BTreeSet::new();
    for (name, ty, value) in KNOWN {
        if !referenced.contains(*name) {
            continue;
        }
        if *ty != "c_int" {
            known_types.insert(ty);
        }
        known.push_str(&format!("    pub const {name}: {ty} = {value};\n"));
    }

    let mut uses: Vec<String> = Vec::new();
    uses.push("use core::ffi::{CStr, c_int};".into());
    for (module, names) in &api_imports {
        uses.push(format!(
            "use crate::src::nvim::api::{module}::{{{}}};",
            names.iter().cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    // The keyset tables and their perfect-hash lookups still live in the
    // hand-maintained dispatch module.
    let dispatch: Vec<String> = referenced
        .iter()
        .filter(|n| n.ends_with("_get_field") || n.ends_with("_table"))
        .cloned()
        .collect();
    if !dispatch.is_empty() {
        uses.push(format!(
            "use crate::src::nvim::api::private::dispatch::{{{}}};",
            dispatch.join(", ")
        ));
    }
    let helpers: Vec<&str> = [
        "api_dict_to_keydict",
        "api_keydict_to_dict",
        "api_set_error",
    ]
    .into_iter()
    .filter(|n| referenced.contains(*n))
    .collect();
    uses.push(format!(
        "use crate::src::nvim::api::private::helpers::{{{}}};",
        helpers.join(", ")
    ));
    if referenced.contains("expr_map_locked") {
        uses.push("use crate::src::nvim::ex_docmd::expr_map_locked;".into());
        uses.push("use crate::src::nvim::main::{e_textlock, textlock};".into());
    }
    if referenced.contains("text_locked") {
        uses.push("use crate::src::nvim::ex_getln::{get_text_locked_msg, text_locked};".into());
    }
    uses.push("use crate::src::nvim::log::logmsg;".into());
    // `mod known` names its constants' types, so it counts as a reference.
    let referenced_all = idents(&format!("{support}{body}{known}"));
    let types: Vec<String> = TYPE_NAMES
        .iter()
        .map(|s| (*s).to_string())
        .chain(
            referenced
                .iter()
                .filter(|n| n.starts_with("KeyDict_") && !n.ends_with("_get_field"))
                .cloned(),
        )
        .filter(|n| referenced_all.contains(n))
        .collect();
    uses.push(format!(
        "use crate::src::nvim::types::{{{}}};",
        types.join(", ")
    ));

    let mut out = String::from(HEADER);
    out.push('\n');
    for file in &files {
        let module = file.name.strip_suffix(".rs").unwrap();
        out.push_str(&format!("mod {module};\n"));
    }
    out.push('\n');
    for file in &files {
        let module = file.name.strip_suffix(".rs").unwrap();
        out.push_str(&format!("pub use self::{module}::*;\n"));
    }
    out.push('\n');
    for line in uses {
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str(&format!(
        r#"
/// Values that belong to other modules; nested so they stay out of the flat
/// namespace the unit-test header generator collects constants into.
mod known {{
    use super::{{{}}};
    use core::ffi::c_int;

{known}}}

use known::*;
"#,
        known_types.iter().cloned().collect::<Vec<_>>().join(", ")
    ));
    out.push_str(&support);

    files.insert(
        0,
        Emitted {
            name: "mod.rs".into(),
            text: out,
        },
    );
    Ok(files)
}

// ------------------------------------------------------- codegen: tables

const TABLES_HEADER: &str = r#"//! The msgpack-RPC dispatch tables.
//!
//! GENERATED by tools/apigen from the `KeyDict_*` structs in
//! `crate::src::nvim::types::keysets` plus `tools/apigen/functions.txt`. Do
//! not edit; run `just apigen` (`just apigen --check` fails on drift).
//!
//! Two lookups live here. `KeyDict_<name>_get_field` turns an options-dict
//! key into the table row that says where the value goes, and
//! `msgpack_rpc_get_handler_for` turns a method name into the wrapper that
//! serves it.
//!
//! Upstream generated both with a hand-rolled perfect hash (`src/gen/hashy.lua`
//! at tag v0.12.4): a switch on the key's length, then on the one character
//! position that best split that length's keys, then a `memcmp` to confirm.
//! Here each is a `match` on the key bytes instead. rustc lowers that to the
//! same shape — a switch on the length and a decision tree over the bytes —
//! without the confirming compare, and it is one readable line per key rather
//! than a table plus a switch plus a loop.
//!
//! What did survive from `hashy` is the *table order*, because it is not an
//! implementation detail: a key's row index is its `opt_index`, the bit it
//! owns in its keyset's `is_set__*_` mask, and a method's row index is what
//! `eval/funcs.rs` stores to bind the builtin `nvim_*()` Vimscript functions.
//! `tools/apigen`'s `table_order` reproduces the layout upstream's hash
//! implied.

#![deny(unsafe_op_in_unsafe_fn)]
"#;

/// Header for a child module of the tables directory. A chunk that came out
/// with no `unsafe` in it — the handler table and its lookup are plain data —
/// forbids it outright rather than settling for the weaker lint.
fn tables_child_header(what: &str, body: &str) -> String {
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
         \n\
         // A chunk may hold nothing that needs the parent's support code.\n\
         #[allow(unused_imports)]\n\
         use super::*;\n\
         \n"
    )
}

/// Shared support for the tables. `key`/`hl_key`/`END` build the rows;
/// `key_bytes` is how every lookup gets at the bytes it was handed.
const TABLES_SUPPORT: &str = r#"
/// One row of a keyset table: the key's name, the offset of the field its
/// value lands in, the tag that value must arrive as, and the bit it owns in
/// the keyset's `is_set__*_` mask (-1 when the keyset has no mask).
const fn key(name: &'static CStr, ptr_off: usize, type_0: c_int, opt_index: c_int) -> KeySetLink {
    KeySetLink {
        str: name.as_ptr().cast_mut(),
        ptr_off,
        type_0,
        opt_index,
        is_hlgroup: false,
    }
}

/// A row whose value names a highlight group. It arrives as a String and is
/// stored as the id the converter resolves it to, so its tag is an Integer.
const fn hl_key(name: &'static CStr, ptr_off: usize, opt_index: c_int) -> KeySetLink {
    KeySetLink {
        str: name.as_ptr().cast_mut(),
        ptr_off,
        type_0: TAG_INTEGER,
        opt_index,
        is_hlgroup: true,
    }
}

/// The null row every keyset table ends with: `api_keydict_to_dict` walks a
/// table until it sees one.
const END: KeySetLink = KeySetLink {
    str: ptr::null_mut(),
    ptr_off: 0,
    type_0: TAG_NIL,
    opt_index: -1,
    is_hlgroup: false,
};

/// The key bytes a lookup was handed. An empty key carries no pointer worth
/// dereferencing — and may carry a null one — so length zero short-circuits.
///
/// # Safety
/// `str` points at `len` readable bytes.
unsafe fn key_bytes<'a>(str: *const c_char, len: size_t) -> &'a [u8] {
    if len == 0 {
        return &[];
    }
    // SAFETY: the caller promises `len` readable bytes at `str`.
    unsafe { slice::from_raw_parts(str.cast::<u8>(), len) }
}

/// One row of [`method_handlers`]: the name a client calls the method by, the
/// wrapper that serves it, whether it may run straight from the RPC read
/// callback instead of being deferred to the main loop, and whether its result
/// is heap-allocated for the caller to free rather than owned by the request
/// arena.
const fn handler(
    name: &'static CStr,
    f: unsafe extern "C" fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
    fast: bool,
    ret_alloc: bool,
) -> MsgpackRpcRequestHandler {
    MsgpackRpcRequestHandler {
        name: name.as_ptr(),
        fn_0: Some(f),
        fast,
        ret_alloc,
    }
}

/// What [`msgpack_rpc_get_handler_for`] returns when it refused; the caller
/// looks at `*error`.
const NO_HANDLER: MsgpackRpcRequestHandler = MsgpackRpcRequestHandler {
    name: ptr::null(),
    fn_0: None,
    fast: false,
    ret_alloc: false,
};

/// Look a method up by name.
///
/// # Safety
/// `name` points at `name_len` readable bytes; `error` at a live `Error`.
pub unsafe extern "C" fn msgpack_rpc_get_handler_for(
    name: *const c_char,
    name_len: size_t,
    error: *mut Error,
) -> MsgpackRpcRequestHandler {
    // SAFETY: the caller passes a method name of `name_len` bytes.
    if let Some(index) = handler_index(unsafe { key_bytes(name, name_len) }) {
        // SAFETY: `handler_index` only ever returns an index into the table.
        return unsafe { (*method_handlers.ptr())[index] };
    }
    // `%.*s`: the name is not NUL-terminated, so its length goes along. The
    // stand-in for an empty one is, and upstream passed `sizeof("<empty>")`.
    let (len, text) = if name_len > 0 {
        (name_len as c_int, name)
    } else {
        let empty = c"<empty>";
        (empty.to_bytes_with_nul().len() as c_int, empty.as_ptr())
    };
    // SAFETY: `error` is live and the format string matches its arguments.
    unsafe {
        api_set_error(
            error,
            kErrorTypeException,
            c"Invalid method: %.*s".as_ptr(),
            len,
            text,
        );
    }
    NO_HANDLER
}
"#;

/// The `TAG_*` constants, in tag order. Only the referenced ones are emitted.
const TAGS: &[(&str, &str, &str)] = &[
    (
        "TAG_STRING_ARRAY",
        "-1",
        "ShaDa's own unpacked-in-place array of strings",
    ),
    ("TAG_NIL", "0", "any Object"),
    ("TAG_BOOLEAN", "1", ""),
    ("TAG_INTEGER", "2", ""),
    ("TAG_FLOAT", "3", ""),
    ("TAG_STRING", "4", ""),
    ("TAG_ARRAY", "5", ""),
    ("TAG_DICT", "6", ""),
    ("TAG_LUAREF", "7", ""),
    ("TAG_BUFFER", "8", ""),
    ("TAG_WINDOW", "9", ""),
    ("TAG_TABPAGE", "10", ""),
];

/// One keyset's table plus the lookup that indexes it.
fn emit_keyset(out: &mut String, k: &Keyset) {
    let name = &k.name;
    writeln!(
        out,
        "pub static {name}_table: GlobalCell<[KeySetLink; {}]> = GlobalCell::new({{",
        k.len()
    )
    .unwrap();
    writeln!(out, "    type K = KeyDict_{name};").unwrap();
    writeln!(out, "    [").unwrap();
    for (i, key) in k.keys.iter().enumerate() {
        let opt_index: i64 = if k.has_optional { i as i64 + 1 } else { -1 };
        let (ctor, tag) = if key.is_hlgroup {
            ("hl_key", String::new())
        } else {
            ("key", format!("{}, ", key.tag))
        };
        writeln!(
            out,
            "        {ctor}(c\"{}\", offset_of!(K, {}), {tag}{opt_index}),",
            key.wire, key.field
        )
        .unwrap();
    }
    writeln!(out, "        END,").unwrap();
    writeln!(out, "    ]").unwrap();
    writeln!(out, "}});").unwrap();
    out.push('\n');

    writeln!(out, "/// Look a key up in [`{name}_table`].").unwrap();
    writeln!(out, "///").unwrap();
    writeln!(out, "/// # Safety").unwrap();
    writeln!(out, "/// `str` points at `len` readable bytes.").unwrap();
    if k.keys.is_empty() {
        writeln!(
            out,
            "pub unsafe extern \"C\" fn KeyDict_{name}_get_field(_str: *const c_char, _len: size_t) -> *mut KeySetLink {{"
        )
        .unwrap();
        writeln!(out, "    // The keyset has no keys, so nothing matches.").unwrap();
        writeln!(out, "    ptr::null_mut()").unwrap();
        writeln!(out, "}}").unwrap();
        out.push('\n');
        return;
    }
    writeln!(
        out,
        "pub unsafe extern \"C\" fn KeyDict_{name}_get_field(str: *const c_char, len: size_t) -> *mut KeySetLink {{"
    )
    .unwrap();
    writeln!(
        out,
        "    // SAFETY: the caller passes a key of `len` bytes."
    )
    .unwrap();
    writeln!(
        out,
        "    let index: usize = match unsafe {{ key_bytes(str, len) }} {{"
    )
    .unwrap();
    for (i, key) in k.keys.iter().enumerate() {
        writeln!(out, "        b\"{}\" => {i},", key.wire).unwrap();
    }
    writeln!(out, "        _ => return ptr::null_mut(),").unwrap();
    writeln!(out, "    }};").unwrap();
    writeln!(
        out,
        "    let table: *mut KeySetLink = {name}_table.ptr().cast();"
    )
    .unwrap();
    writeln!(out, "    table.wrapping_add(index)").unwrap();
    writeln!(out, "}}").unwrap();
    out.push('\n');
}

/// Cross-check the handler table's layout against the row numbers
/// `eval/funcs.rs` has baked in.
///
/// The builtin `nvim_*()` Vimscript functions do not look their handler up by
/// name: each `EvalFuncDef` stores `&method_handlers[N]` outright. That makes
/// the table's order part of the crate's meaning rather than an internal
/// detail of this generator, and a silent renumbering would bind every one of
/// those builtins to the wrong API function. So it is an error here, not a
/// mystery at runtime.
fn check_eval_indices(root: &Path, order: &BTreeMap<&str, usize>) -> Result<(), String> {
    let path = root.join("src/nvim/eval/funcs.rs");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    // Each entry reads `name: b"nvim_foo\0"…` and, further down the same
    // struct literal, `api_handler: (&raw const method_handlers …).offset(N …)`.
    let mut name = None;
    let mut binding = false;
    let mut checked = 0;
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("name: b\"") {
            name = rest.split_once("\\0").map(|(n, _)| n.to_string());
        }
        if line.starts_with("api_handler: (&raw const method_handlers") {
            binding = true;
            continue;
        }
        if !binding {
            continue;
        }
        binding = false;
        let rest = line.strip_prefix(".offset(").ok_or_else(|| {
            format!("eval/funcs.rs: an api_handler binding lacks an .offset(): {line}")
        })?;
        let name = name
            .take()
            .ok_or_else(|| "eval/funcs.rs: an api_handler binding has no name".to_string())?;
        let row = rest
            .split_whitespace()
            .next()
            .unwrap_or("")
            .parse::<usize>()
            .map_err(|_| format!("eval/funcs.rs: unreadable handler row in `{line}`"))?;
        let want = order
            .get(name.as_str())
            .ok_or_else(|| format!("eval/funcs.rs binds {name}, which is not an RPC method"))?;
        if *want != row {
            return Err(format!(
                "eval/funcs.rs binds {name} to handler row {row}, but it is now row {want}; \
                 the table order changed and those bindings have to follow"
            ));
        }
        checked += 1;
    }
    if checked == 0 {
        return Err("eval/funcs.rs bound no handlers; the cross-check has gone blind".into());
    }
    Ok(())
}

/// The handler table and the lookup that indexes it.
///
/// The layout is `table_order` over the method names *sorted*, not over the
/// spec file's arrangement. Upstream fed its own header-declaration order in;
/// sorting reproduces the same table (checked against the frozen one) while
/// making the result independent of how `functions.txt` is grouped.
fn emit_handlers<'a>(out: &mut String, specs: &'a [Spec]) -> BTreeMap<&'a str, usize> {
    let by_name: BTreeMap<&str, &Spec> = specs.iter().map(|s| (s.name.as_str(), s)).collect();
    // A lua_only method is not dispatched, so it takes no row.
    let mut sorted: Vec<&Spec> = specs.iter().filter(|s| s.is_method()).collect();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    let names: Vec<String> = sorted.iter().map(|s| s.name.clone()).collect();
    let order = table_order(&names);
    let specs: Vec<&Spec> = sorted;

    writeln!(
        out,
        "pub static method_handlers: GlobalCell<[MsgpackRpcRequestHandler; {}]> = GlobalCell::new([",
        specs.len()
    )
    .unwrap();
    for &i in &order {
        let spec = &specs[i];
        // An alias serves its target's wrapper, with its flags.
        let flags = match &spec.alias {
            Some(target) => by_name[target.as_str()],
            None => spec,
        };
        let f = match (&spec.alias, &spec.handler) {
            (Some(target), _) => format!("handle_{target}"),
            (_, Some(path)) => path.rsplit("::").next().unwrap().to_string(),
            _ => format!("handle_{}", spec.name),
        };
        writeln!(
            out,
            "    handler(c\"{}\", {f}, {}, {}),",
            spec.name, flags.fast, flags.ret_alloc
        )
        .unwrap();
    }
    writeln!(out, "]);").unwrap();
    out.push('\n');

    writeln!(
        out,
        "/// The row of [`method_handlers`] the method called `name` sits in."
    )
    .unwrap();
    writeln!(out, "pub fn handler_index(name: &[u8]) -> Option<usize> {{").unwrap();
    writeln!(out, "    Some(match name {{").unwrap();
    for (row, &i) in order.iter().enumerate() {
        writeln!(out, "        b\"{}\" => {row},", specs[i].name).unwrap();
    }
    writeln!(out, "        _ => return None,").unwrap();
    writeln!(out, "    }})").unwrap();
    writeln!(out, "}}").unwrap();
    out.push('\n');
    order
        .iter()
        .enumerate()
        .map(|(row, &i)| (specs[i].name.as_str(), row))
        .collect()
}

/// Cut formatted top-level items apart on the blank lines between them. The
/// tables carry doc comments, so `split_items`' "a line starting with `pub`
/// opens an item" would orphan them.
fn split_paragraphs(text: &str) -> Vec<String> {
    let mut out: Vec<String> = vec![String::new()];
    let mut fresh = false;
    for line in text.lines() {
        if fresh && !line.is_empty() {
            out.push(String::new());
        }
        fresh = line.is_empty();
        out.last_mut().unwrap().push_str(line);
        out.last_mut().unwrap().push('\n');
    }
    out.retain(|s| !s.trim().is_empty());
    out
}

/// The dispatch tables module: the keyset tables and their lookups, the
/// handler table and its lookup.
fn generate_tables(
    root: &Path,
    keysets: &[Keyset],
    specs: &[Spec],
    config: &Path,
) -> Result<Vec<Emitted>, String> {
    // What goes in which child module, formatted as one batch each so the
    // chunker can count real lines.
    let mut sections: Vec<(&str, String)> = Vec::new();
    let mut all = String::new();
    for k in keysets {
        emit_keyset(&mut all, k);
    }
    for chunk in chunked(&rustfmt(config, &all)?) {
        sections.push(("keysets", chunk));
    }
    let mut all = String::new();
    check_eval_indices(root, &emit_handlers(&mut all, specs))?;
    for chunk in chunked(&rustfmt(config, &all)?) {
        sections.push(("handlers", chunk));
    }

    let mut files: Vec<Emitted> = Vec::new();
    let mut body = String::new();
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
            "keysets" => "The keyset tables: which key fills which field.",
            _ => "The handler table: which method calls which wrapper.",
        };
        body.push_str(chunk);
        files.push(Emitted {
            name,
            text: format!("{}{chunk}", tables_child_header(what, chunk)),
        });
    }

    let referenced = idents(&format!("{TABLES_SUPPORT}{body}"));
    let mut tags = String::new();
    for (name, value, note) in TAGS {
        if !referenced.contains(*name) {
            continue;
        }
        let note = if note.is_empty() {
            String::new()
        } else {
            format!(" // {note}")
        };
        writeln!(tags, "    pub const {name}: c_int = {value};{note}").unwrap();
    }

    let mut out = String::from(TABLES_HEADER);
    out.push('\n');
    for file in &files {
        writeln!(out, "mod {};", file.name.strip_suffix(".rs").unwrap()).unwrap();
    }
    out.push('\n');
    for file in &files {
        writeln!(
            out,
            "pub use self::{}::*;",
            file.name.strip_suffix(".rs").unwrap()
        )
        .unwrap();
    }
    out.push('\n');
    out.push_str("use core::ffi::{CStr, c_char, c_int};\n");
    out.push_str("use core::mem::offset_of;\n");
    out.push_str("use core::{ptr, slice};\n");
    out.push('\n');
    out.push_str("// Every generated wrapper; the handler table names most of them.\n");
    out.push_str("use crate::src::nvim::api::private::dispatch_wrappers::*;\n");
    out.push_str("use crate::src::nvim::api::private::helpers::api_set_error;\n");
    out.push_str("use crate::src::nvim::global_cell::GlobalCell;\n");
    // Handlers the spec names outright, which live outside the generated
    // wrappers.
    let mut externs: BTreeSet<&str> = BTreeSet::new();
    for spec in specs {
        if let Some(path) = &spec.handler {
            externs.insert(path.as_str());
        }
    }
    for path in &externs {
        writeln!(out, "use {path};").unwrap();
    }
    let mut types: Vec<String> = [
        "Arena",
        "Array",
        "Error",
        "ErrorType",
        "KeySetLink",
        "MsgpackRpcRequestHandler",
        "Object",
        "size_t",
        "uint64_t",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .chain(keysets.iter().map(|k| format!("KeyDict_{}", k.name)))
    .collect();
    types.sort();
    writeln!(
        out,
        "use crate::src::nvim::types::{{{}}};",
        types.join(", ")
    )
    .unwrap();
    write!(
        out,
        r#"
/// Values that belong to other modules; nested so they stay out of the flat
/// namespace the unit-test header generator collects constants into.
mod known {{
    use super::ErrorType;
    use core::ffi::c_int;

    pub const kErrorTypeException: ErrorType = 0;

    // `KeySetLink::type_0`: the `ObjectType` a key's value must arrive as, as
    // the `c_int` that field holds.
{tags}}}

use known::*;
"#
    )
    .unwrap();
    out.push_str(TABLES_SUPPORT);

    files.insert(
        0,
        Emitted {
            name: "mod.rs".into(),
            text: out,
        },
    );
    Ok(files)
}

/// Split formatted item text into files no wider than the budget.
fn chunked(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chunk = String::new();
    for one in split_paragraphs(text) {
        if !chunk.is_empty() && chunk.lines().count() + one.lines().count() > CHUNK_BUDGET {
            out.push(std::mem::take(&mut chunk));
        }
        chunk.push_str(&one);
    }
    out.push(chunk);
    out
}

// ---------------------------------------------------- codegen: lua binding

const LUA_HEADER: &str = r#"//! The `vim.api` Lua binding.
//!
//! GENERATED by tools/apigen from the `nvim_*` signatures under
//! `crate::src::nvim::api` plus `tools/apigen/functions.txt`. Do not edit;
//! run `just apigen` (`just apigen --check` fails on drift).
//!
//! One `lua_CFunction` per API function, plus [`nlua_add_api_functions`],
//! which builds the table they hang off. A binding checks that it was handed
//! exactly the arguments the API function declares, pops each one off the Lua
//! stack and converts it, calls the function, and converts the result back.
//!
//! Everything that fails — the wrong number of arguments, a value Lua cannot
//! convert, the API function itself — leaves its reason in one `Error`, and
//! the binding raises it as a Lua error on the way out. Getting there means
//! unwinding past whatever was already converted, which is what the labelled
//! blocks in each binding are for: breaking out of one runs the releases
//! between it and the end, and nothing else. That is the shape upstream's
//! generator built out of `goto exit_N` (src/gen/gen_api_dispatch.lua at tag
//! v0.12.4).
//!
//! This module holds the shared support code and the imports; the bindings
//! themselves live in one child module per API source file, re-exported here
//! so callers see one flat namespace. A source file whose bindings would
//! overflow the tree's 1,000-line file cap is split into numbered parts.

#![deny(unsafe_op_in_unsafe_fn)]
"#;

/// The fixed part of the Lua binding support code.
const LUA_SUPPORT: &str = r#"
/// A fresh, unset error.
const ERROR_INIT: Error = Error {
    type_0: kErrorTypeNone,
    msg: ptr::null_mut(),
};

/// Flags for handing a result back to Lua.
///
/// `kNluaPushFreeRefs` always applies: the binding owns the value and
/// releases the Lua references it holds as it converts. `kNluaPushSpecial`
/// additionally converts `nil` and the other special values the pre-0.11
/// way, which is what clients of methods older than API level 11 expect.
const PUSH: c_int = kNluaPushFreeRefs as c_int;
const PUSH_SPECIAL: c_int = (kNluaPushSpecial | kNluaPushFreeRefs) as c_int;

/// Release the request arena and, if the call failed, stage the Lua error:
/// the source position, the parameter a failed conversion blamed, and the
/// message itself, left on the stack as one string. Returns whether the
/// caller must now raise it — which it has to do itself, so that the
/// non-returning `lua_error` unwinds out of the binding's own frame.
///
/// # Safety
/// `lstate` is the running Lua state; `arena` and `err` point at the
/// binding's own locals; `err_param`, when set, points at a NUL-terminated
/// name that outlives the call.
unsafe fn finish(
    lstate: *mut lua_State,
    arena: *mut Arena,
    err: *mut Error,
    err_param: *const c_char,
) -> bool {
    unsafe {
        arena_mem_free(arena_finish(arena));
        if (*err).type_0 == kErrorTypeNone {
            return false;
        }
        luaL_where(lstate, 1);
        if !err_param.is_null() {
            lua_pushstring(lstate, c"Invalid '".as_ptr());
            lua_pushstring(lstate, err_param);
            lua_pushstring(lstate, c"': ".as_ptr());
            lua_pushstring(lstate, (*err).msg);
            api_clear_error(err);
            lua_concat(lstate, 5);
        } else {
            lua_pushstring(lstate, (*err).msg);
            api_clear_error(err);
            lua_concat(lstate, 2);
        }
        true
    }
}

/// One entry of the `vim.api` table.
///
/// # Safety
/// `lstate` has the table under construction on top of its stack.
unsafe fn bind(
    lstate: *mut lua_State,
    f: unsafe extern "C-unwind" fn(*mut lua_State) -> c_int,
    name: &CStr,
) {
    unsafe {
        lua_pushcclosure(lstate, Some(f), 0);
        lua_setfield(lstate, -2, name.as_ptr());
    }
}
"#;

/// The `nlua_pop_*` that turns the top of the Lua stack into a parameter, and
/// whatever leading argument it takes beyond the state.
fn popper(ty: &ApiType) -> (String, &'static str) {
    match ty {
        ApiType::Boolean => ("nlua_pop_Boolean".into(), ""),
        ApiType::Integer => ("nlua_pop_Integer".into(), ""),
        ApiType::Float => ("nlua_pop_Float".into(), ""),
        ApiType::String => ("nlua_pop_String".into(), ""),
        ApiType::Array => ("nlua_pop_Array".into(), ""),
        // The flag says whether to keep Lua references to the functions the
        // value holds. Only a `DictOf(LuaRef)` parameter wants them.
        ApiType::Dict => ("nlua_pop_Dict".into(), "false, "),
        ApiType::Object => ("nlua_pop_Object".into(), "true, "),
        ApiType::LuaRef => ("nlua_pop_LuaRef".into(), ""),
        ApiType::Handle(_) => ("nlua_pop_handle".into(), ""),
        ApiType::KeyDict(_) => unreachable!("keysets are filled in place"),
    }
}

/// The `nlua_push_*` that hands a result back, and whether it takes the value
/// by pointer.
fn pusher(ret: &RetType) -> (String, bool) {
    match ret {
        RetType::Boolean => ("nlua_push_Boolean".into(), false),
        RetType::Integer => ("nlua_push_Integer".into(), false),
        RetType::Float => ("nlua_push_Float".into(), false),
        RetType::String => ("nlua_push_String".into(), false),
        RetType::Array => ("nlua_push_Array".into(), false),
        RetType::Dict => ("nlua_push_Dict".into(), false),
        RetType::Object => ("nlua_push_Object".into(), true),
        RetType::Handle(_) => ("nlua_push_handle".into(), false),
        RetType::Void | RetType::KeyDict(_) => unreachable!("handled separately"),
    }
}

/// One `lua_CFunction`.
fn emit_lua_fn(out: &mut String, f: &ApiFn, spec: &Spec) -> Result<(), String> {
    let name = &f.name;
    let values: Vec<(usize, &ApiType, &str)> = f
        .params
        .iter()
        .filter_map(|p| match p {
            Param::Value { index, ty, name } => Some((*index, ty, name.as_str())),
            _ => None,
        })
        .collect();
    let argc = values.len();
    // Whether the API function has a Lua implementation of its own, which it
    // needs the state for.
    let has_lua_imp = f.params.contains(&Param::LuaState);

    for (index, ty, _) in &values {
        if matches!(ty, ApiType::Dict)
            && spec
                .declared
                .get(&(index + 1))
                .is_some_and(|d| d.contains("LuaRef"))
        {
            return Err(format!(
                "{name}: a DictOf(LuaRef) parameter has to keep its Lua references; \
                 nothing needed that when this was written"
            ));
        }
    }

    // Slot (1-based) -> what releases the value that pop left behind. The
    // rest own nothing: their arena memory goes back with the arena.
    let mut frees: BTreeMap<usize, String> = BTreeMap::new();
    for (index, ty, _) in &values {
        let slot = index + 1;
        let code = match ty {
            ApiType::Object => format!("api_luarefs_free_object(arg_{slot});"),
            ApiType::LuaRef => format!("api_free_luaref(arg_{slot});"),
            ApiType::KeyDict(keyset) => format!(
                "api_luarefs_free_keydict((&raw mut arg_{slot}).cast(), {keyset}_table.ptr().cast());"
            ),
            _ => continue,
        };
        frees.insert(slot, code);
    }

    // Where a failed pop breaks to: the outermost release still pending.
    // A keyset may have been half filled before it failed, so its own
    // release counts; every other pop leaves nothing behind when it fails.
    let pending = |slot: usize, keyset: bool| {
        frees
            .range(slot + usize::from(!keyset)..)
            .next()
            .map(|(pending, _)| *pending)
    };
    let target = |slot: usize, keyset: bool| match pending(slot, keyset) {
        Some(pending) => format!("'free_arg_{pending}"),
        None => "'done".to_string(),
    };
    // A release the pops can never skip past needs no block of its own: it
    // sits on the one path that reaches it.
    let jumped_to: BTreeSet<usize> = values
        .iter()
        .filter_map(|(index, ty, _)| pending(index + 1, matches!(ty, ApiType::KeyDict(_))))
        .collect();

    writeln!(
        out,
        "pub unsafe extern \"C-unwind\" fn nlua_api_{name}(lstate: *mut lua_State) -> c_int {{"
    )
    .unwrap();
    writeln!(out, "    unsafe {{").unwrap();
    writeln!(out, "        let mut err = ERROR_INIT;").unwrap();
    writeln!(out, "        let mut arena = ARENA_EMPTY;").unwrap();
    writeln!(
        out,
        "        let mut err_param: *mut c_char = ptr::null_mut();"
    )
    .unwrap();
    writeln!(out, "        'done: {{").unwrap();
    writeln!(out, "            if lua_gettop(lstate) != {argc} {{").unwrap();
    writeln!(
        out,
        "                api_set_error(&raw mut err, kErrorTypeValidation, c\"Expected {argc} argument{}\".as_ptr());",
        if argc == 1 { "" } else { "s" }
    )
    .unwrap();
    writeln!(out, "                break 'done;").unwrap();
    writeln!(out, "            }}").unwrap();
    if !spec.fast {
        writeln!(out, "            if !nlua_is_deferred_safe() {{").unwrap();
        writeln!(
            out,
            "                return luaL_error(lstate, (&raw const e_fast_api_disabled).cast(), c\"{name}\".as_ptr());"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
    }
    if spec.textlock {
        writeln!(out, "            if text_locked() {{").unwrap();
        writeln!(
            out,
            "                api_set_error(&raw mut err, kErrorTypeException, c\"%s\".as_ptr(), get_text_locked_msg());"
        )
        .unwrap();
        writeln!(out, "                break 'done;").unwrap();
        writeln!(out, "            }}").unwrap();
    } else if spec.textlock_allow_cmdwin {
        writeln!(
            out,
            "            if textlock.get() != 0 || expr_map_locked() {{"
        )
        .unwrap();
        writeln!(
            out,
            "                api_set_error(&raw mut err, kErrorTypeException, c\"%s\".as_ptr(), &raw const e_textlock);"
        )
        .unwrap();
        writeln!(out, "                break 'done;").unwrap();
        writeln!(out, "            }}").unwrap();
    }

    // Anything a release touches outlives the block that filled it, so it is
    // declared out here. A keyset is also zeroed here: the pop fills it field
    // by field and a partial fill still has to be walkable.
    let types: BTreeMap<usize, &ApiType> = values.iter().map(|(i, ty, _)| (i + 1, *ty)).collect();
    for slot in frees.keys().rev() {
        match types[slot] {
            ApiType::KeyDict(keyset) => writeln!(
                out,
                "            let mut arg_{slot}: KeyDict_{keyset} = core::mem::zeroed();"
            ),
            ApiType::LuaRef => writeln!(out, "            let arg_{slot}: LuaRef;"),
            _ => writeln!(out, "            let arg_{slot}: Object;"),
        }
        .unwrap();
    }
    for slot in frees.keys().rev().filter(|slot| jumped_to.contains(slot)) {
        writeln!(out, "            'free_arg_{slot}: {{").unwrap();
    }

    // The Lua stack hands the arguments back last first.
    for (index, ty, param) in values.iter().rev() {
        let slot = index + 1;
        if let ApiType::KeyDict(keyset) = ty {
            writeln!(out, "            nlua_pop_keydict(").unwrap();
            writeln!(out, "                lstate,").unwrap();
            writeln!(out, "                (&raw mut arg_{slot}).cast(),").unwrap();
            writeln!(out, "                Some(KeyDict_{keyset}_get_field),").unwrap();
            // The keyset pop names the offending key itself.
            writeln!(out, "                &raw mut err_param,").unwrap();
            writeln!(out, "                &raw mut arena,").unwrap();
            writeln!(out, "                &raw mut err,").unwrap();
            writeln!(out, "            );").unwrap();
            writeln!(out, "            if err.type_0 != kErrorTypeNone {{").unwrap();
            writeln!(out, "                break {};", target(slot, true)).unwrap();
            writeln!(out, "            }}").unwrap();
            continue;
        }
        let (pop, extra) = popper(ty);
        let bind = if frees.contains_key(&slot) {
            format!("arg_{slot} = ")
        } else {
            format!("let arg_{slot} = ")
        };
        writeln!(
            out,
            "            {bind}{pop}(lstate, {extra}&raw mut arena, &raw mut err);"
        )
        .unwrap();
        writeln!(out, "            if err.type_0 != kErrorTypeNone {{").unwrap();
        writeln!(
            out,
            "                err_param = c\"{param}\".as_ptr().cast_mut();"
        )
        .unwrap();
        writeln!(out, "                break {};", target(slot, false)).unwrap();
        writeln!(out, "            }}").unwrap();
    }

    let call_args: Vec<String> = f
        .params
        .iter()
        .map(|p| match p {
            Param::ChannelId => "LUA_INTERNAL_CALL".into(),
            Param::Arena => "&raw mut arena".into(),
            Param::Error => "&raw mut err".into(),
            Param::LuaState => "lstate".into(),
            Param::Value {
                index,
                ty: ApiType::KeyDict(_),
                ..
            } => format!("&raw mut arg_{}", index + 1),
            Param::Value { index, .. } => format!("arg_{}", index + 1),
        })
        .collect();
    let call = format!("{name}({})", call_args.join(", "));
    // The API function may reach back into Lua; while it runs, this is the
    // state it reaches into.
    writeln!(out, "            let saved_lstate = active_lstate.get();").unwrap();
    writeln!(out, "            active_lstate.set(lstate);").unwrap();
    let by_pointer = matches!(f.ret, RetType::Object | RetType::KeyDict(_));
    match f.ret {
        RetType::Void => writeln!(out, "            {call};").unwrap(),
        _ => writeln!(
            out,
            "            let {}ret = {call};",
            if by_pointer { "mut " } else { "" }
        )
        .unwrap(),
    }
    let flags = if spec.push_special() {
        "PUSH_SPECIAL"
    } else {
        "PUSH"
    };
    let push = match &f.ret {
        RetType::Void => String::new(),
        RetType::KeyDict(keyset) => format!(
            "nlua_push_keydict(lstate, (&raw mut ret).cast(), {keyset}_table.ptr().cast());"
        ),
        ret => {
            let (push, by_pointer) = pusher(ret);
            let value = if by_pointer { "&raw mut ret" } else { "ret" };
            format!("{push}(lstate, {value}, {flags});")
        }
    };
    // A function with a Lua implementation of its own may have pushed the
    // result already; only convert what it left behind.
    if !push.is_empty() && has_lua_imp {
        writeln!(out, "            if lua_gettop(lstate) == 0 {{").unwrap();
        writeln!(out, "                {push}").unwrap();
        writeln!(out, "            }}").unwrap();
    } else if !push.is_empty() {
        writeln!(out, "            {push}").unwrap();
    }
    writeln!(out, "            active_lstate.set(saved_lstate);").unwrap();
    if spec.ret_alloc {
        let free = match &f.ret {
            RetType::String => "api_free_string",
            RetType::Object => "api_free_object",
            RetType::Dict => "api_free_dict",
            RetType::Array => "api_free_array",
            other => return Err(format!("{name}: nothing frees a {other:?} result")),
        };
        writeln!(out, "            {free}(ret);").unwrap();
    }

    for (slot, free) in frees.iter() {
        if jumped_to.contains(slot) {
            writeln!(out, "            }}").unwrap();
        }
        writeln!(out, "            {free}").unwrap();
    }
    writeln!(out, "        }}").unwrap();
    writeln!(
        out,
        "        if finish(lstate, &raw mut arena, &raw mut err, err_param) {{"
    )
    .unwrap();
    writeln!(out, "            return lua_error(lstate);").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(
        out,
        "        {}",
        if f.ret == RetType::Void { 0 } else { 1 }
    )
    .unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}").unwrap();
    Ok(())
}

/// The table `vim.api` is: one entry per binding.
fn emit_lua_registration(out: &mut String, bound: &[&str]) {
    writeln!(
        out,
        "/// Build the `vim.api` table and set it on the table at the top of the stack."
    )
    .unwrap();
    writeln!(out, "///").unwrap();
    writeln!(out, "/// # Safety").unwrap();
    writeln!(
        out,
        "/// `lstate` is the running Lua state, with a table on top."
    )
    .unwrap();
    writeln!(
        out,
        "pub unsafe extern \"C-unwind\" fn nlua_add_api_functions(lstate: *mut lua_State) {{"
    )
    .unwrap();
    writeln!(out, "    unsafe {{").unwrap();
    writeln!(out, "        lua_createtable(lstate, 0, {});", bound.len()).unwrap();
    for name in bound {
        writeln!(out, "        bind(lstate, nlua_api_{name}, c\"{name}\");").unwrap();
    }
    writeln!(out, "        lua_setfield(lstate, -2, c\"api\".as_ptr());").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}").unwrap();
}

/// Header for a child module of the Lua binding directory.
fn lua_child_header(what: &str) -> String {
    format!(
        "//! {what}\n\
         //!\n\
         //! GENERATED by tools/apigen; see the parent module. Do not edit;\n\
         //! run `just apigen`.\n\
         \n\
         #![deny(unsafe_op_in_unsafe_fn)]\n\
         \n\
         use super::*;\n\
         \n"
    )
}

/// The `vim.api` Lua binding module.
fn generate_lua(
    api: &BTreeMap<String, ApiFn>,
    specs: &[Spec],
    config: &Path,
) -> Result<Vec<Emitted>, String> {
    let bound: Vec<(&ApiFn, &Spec)> = specs
        .iter()
        .filter(|s| s.has_lua_binding())
        .map(|spec| {
            api.get(&spec.name)
                .map(|f| (f, spec))
                .ok_or_else(|| format!("{}: no such API function in the crate", spec.name))
        })
        .collect::<Result<_, _>>()?;

    let mut by_module: BTreeMap<&str, Vec<(&ApiFn, &Spec)>> = BTreeMap::new();
    for (f, spec) in &bound {
        by_module
            .entry(f.module.as_str())
            .or_default()
            .push((f, spec));
    }
    for fns in by_module.values_mut() {
        fns.sort_by(|a, b| a.0.name.cmp(&b.0.name));
    }

    // As in the RPC wrappers: format one API source file's bindings as a
    // batch, then split them on their openers, so the 1,000-line cap is
    // measured on the text that actually lands on disk.
    let mut children: Vec<(&str, String)> = Vec::new();
    for (module, fns) in &by_module {
        let mut all = String::new();
        for (f, spec) in fns {
            emit_lua_fn(&mut all, f, spec)?;
            all.push('\n');
        }
        let mut chunk = String::new();
        for one in split_items(&rustfmt(config, &all)?) {
            if !chunk.is_empty() && chunk.lines().count() + one.lines().count() > CHUNK_BUDGET {
                children.push((module, std::mem::take(&mut chunk)));
            }
            chunk.push_str(&one);
        }
        children.push((module, chunk));
    }

    let mut files: Vec<Emitted> = Vec::new();
    let mut body = String::new();
    let mut part = 0;
    for (i, (module, chunk)) in children.iter().enumerate() {
        let parts = children.iter().filter(|(m, _)| m == module).count();
        part = if i > 0 && children[i - 1].0 == *module {
            part + 1
        } else {
            1
        };
        let name = if part == 1 {
            format!("{module}.rs")
        } else {
            format!("{module}_{part}.rs")
        };
        let of = if parts > 1 {
            format!(", part {part} of {parts}")
        } else {
            String::new()
        };
        body.push_str(chunk);
        files.push(Emitted {
            name,
            text: format!(
                "{}{chunk}",
                lua_child_header(&format!(
                    "Lua bindings for `crate::src::nvim::api::{module}`{of}."
                ))
            ),
        });
    }

    let mut names: Vec<&str> = bound.iter().map(|(f, _)| f.name.as_str()).collect();
    names.sort();
    let mut registration = String::new();
    emit_lua_registration(&mut registration, &names);
    let registration = rustfmt(config, &registration)?;
    body.push_str(&registration);
    files.push(Emitted {
        name: "register.rs".into(),
        text: format!(
            "{}{registration}",
            lua_child_header("The `vim.api` table: which name calls which binding.")
        ),
    });

    // module path segment -> API functions to import from it
    let mut api_imports: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (f, _) in &bound {
        api_imports
            .entry(f.module.as_str())
            .or_default()
            .insert(f.name.as_str());
    }

    let referenced = idents(&format!("{LUA_SUPPORT}{body}"));
    let referenced_names = |names: &[&str]| -> Vec<String> {
        names
            .iter()
            .filter(|n| referenced.contains(**n))
            .map(|n| (*n).to_string())
            .collect()
    };
    let mut uses: Vec<String> = Vec::new();
    uses.push("use core::ffi::{CStr, c_char, c_int};".into());
    uses.push("use core::ptr;".into());
    for (module, names) in &api_imports {
        uses.push(format!(
            "use crate::src::nvim::api::{module}::{{{}}};",
            names.iter().copied().collect::<Vec<_>>().join(", ")
        ));
    }
    let dispatch: Vec<String> = referenced
        .iter()
        .filter(|n| n.ends_with("_get_field") || n.ends_with("_table"))
        .cloned()
        .collect();
    if !dispatch.is_empty() {
        uses.push(format!(
            "use crate::src::nvim::api::private::dispatch::{{{}}};",
            dispatch.join(", ")
        ));
    }
    uses.push(format!(
        "use crate::src::nvim::api::private::helpers::{{{}}};",
        referenced_names(&[
            "api_clear_error",
            "api_free_dict",
            "api_free_object",
            "api_free_string",
            "api_luarefs_free_keydict",
            "api_luarefs_free_object",
            "api_set_error",
        ])
        .join(", ")
    ));
    if referenced.contains("expr_map_locked") {
        uses.push("use crate::src::nvim::ex_docmd::expr_map_locked;".into());
    }
    if referenced.contains("text_locked") {
        uses.push("use crate::src::nvim::ex_getln::{get_text_locked_msg, text_locked};".into());
    }
    uses.push(format!(
        "use crate::src::nvim::lua::converter::{{{}}};",
        referenced_names(&[
            "kNluaPushFreeRefs",
            "kNluaPushSpecial",
            "nlua_pop_Array",
            "nlua_pop_Boolean",
            "nlua_pop_Dict",
            "nlua_pop_Float",
            "nlua_pop_Integer",
            "nlua_pop_LuaRef",
            "nlua_pop_Object",
            "nlua_pop_String",
            "nlua_pop_handle",
            "nlua_pop_keydict",
            "nlua_push_Array",
            "nlua_push_Boolean",
            "nlua_push_Dict",
            "nlua_push_Float",
            "nlua_push_Integer",
            "nlua_push_Object",
            "nlua_push_String",
            "nlua_push_handle",
            "nlua_push_keydict",
        ])
        .join(", ")
    ));
    uses.push(format!(
        "use crate::src::nvim::lua::executor::{{{}}};",
        referenced_names(&[
            "LUA_INTERNAL_CALL",
            "active_lstate",
            "api_free_luaref",
            "nlua_is_deferred_safe",
        ])
        .join(", ")
    ));
    uses.push(format!(
        "use crate::src::nvim::lua::ffi::{{{}}};",
        referenced_names(&[
            "lua_concat",
            "lua_createtable",
            "lua_error",
            "lua_gettop",
            "lua_pushcclosure",
            "lua_pushstring",
            "lua_setfield",
            "luaL_error",
            "luaL_where",
        ])
        .join(", ")
    ));
    uses.push(format!(
        "use crate::src::nvim::main::{{{}}};",
        referenced_names(&["e_fast_api_disabled", "e_textlock", "textlock"]).join(", ")
    ));
    uses.push("use crate::src::nvim::memory::{ARENA_EMPTY, arena_finish, arena_mem_free};".into());
    // `ErrorType` is unconditional: `mod known` names it, and that block is
    // outside the text the reference scan covers.
    let types: Vec<String> = ["ErrorType"]
        .iter()
        .map(|s| (*s).to_string())
        .chain(referenced_names(&[
            "Arena",
            "Error",
            "LuaRef",
            "Object",
            "lua_State",
        ]))
        .chain(
            referenced
                .iter()
                .filter(|n| n.starts_with("KeyDict_") && !n.ends_with("_get_field"))
                .cloned(),
        )
        .collect();
    uses.push(format!(
        "use crate::src::nvim::types::{{{}}};",
        types.join(", ")
    ));

    let mut out = String::from(LUA_HEADER);
    out.push('\n');
    for file in &files {
        writeln!(out, "mod {};", file.name.strip_suffix(".rs").unwrap()).unwrap();
    }
    out.push('\n');
    for file in &files {
        writeln!(
            out,
            "pub use self::{}::*;",
            file.name.strip_suffix(".rs").unwrap()
        )
        .unwrap();
    }
    out.push('\n');
    for line in uses {
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str(
        r#"
/// Values that belong to other modules; nested so they stay out of the flat
/// namespace the unit-test header generator collects constants into.
mod known {
    use super::ErrorType;

    pub const kErrorTypeNone: ErrorType = -1;
    pub const kErrorTypeException: ErrorType = 0;
    pub const kErrorTypeValidation: ErrorType = 1;
}

use known::*;
"#,
    );
    out.push_str(LUA_SUPPORT);

    files.insert(
        0,
        Emitted {
            name: "mod.rs".into(),
            text: out,
        },
    );
    Ok(files)
}

// ---------------------------------------------------------------- driver

/// Format a generated module through rustfmt's stdin. `--config-path` is
/// explicit because reading from stdin gives rustfmt no file to discover
/// rustfmt.toml (edition 2024) from, and the default edition formats
/// differently.
fn rustfmt(config: &Path, text: &str) -> Result<String, String> {
    let mut child = Command::new("rustfmt")
        .arg("--config-path")
        .arg(config)
        .arg("--emit")
        .arg("stdout")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("rustfmt: {e}"))?;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(text.as_bytes())
        .map_err(|e| format!("rustfmt: {e}"))?;
    let out = child
        .wait_with_output()
        .map_err(|e| format!("rustfmt: {e}"))?;
    if !out.status.success() {
        return Err("rustfmt rejected the generated module".into());
    }
    String::from_utf8(out.stdout).map_err(|e| format!("rustfmt: {e}"))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("apigen: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut root = None;
    let mut spec_path = None;
    let mut out_dir = None;
    let mut tables_dir = None;
    let mut lua_dir = None;
    let mut config = None;
    let mut check = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{arg} needs a value"));
        match arg.as_str() {
            "--root" => root = Some(PathBuf::from(value()?)),
            "--spec" => spec_path = Some(PathBuf::from(value()?)),
            "--out-dir" => out_dir = Some(PathBuf::from(value()?)),
            "--tables-dir" => tables_dir = Some(PathBuf::from(value()?)),
            "--lua-dir" => lua_dir = Some(PathBuf::from(value()?)),
            "--rustfmt-config" => config = Some(PathBuf::from(value()?)),
            "--check" => check = true,
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    let root = root.ok_or("--root is required")?;
    let spec_path = spec_path.ok_or("--spec is required")?;
    let out_dir = out_dir.ok_or("--out-dir is required")?;
    let tables_dir = tables_dir.ok_or("--tables-dir is required")?;
    let lua_dir = lua_dir.ok_or("--lua-dir is required")?;
    let config = config.ok_or("--rustfmt-config is required")?;

    let api = collect_api_fns(&root)?;
    let keysets = collect_keysets(&root)?;
    let specs = parse_spec(&spec_path)?;
    let sizes: BTreeMap<String, usize> =
        keysets.iter().map(|k| (k.name.clone(), k.len())).collect();
    let trees = [
        (
            out_dir,
            generate(&api, &specs, &sizes, &config)?,
            "wrappers",
        ),
        (
            tables_dir,
            generate_tables(&root, &keysets, &specs, &config)?,
            "tables",
        ),
        (lua_dir, generate_lua(&api, &specs, &config)?, "Lua binding"),
    ];

    let mut wrote = false;
    for (dir, mut files, what) in trees {
        for file in &mut files {
            file.text = rustfmt(&config, &file.text)?;
            // The chunker works on unformatted text; if the margin it leaves
            // was not enough, say so rather than let the ratchet find out.
            let lines = file.text.lines().count();
            if lines > 1000 {
                return Err(format!(
                    "{} came out {lines} lines; lower CHUNK_BUDGET",
                    file.name
                ));
            }
        }

        let mut stale: BTreeSet<String> = std::fs::read_dir(&dir)
            .map(|dir| {
                dir.filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().into_owned())
                    .filter(|n| n.ends_with(".rs"))
                    .collect()
            })
            .unwrap_or_default();
        let mut changed = Vec::new();
        for file in &files {
            stale.remove(&file.name);
            let path = dir.join(&file.name);
            if std::fs::read_to_string(&path).unwrap_or_default() != file.text {
                changed.push(path);
            }
        }
        if check {
            if let Some(path) = changed.first() {
                return Err(format!("{} is stale; run `just apigen`", path.display()));
            }
            if let Some(name) = stale.iter().next() {
                return Err(format!(
                    "{} is left over from an older spec; run `just apigen`",
                    dir.join(name).display()
                ));
            }
            continue;
        }
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for file in &files {
            let path = dir.join(&file.name);
            if changed.contains(&path) {
                std::fs::write(&path, &file.text)
                    .map_err(|e| format!("{}: {e}", path.display()))?;
            }
        }
        for name in &stale {
            std::fs::remove_file(dir.join(name)).ok();
        }
        if !changed.is_empty() || !stale.is_empty() {
            wrote = true;
            eprintln!(
                "apigen: wrote {} ({what}, {} files)",
                dir.display(),
                files.len()
            );
        }
    }
    if wrote {
        eprintln!(
            "apigen: {} wrappers, {} handlers, {} keysets, {} Lua bindings",
            specs.iter().filter(|s| s.is_wrapper()).count(),
            specs.iter().filter(|s| s.is_method()).count(),
            keysets.len(),
            specs.iter().filter(|s| s.has_lua_binding()).count(),
        );
    }
    Ok(())
}
