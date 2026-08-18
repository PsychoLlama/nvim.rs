// apigen: generate the msgpack-RPC dispatch layer from the Rust sources
// themselves.
//
// Upstream generated `dispatch_wrappers.generated.h` and
// `keysets_defs.generated.h` by parsing the C headers under src/api/
// (src/gen/gen_api_dispatch.lua at tag v0.12.4). The transpile froze that
// output into one 27k-line Rust module. This tool takes the job back, from
// the two real sources of truth in the crate:
//
//   --out-dir     one wrapper per API function under <root>/src/api/ (bar
//                 `private/`): it validates an `Array` of msgpack arguments,
//                 converts them, calls the API function and boxes the result
//                 back into an `Object`. An API function is either transpiled
//                 (`pub unsafe extern "C" fn`, reporting failure through an
//                 `*mut Error` out-parameter) or converted (`pub [unsafe] fn`
//                 answering `Result<T, Error>`); see `ApiFn`.
//   --tables-dir  the keyset tables and their key lookups, read off the
//                 `KeyDict_*` structs in <root>/src/types/keysets.rs,
//                 plus the handler table and its method lookup.
//   --lua-dir     the `vim.api` Lua binding: the same conversion job again,
//                 against the Lua stack rather than an argument `Array`, plus
//                 the table that hangs the bindings off their names.
//   --metadata-file
//                 the packed api-info metadata: the msgpack dict
//                 `nvim --api-info` and `nvim_get_api_info()` hand back, so
//                 clients can discover the API. Its sections that no Rust
//                 source declares — the UI events, the error kinds, the
//                 handle types and the version's API levels — come from a
//                 second spec (`--metadata-spec`).
//   --cmdtable-file
//                 the Ex command table and its lookup indices, and
//   --cmdidx-file the `CMD_*` names that index it, both from the vendored
//                 `ex_cmds.lua` (`--ex-cmds-lua`).
//   --options-dir the option table, from the vendored `options.lua` upstream
//                 fed to src/gen/gen_options.lua. That one is metadata, not
//                 Rust, so it is read by the small Lua reader in `lua.rs`
//                 rather than by syn; see `options.rs`.
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
// under the row numbers eval/funcs/ has baked in.
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

mod eval_funcs;
mod ex_cmds;
mod lua;
mod options;

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

impl RetType {
    /// The C name of the return type, as the metadata's `return_type` starts
    /// from. Only the decorations Rust cannot express need a `ret=` override.
    fn declared(&self) -> String {
        match self {
            RetType::Void => "void".into(),
            RetType::Boolean => "Boolean".into(),
            RetType::Integer => "Integer".into(),
            RetType::Float => "Float".into(),
            RetType::String => "String".into(),
            RetType::Array => "Array".into(),
            RetType::Dict => "Dict".into(),
            RetType::Object => "Object".into(),
            RetType::Handle(name) => (*name).into(),
            RetType::KeyDict(name) => format!("Dict({name})"),
        }
    }
}

/// One API function as parsed out of the crate.
///
/// Two signature shapes say the same thing, and the crate holds both while
/// the hand-written layer is converted one function at a time:
///
/// ```ignore
/// pub unsafe extern "C" fn nvim_x(args.., arena: *mut Arena, err: *mut Error) -> T
/// pub [unsafe] fn nvim_x(args.., arena: *mut Arena) -> Result<T, Error>
/// ```
///
/// The first reports failure through an out-parameter the caller has to
/// inspect afterwards; the second through its result. Everything downstream
/// — the RPC wrapper, the Lua binding, the metadata — sees the same `params`
/// and the same `ret` either way; only [`ApiFn::fallible`] differs, and only
/// the shape of the call the wrappers emit follows from it.
struct ApiFn {
    name: String,
    /// Module path segment under `crate::api::` (the file stem).
    module: String,
    params: Vec<Param>,
    /// What a successful call answers with. `Result<T, Error>` records `T`,
    /// so a conversion to the `Result` shape leaves the metadata untouched.
    ret: RetType,
    /// The function reports failure by returning `Err`, rather than through
    /// an `*mut Error` out-parameter. Mutually exclusive with [`Param::Error`].
    fallible: bool,
    /// The function is `unsafe` to call, so the wrappers wrap the call. A
    /// converted function whose arguments are all values need not be.
    is_unsafe: bool,
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
    /// Upstream's `FUNC_API_REMOTE_ONLY`.
    remote_only: bool,
    /// Upstream's `FUNC_API_LUA_ONLY`.
    lua_only: bool,
    /// The API level the method appeared at, or -1 for the internal `nvim__*`
    /// ones. Required on every `nvim_`-prefixed entry: the metadata publishes
    /// it, and the Lua binding reads it to decide how to convert the result —
    /// see [`Spec::push_special`]. The deprecated spellings, which are not
    /// `nvim_`-prefixed, were all there from the start and are pinned at 0.
    since: Option<i32>,
    /// The API level the method was deprecated at, if it has been. Only the
    /// metadata reads it. The deprecated spellings are pinned at 1.
    deprecated_since: Option<i32>,
    /// The declared C name of the return type, for returns whose Rust type has
    /// lost the decoration: `ret=ArrayOf(String)`. Only the metadata reads it
    /// — nothing in a wrapper depends on what the result is called.
    ret: Option<String>,
}

impl Spec {
    /// Whether this entry gets an RPC wrapper of its own.
    fn is_wrapper(&self) -> bool {
        self.alias.is_none() && self.handler.is_none() && self.is_method()
    }

    /// Whether this entry is one of the methods the RPC dispatcher answers
    /// to, and so takes a row in the handler table. Upstream's `f.remote`.
    fn is_method(&self) -> bool {
        self.remote_only || !self.lua_only
    }

    /// Whether this entry gets a `vim.api.<name>` Lua binding — upstream's
    /// `f.lua`. A deprecated spelling does not: upstream exposed the old
    /// names over RPC only.
    fn has_lua_binding(&self) -> bool {
        self.alias.is_none() && self.handler.is_none() && (self.lua_only || !self.remote_only)
    }

    /// Whether this entry also answers to its own name in Vimscript, called
    /// through its row of the handler table — upstream's `f.eval`. Both
    /// `remote_only` and `lua_only` suppress it, so a method spelled with
    /// *both* (`nvim_chan_send`) has an RPC wrapper and a Lua binding but no
    /// Vimscript one. Only the `nvim_`-prefixed names qualify: the legacy
    /// spellings were never given a builtin.
    fn has_eval_binding(&self) -> bool {
        self.name.starts_with("nvim_")
            && self.alias.is_none()
            && self.handler.is_none()
            && !self.remote_only
            && !self.lua_only
    }

    /// Whether the Lua binding converts this method's result the pre-0.11 way
    /// — `nil` and the other special values keep their old spelling. Upstream
    /// froze that for everything that predates API level 11 because clients
    /// depend on it, and used the modern conversion for newer methods.
    fn push_special(&self) -> bool {
        self.metadata_since() < 11
    }

    /// The API level the metadata publishes: the declared one, or 0 for a
    /// deprecated spelling.
    fn metadata_since(&self) -> i32 {
        self.since.unwrap_or(0)
    }

    /// Whether this entry is one of the deprecated spellings upstream kept
    /// for clients that predate the `nvim_` naming. They were all present at
    /// API level 0 and all deprecated at level 1, so neither is spelled out.
    fn is_legacy_spelling(&self) -> bool {
        !self.name.starts_with("nvim_")
    }

