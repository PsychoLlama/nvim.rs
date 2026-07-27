// apigen: generate the msgpack-RPC dispatch wrappers from the Rust API
// signatures themselves.
//
// Upstream generated `dispatch_wrappers.generated.h` by parsing the C headers
// under src/nvim/api/ (src/gen/gen_api_dispatch.lua at tag v0.12.4). The
// transpile froze that output into one 27k-line Rust module. This tool takes
// the job back: it reads the real source of truth — the `pub unsafe extern
// "C" fn nvim_*` signatures in <root>/src/nvim/api/*.rs — and emits the
// per-function wrapper that validates an `Array` of msgpack arguments,
// converts them, calls the API function and boxes the result back into an
// `Object`.
//
// A signature alone does not say everything upstream's `FUNC_API_*` markers
// said: whether a call is refused under textlock, which declared C type name
// appears in a "Wrong type for argument" message ("ArrayOf(Integer, 2)" is a
// plain `Array` in Rust), and the `since`/`fast`/`ret_alloc` metadata that
// feeds the handler table and the api-info blob. That lives in the spec file
// (`--spec`), one line per exported function. The two inputs cross-check: a
// spec entry naming a function that no longer exists, or whose declared
// parameter count disagrees with the Rust signature, is a hard error.
//
// Output is committed, rustfmt'd Rust. `--check` re-generates in memory and
// diffs against the committed file, so drift is a build failure rather than a
// silent hazard (scripts/gen-api-dispatch.sh, `just apigen`).
//
// Build with the repo dev shell; syn is pinned exactly, as in tools/ffigen.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
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
    Value { index: usize, ty: ApiType },
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
    name: String,
    /// Refuse the call while the text is locked (upstream `FUNC_API_TEXTLOCK`).
    textlock: bool,
    /// Refuse under textlock, but tolerate the command-line window
    /// (`FUNC_API_TEXTLOCK_ALLOW_CMDWIN`).
    textlock_allow_cmdwin: bool,
    /// Declared C type names for arguments whose Rust type has lost the
    /// decoration, keyed by 1-based argument position: `arg1=ArrayOf(Integer, 2)`.
    declared: BTreeMap<usize, String>,
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
            _ if name == "channel_id" => Param::ChannelId,
            _ => {
                let ty = value_type(&arg.ty)
                    .ok_or_else(|| format!("parameter `{name}` has an unmapped type"))?;
                let param = Param::Value { index, ty };
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

/// Element counts of the `<name>_table: GlobalCell<[KeySetLink; N]>` statics,
/// which `api_keydict_to_dict` needs as its bound. They still live in the
/// hand-maintained dispatch module; reading them keeps the two in step.
fn collect_keyset_tables(root: &Path) -> Result<BTreeMap<String, usize>, String> {
    let path = root.join("src/nvim/api/private/dispatch.rs");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let file = syn::parse_file(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut out = BTreeMap::new();
    for item in &file.items {
        let syn::Item::Static(s) = item else { continue };
        let Some(name) = s
            .ident
            .to_string()
            .strip_suffix("_table")
            .map(str::to_string)
        else {
            continue;
        };
        // GlobalCell<[KeySetLink; N]>
        let syn::Type::Path(p) = &*s.ty else { continue };
        let last = p.path.segments.last().unwrap();
        let syn::PathArguments::AngleBracketed(args) = &last.arguments else {
            continue;
        };
        let Some(syn::GenericArgument::Type(syn::Type::Array(arr))) = args.args.first() else {
            continue;
        };
        if type_name(&arr.elem).as_deref() != Some("KeySetLink") {
            continue;
        }
        let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(n),
            ..
        }) = &arr.len
        else {
            continue;
        };
        out.insert(name, n.base10_parse::<usize>().map_err(|e| e.to_string())?);
    }
    Ok(out)
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
        out.push(spec);
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
            Param::Value { index, ty } => Some((*index, ty)),
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

#![deny(unsafe_op_in_unsafe_fn)]
"#;

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

fn generate(
    api: &BTreeMap<String, ApiFn>,
    specs: &[Spec],
    tables: &BTreeMap<String, usize>,
) -> Result<String, String> {
    let mut body = String::new();
    // module path segment -> API functions to import from it
    let mut api_imports: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for spec in specs {
        let f = api
            .get(&spec.name)
            .ok_or_else(|| format!("{}: no such API function in the crate", spec.name))?;
        api_imports
            .entry(f.module.clone())
            .or_default()
            .insert(f.name.clone());
        emit_fn(&mut body, f, spec, tables)?;
        body.push('\n');
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
    out.push('\n');
    out.push_str(&body);
    Ok(out)
}

// ---------------------------------------------------------------- driver

fn rustfmt(path: &Path) -> Result<(), String> {
    let status = Command::new("rustfmt")
        .arg(path)
        .status()
        .map_err(|e| format!("rustfmt: {e}"))?;
    if !status.success() {
        return Err(format!("rustfmt failed on {}", path.display()));
    }
    Ok(())
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
    let mut out_path = None;
    let mut check = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{arg} needs a value"));
        match arg.as_str() {
            "--root" => root = Some(PathBuf::from(value()?)),
            "--spec" => spec_path = Some(PathBuf::from(value()?)),
            "--out" => out_path = Some(PathBuf::from(value()?)),
            "--check" => check = true,
            other => return Err(format!("unknown argument `{other}`")),
        }
    }
    let root = root.ok_or("--root is required")?;
    let spec_path = spec_path.ok_or("--spec is required")?;
    let out_path = out_path.ok_or("--out is required")?;

    let api = collect_api_fns(&root)?;
    let tables = collect_keyset_tables(&root)?;
    let specs = parse_spec(&spec_path)?;
    let generated = generate(&api, &specs, &tables)?;

    // rustfmt only reads files, so both modes go through a temporary and the
    // committed file is compared afterwards. It has to sit next to the real
    // output: rustfmt discovers rustfmt.toml (edition 2024) by walking the
    // file's ancestors, and a system temp dir would silently format to a
    // different edition's defaults.
    let scratch = out_path.with_extension("rs.apigen");
    std::fs::write(&scratch, &generated).map_err(|e| format!("{}: {e}", scratch.display()))?;
    rustfmt(&scratch)?;
    let formatted =
        std::fs::read_to_string(&scratch).map_err(|e| format!("{}: {e}", scratch.display()))?;
    std::fs::remove_file(&scratch).ok();

    let committed = std::fs::read_to_string(&out_path).unwrap_or_default();
    if check {
        if committed != formatted {
            return Err(format!(
                "{} is stale; run `just apigen`",
                out_path.display()
            ));
        }
        return Ok(());
    }
    if committed != formatted {
        std::fs::write(&out_path, &formatted)
            .map_err(|e| format!("{}: {e}", out_path.display()))?;
        eprintln!(
            "apigen: wrote {} ({} wrappers)",
            out_path.display(),
            specs.len()
        );
    }
    Ok(())
}