    /// Whether the api-info metadata describes this entry. The internal
    /// `nvim__*` functions and the internal error event are deliberately
    /// unpublished, and `redraw` is not an API function at all — it is the UI
    /// client's own notification handler.
    fn in_metadata(&self) -> bool {
        !self.name.starts_with("nvim__")
            && self.name != "nvim_error_event"
            && self.handler.is_none()
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

/// For `Result<T, Error>`, the `T`. The error half has to be the API's own
/// `Error`, however it is spelled: a `Result` over anything else is not an
/// API signature, and is left for the caller to reject.
fn result_ok_type(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(path) = ty else {
        return None;
    };
    let last = path.path.segments.last()?;
    if last.ident != "Result" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
        return None;
    };
    let mut types = args.args.iter().filter_map(|arg| match arg {
        syn::GenericArgument::Type(ty) => Some(ty),
        _ => None,
    });
    let (ok, err) = (types.next()?, types.next()?);
    (type_name(err)? == "Error").then_some(ok)
}

/// What a call answers with, and whether it says so through a `Result` — the
/// two shapes [`ApiFn`] documents. `Result<T, Error>` classifies as `T`, and
/// `Result<(), Error>` as void, so the two shapes agree on everything but
/// how failure travels.
fn ret_type(ret: &syn::ReturnType) -> Option<(RetType, bool)> {
    let ty = match ret {
        syn::ReturnType::Default => return Some((RetType::Void, false)),
        syn::ReturnType::Type(_, ty) => ty,
    };
    match result_ok_type(ty) {
        Some(ok) => Some((value_ret(ok)?, true)),
        None => Some((value_ret(ty)?, false)),
    }
}

/// The value half of a return type: what the wrappers box into an `Object`.
fn value_ret(ty: &syn::Type) -> Option<RetType> {
    // `Result<(), Error>` is the converted spelling of a `void` function.
    if matches!(ty, syn::Type::Tuple(t) if t.elems.is_empty()) {
        return Some(RetType::Void);
    }
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

/// Every `.rs` file at or below `dir`, in path order, skipping `api/private`
/// — the plumbing, which is not an API surface and which holds this tool's
/// own output.
fn api_sources(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "private") {
                continue;
            }
            api_sources(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
    Ok(())
}

/// Collect the API functions in `<root>/src/api/`: every
/// `pub unsafe extern "C" fn`, plus every `pub [unsafe] fn` that answers with
/// a `Result<_, Error>` — the two shapes [`ApiFn`] documents. Anything else a
/// source file holds is support code, and is passed over.
///
/// An API source file over the tree's 1,000-line cap is carved into
/// `api/<stem>/mod.rs` plus `api/<stem>/*.rs`, with the parent re-exporting its
/// children, so an API function may sit in a child and still be reached as
/// `api::<stem>::nvim_foo`. The module recorded here is therefore always the
/// *top-level* stem, however deep the function itself lives: it is what the
/// generated `use` lines name and what the per-module wrapper split is keyed
/// on, so a carve moves no wrapper.
fn collect_api_fns(root: &Path) -> Result<BTreeMap<String, ApiFn>, String> {
    let dir = root.join("src/api");
    let mut out: BTreeMap<String, ApiFn> = BTreeMap::new();
    let mut entries: Vec<PathBuf> = Vec::new();
    api_sources(&dir, &mut entries)?;
    for path in entries {
        let rel = path.strip_prefix(&dir).expect("walked from dir");
        let top = rel.components().next().expect("a file has one component");
        let module = Path::new(top.as_os_str())
            .file_stem()
            .expect("a path component has a stem")
            .to_string_lossy()
            .into_owned();
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let file = syn::parse_file(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        for item in &file.items {
            let syn::Item::Fn(f) = item else { continue };
            if !matches!(f.vis, syn::Visibility::Public(_)) {
                continue;
            }
            let Some((ret, fallible)) = ret_type(&f.sig.output) else {
                continue;
            };
            // The transpiled shape is recognised by its calling convention;
            // the converted one by its `Result`, which no support function in
            // an API source file returns.
            let transpiled = f.sig.unsafety.is_some()
                && matches!(&f.sig.abi, Some(abi) if abi.name.as_ref().is_none_or(|n| n.value() == "C"));
            if !transpiled && !fallible {
                continue;
            }
            let name = f.sig.ident.to_string();
            let Ok(params) = classify(&f.sig) else {
                continue;
            };
            // A `Result` is the whole story: a function that also took the
            // out-parameter would have two places to say the same thing, and
            // the wrappers would have to check both.
            if fallible && params.contains(&Param::Error) {
                return Err(format!(
                    "{name} answers with a Result and takes an `*mut Error` out-parameter; \
                     one or the other"
                ));
            }
            // Two files may hold a same-named private helper, but a name the
            // spec can reach must resolve to exactly one function.
            if let Some(prev) = out.insert(
                name.clone(),
                ApiFn {
                    name: name.clone(),
                    module: module.clone(),
                    params,
                    ret,
                    fallible,
                    is_unsafe: f.sig.unsafety.is_some(),
                },
            ) {
                return Err(format!(
                    "{name} is declared twice: in api::{} and in api::{module}",
                    prev.module
                ));
            }
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
    let path = root.join("src/types/keysets.rs");
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
/// of the handler table, whose indices `eval/funcs/` has baked in.
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
                Some(("deprecated_since", value)) => {
                    spec.deprecated_since = Some(
                        value
                            .parse::<i32>()
                            .map_err(|_| at(format!("bad API level in `{word}`")))?,
                    )
                }
                Some(("ret", value)) => spec.ret = Some(value.replace('_', " ")),
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
        if spec.lua_only && (spec.alias.is_some() || spec.handler.is_some()) {
            return Err(at(
                "a lua_only method is not dispatched, so it has no alias or handler".into(),
            ));
        }
        if spec.is_legacy_spelling() && (spec.since.is_some() || spec.deprecated_since.is_some()) {
            return Err(at(
                "a deprecated spelling is pinned at since=0 deprecated_since=1".into(),
            ));
        }
        if !spec.is_legacy_spelling() && spec.since.is_none() {
            return Err(at("since= is required on every nvim_ method".into()));
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

/// The `as_*` reader that turns argument `index` into a parameter, or `None`
/// if the tag does not match.
fn reader(ty: &ApiType, index: usize) -> String {
    let item = format!("args[{index}]");
    match ty {
        ApiType::Boolean => format!("as_boolean({item})"),
        ApiType::Integer => format!("as_integer({item})"),
        ApiType::Float => format!("as_float({item})"),
        ApiType::String => format!("as_string({item})"),
        ApiType::Array => format!("as_array({item})"),
        ApiType::Dict => format!("as_dict({item})"),
        ApiType::LuaRef => format!("as_luaref({item})"),
        ApiType::Handle(_) => format!("as_handle({item}, {})", object_tag(ty)),
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
    let can_fail = f.fallible || f.params.contains(&Param::Error);

    for n in spec.declared.keys() {
        if *n > arity {
            return Err(format!(
                "{name}: spec overrides arg{n} but it takes {arity}"
            ));
        }
    }

    writeln!(out, "pub unsafe fn {handler}(").unwrap();
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
    writeln!(
        out,
        "    // SAFETY: the dispatcher hands over an argument array of `size` initialized\n\
         \x20   // objects and an `Error` slot that is live and ours alone until we return;\n\
         \x20   // both outlive the call."
    )
    .unwrap();
    writeln!(
        out,
        "    let (args, error) = unsafe {{ (args_slice(&args), &mut *error) }};"
    )
    .unwrap();
    writeln!(
        out,
        "    log_invoke(c\"{handler}\", c\"{name}\", line!() as c_int, channel_id);"
    )
    .unwrap();
    writeln!(out, "    if args.len() != {arity} {{").unwrap();
    writeln!(out, "        wrong_arity(error, {arity}, args.len());").unwrap();
    writeln!(out, "        return NIL;").unwrap();
    writeln!(out, "    }}").unwrap();

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
                writeln!(out, "    let arg_{slot} = args[{index}];").unwrap();
            }
            ApiType::KeyDict(keyset) => {
                let get_field = format!("KeyDict_{keyset}_get_field");
                writeln!(out, "    let mut arg_{slot}: KeyDict_{keyset} =").unwrap();
                writeln!(
                    out,
                    "        match read_keydict(Some({get_field}), args[{index}], error) {{"
                )
                .unwrap();
                writeln!(out, "            KeySetArg::Read(v) => v,").unwrap();
                writeln!(out, "            KeySetArg::Refused => return NIL,").unwrap();
                writeln!(out, "            KeySetArg::WrongType => {{").unwrap();
                writeln!(out, "                {bad}").unwrap();
                writeln!(out, "                return NIL;").unwrap();
                writeln!(out, "            }}").unwrap();
                writeln!(out, "        }};").unwrap();
            }
            _ => {
                writeln!(
                    out,
                    "    let Some(arg_{slot}) = {} else {{",
                    reader(ty, *index)
                )
                .unwrap();
                writeln!(out, "        {bad}").unwrap();
                writeln!(out, "        return NIL;").unwrap();
                writeln!(out, "    }};").unwrap();
            }
        }
    }

    // The two locks a wrapper may be refused by. Reading them touches editor
    // globals, which only the main loop -- where a wrapper runs -- has set up.
    if spec.textlock {
        writeln!(out, "    // SAFETY: a wrapper runs on the main loop.").unwrap();
        writeln!(out, "    if unsafe {{ text_locked() }} {{").unwrap();
        writeln!(out, "        text_locked_error(error);").unwrap();
        writeln!(out, "        return NIL;").unwrap();
        writeln!(out, "    }}").unwrap();
    } else if spec.textlock_allow_cmdwin {
        writeln!(out, "    // SAFETY: a wrapper runs on the main loop.").unwrap();
        writeln!(
            out,
            "    if textlock.get() != 0 || unsafe {{ expr_map_locked() }} {{"
        )
        .unwrap();
        writeln!(out, "        expr_map_locked_error(error);").unwrap();
        writeln!(out, "        return NIL;").unwrap();
        writeln!(out, "    }}").unwrap();
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
    let args = call_args.join(", ");
    let call = match f.is_unsafe {
        // An API function still on the transpiled shape is the one edge every
        // wrapper has. Everything it is handed was checked above.
        true => format!("unsafe {{ {name}({args}) }}"),
        false => format!("{name}({args})"),
    };
    if f.is_unsafe {
        writeln!(
            out,
            "    // SAFETY: each argument was checked against the type the signature declares;\n\
             \x20   // `arena` and `error` are the dispatcher's own."
        )
        .unwrap();
    }
    // An `Object` result needs no boxing, so when nothing follows the call it
    // is the wrapper's tail expression rather than a binding.
    if f.ret == RetType::Object && !can_fail {
        writeln!(out, "    {call}").unwrap();
        writeln!(out, "}}").unwrap();
        return Ok(());
    }
    // Converting a keyset result to a Dict takes it by pointer.
    let bind = match &f.ret {
        RetType::KeyDict(_) => "mut rv",
        _ => "rv",
    };
    if f.fallible {
        // The failure travels back in the result, so it is over as soon as it
        // is matched: `failure` moves it into the dispatcher's slot.
        match &f.ret {
            RetType::Void => {
                writeln!(out, "    if let Err(e) = {call} {{").unwrap();
                writeln!(out, "        return failure(error, e);").unwrap();
                writeln!(out, "    }}").unwrap();
            }
            _ => {
                writeln!(out, "    let {bind} = match {call} {{").unwrap();
                writeln!(out, "        Ok(rv) => rv,").unwrap();
                writeln!(out, "        Err(e) => return failure(error, e),").unwrap();
                writeln!(out, "    }};").unwrap();
            }
        }
    } else {
        let bind = match &f.ret {
            RetType::Void => String::new(),
            _ => format!("let {bind} = "),
        };
        writeln!(out, "    {bind}{call};").unwrap();
        if can_fail {
            writeln!(out, "    if error.type_0 != kErrorTypeNone {{").unwrap();
            writeln!(out, "        return NIL;").unwrap();
            writeln!(out, "    }}").unwrap();
        }
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
            writeln!(
                out,
                "    // SAFETY: `rv` is a `KeyDict_{keyset}`, whose field table is\n\
                 \x20   // `{keyset}_table` and whose length is {size}."
            )
            .unwrap();
            writeln!(out, "    let dict = unsafe {{").unwrap();
            writeln!(
                out,
                "        api_keydict_to_dict((&raw mut rv).cast(), {keyset}_table.ptr().cast(), {size} as size_t, arena)"
            )
            .unwrap();
            writeln!(out, "    }};").unwrap();
            "obj(kObjectTypeDict, object_data { dict })".into()
        }
    };
    writeln!(out, "    {boxed}").unwrap();
    writeln!(out, "}}").unwrap();
    Ok(())
}

const HEADER: &str = r#"//! Dispatch wrappers for the msgpack-RPC API.
//!
//! GENERATED by tools/apigen from the `nvim_*` signatures under
//! `crate::api` plus `tools/apigen/functions.txt`. Do not edit;
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
        "//! Dispatch wrappers for `crate::api::{module}`{of}.\n\
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

/// The arguments a client sent, as a slice. An empty array need not carry a
/// pointer at all, so that case answers without forming one.
///
/// # Safety
/// `args.items` points at `args.size` initialized `Object`s that outlive the
/// borrow.
unsafe fn args_slice(args: &Array) -> &[Object] {
    if args.size == 0 {
        return &[];
    }
    // SAFETY: the caller vouches for `size` objects at `items`.
    unsafe { core::slice::from_raw_parts(args.items, args.size) }
}

/// One "RPC: ch N: invoke nvim_foo" debug line. Below the configured log
/// level — the default — this is a load and a compare.
fn log_invoke(handler: &CStr, method: &CStr, line: c_int, channel_id: uint64_t) {
    let fmt = c"RPC: ch %lu: invoke %s".as_ptr();
    let (handler, method) = (handler.as_ptr(), method.as_ptr());
    // SAFETY: the format string is this function's own and matches the two
    // arguments after it; every string here is NUL-terminated and outlives
    // the call.
    unsafe {
        logmsg_c!(LOGLVL_DBG, core::ptr::null(), handler, line, true, fmt, channel_id, method);
    }
}

/// Refuses a call that arrived with the wrong number of arguments.
fn wrong_arity(error: &mut Error, expected: usize, got: usize) {
    let fmt = c"Wrong number of arguments: expecting %zu but got %zu".as_ptr();
    // SAFETY: `error` is live and the format string matches its two arguments.
    unsafe { api_set_error(error, kErrorTypeException, fmt, expected as size_t, got as size_t) };
}

/// Refuses a call whose argument in `slot` carried a tag the parameter does
/// not accept.
fn wrong_type(error: &mut Error, slot: usize, func: &CStr, expected: &CStr) {
    let fmt = c"Wrong type for argument %zu when calling %s, expecting %s".as_ptr();
    let (slot, func, expected) = (slot as size_t, func.as_ptr(), expected.as_ptr());
    // SAFETY: `error` is live, the format string matches its three arguments,
    // and both names are NUL-terminated and outlive the call.
    unsafe { api_set_error(error, kErrorTypeException, fmt, slot, func, expected) };
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
    (
        "read_keydict",
        r#"
/// What reading a keyset argument produced.
enum KeySetArg<K> {
    /// Decoded, with `error` untouched.
    Read(K),
    /// The decoder rejected a key; `error` says which and why.
    Refused,
    /// The argument was neither a Dict nor the empty list that stands in for
    /// an empty one.
    WrongType,
}

/// Decode one keyset argument into a fresh `K`.
///
/// `get_field` must be `K`'s own generated field lookup: the decoder writes
/// through the offsets it hands back, so pairing it with a different keyset
/// would write outside `K`.
fn read_keydict<K>(get_field: FieldHashfn, item: Object, error: &mut Error) -> KeySetArg<K> {
    if item.type_0 != kObjectTypeDict {
        if !is_empty_array(item) {
            return KeySetArg::WrongType;
        }
        // SAFETY: as below; an empty list sets no field.
        return KeySetArg::Read(unsafe { core::mem::zeroed() });
    }
    // SAFETY: as above.
    let mut out: K = unsafe { core::mem::zeroed() };
    // SAFETY: `get_field` is `K`'s own lookup, per the contract above, so the
    // offsets it hands back are inside `out`; the tag says the dict arm of the
    // union is the live one.
    let read = unsafe { api_dict_to_keydict((&raw mut out).cast(), get_field, item.data.dict, error) };
    if read {
        KeySetArg::Read(out)
    } else {
        KeySetArg::Refused
    }
}
"#,
    ),
    (
        "text_locked_error",
        r#"
/// Refuses a call made while the text is locked.
fn text_locked_error(error: &mut Error) {
    let fmt = c"%s".as_ptr();
    // SAFETY: `error` is live and `get_text_locked_msg` answers with a static
    // NUL-terminated message, which is what `%s` takes.
    unsafe { api_set_error(error, kErrorTypeException, fmt, get_text_locked_msg()) };
}
"#,
    ),
    (
        "expr_map_locked_error",
        r#"
/// Refuses a call made from an expression mapping, which the cmdline window
/// alone would have allowed.
fn expr_map_locked_error(error: &mut Error) {
    let fmt = c"%s".as_ptr();
    // SAFETY: `error` is live and `e_textlock` is a static NUL-terminated
    // message, which is what `%s` takes.
    unsafe { api_set_error(error, kErrorTypeException, fmt, &raw const e_textlock) };
}
"#,
    ),
    (
        "failure",
        r#"
/// Hands the error an API function answered with to the dispatcher, which
/// reads it out of the slot it lent the wrapper. The wrapper's own result is
/// nil, as it is for every other way of refusing.
fn failure(error: &mut Error, e: Error) -> Object {
    *error = e;
    NIL
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

/// Names re-exported by `crate::types`, emitted when referenced.
const TYPE_NAMES: &[&str] = &[
    "Arena",
    "Array",
    "Boolean",
    "Dict",
    "Error",
    "ErrorType",
    "FieldHashfn",
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
    // batch here and are re-split on their `pub unsafe fn` openers.
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
        if name.starts_with("kObjectType") || name.starts_with("kErrorType") {
            // These have one definition, in `types`. Re-export rather than
            // repeat it -- a `use` is not a constant, so it still stays out
            // of what the unit-test header generator collects.
            known.push_str(&format!("    pub use crate::types::{name};\n"));
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
            "use crate::api::{module}::{{{}}};",
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
            "use crate::api::private::dispatch::{{{}}};",
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
        "use crate::api::private::helpers::{{{}}};",
        helpers.join(", ")
    ));
    if referenced.contains("expr_map_locked") {
        uses.push("use crate::ex_docmd::expr_map_locked;".into());
        uses.push("use crate::main::{e_textlock, textlock};".into());
    }
    if referenced.contains("text_locked") {
        uses.push("use crate::ex_getln::{get_text_locked_msg, text_locked};".into());
    }
    uses.push("use crate::log::logmsg_c;".into());
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
    uses.push(format!("use crate::types::{{{}}};", types.join(", ")));

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
//! `crate::types::keysets` plus `tools/apigen/functions.txt`. Do
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
//! `eval/funcs/` stores to bind the builtin `nvim_*()` Vimscript functions.
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
    f: unsafe fn(uint64_t, Array, *mut Arena, *mut Error) -> Object,
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
pub unsafe fn msgpack_rpc_get_handler_for(
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

/// The handler table's layout: the dispatched methods, and the order their
/// rows sit in.
///
/// The order is `table_order` over the method names *sorted*, not over the
/// spec file's arrangement. Upstream fed its own header-declaration order in;
/// sorting reproduces the same table (checked against the frozen one) while
/// making the result independent of how `functions.txt` is grouped.
fn handler_layout(specs: &[Spec]) -> (Vec<&Spec>, Vec<usize>) {
    // A lua_only method is not dispatched, so it takes no row.
    let mut sorted: Vec<&Spec> = specs.iter().filter(|s| s.is_method()).collect();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    let names: Vec<String> = sorted.iter().map(|s| s.name.clone()).collect();
    let order = table_order(&names);
    (sorted, order)
}

/// Which row of the handler table each method sits in. The builtin-function
/// table bakes these numbers into its `nvim_*()` bindings, so both generators
/// read the layout from here and the two cannot drift.
fn handler_rows(specs: &[Spec]) -> BTreeMap<&str, usize> {
    let (specs, order) = handler_layout(specs);
    order
        .iter()
        .enumerate()
        .map(|(row, &i)| (specs[i].name.as_str(), row))
        .collect()
}

/// The handler table and the lookup that indexes it.
fn emit_handlers(out: &mut String, specs: &[Spec]) {
    let by_name: BTreeMap<&str, &Spec> = specs.iter().map(|s| (s.name.as_str(), s)).collect();
    let (specs, order) = handler_layout(specs);

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
    emit_handlers(&mut all, specs);
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
    out.push_str("use crate::api::private::dispatch_wrappers::*;\n");
    out.push_str("use crate::api::private::helpers::api_set_error;\n");
    out.push_str("use crate::global_cell::GlobalCell;\n");
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
    writeln!(out, "use crate::types::{{{}}};", types.join(", ")).unwrap();
    write!(
        out,
        r#"
/// Values that belong to other modules; nested so they stay out of the flat
/// namespace the unit-test header generator collects constants into.
mod known {{
    use core::ffi::c_int;

    pub use crate::types::kErrorTypeException;

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
//! `crate::api` plus `tools/apigen/functions.txt`. Do not edit;
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
//! releasing whatever was already converted, and only that: each converted
//! argument arms a guard, so returning from a failed conversion drops the
//! guards armed before it and no others, in declaration order. Upstream's
//! generator wrote that order out by hand as a chain of `goto exit_N`
//! (src/gen/gen_api_dispatch.lua at tag v0.12.4).
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

/// What one binding carries from its first conversion to its last release.
struct Call {
    /// Where the conversions and the API function allocate. Released once
    /// every argument has been, since the values the releases walk live in
    /// it.
    arena: Arena,
    /// Why the call failed, if it did.
    err: Error,
    /// The parameter a failed conversion blamed, named in the message.
    err_param: *mut c_char,
}

impl Call {
    const fn new() -> Self {
        Call {
            arena: ARENA_EMPTY,
            err: ERROR_INIT,
            err_param: ptr::null_mut(),
        }
    }
}

/// A binding's own half: pop each argument off the Lua stack, call the API
/// function, hand the result back. Returning — off the end or out of a failed
/// conversion — releases exactly the arguments that were converted, in
/// declaration order, because each one's release is a guard dropped on the
/// way out.
type Convert = unsafe fn(*mut lua_State, &mut Call);

// -- keysets ---------------------------------------------------------------

/// A generated keyset, tied to the two generated items that describe it.
///
/// # Safety
/// `GET_FIELD` must be `Self`'s own field lookup and [`table`](Self::table)
/// answer `Self`'s own key table: the decoder writes through the offsets the
/// first hands back and the release walks the second, so either one belonging
/// to a different keyset would read and write outside `Self`. All zeroes must
/// be a valid `Self`, which is how a keyset argument starts out.
unsafe trait KeySet: Sized {
    const GET_FIELD: FieldHashfn;

    fn table() -> *mut KeySetLink;
}

/// A keyset's generated table, as the code that walks one takes it.
fn keyset_table<const N: usize>(table: &GlobalCell<[KeySetLink; N]>) -> *mut KeySetLink {
    table.ptr().cast()
}

/// A keyset argument, with its release armed from the moment it exists: the
/// decoder fills it field by field, and a fill that stops halfway still holds
/// whatever references it took before it stopped.
struct KeyDictArg<K: KeySet> {
    dict: K,
}

impl<K: KeySet> KeyDictArg<K> {
    fn zeroed() -> Self {
        // SAFETY: all zeroes is a valid `K`, per `KeySet`'s contract.
        KeyDictArg {
            dict: unsafe { core::mem::zeroed() },
        }
    }
}

impl<K: KeySet> Drop for KeyDictArg<K> {
    fn drop(&mut self) {
        // SAFETY: `K::table()` describes `K`'s fields, per `KeySet`'s
        // contract, and the binding owns the references they name.
        unsafe { api_luarefs_free_keydict((&raw mut self.dict).cast(), K::table()) };
    }
}

/// Fill a keyset argument from the Lua value on top of the stack. On refusal
/// `*err_param` names the key that failed.
///
/// # Safety
/// `lstate` is the running Lua state with the argument on top; `arena`,
/// `err` and `err_param` are the binding's own.
unsafe fn pop_keydict<K: KeySet>(
    lstate: *mut lua_State,
    arg: &mut KeyDictArg<K>,
    arena: &mut Arena,
    err: &mut Error,
    err_param: &mut *mut c_char,
) {
    // SAFETY: the caller's stack, and `K::GET_FIELD` is `K`'s own lookup per
    // `KeySet`'s contract, which is what the decoder needs of it.
    unsafe {
        nlua_pop_keydict(
            lstate,
            (&raw mut arg.dict).cast(),
            K::GET_FIELD,
            err_param,
            arena,
            err,
        )
    };
}

/// Hand a keyset result back as a Lua table.
///
/// # Safety
/// `lstate` is the running Lua state and `value` points at the binding's own
/// result.
unsafe fn push_keydict<K: KeySet>(lstate: *mut lua_State, value: *mut K) {
    // SAFETY: the caller's stack, and `K::table()` describes `K`'s fields per
    // `KeySet`'s contract.
    unsafe { nlua_push_keydict(lstate, value.cast(), K::table()) };
}

// -- argument guards -------------------------------------------------------

/// An `Object` argument, which puts the Lua references the conversion took
/// out of the registry back when the binding leaves.
struct ObjectArg {
    value: Object,
}

impl ObjectArg {
    /// # Safety
    /// The binding owns the references `value` names and nothing else
    /// releases them.
    unsafe fn new(value: Object) -> Self {
        ObjectArg { value }
    }
}

impl Drop for ObjectArg {
    fn drop(&mut self) {
        // SAFETY: `new`'s contract.
        unsafe { api_luarefs_free_object(self.value) };
    }
}

/// A `LuaRef` argument, released the same way.
struct LuaRefArg {
    value: LuaRef,
}

impl LuaRefArg {
    /// # Safety
    /// As [`ObjectArg::new`].
    unsafe fn new(value: LuaRef) -> Self {
        LuaRefArg { value }
    }
}

impl Drop for LuaRefArg {
    fn drop(&mut self) {
        // SAFETY: `new`'s contract.
        unsafe { api_free_luaref(self.value) };
    }
}

// -- refusals --------------------------------------------------------------

/// Refuses a call that arrived with a different number of arguments than the
/// API function declares.
fn wrong_arity(err: &mut Error, argc: c_int) {
    let fmt = if argc == 1 {
        c"Expected %d argument".as_ptr()
    } else {
        c"Expected %d arguments".as_ptr()
    };
    // SAFETY: `err` is live and the format string matches its one argument.
    unsafe { api_set_error(err, kErrorTypeValidation, fmt, argc) };
}

// -- the shared half of every binding --------------------------------------

/// Everything a binding does around its own conversions: check that Lua
/// handed it the arguments the API function declares, hand over to `convert`,
/// release the request arena, and raise whatever error the call left behind.
/// Answers the number of results `convert` left on the stack.
///
/// `deferred` names the binding when its API function is not `fast` and so
/// may not run from a context where deferring is unsafe; it is `None` for one
/// that may run anywhere.
///
/// The error is raised from here rather than from `convert`, because by the
/// time `convert` has returned every argument guard has been dropped and the
/// arena is gone — and `lua_error` does not come back.
///
/// # Safety
/// `lstate` is the running Lua state with the binding's arguments on top;
/// `deferred`, when set, outlives the call; `convert` reads `argc` arguments
/// and leaves `nret` results.
unsafe fn run(
    lstate: *mut lua_State,
    deferred: Option<&CStr>,
    argc: c_int,
    nret: c_int,
    convert: Convert,
) -> c_int {
    let mut call = Call::new();
    // SAFETY: the caller's stack.
    if unsafe { lua_gettop(lstate) } != argc {
        wrong_arity(&mut call.err, argc);
    } else {
        // SAFETY: as above; the query has no side effects.
        let refused = deferred.filter(|_| !unsafe { nlua_is_deferred_safe() });
        if let Some(name) = refused {
            let (fmt, name) = ((&raw const e_fast_api_disabled).cast(), name.as_ptr());
            // SAFETY: as above; both strings are static and NUL-terminated,
            // and nothing is left to release.
            return unsafe { luaL_error(lstate, fmt, name) };
        }
        // SAFETY: as above; `call` is this frame's own.
        unsafe { convert(lstate, &mut call) };
    }
    // SAFETY: the arena is this frame's own, and every argument that borrowed
    // from it has been released.
    unsafe { arena_mem_free(arena_finish(&raw mut call.arena)) };
    if call.err.type_0 == kErrorTypeNone {
        return nret;
    }
    // SAFETY: as above; `call.err` carries a message.
    unsafe { stage_error(lstate, &mut call) };
    // SAFETY: the message is on the stack, and `lua_error` does not return.
    unsafe { lua_error(lstate) }
}

/// [`run`] for a binding whose API function is not `fast`.
///
/// # Safety
/// As [`run`].
unsafe fn dispatch(
    lstate: *mut lua_State,
    name: &CStr,
    argc: c_int,
    nret: c_int,
    convert: Convert,
) -> c_int {
    // SAFETY: the caller's.
    unsafe { run(lstate, Some(name), argc, nret, convert) }
}

/// [`run`] for a binding whose API function is `fast`, and so has no context
/// to refuse.
///
/// # Safety
/// As [`run`].
unsafe fn dispatch_fast(
    lstate: *mut lua_State,
    argc: c_int,
    nret: c_int,
    convert: Convert,
) -> c_int {
    // SAFETY: the caller's.
    unsafe { run(lstate, None, argc, nret, convert) }
}

/// Leave the failed call's message on the Lua stack as one string: the source
/// position, the parameter a failed conversion blamed, and the message
/// itself, ready for `lua_error` to raise.
///
/// # Safety
/// `lstate` is the running Lua state and `call` is the binding's own, with
/// an error set.
unsafe fn stage_error(lstate: *mut lua_State, call: &mut Call) {
    let err = &raw mut call.err;
    // SAFETY: the caller's stack; `err` points at the binding's own error,
    // whose message this consumes, and `err_param`, when set, at a static
    // NUL-terminated name.
    unsafe {
        luaL_where(lstate, 1);
        if !call.err_param.is_null() {
            lua_pushstring(lstate, c"Invalid '".as_ptr());
            lua_pushstring(lstate, call.err_param);
            lua_pushstring(lstate, c"': ".as_ptr());
            lua_pushstring(lstate, (*err).msg);
            api_clear_error(err);
            lua_concat(lstate, 5);
        } else {
            lua_pushstring(lstate, (*err).msg);
            api_clear_error(err);
            lua_concat(lstate, 2);
        }
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

/// Lua binding support code emitted only when a binding names it, keyed by
/// that name. Same shape as the RPC wrappers' [`READERS`], and swept the same
/// way.
const LUA_READERS: &[(&str, &str)] = &[
    (
        "text_locked_error",
        r#"
/// Refuses a call made while the text is locked.
fn text_locked_error(err: &mut Error) {
    let fmt = c"%s".as_ptr();
    // SAFETY: `err` is live and `get_text_locked_msg` answers with a static
    // NUL-terminated message, which is what `%s` takes.
    unsafe { api_set_error(err, kErrorTypeException, fmt, get_text_locked_msg()) };
}
"#,
    ),
    (
        "expr_map_locked_error",
        r#"
/// Refuses a call made from an expression mapping, which the cmdline window
/// alone would have allowed.
fn expr_map_locked_error(err: &mut Error) {
    let fmt = c"%s".as_ptr();
    // SAFETY: `err` is live and `e_textlock` is a static NUL-terminated
    // message, which is what `%s` takes.
    unsafe { api_set_error(err, kErrorTypeException, fmt, &raw const e_textlock) };
}
"#,
    ),
];

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
    let can_fail = f.fallible || f.params.contains(&Param::Error);

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

    // Slot (1-based) -> the guard that releases what the pop left behind, and
    // how the call names the value through it. The rest own nothing: their
    // arena memory goes back with the arena.
    let guard = |ty: &ApiType| match ty {
        ApiType::Object => Some("ObjectArg"),
        ApiType::LuaRef => Some("LuaRefArg"),
        _ => None,
    };
    // How the call to the API function names slot `n`.
    let value = |index: usize, ty: &ApiType| {
        let slot = index + 1;
        match ty {
            ApiType::KeyDict(_) => format!("&raw mut arg_{slot}.dict"),
            _ if guard(ty).is_some() => format!("arg_{slot}.value"),
            _ => format!("arg_{slot}"),
        }
    };

    // Which of `Call`'s fields the body reads. Every conversion needs all
    // three; with no arguments to convert only the call itself is left.
    let locked = spec.textlock || spec.textlock_allow_cmdwin;
    let uses_arena = argc > 0 || f.params.contains(&Param::Arena);
    let uses_err = argc > 0 || locked || can_fail;
    let fields: Vec<&str> = [
        ("arena", uses_arena),
        ("err", uses_err),
        ("err_param", argc > 0),
    ]
    .into_iter()
    .filter_map(|(name, used)| used.then_some(name))
    .collect();

    writeln!(
        out,
        "pub unsafe extern \"C-unwind\" fn nlua_api_{name}(lstate: *mut lua_State) -> c_int {{"
    )
    .unwrap();
    writeln!(
        out,
        "    /// Pop the arguments, call the API function, hand the result back."
    )
    .unwrap();
    writeln!(
        out,
        "    /// Each argument that owns Lua references arms a guard, so every way"
    )
    .unwrap();
    writeln!(
        out,
        "    /// out releases exactly what was converted, in declaration order."
    )
    .unwrap();
    writeln!(out, "    ///").unwrap();
    writeln!(out, "    /// # Safety").unwrap();
    writeln!(
        out,
        "    /// The dispatcher's contract, which is what every `unsafe` below rests"
    )
    .unwrap();
    writeln!(
        out,
        "    /// on: `lstate` is the running Lua state with this binding's arguments"
    )
    .unwrap();
    writeln!(out, "    /// on top, and `call` is the binding's own.").unwrap();
    writeln!(
        out,
        "    unsafe fn convert(lstate: *mut lua_State, {}: &mut Call) {{",
        if fields.is_empty() { "_call" } else { "call" }
    )
    .unwrap();
    if !fields.is_empty() {
        let rest = if fields.len() == 3 { "" } else { ", .." };
        writeln!(
            out,
            "        let Call {{ {}{rest} }} = call;",
            fields.join(", ")
        )
        .unwrap();
    }

    if spec.textlock {
        writeln!(out, "        // SAFETY: as above.").unwrap();
        writeln!(out, "        if unsafe {{ text_locked() }} {{").unwrap();
        writeln!(out, "            text_locked_error(err);").unwrap();
        writeln!(out, "            return;").unwrap();
        writeln!(out, "        }}").unwrap();
    } else if spec.textlock_allow_cmdwin {
        writeln!(out, "        // SAFETY: as above.").unwrap();
        writeln!(
            out,
            "        if textlock.get() != 0 || unsafe {{ expr_map_locked() }} {{"
        )
        .unwrap();
        writeln!(out, "            expr_map_locked_error(err);").unwrap();
        writeln!(out, "            return;").unwrap();
        writeln!(out, "        }}").unwrap();
    }

    // The Lua stack hands the arguments back last first. A keyset is zeroed
    // and armed before its pop, which fills it field by field and may stop
    // halfway; everything else arms only once its pop has succeeded, which
    // is what leaves a failed conversion owning nothing. Both orders put the
    // guards on the stack highest slot first, so they drop lowest slot first
    // — the order upstream's `goto exit_N` chain released them in.
    for (index, ty, param) in values.iter().rev() {
        let slot = index + 1;
        if let ApiType::KeyDict(keyset) = ty {
            writeln!(
                out,
                "        let mut arg_{slot} = KeyDictArg::<KeyDict_{keyset}>::zeroed();"
            )
            .unwrap();
            writeln!(out, "        // SAFETY: as above.").unwrap();
            writeln!(
                out,
                "        unsafe {{ pop_keydict(lstate, &mut arg_{slot}, arena, err, err_param) }};"
            )
            .unwrap();
            // The keyset pop names the offending key itself.
            writeln!(out, "        if err.type_0 != kErrorTypeNone {{").unwrap();
            writeln!(out, "            return;").unwrap();
            writeln!(out, "        }}").unwrap();
            continue;
        }
        let (pop, extra) = popper(ty);
        writeln!(out, "        // SAFETY: as above.").unwrap();
        writeln!(
            out,
            "        let arg_{slot} = unsafe {{ {pop}(lstate, {extra}arena, err) }};"
        )
        .unwrap();
        writeln!(out, "        if err.type_0 != kErrorTypeNone {{").unwrap();
        writeln!(
            out,
            "            *err_param = c\"{param}\".as_ptr().cast_mut();"
        )
        .unwrap();
        writeln!(out, "            return;").unwrap();
        writeln!(out, "        }}").unwrap();
        if let Some(guard) = guard(ty) {
            writeln!(
                out,
                "        // SAFETY: the conversion took the references and nothing else"
            )
            .unwrap();
            writeln!(out, "        // releases them.").unwrap();
            writeln!(
                out,
                "        let arg_{slot} = unsafe {{ {guard}::new(arg_{slot}) }};"
            )
            .unwrap();
        }
    }

    let call_args: Vec<String> = f
        .params
        .iter()
        .map(|p| match p {
            Param::ChannelId => "LUA_INTERNAL_CALL".into(),
            Param::Arena => "arena".into(),
            Param::Error => "err".into(),
            Param::LuaState => "lstate".into(),
            Param::Value { index, ty, .. } => value(*index, ty),
        })
        .collect();
    let args = call_args.join(", ");
    let call = match f.is_unsafe {
        true => format!("unsafe {{ {name}({args}) }}"),
        false => format!("{name}({args})"),
    };
    // The API function may reach back into Lua; while it runs, this is the
    // state it reaches into.
    writeln!(out, "        let saved_lstate = active_lstate.get();").unwrap();
    writeln!(out, "        active_lstate.set(lstate);").unwrap();
    let by_pointer = matches!(f.ret, RetType::Object | RetType::KeyDict(_));
    if f.is_unsafe {
        writeln!(
            out,
            "        // SAFETY: as above; the arguments are this binding's own."
        )
        .unwrap();
    }
    let bind = if by_pointer { "mut ret" } else { "ret" };
    if f.fallible {
        // A failed call has no result to hand back and nothing left to
        // release, so it restores the state it borrowed and leaves the error
        // for `run` to raise.
        match f.ret {
            RetType::Void => writeln!(out, "        if let Err(e) = {call} {{").unwrap(),
            _ => {
                writeln!(out, "        let {bind} = match {call} {{").unwrap();
                writeln!(out, "            Ok(ret) => ret,").unwrap();
                writeln!(out, "            Err(e) => {{").unwrap();
            }
        }
        let indent = if f.ret == RetType::Void {
            "    "
        } else {
            "        "
        };
        writeln!(out, "        {indent}active_lstate.set(saved_lstate);").unwrap();
        writeln!(out, "        {indent}*err = e;").unwrap();
        writeln!(out, "        {indent}return;").unwrap();
        match f.ret {
            RetType::Void => writeln!(out, "        }}").unwrap(),
            _ => {
                writeln!(out, "            }}").unwrap();
                writeln!(out, "        }};").unwrap();
            }
        }
    } else {
        match f.ret {
            RetType::Void => writeln!(out, "        {call};").unwrap(),
            _ => writeln!(out, "        let {bind} = {call};").unwrap(),
        }
    }
    // A function with a Lua implementation converts its own result, so this
    // is only the fallback path, and upstream left it on the old conversion
    // whatever the API level says.
    let flags = if spec.push_special() || has_lua_imp {
        "PUSH_SPECIAL"
    } else {
        "PUSH"
    };
    let push = match &f.ret {
        RetType::Void => String::new(),
        RetType::KeyDict(_) => "unsafe { push_keydict(lstate, &raw mut ret) };".to_string(),
        ret => {
            let (push, by_pointer) = pusher(ret);
            let value = if by_pointer { "&raw mut ret" } else { "ret" };
            format!("unsafe {{ {push}(lstate, {value}, {flags}) }};")
        }
    };
    // A function with a Lua implementation of its own may have pushed the
    // result already; only convert what it left behind.
    if !push.is_empty() && has_lua_imp {
        writeln!(out, "        // SAFETY: as above.").unwrap();
        writeln!(out, "        if unsafe {{ lua_gettop(lstate) }} == 0 {{").unwrap();
        writeln!(out, "            // SAFETY: as above.").unwrap();
        writeln!(out, "            {push}").unwrap();
        writeln!(out, "        }}").unwrap();
    } else if !push.is_empty() {
        writeln!(out, "        // SAFETY: as above.").unwrap();
        writeln!(out, "        {push}").unwrap();
    }
    writeln!(out, "        active_lstate.set(saved_lstate);").unwrap();
    if spec.ret_alloc {
        let free = match &f.ret {
            RetType::String => "api_free_string",
            RetType::Object => "api_free_object",
            RetType::Dict => "api_free_dict",
            RetType::Array => "api_free_array",
            other => return Err(format!("{name}: nothing frees a {other:?} result")),
        };
        writeln!(
            out,
            "        // SAFETY: as above; the result is the binding's."
        )
        .unwrap();
        writeln!(out, "        unsafe {{ {free}(ret) }};").unwrap();
    }
    writeln!(out, "    }}").unwrap();

    let nret = if f.ret == RetType::Void { 0 } else { 1 };
    writeln!(
        out,
        "    // SAFETY: `lstate` is the state Lua called this binding on."
    )
    .unwrap();
    if spec.fast {
        writeln!(
            out,
            "    unsafe {{ dispatch_fast(lstate, {argc}, {nret}, convert) }}"
        )
        .unwrap();
    } else {
        writeln!(
            out,
            "    unsafe {{ dispatch(lstate, c\"{name}\", {argc}, {nret}, convert) }}"
        )
        .unwrap();
    }
    writeln!(out, "}}").unwrap();
    Ok(())
}

/// The table `vim.api` is: one entry per binding.
fn emit_lua_registration(out: &mut String, bound: &[&str]) {
    writeln!(out, "/// Every binding, by the name it answers to.").unwrap();
    writeln!(
        out,
        "static BINDINGS: [(&CStr, unsafe extern \"C-unwind\" fn(*mut lua_State) -> c_int); {}] = [",
        bound.len()
    )
    .unwrap();
    for name in bound {
        writeln!(out, "    (c\"{name}\", nlua_api_{name}),").unwrap();
    }
    writeln!(out, "];").unwrap();
    out.push('\n');
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
    writeln!(out, "    // SAFETY: the caller's stack.").unwrap();
    writeln!(out, "    unsafe {{").unwrap();
    writeln!(
        out,
        "        lua_createtable(lstate, 0, BINDINGS.len() as c_int);"
    )
    .unwrap();
    writeln!(out, "        for (name, binding) in BINDINGS {{").unwrap();
    writeln!(out, "            bind(lstate, binding, name);").unwrap();
    writeln!(out, "        }}").unwrap();
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
                lua_child_header(&format!("Lua bindings for `crate::api::{module}`{of}."))
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

    // Every keyset a binding names, whether as a parameter or as a result.
    let mut keysets: BTreeSet<&str> = BTreeSet::new();
    for (f, _) in &bound {
        for param in &f.params {
            if let Param::Value {
                ty: ApiType::KeyDict(keyset),
                ..
            } = param
            {
                keysets.insert(keyset.as_str());
            }
        }
        if let RetType::KeyDict(keyset) = &f.ret {
            keysets.insert(keyset.as_str());
        }
    }
    let mut impls = String::from(
        "\n// The keysets the bindings name, each tied to its own generated table\n\
         // and lookup.\n",
    );
    for keyset in &keysets {
        writeln!(
            impls,
            "\n// SAFETY: `{keyset}_table` and `KeyDict_{keyset}_get_field` are the\n\
             // generated table and lookup for `KeyDict_{keyset}`, which is all integers\n\
             // and pointers, so all zeroes is one.\n\
             unsafe impl KeySet for KeyDict_{keyset} {{\n\
             \x20   const GET_FIELD: FieldHashfn = Some(KeyDict_{keyset}_get_field);\n\
             \n\
             \x20   fn table() -> *mut KeySetLink {{\n\
             \x20       keyset_table(&{keyset}_table)\n\
             \x20   }}\n\
             }}"
        )
        .unwrap();
    }

    // Support code only some bindings need, swept until nothing new turns up.
    let mut extras = String::new();
    loop {
        let seen = idents(&format!("{LUA_SUPPORT}{extras}{impls}{body}"));
        let Some((_, code)) = LUA_READERS
            .iter()
            .find(|(name, code)| seen.contains(*name) && !extras.contains(code))
        else {
            break;
        };
        extras.push_str(code);
    }
    let support = format!("{LUA_SUPPORT}{extras}{impls}\n");

    let referenced = idents(&format!("{support}{body}"));
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
            "use crate::api::{module}::{{{}}};",
            names.iter().copied().collect::<Vec<_>>().join(", ")
        ));
    }
    // `keyset_table` is the support code's own; every other `_table` is a
    // generated keyset table.
    let dispatch: Vec<String> = referenced
        .iter()
        .filter(|n| n.ends_with("_get_field") || (n.ends_with("_table") && *n != "keyset_table"))
        .cloned()
        .collect();
    if !dispatch.is_empty() {
        uses.push(format!(
            "use crate::api::private::dispatch::{{{}}};",
            dispatch.join(", ")
        ));
    }
    uses.push(format!(
        "use crate::api::private::helpers::{{{}}};",
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
        uses.push("use crate::ex_docmd::expr_map_locked;".into());
    }
    if referenced.contains("text_locked") {
        uses.push("use crate::ex_getln::{get_text_locked_msg, text_locked};".into());
    }
    uses.push(format!(
        "use crate::lua::converter::{{{}}};",
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
        "use crate::lua::executor::{{{}}};",
        referenced_names(&[
            "LUA_INTERNAL_CALL",
            "active_lstate",
            "api_free_luaref",
            "nlua_is_deferred_safe",
        ])
        .join(", ")
    ));
    uses.push(format!(
        "use crate::lua::ffi::{{{}}};",
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
        "use crate::main::{{{}}};",
        referenced_names(&["e_fast_api_disabled", "e_textlock", "textlock"]).join(", ")
    ));
    uses.push("use crate::global_cell::GlobalCell;".into());
    uses.push("use crate::memory::{ARENA_EMPTY, arena_finish, arena_mem_free};".into());
    let types: Vec<String> = referenced_names(&[
        "Arena",
        "Error",
        "FieldHashfn",
        "KeySetLink",
        "LuaRef",
        "Object",
        "lua_State",
    ])
    .into_iter()
    .chain(
        referenced
            .iter()
            .filter(|n| n.starts_with("KeyDict_") && !n.ends_with("_get_field"))
            .cloned(),
    )
    .collect();
    uses.push(format!("use crate::types::{{{}}};", types.join(", ")));

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
    pub use crate::types::{kErrorTypeException, kErrorTypeNone, kErrorTypeValidation};
}

use known::*;
"#,
    );
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

// ---------------------------------------------------------------- metadata

/// One UI event, as the metadata describes it: the parameters are wire types,
/// which is not quite what the event's own signature says.
struct UiEvent {
    name: String,
    since: i64,
    params: Vec<(String, String)>,
}

/// One of the handle types the API takes and returns.
struct HandleType {
    name: String,
    id: i64,
    /// Method names of this type start with it, which is also what makes
    /// `method` true in the metadata.
    prefix: String,
}

/// The api-info sections that nothing in the crate declares (`--metadata-spec`).
#[derive(Default)]
struct Sidecar {
    api_level: i64,
    api_compatible: i64,
    api_prerelease: bool,
    prerelease: bool,
    build: String,
    error_types: Vec<(String, i64)>,
    handles: Vec<HandleType>,
    ui_events: Vec<UiEvent>,
}

fn parse_sidecar(path: &Path) -> Result<Sidecar, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut out = Sidecar::default();
    let mut versions = 0;
    for (lineno, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        let at = |msg: String| format!("{}:{}: {msg}", path.display(), lineno + 1);
        let mut words = line.split_whitespace();
        let section = words.next().unwrap();
        // Everything but `version` is named; the rest of the line is
        // key=value pairs, except a ui_event's trailing parameter list.
        let mut name = String::new();
        if section != "version" {
            name = words
                .next()
                .ok_or_else(|| at(format!("{section} needs a name")))?
                .to_string();
        }
        let (mut id, mut since, mut prefix) = (None, None, None);
        let mut params = Vec::new();
        for word in words {
            let Some((key, value)) = word.split_once('=') else {
                let (ty, pname) = word
                    .split_once(':')
                    .ok_or_else(|| at(format!("`{word}` is not <type>:<name>")))?;
                params.push((ty.to_string(), pname.to_string()));
                continue;
            };
            let number = |what: &str| {
                value
                    .parse::<i64>()
                    .map_err(|_| at(format!("bad {what} in `{word}`")))
            };
            let flag = || match value {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(at(format!("`{word}` wants true or false"))),
            };
            match (section, key) {
                ("error_type" | "type", "id") => id = Some(number("id")?),
                ("ui_event", "since") => since = Some(number("API level")?),
                ("type", "prefix") => prefix = Some(value.to_string()),
                ("version", "api_level") => out.api_level = number("API level")?,
                ("version", "api_compatible") => out.api_compatible = number("API level")?,
                ("version", "api_prerelease") => out.api_prerelease = flag()?,
                ("version", "prerelease") => out.prerelease = flag()?,
                ("version", "build") => out.build = value.to_string(),
                _ => return Err(at(format!("unknown key `{key}` in a {section} line"))),
            }
        }
        let want = |v: Option<i64>, what: &str| v.ok_or_else(|| at(format!("{what} is required")));
        match section {
            "version" => versions += 1,
            "error_type" => out.error_types.push((name, want(id, "id")?)),
            "type" => out.handles.push(HandleType {
                name,
                id: want(id, "id")?,
                prefix: prefix.ok_or_else(|| at("prefix is required".into()))?,
            }),
            "ui_event" => out.ui_events.push(UiEvent {
                name,
                since: want(since, "since")?,
                params,
            }),
            other => return Err(at(format!("unknown section `{other}`"))),
        }
    }
    if versions != 1 {
        return Err(format!("{}: expected one version line", path.display()));
    }
    if out.build.is_empty() || out.handles.is_empty() || out.error_types.is_empty() {
        return Err(format!("{}: a section is empty", path.display()));
    }
    Ok(out)
}

/// Split `Container(inner)` — tolerating the trailing `*` a keyset parameter
/// is declared with — into its two halves.
fn split_container(declared: &str) -> Option<(&str, &str)> {
    let s = declared.trim_end().trim_end_matches('*').trim_end();
    let open = s.find('(')?;
    let name = &s[..open];
    if !s.ends_with(')') || name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    Some((name, &s[open + 1..s.len() - 1]))
}

/// Split an `ArrayOf` body into its element type and, if the declaration
/// fixed one, its length. Only a top-level comma separates them: the element
/// may itself be a container with commas of its own.
fn split_elem(inner: &str) -> (&str, Option<&str>) {
    let mut depth = 0;
    for (i, c) in inner.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                let (elem, size) = (inner[..i].trim(), inner[i + 1..].trim());
                if !size.is_empty() && size.bytes().all(|b| b.is_ascii_digit()) {
                    return (elem, Some(size));
                }
            }
            _ => {}
        }
    }
    (inner.trim(), None)
}

/// The type name the metadata publishes for a declared C type. Upstream's
/// header grammar recognised a handful of containers that decorate a plain
/// API type with what it holds; only `ArrayOf` survives into the metadata,
/// the rest collapse to the wire type they are spelled on top of.
fn exported_type(declared: &str) -> String {
    let Some((container, inner)) = split_container(declared) else {
        return declared.to_string();
    };
    match container {
        "Union" => "Object".into(),
        "Tuple" => "Array".into(),
        "Dict" | "DictOf" | "DictAs" => "Dict".into(),
        "LuaRefOf" => "LuaRef".into(),
        "Enum" => "String".into(),
        "ArrayOf" => match split_elem(inner) {
            (elem, Some(size)) => format!("ArrayOf({}, {size})", exported_type(elem)),
            (elem, None) => format!("ArrayOf({})", exported_type(elem)),
        },
        _ => declared.to_string(),
    }
}

/// A minimal msgpack writer: the value kinds the metadata uses, each encoded
/// the narrowest way, which is what upstream's mpack did.
#[derive(Default)]
struct Packer(Vec<u8>);

impl Packer {
    fn map(&mut self, len: usize) {
        // Every map in the metadata is a fixmap. Upstream's generator made
        // the same assumption and would rather stop than emit a wider code
        // nobody has compared against a client.
        assert!(len <= 15, "a map of {len} needs a wider code");
        self.0.push(0x80 | len as u8);
    }

    fn array(&mut self, len: usize) {
        if len <= 15 {
            self.0.push(0x90 | len as u8);
        } else if let Ok(len) = u16::try_from(len) {
            self.0.push(0xdc);
            self.0.extend(len.to_be_bytes());
        } else {
            self.0.push(0xdd);
            self.0.extend((len as u32).to_be_bytes());
        }
    }

    fn str(&mut self, s: &str) {
        let len = s.len();
        if len < 32 {
            self.0.push(0xa0 | len as u8);
        } else if let Ok(len) = u8::try_from(len) {
            self.0.extend([0xd9, len]);
        } else if let Ok(len) = u16::try_from(len) {
            self.0.push(0xda);
            self.0.extend(len.to_be_bytes());
        } else {
            self.0.push(0xdb);
            self.0.extend((len as u32).to_be_bytes());
        }
        self.0.extend(s.as_bytes());
    }

    fn int(&mut self, v: i64) {
        if (-32..128).contains(&v) {
            self.0.push(v as i8 as u8);
        } else if let Ok(v) = u8::try_from(v) {
            self.0.extend([0xcc, v]);
        } else if let Ok(v) = u16::try_from(v) {
            self.0.push(0xcd);
            self.0.extend(v.to_be_bytes());
        } else if let Ok(v) = i8::try_from(v) {
            self.0.extend([0xd0, v as u8]);
        } else if let Ok(v) = i16::try_from(v) {
            self.0.push(0xd1);
            self.0.extend(v.to_be_bytes());
        } else if let Ok(v) = u32::try_from(v) {
            self.0.push(0xce);
            self.0.extend(v.to_be_bytes());
        } else if let Ok(v) = i32::try_from(v) {
            self.0.push(0xd2);
            self.0.extend(v.to_be_bytes());
        } else if let Ok(v) = u64::try_from(v) {
            self.0.push(0xcf);
            self.0.extend(v.to_be_bytes());
        } else {
            self.0.push(0xd3);
            self.0.extend(v.to_be_bytes());
        }
    }

    fn bool(&mut self, v: bool) {
        self.0.push(if v { 0xc3 } else { 0xc2 });
    }

    /// A `[type, name]` pair, which is how both a function parameter and a UI
    /// event parameter travel.
    fn pair(&mut self, ty: &str, name: &str) {
        self.array(2);
        self.str(ty);
        self.str(name);
    }
}

/// Every byte-string literal in an expression, NUL terminator stripped. The
/// transpiled statics spell a C string table as
/// `[b"name\0".as_ptr() as *const c_char, …]`.
fn byte_strings(expr: &syn::Expr, out: &mut Vec<String>) {
    let mut nested = |e| byte_strings(e, out);
    match expr {
        // Both spellings of a C string literal. `c"..."` is what the tree
        // writes since the `manual_c_str_literals` sweep; `b"...\0"` is what
        // c2rust emitted, and is still what the untouched files hold.
        syn::Expr::Lit(lit) => {
            let bytes = match &lit.lit {
                syn::Lit::ByteStr(bytes) => bytes.value(),
                syn::Lit::CStr(bytes) => bytes.value().into_bytes(),
                _ => return,
            };
            out.push(
                String::from_utf8_lossy(&bytes)
                    .trim_end_matches('\0')
                    .into(),
            );
        }
        syn::Expr::Array(array) => array.elems.iter().for_each(nested),
        syn::Expr::Call(call) => call.args.iter().for_each(nested),
        syn::Expr::MethodCall(call) => nested(&call.receiver),
        syn::Expr::Cast(cast) => nested(&cast.expr),
        syn::Expr::Paren(paren) => nested(&paren.expr),
        syn::Expr::Reference(reference) => nested(&reference.expr),
        syn::Expr::Unary(unary) => nested(&unary.expr),
        _ => {}
    }
}

/// The `ui_options` the metadata advertises: the externalisable UI features,
/// read off `ui_ext_names` — the table `nvim_ui_attach` validates its options
/// against, so a new one cannot be added without the metadata following.
/// `rgb` leads, and is not in the table because it is not an extension. Names
/// that open with an underscore are the tree's own debugging switches and stay
/// unadvertised.
fn read_ui_options(root: &Path) -> Result<Vec<String>, String> {
    let path = root.join("src/main/mod.rs");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let file = syn::parse_file(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut names = vec!["rgb".to_string()];
    let mut found = false;
    for item in &file.items {
        let syn::Item::Static(item) = item else {
            continue;
        };
        if item.ident != "ui_ext_names" {
            continue;
        }
        found = true;
        let mut all = Vec::new();
        byte_strings(&item.expr, &mut all);
        names.extend(
            all.into_iter()
                .filter(|n| n.starts_with(|c: char| c.is_ascii_lowercase())),
        );
    }
    if !found {
        return Err(format!("{}: no ui_ext_names table", path.display()));
    }
    Ok(names)
}

/// The three `NVIM_VERSION_*` constants, which are the tree's own statement of
/// what version it is.
fn read_version(root: &Path) -> Result<[i64; 3], String> {
    let path = root.join("src/version/mod.rs");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let file = syn::parse_file(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let wanted = [
        "NVIM_VERSION_MAJOR",
        "NVIM_VERSION_MINOR",
        "NVIM_VERSION_PATCH",
    ];
    let mut out = [None; 3];
    for item in &file.items {
        let syn::Item::Const(item) = item else {
            continue;
        };
        let Some(slot) = wanted.iter().position(|w| item.ident == w) else {
            continue;
        };
        // `pub const NVIM_VERSION_MAJOR: c_int = 0 as c_int;`
        let mut expr = &*item.expr;
        while let syn::Expr::Cast(cast) = expr {
            expr = &cast.expr;
        }
        let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(int),
            ..
        }) = expr
        else {
            return Err(format!(
                "{}: {} is not a literal",
                path.display(),
                item.ident
            ));
        };
        out[slot] = Some(int.base10_parse::<i64>().map_err(|e| e.to_string())?);
    }
    let mut version = [0; 3];
    for (slot, value) in out.iter().enumerate() {
        version[slot] = value.ok_or_else(|| format!("{}: no {}", path.display(), wanted[slot]))?;
    }
    Ok(version)
}

/// One entry of the metadata's `functions` array.
struct MetaFn<'a> {
    name: &'a str,
    params: Vec<(String, String)>,
    ret: String,
    since: i64,
    deprecated_since: Option<i64>,
    /// Whether the method belongs to one of the handle types, which clients
    /// use to hang it off a Buffer/Window/Tabpage object.
    method: bool,
}

/// Describe every method the metadata publishes, in spec order — which is the
/// order upstream's generator read the API headers in, and the order the
/// frozen blob lists.
fn metadata_functions<'a>(
    api: &BTreeMap<String, ApiFn>,
    specs: &'a [Spec],
    handles: &[HandleType],
) -> Result<Vec<MetaFn<'a>>, String> {
    let by_name: BTreeMap<&str, &Spec> = specs.iter().map(|s| (s.name.as_str(), s)).collect();
    let mut out = Vec::new();
    for spec in specs.iter().filter(|s| s.in_metadata()) {
        // A deprecated spelling describes the method it stands for, under its
        // own name and with its own levels.
        let implementation = match &spec.alias {
            Some(target) => by_name[target.as_str()],
            None => spec,
        };
        let f = api
            .get(&implementation.name)
            .ok_or_else(|| format!("{}: no such API function in the crate", implementation.name))?;
        let params = f
            .params
            .iter()
            .filter_map(|p| match p {
                Param::Value { index, ty, name } => Some((
                    exported_type(
                        &implementation
                            .declared
                            .get(&(index + 1))
                            .cloned()
                            .unwrap_or_else(|| ty.declared()),
                    ),
                    name.clone(),
                )),
                _ => None,
            })
            .collect();
        let declared_ret = spec
            .ret
            .clone()
            .or_else(|| implementation.ret.clone())
            .unwrap_or_else(|| f.ret.declared());
        out.push(MetaFn {
            name: &spec.name,
            params,
            ret: exported_type(&declared_ret),
            since: spec.metadata_since().into(),
            deprecated_since: match spec.is_legacy_spelling() {
                true => Some(1),
                false => spec.deprecated_since.map(i64::from),
            },
            method: handles
                .iter()
                .any(|h| implementation.name.starts_with(&h.prefix)),
        });
    }
    Ok(out)
}

/// Pack the whole metadata dict. Key order is upstream's and is load-bearing
/// only in that the committed bytes have to keep matching: msgpack maps are
/// unordered, but the blob is checked byte for byte.
fn pack_metadata(
    version: [i64; 3],
    sidecar: &Sidecar,
    ui_options: &[String],
    functions: &[MetaFn],
) -> Vec<u8> {
    let mut p = Packer::default();
    p.map(6);

    p.str("version");
    p.map(8);
    for (key, value) in [
        ("major", version[0]),
        ("minor", version[1]),
        ("patch", version[2]),
    ] {
        p.str(key);
        p.int(value);
    }
    p.str("prerelease");
    p.bool(sidecar.prerelease);
    p.str("api_level");
    p.int(sidecar.api_level);
    p.str("api_compatible");
    p.int(sidecar.api_compatible);
    p.str("api_prerelease");
    p.bool(sidecar.api_prerelease);
    p.str("build");
    p.str(&sidecar.build);

    p.str("functions");
    p.array(functions.len());
    for f in functions {
        p.map(if f.deprecated_since.is_some() { 6 } else { 5 });
        p.str("parameters");
        p.array(f.params.len());
        for (ty, name) in &f.params {
            p.pair(ty, name);
        }
        p.str("since");
        p.int(f.since);
        p.str("return_type");
        p.str(&f.ret);
        p.str("name");
        p.str(f.name);
        p.str("method");
        p.bool(f.method);
        if let Some(level) = f.deprecated_since {
            p.str("deprecated_since");
            p.int(level);
        }
    }

    p.str("ui_events");
    p.array(sidecar.ui_events.len());
    for event in &sidecar.ui_events {
        p.map(3);
        p.str("name");
        p.str(&event.name);
        p.str("parameters");
        p.array(event.params.len());
        for (ty, name) in &event.params {
            p.pair(ty, name);
        }
        p.str("since");
        p.int(event.since);
    }

    p.str("ui_options");
    p.array(ui_options.len());
    for option in ui_options {
        p.str(option);
    }

    p.str("error_types");
    p.map(sidecar.error_types.len());
    for (name, id) in &sidecar.error_types {
        p.str(name);
        p.map(1);
        p.str("id");
        p.int(*id);
    }

    p.str("types");
    p.map(sidecar.handles.len());
    for handle in &sidecar.handles {
        p.str(&handle.name);
        p.map(2);
        p.str("id");
        p.int(handle.id);
        p.str("prefix");
        p.str(&handle.prefix);
    }

    p.0
}

/// How wide a rendered byte-string line may get before it is continued.
const LITERAL_WIDTH: usize = 92;

/// Render bytes as a Rust byte-string literal, continued across lines with
/// the `\<newline>` escape. Most of the blob is the ASCII of method and
/// parameter names, so the literal stays readable — and diffable — where an
/// array of 32,000 numbers would not.
fn byte_literal(data: &[u8]) -> String {
    let mut out = String::from("b\"");
    let mut column = out.len();
    for &byte in data {
        // A continuation eats the newline *and* the whitespace after it, so a
        // line may not open with a space of its own.
        let escaped = match byte {
            b'"' => "\\\"".to_string(),
            b'\\' => "\\\\".to_string(),
            b' ' if column == 0 => "\\x20".to_string(),
            0x20..=0x7e => (byte as char).to_string(),
            _ => format!("\\x{byte:02x}"),
        };
        if column + escaped.len() > LITERAL_WIDTH {
            out.push_str("\\\n");
            column = 0;
        }
        column += escaped.len();
        out.push_str(&escaped);
    }
    out.push('"');
    out
}

const METADATA_HEADER: &str = r#"//! The API metadata blob.
//!
//! `nvim --api-info` and `nvim_get_api_info()` hand this back verbatim: one
//! msgpack dict describing every method the API publishes, the UI events and
//! options, the error kinds and the handle types, so that a client can
//! discover the API rather than hard-code it.
//!
//! Generated by tools/apigen from tools/apigen/functions.txt, the API
//! signatures, tools/apigen/metadata.txt and the tree's own `ui_ext_names`
//! and `NVIM_VERSION_*`. Do not edit: run `just apigen`.
#![forbid(unsafe_code)]

"#;

fn generate_metadata(
    root: &Path,
    api: &BTreeMap<String, ApiFn>,
    specs: &[Spec],
    sidecar: &Sidecar,
) -> Result<String, String> {
    let functions = metadata_functions(api, specs, &sidecar.handles)?;
    let packed = pack_metadata(
        read_version(root)?,
        sidecar,
        &read_ui_options(root)?,
        &functions,
    );
    let mut out = String::from(METADATA_HEADER);
    writeln!(
        out,
        "/// The packed metadata, describing {} methods and {} UI events.",
        functions.len(),
        sidecar.ui_events.len(),
    )
    .unwrap();
    writeln!(
        out,
        "pub static PACKED_API_METADATA: &[u8] = {};",
        byte_literal(&packed)
    )
    .unwrap();
    Ok(out)
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
    let mut metadata_spec = None;
    let mut out_dir = None;
    let mut tables_dir = None;
    let mut lua_dir = None;
    let mut options_lua = None;
    let mut options_dir = None;
    let mut eval_lua = None;
    let mut eval_dir = None;
    let mut ex_cmds_lua = None;
    let mut cmdtable_file = None;
    let mut cmdidx_file = None;
    let mut metadata_file = None;
    let mut config = None;
    let mut check = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{arg} needs a value"));
        match arg.as_str() {
            "--root" => root = Some(PathBuf::from(value()?)),
            "--spec" => spec_path = Some(PathBuf::from(value()?)),
            "--metadata-spec" => metadata_spec = Some(PathBuf::from(value()?)),
            "--out-dir" => out_dir = Some(PathBuf::from(value()?)),
            "--tables-dir" => tables_dir = Some(PathBuf::from(value()?)),
            "--lua-dir" => lua_dir = Some(PathBuf::from(value()?)),
            "--options-lua" => options_lua = Some(PathBuf::from(value()?)),
            "--options-dir" => options_dir = Some(PathBuf::from(value()?)),
            "--eval-lua" => eval_lua = Some(PathBuf::from(value()?)),
            "--eval-dir" => eval_dir = Some(PathBuf::from(value()?)),
            "--ex-cmds-lua" => ex_cmds_lua = Some(PathBuf::from(value()?)),
            "--cmdtable-file" => cmdtable_file = Some(PathBuf::from(value()?)),
            "--cmdidx-file" => cmdidx_file = Some(PathBuf::from(value()?)),
            "--metadata-file" => metadata_file = Some(PathBuf::from(value()?)),
            "--rustfmt-config" => config = Some(PathBuf::from(value()?)),
            "--check" => check = true,
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    let root = root.ok_or("--root is required")?;
    let spec_path = spec_path.ok_or("--spec is required")?;
    let metadata_spec = metadata_spec.ok_or("--metadata-spec is required")?;
    let out_dir = out_dir.ok_or("--out-dir is required")?;
    let tables_dir = tables_dir.ok_or("--tables-dir is required")?;
    let lua_dir = lua_dir.ok_or("--lua-dir is required")?;
    let options_lua = options_lua.ok_or("--options-lua is required")?;
    let options_dir = options_dir.ok_or("--options-dir is required")?;
    let eval_lua = eval_lua.ok_or("--eval-lua is required")?;
    let eval_dir = eval_dir.ok_or("--eval-dir is required")?;
    let ex_cmds_lua = ex_cmds_lua.ok_or("--ex-cmds-lua is required")?;
    let cmdtable_file = cmdtable_file.ok_or("--cmdtable-file is required")?;
    let cmdidx_file = cmdidx_file.ok_or("--cmdidx-file is required")?;
    let metadata_file = metadata_file.ok_or("--metadata-file is required")?;
    let config = config.ok_or("--rustfmt-config is required")?;

    let api = collect_api_fns(&root)?;
    let keysets = collect_keysets(&root)?;
    let specs = parse_spec(&spec_path)?;
    let sidecar = parse_sidecar(&metadata_spec)?;
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
            generate_tables(&keysets, &specs, &config)?,
            "tables",
        ),
        (lua_dir, generate_lua(&api, &specs, &config)?, "Lua binding"),
        (
            options_dir.clone(),
            options::generate(&root, &options_lua, &options_dir, &config)?,
            "option table",
        ),
        (
            eval_dir.clone(),
            eval_funcs::generate(&root, &api, &specs, &eval_lua, &eval_dir, &config)?,
            "builtin table",
        ),
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
    // These live beside hand-written modules rather than in a tree of their
    // own, so they get no stale-file sweep.
    let (cmdtable, cmdidx) = ex_cmds::generate(&ex_cmds_lua)?;
    let singles = [
        (
            metadata_file,
            generate_metadata(&root, &api, &specs, &sidecar)?,
            "metadata",
        ),
        (cmdtable_file, cmdtable, "Ex command table"),
        (cmdidx_file, cmdidx, "cmdidx_T"),
    ];
    for (path, text, what) in singles {
        let text = rustfmt(&config, &text)?;
        if std::fs::read_to_string(&path).unwrap_or_default() == text {
            continue;
        }
        if check {
            return Err(format!("{} is stale; run `just apigen`", path.display()));
        }
        std::fs::write(&path, &text).map_err(|e| format!("{}: {e}", path.display()))?;
        wrote = true;
        eprintln!("apigen: wrote {} ({what})", path.display());
    }

    if wrote {
        eprintln!(
            "apigen: {} wrappers, {} handlers, {} keysets, {} Lua bindings, {} published methods, {} eval bindings",
            specs.iter().filter(|s| s.is_wrapper()).count(),
            specs.iter().filter(|s| s.is_method()).count(),
            keysets.len(),
            specs.iter().filter(|s| s.has_lua_binding()).count(),
            specs.iter().filter(|s| s.in_metadata()).count(),
            specs.iter().filter(|s| s.has_eval_binding()).count(),
        );
    }
    Ok(())
}
