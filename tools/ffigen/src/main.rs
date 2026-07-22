// ffigen: generate the unit suite's LuaJIT ffi.cdef surface from
// the Rust crate itself, replacing the preprocessed v0.12.4 C headers.
//
// Reads the transpiled crate (src/**/*.rs), collects
//   - #[repr(C)] structs/unions and type aliases (canonical copies preferred
//     from src/nvim/types/),
//   - #[no_mangle] fns and statics (the linkable ABI surface),
//   - integer `pub const`s (c2rust's rendering of C enums and #defines),
// and emits one C-syntax declaration chunk (`unit-cdefs.h`) that ffi.cdef can
// digest, plus a manifest that lets scripts/check-unit-cdefs.py diff every
// emitted layout/constant/prototype against the old preprocessed-header
// pipeline while both still exist.
//
// Names on the deny list (types/constants already defined by the system
// preamble headers the harness cdefs first: uv.h, string.h, ...) are
// referenced but never defined here.
//
// Build with the repo dev shell (Cargo.lock is v3 on purpose: the pinned
// toolchain's cargo must be able to build this at test time).

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------- model

#[derive(Clone, Debug)]
enum Kind {
    Struct(Vec<Field>),
    Union(Vec<Field>),
    Alias(syn::Type),
    Opaque,
}

#[derive(Clone, Debug)]
struct Def {
    file: String, // repo-relative
    kind: Kind,
    align: Option<u64>,
    is_types: bool, // lives under src/nvim/types/
}

#[derive(Clone, Debug)]
struct Field {
    name: String,
    ty: syn::Type,
    bits: Vec<BitSpec>, // non-empty: this is a c2rust bitfield storage unit
    padding: bool,      // #[bitfield(padding)]: emit nothing
}

#[derive(Clone, Debug)]
struct BitSpec {
    name: String,
    ty: String, // c2rust `ty = "..."` string
    width: u64,
}

struct Konst {
    file: String,
    expr: syn::Expr,
}

struct ExportFn {
    name: String,
    file: String,
    sig: syn::Signature,
}

struct ExportStatic {
    name: String,
    file: String,
    ty: syn::Type,
}

#[derive(Default)]
struct World {
    // (file, name) -> Def for file-local resolution
    by_file: HashMap<(String, String), Def>,
    // name -> all defs
    by_name: HashMap<String, Vec<Def>>,
    consts: HashMap<String, Vec<Konst>>,
    fns: Vec<ExportFn>,
    statics: Vec<ExportStatic>,
    deny: HashSet<String>,
    // type name -> files whose #[no_mangle] fn/static signatures mention it
    sig_files: HashMap<String, BTreeSet<String>>,
}

/// Collect the last-segment identifiers of every named type mentioned in a
/// type tree (fn-pointer args included, libc-qualified and primitive names
/// skipped).
fn type_names(ty: &syn::Type, out: &mut BTreeSet<String>) {
    match ty {
        syn::Type::Path(tp) => {
            if tp.path.segments.len() > 1
                && tp
                    .path
                    .segments
                    .first()
                    .map_or(false, |s| s.ident == "libc")
            {
                return;
            }
            if let Some(seg) = tp.path.segments.last() {
                if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
                    for a in &ab.args {
                        if let syn::GenericArgument::Type(t) = a {
                            type_names(t, out);
                        }
                    }
                    return;
                }
                let name = seg.ident.to_string();
                if prim(&name).is_none() && !name.starts_with("C2Rust_Unnamed") {
                    out.insert(name);
                }
            }
        }
        syn::Type::Ptr(p) => type_names(&p.elem, out),
        syn::Type::Array(a) => type_names(&a.elem, out),
        syn::Type::BareFn(f) => {
            for a in &f.inputs {
                type_names(&a.ty, out);
            }
            if let syn::ReturnType::Type(_, t) = &f.output {
                type_names(t, out);
            }
        }
        syn::Type::Paren(p) => type_names(&p.elem, out),
        syn::Type::Group(g) => type_names(&g.elem, out),
        _ => {}
    }
}

// ------------------------------------------------------------ collection

fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "async"
            | "await"
            | "try"
    )
}

fn is_rust_prelude(s: &str) -> bool {
    matches!(s, "String" | "Option" | "Result" | "Box" | "Vec" | "Error")
}

/// c2rust renames C identifiers that collide with Rust keywords/prelude by
/// appending `_0` (`type` -> `type_0`, `String` -> `String_0`). Recover the C
/// name for emission.
fn c_name(rust: &str) -> String {
    if let Some(stem) = rust.strip_suffix("_0") {
        if is_rust_keyword(stem) || is_rust_prelude(stem) {
            return stem.to_string();
        }
    }
    rust.to_string()
}

fn repr_of(attrs: &[syn::Attribute]) -> (bool, Option<u64>) {
    let mut is_c = false;
    let mut align = None;
    for attr in attrs {
        if !attr.path().is_ident("repr") {
            continue;
        }
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("C") {
                is_c = true;
            } else if meta.path.is_ident("align") {
                let content;
                syn::parenthesized!(content in meta.input);
                let lit: syn::LitInt = content.parse()?;
                align = Some(lit.base10_parse::<u64>()?);
            } else if meta.path.is_ident("packed") {
                // none exist in the emitted closure today; scream if that changes
                panic!("repr(packed) encountered; teach ffigen to emit it");
            }
            Ok(())
        });
    }
    (is_c, align)
}

fn bitfields_of(attrs: &[syn::Attribute]) -> (Vec<BitSpec>, bool) {
    let mut specs = Vec::new();
    let mut padding = false;
    for attr in attrs {
        if !attr.path().is_ident("bitfield") {
            continue;
        }
        let mut name = None;
        let mut ty = None;
        let mut bits = None;
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("padding") {
                padding = true;
                return Ok(());
            }
            let value: syn::LitStr = meta.value()?.parse()?;
            if meta.path.is_ident("name") {
                name = Some(value.value());
            } else if meta.path.is_ident("ty") {
                ty = Some(value.value());
            } else if meta.path.is_ident("bits") {
                bits = Some(value.value());
            }
            Ok(())
        })
        .expect("parse #[bitfield] attribute");
        if let (Some(name), Some(ty), Some(bits)) = (name, ty, bits) {
            // bits = "lo..=hi"
            let (lo, hi) = bits.split_once("..=").expect("bitfield bits form");
            let lo: u64 = lo.parse().unwrap();
            let hi: u64 = hi.parse().unwrap();
            specs.push(BitSpec {
                name,
                ty,
                width: hi - lo + 1,
            });
        }
    }
    (specs, padding)
}

fn named_fields(fields: &syn::FieldsNamed) -> Vec<Field> {
    fields
        .named
        .iter()
        .map(|f| {
            let (bits, padding) = bitfields_of(&f.attrs);
            Field {
                name: f.ident.as_ref().unwrap().to_string(),
                ty: f.ty.clone(),
                bits,
                padding,
            }
        })
        .collect()
}

fn has_no_mangle(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| a.path().is_ident("no_mangle"))
}

fn collect_file(world: &mut World, rel: &str, ast: syn::File) {
    let is_types = rel.starts_with("src/nvim/types/");
    let add = |world: &mut World, name: String, kind: Kind, align: Option<u64>| {
        let def = Def {
            file: rel.to_string(),
            kind,
            align,
            is_types,
        };
        world
            .by_file
            .insert((rel.to_string(), name.clone()), def.clone());
        world.by_name.entry(name).or_default().push(def);
    };
    for item in ast.items {
        match item {
            syn::Item::Struct(s) if s.generics.params.is_empty() => {
                let (is_c, align) = repr_of(&s.attrs);
                if is_c {
                    if let syn::Fields::Named(named) = &s.fields {
                        add(
                            world,
                            s.ident.to_string(),
                            Kind::Struct(named_fields(named)),
                            align,
                        );
                    }
                }
            }
            syn::Item::Union(u) if u.generics.params.is_empty() => {
                let (is_c, align) = repr_of(&u.attrs);
                if is_c {
                    add(
                        world,
                        u.ident.to_string(),
                        Kind::Union(named_fields(&u.fields)),
                        align,
                    );
                }
            }
            syn::Item::Type(t) if t.generics.params.is_empty() => {
                add(
                    world,
                    t.ident.to_string(),
                    Kind::Alias((*t.ty).clone()),
                    None,
                );
            }
            syn::Item::Const(c) => {
                world
                    .consts
                    .entry(c.ident.to_string())
                    .or_default()
                    .push(Konst {
                        file: rel.to_string(),
                        expr: (*c.expr).clone(),
                    });
            }
            syn::Item::Fn(f) if has_no_mangle(&f.attrs) => {
                world.fns.push(ExportFn {
                    name: f.sig.ident.to_string(),
                    file: rel.to_string(),
                    sig: f.sig.clone(),
                });
            }
            syn::Item::Static(s) if has_no_mangle(&s.attrs) => {
                world.statics.push(ExportStatic {
                    name: s.ident.to_string(),
                    file: rel.to_string(),
                    ty: (*s.ty).clone(),
                });
            }
            syn::Item::ForeignMod(fm) => {
                for fitem in fm.items {
                    if let syn::ForeignItem::Type(ft) = fitem {
                        add(world, ft.ident.to_string(), Kind::Opaque, None);
                    }
                }
            }
            _ => {}
        }
    }
}

fn discover(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![root.join("src")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read_dir") {
            let path = entry.expect("dirent").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().map_or(false, |e| e == "rs") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

// --------------------------------------------------------- const evaluator

fn eval_const(world: &World, file: &str, expr: &syn::Expr, depth: u32) -> Option<i128> {
    if depth > 32 {
        return None;
    }
    match expr {
        syn::Expr::Lit(l) => match &l.lit {
            syn::Lit::Int(i) => i.base10_parse::<i128>().ok(),
            syn::Lit::Byte(b) => Some(b.value() as i128),
            syn::Lit::Char(c) => Some(c.value() as i128),
            _ => None,
        },
        syn::Expr::Cast(c) => {
            let v = eval_const(world, file, &c.expr, depth + 1)?;
            // Wrap to the cast target's width/signedness so `-1 as c_uint`
            // and friends match the C value.
            let name = match &*c.ty {
                syn::Type::Path(tp) => tp.path.segments.last()?.ident.to_string(),
                _ => return Some(v),
            };
            Some(wrap_to(&name, v))
        }
        syn::Expr::Unary(u) => {
            let v = eval_const(world, file, &u.expr, depth + 1)?;
            match u.op {
                syn::UnOp::Neg(_) => Some(-v),
                syn::UnOp::Not(_) => Some(!v),
                _ => None,
            }
        }
        syn::Expr::Binary(b) => {
            let l = eval_const(world, file, &b.left, depth + 1)?;
            let r = eval_const(world, file, &b.right, depth + 1)?;
            match b.op {
                syn::BinOp::BitOr(_) => Some(l | r),
                syn::BinOp::BitAnd(_) => Some(l & r),
                syn::BinOp::BitXor(_) => Some(l ^ r),
                syn::BinOp::Shl(_) => l.checked_shl(r as u32),
                syn::BinOp::Shr(_) => l.checked_shr(r as u32),
                syn::BinOp::Add(_) => l.checked_add(r),
                syn::BinOp::Sub(_) => l.checked_sub(r),
                syn::BinOp::Mul(_) => l.checked_mul(r),
                syn::BinOp::Div(_) => l.checked_div(r),
                _ => None,
            }
        }
        syn::Expr::Paren(p) => eval_const(world, file, &p.expr, depth + 1),
        syn::Expr::Group(g) => eval_const(world, file, &g.expr, depth + 1),
        syn::Expr::Path(p) => {
            let name = p.path.segments.last()?.ident.to_string();
            let candidates = world.consts.get(&name)?;
            let pick = candidates
                .iter()
                .find(|k| k.file == file)
                .or_else(|| candidates.first())?;
            let pick_file = pick.file.clone();
            let pick_expr = pick.expr.clone();
            eval_const(world, &pick_file, &pick_expr, depth + 1)
        }
        _ => None,
    }
}

fn wrap_to(ty: &str, v: i128) -> i128 {
    match ty {
        "c_int" | "i32" | "int32_t" => v as i32 as i128,
        "c_uint" | "u32" | "uint32_t" => v as u32 as i128,
        "c_char" | "c_schar" | "i8" | "int8_t" => v as i8 as i128,
        "c_uchar" | "u8" | "uint8_t" => v as u8 as i128,
        "c_short" | "i16" | "int16_t" => v as i16 as i128,
        "c_ushort" | "u16" | "uint16_t" => v as u16 as i128,
        "c_long" | "c_longlong" | "i64" | "int64_t" | "isize" | "ssize_t" => v as i64 as i128,
        "c_ulong" | "c_ulonglong" | "u64" | "uint64_t" | "usize" | "size_t" => v as u64 as i128,
        _ => v,
    }
}

// --------------------------------------------------------------- C types

#[derive(Clone, Debug)]
enum CTy {
    Void,
    Named {
        name: String,
        konst: bool,
    },
    // the bool marks a const pointer (`T *const`), used for immutable statics
    Ptr(Box<CTy>, bool),
    Arr(Box<CTy>, u128),
    Fun {
        ret: Box<CTy>,
        args: Vec<CTy>,
        variadic: bool,
    },
    // inline anonymous struct/union (from file-local C2Rust_Unnamed_N)
    Anon {
        is_union: bool,
        body: Vec<String>,
    },
}

fn prim(name: &str) -> Option<String> {
    Some(
        match name {
            "c_void" => "void",
            "c_char" => "char",
            "c_schar" => "signed char",
            "c_uchar" => "unsigned char",
            "c_short" => "short",
            "c_ushort" => "unsigned short",
            "c_int" => "int",
            "c_uint" => "unsigned int",
            "c_long" => "long",
            "c_ulong" => "unsigned long",
            "c_longlong" => "long long",
            "c_ulonglong" => "unsigned long long",
            "c_float" | "f32" => "float",
            "c_double" | "f64" => "double",
            "i8" => "int8_t",
            "u8" => "uint8_t",
            "i16" => "int16_t",
            "u16" => "uint16_t",
            "i32" => "int32_t",
            "u32" => "uint32_t",
            "i64" => "int64_t",
            "u64" => "uint64_t",
            "usize" => "size_t",
            "isize" => "ssize_t",
            "bool" => "bool",
            "int8_t" | "uint8_t" | "int16_t" | "uint16_t" | "int32_t" | "uint32_t" | "int64_t"
            | "uint64_t" | "size_t" | "ssize_t" | "intptr_t" | "uintptr_t" | "ptrdiff_t"
            | "va_list" | "wchar_t" | "double" | "float" | "char" | "int" => name,
            _ => return None,
        }
        .to_string(),
    )
}

/// Render a declarator: the classic inside-out C declaration algorithm.
fn decl(c: &CTy, inner: &str) -> String {
    match c {
        CTy::Void => format!("void {}", inner).trim_end().to_string(),
        CTy::Named { name, konst } => {
            let k = if *konst { "const " } else { "" };
            format!("{}{} {}", k, name, inner).trim_end().to_string()
        }
        CTy::Ptr(to, konst) => {
            let inner = format!("*{}{}", if *konst { "const " } else { "" }, inner);
            match **to {
                CTy::Arr(..) | CTy::Fun { .. } => decl(to, &format!("({})", inner)),
                _ => decl(to, &inner),
            }
        }
        CTy::Arr(of, n) => decl(of, &format!("{}[{}]", inner, n)),
        CTy::Fun {
            ret,
            args,
            variadic,
        } => {
            let mut rendered: Vec<String> = args.iter().map(|a| decl(a, "")).collect();
            if *variadic {
                rendered.push("...".to_string());
            }
            if rendered.is_empty() {
                rendered.push("void".to_string());
            }
            decl(ret, &format!("{}({})", inner, rendered.join(", ")))
        }
        CTy::Anon { is_union, body } => {
            let kw = if *is_union { "union" } else { "struct" };
            format!("{} {{ {} }} {}", kw, body.join(" "), inner)
        }
    }
}

// ------------------------------------------------------------- the engine

struct Emitter<'w> {
    world: &'w World,
    emitted: BTreeMap<String, Def>, // name -> def chosen
    opaque: BTreeSet<String>,
    queue: Vec<(String, String)>, // (file context, name)
    unknown: BTreeSet<String>,
    denied_refs: BTreeSet<String>,
    notes: Vec<String>,
}

impl<'w> Emitter<'w> {
    fn new(world: &'w World) -> Self {
        Emitter {
            world,
            emitted: BTreeMap::new(),
            opaque: BTreeSet::new(),
            queue: Vec::new(),
            unknown: BTreeSet::new(),
            denied_refs: BTreeSet::new(),
            notes: Vec::new(),
        }
    }

    fn resolve(&self, file: &str, name: &str) -> Option<Def> {
        if name.starts_with("C2Rust_Unnamed") {
            return self
                .world
                .by_file
                .get(&(file.to_string(), name.to_string()))
                .cloned();
        }
        let defs = self.world.by_name.get(name)?;
        let concrete: Vec<&Def> = defs
            .iter()
            .filter(|d| !matches!(d.kind, Kind::Opaque))
            .collect();
        // 1. canonical copy in types/ (5a may have left only an opaque
        //    declaration there; a concrete def elsewhere then wins)
        if let Some(d) = concrete.iter().find(|d| d.is_types) {
            return Some((*d).clone());
        }
        // 2. the referencing file's own definition
        if let Some(d) = concrete.iter().find(|d| d.file == file) {
            return Some((*d).clone());
        }
        // 3. the copy owned by the module whose exported fns/statics carry
        //    this type in their signatures (same-named private types exist:
        //    lmpack's Unpacker vs msgpack_rpc's)
        if let Some(files) = self.world.sig_files.get(name) {
            if let Some(d) = concrete.iter().find(|d| files.contains(&d.file)) {
                return Some((*d).clone());
            }
        }
        match concrete.first() {
            Some(d) => Some((*d).clone()),
            None => defs.first().cloned(),
        }
    }

    fn want(&mut self, file: &str, name: &str) {
        if self.world.deny.contains(name) {
            // referenced but system-owned: emit a compat typedef to the tag
            self.denied_refs.insert(name.to_string());
            return;
        }
        if self.emitted.contains_key(name) || self.opaque.contains(name) || prim(name).is_some() {
            return;
        }
        self.queue.push((file.to_string(), name.to_string()));
    }

    /// Convert a syn type to a CTy, enqueueing referenced named types.
    fn cty(&mut self, file: &str, ty: &syn::Type) -> Option<CTy> {
        match ty {
            syn::Type::Path(tp) => {
                let seg = tp.path.segments.last()?;
                let name = seg.ident.to_string();
                // `::libc::X` re-exports: the system preamble owns these names.
                if tp.path.segments.len() > 1
                    && tp
                        .path
                        .segments
                        .first()
                        .map_or(false, |s| s.ident == "libc")
                {
                    return Some(CTy::Named { name, konst: false });
                }
                if name == "Option" || name == "GlobalCell" || name == "SharedCell" {
                    // Option<extern "C" fn ...> / GlobalCell<T>: unwrap.
                    if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
                        if let Some(syn::GenericArgument::Type(t)) = ab.args.first() {
                            let t = t.clone();
                            return self.cty(file, &t);
                        }
                    }
                    return None;
                }
                if !matches!(seg.arguments, syn::PathArguments::None) {
                    return None; // other generics: not C
                }
                if let Some(p) = prim(&name) {
                    return Some(CTy::Named {
                        name: p,
                        konst: false,
                    });
                }
                // File-local anonymous types get inlined at the use site.
                if name.starts_with("C2Rust_Unnamed") {
                    let def = self.resolve(file, &name)?;
                    let deffile = def.file.clone();
                    match def.kind {
                        Kind::Alias(ref t) => {
                            let t = t.clone();
                            return self.cty(&deffile, &t);
                        }
                        Kind::Struct(ref fields) | Kind::Union(ref fields) => {
                            let is_union = matches!(def.kind, Kind::Union(_));
                            let fields = fields.clone();
                            let mut body = Vec::new();
                            for f in &fields {
                                body.extend(self.field_lines(&deffile, f)?);
                            }
                            return Some(CTy::Anon { is_union, body });
                        }
                        Kind::Opaque => return None,
                    }
                }
                self.want(file, &name);
                Some(CTy::Named {
                    name: c_name(&name),
                    konst: false,
                })
            }
            syn::Type::Ptr(p) => {
                let mut inner = self.cty(file, &p.elem)?;
                if p.mutability.is_none() {
                    if let CTy::Named { konst, .. } = &mut inner {
                        *konst = true;
                    }
                }
                Some(CTy::Ptr(Box::new(inner), false))
            }
            syn::Type::Array(a) => {
                let len = eval_const(self.world, file, &a.len, 0)? as u128;
                Some(CTy::Arr(Box::new(self.cty(file, &a.elem)?), len))
            }
            syn::Type::BareFn(f) => {
                let mut args = Vec::new();
                for a in &f.inputs {
                    args.push(self.cty(file, &a.ty)?);
                }
                let ret = match &f.output {
                    syn::ReturnType::Default => CTy::Void,
                    syn::ReturnType::Type(_, t) => match &**t {
                        syn::Type::Tuple(t) if t.elems.is_empty() => CTy::Void,
                        syn::Type::Never(_) => CTy::Void,
                        other => self.cty(file, other)?,
                    },
                };
                // A Rust bare-fn type (with or without Option<>) is a C
                // function *pointer*.
                Some(CTy::Ptr(
                    Box::new(CTy::Fun {
                        ret: Box::new(ret),
                        args,
                        variadic: f.variadic.is_some(),
                    }),
                    false,
                ))
            }
            syn::Type::Tuple(t) if t.elems.is_empty() => Some(CTy::Void),
            syn::Type::Never(_) => Some(CTy::Void),
            syn::Type::Paren(p) => {
                let e = (*p.elem).clone();
                self.cty(file, &e)
            }
            syn::Type::Group(g) => {
                let e = (*g.elem).clone();
                self.cty(file, &e)
            }
            _ => None,
        }
    }

    fn bit_cty(&self, spec: &BitSpec, storage_bytes: u64) -> String {
        let last = spec.ty.rsplit("::").next().unwrap_or(&spec.ty);
        match last {
            "bool" => "bool".to_string(),
            "c_int" | "i32" | "int32_t" => "int".to_string(),
            "c_uint" | "u32" | "uint32_t" => "unsigned int".to_string(),
            "u8" | "uint8_t" => "unsigned char".to_string(),
            "i8" | "int8_t" => "signed char".to_string(),
            "u16" | "uint16_t" => "unsigned short".to_string(),
            "i16" | "int16_t" => "short".to_string(),
            // c2rust records the declared C type, but its storage array is
            // authoritative for the unit size: a 64-bit declared type over a
            // 4-byte array means the original C used a 32-bit unit.
            "size_t" | "u64" | "usize" | "c_ulong" | "uint64_t" => {
                if storage_bytes <= 4 {
                    "uint32_t".to_string()
                } else {
                    "uint64_t".to_string()
                }
            }
            "i64" | "isize" | "c_long" | "int64_t" => {
                if storage_bytes <= 4 {
                    "int32_t".to_string()
                } else {
                    "int64_t".to_string()
                }
            }
            // enum-alias types: emit as the unsigned storage c2rust computed
            _ => "unsigned int".to_string(),
        }
    }

    /// Render one struct/union field to C declaration line(s).
    fn field_lines(&mut self, file: &str, f: &Field) -> Option<Vec<String>> {
        if f.padding {
            return Some(vec![]);
        }
        if !f.bits.is_empty() {
            let storage = match &f.ty {
                syn::Type::Array(a) => eval_const(self.world, file, &a.len, 0).unwrap_or(0) as u64,
                _ => 0,
            };
            let mut out = Vec::new();
            for spec in &f.bits {
                out.push(format!(
                    "{} {} : {};",
                    self.bit_cty(spec, storage),
                    c_name(&spec.name),
                    spec.width
                ));
            }
            return Some(out);
        }
        let cty = self.cty(file, &f.ty)?;
        Some(vec![format!("{};", decl(&cty, &c_name(&f.name)))])
    }

    fn run(&mut self) {
        while let Some((file, name)) = self.queue.pop() {
            if self.emitted.contains_key(&name)
                || self.opaque.contains(&name)
                || self.world.deny.contains(&name)
            {
                continue;
            }
            match self.resolve(&file, &name) {
                Some(def) => match def.kind {
                    Kind::Opaque => {
                        self.opaque.insert(name);
                    }
                    Kind::Alias(ref t) => {
                        let t = t.clone();
                        let deffile = def.file.clone();
                        self.emitted.insert(name, def);
                        let _ = self.cty(&deffile, &t);
                    }
                    Kind::Struct(ref fields) | Kind::Union(ref fields) => {
                        let fields = fields.clone();
                        let deffile = def.file.clone();
                        self.emitted.insert(name, def);
                        for f in &fields {
                            if f.padding || !f.bits.is_empty() {
                                continue;
                            }
                            let _ = self.cty(&deffile, &f.ty);
                        }
                    }
                },
                None => {
                    self.unknown.insert(name);
                }
            }
        }
    }
}

// ------------------------------------------------------------- topo order

fn value_deps(world: &World, def: &Def, out: &mut BTreeSet<String>) {
    fn walk(world: &World, file: &str, ty: &syn::Type, out: &mut BTreeSet<String>) {
        match ty {
            syn::Type::Path(tp) => {
                if tp.path.segments.len() > 1
                    && tp
                        .path
                        .segments
                        .first()
                        .map_or(false, |s| s.ident == "libc")
                {
                    return; // external libc name, not a by-value dep of ours
                }
                if let Some(seg) = tp.path.segments.last() {
                    let name = seg.ident.to_string();
                    if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
                        for a in &ab.args {
                            if let syn::GenericArgument::Type(t) = a {
                                walk(world, file, t, out);
                            }
                        }
                        return;
                    }
                    if prim(&name).is_none() {
                        if name.starts_with("C2Rust_Unnamed") {
                            // inlined at use site: recurse for its deps
                            if let Some(d) = world.by_file.get(&(file.to_string(), name.clone())) {
                                let file = d.file.clone();
                                match &d.kind {
                                    Kind::Struct(fs) | Kind::Union(fs) => {
                                        for f in fs.clone() {
                                            walk(world, &file, &f.ty, out);
                                        }
                                    }
                                    Kind::Alias(t) => {
                                        let t = t.clone();
                                        walk(world, &file, &t, out);
                                    }
                                    Kind::Opaque => {}
                                }
                            }
                        } else {
                            out.insert(name);
                        }
                    }
                }
            }
            syn::Type::Array(a) => walk(world, file, &a.elem, out),
            syn::Type::Paren(p) => walk(world, file, &p.elem, out),
            syn::Type::Group(g) => walk(world, file, &g.elem, out),
            // pointers and fn types are not by-value deps
            _ => {}
        }
    }
    match &def.kind {
        Kind::Struct(fields) | Kind::Union(fields) => {
            for f in fields {
                if f.padding || !f.bits.is_empty() {
                    continue;
                }
                walk(world, &def.file, &f.ty, out);
            }
        }
        Kind::Alias(t) => walk(world, &def.file, t, out),
        Kind::Opaque => {}
    }
}

// ------------------------------------------------------------------ main

fn main() {
    let mut args = std::env::args().skip(1);
    let mut root = PathBuf::from(".");
    let mut out_path = None;
    let mut manifest_path = None;
    let mut deny_paths: Vec<PathBuf> = Vec::new();
    let mut extra_roots_path: Option<PathBuf> = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--root" => root = PathBuf::from(args.next().expect("--root DIR")),
            "--out" => out_path = Some(PathBuf::from(args.next().expect("--out FILE"))),
            "--manifest" => {
                manifest_path = Some(PathBuf::from(args.next().expect("--manifest FILE")))
            }
            "--deny" => deny_paths.push(PathBuf::from(args.next().expect("--deny FILE"))),
            "--extra-roots" => {
                extra_roots_path = Some(PathBuf::from(args.next().expect("--extra-roots FILE")))
            }
            other => panic!("unknown arg: {}", other),
        }
    }
    let out_path = out_path.expect("--out is required");

    let mut world = World::default();
    for p in &deny_paths {
        let text = std::fs::read_to_string(p).expect("deny file");
        for line in text.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                world.deny.insert(line.to_string());
            }
        }
    }

    eprintln!("ffigen: parsing crate under {} ...", root.display());
    for path in discover(&root) {
        let rel = path
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let text = std::fs::read_to_string(&path).expect("read source");
        let ast = match syn::parse_file(&text) {
            Ok(a) => a,
            Err(e) => panic!("ffigen: parse error in {}: {}", rel, e),
        };
        collect_file(&mut world, &rel, ast);
    }
    for f in &world.fns {
        let mut names = BTreeSet::new();
        for input in &f.sig.inputs {
            if let syn::FnArg::Typed(pt) = input {
                type_names(&pt.ty, &mut names);
            }
        }
        if let syn::ReturnType::Type(_, t) = &f.sig.output {
            type_names(t, &mut names);
        }
        for n in names {
            world.sig_files.entry(n).or_default().insert(f.file.clone());
        }
    }
    for s in &world.statics {
        let mut names = BTreeSet::new();
        type_names(&s.ty, &mut names);
        for n in names {
            world.sig_files.entry(n).or_default().insert(s.file.clone());
        }
    }

    eprintln!(
        "ffigen: {} type names, {} const names, {} exported fns, {} exported statics",
        world.by_name.len(),
        world.consts.len(),
        world.fns.len(),
        world.statics.len()
    );

    // ---- closure roots: every canonical type + everything exports touch
    let mut emitter = Emitter::new(&world);
    for (name, defs) in &world.by_name {
        if let Some(d) = defs.iter().find(|d| d.is_types) {
            // types/libc.rs mirrors glibc; the system preamble owns that
            // domain, so its types are emitted only if the closure pulls
            // them in (and not denied).
            if !world.deny.contains(name) && !d.file.ends_with("types/libc.rs") {
                emitter.queue.push((d.file.clone(), name.clone()));
            }
        }
    }
    if let Some(p) = &extra_roots_path {
        for line in std::fs::read_to_string(p).expect("extra roots").lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                emitter.queue.push((String::new(), line.to_string()));
            }
        }
    }
    emitter.run();

    // fn signatures + statics enqueue more types
    let mut protos: Vec<(String, String)> = Vec::new();
    let mut skipped_fns = Vec::new();
    for f in &world.fns {
        let mut args = Vec::new();
        let mut ok = true;
        for input in &f.sig.inputs {
            match input {
                syn::FnArg::Typed(pt) => match emitter.cty(&f.file, &pt.ty) {
                    Some(c) => args.push(c),
                    None => ok = false,
                },
                syn::FnArg::Receiver(_) => ok = false,
            }
        }
        let variadic = f.sig.variadic.is_some();
        let ret = match &f.sig.output {
            syn::ReturnType::Default => Some(CTy::Void),
            syn::ReturnType::Type(_, t) => match &**t {
                syn::Type::Tuple(tu) if tu.elems.is_empty() => Some(CTy::Void),
                syn::Type::Never(_) => Some(CTy::Void),
                other => emitter.cty(&f.file, other),
            },
        };
        match (ok, ret) {
            (true, Some(ret)) => {
                let fun = CTy::Fun {
                    ret: Box::new(ret),
                    args,
                    variadic,
                };
                protos.push((f.name.clone(), format!("{};", decl(&fun, &f.name))));
            }
            _ => skipped_fns.push(f.name.clone()),
        }
    }
    protos.sort();
    let mut static_decls: Vec<(String, String)> = Vec::new();
    let mut skipped_statics = Vec::new();
    fn constify(c: CTy) -> CTy {
        match c {
            CTy::Named { name, .. } => CTy::Named { name, konst: true },
            CTy::Ptr(to, _) => CTy::Ptr(to, true),
            CTy::Arr(of, n) => CTy::Arr(Box::new(constify(*of)), n),
            other => other,
        }
    }
    for s in &world.statics {
        // GlobalCell/SharedCell wrap mutable editor state; a plain Rust
        // static is immutable, i.e. `const` on the C side.
        let cell = match &s.ty {
            syn::Type::Path(tp) => tp.path.segments.last().map_or(false, |seg| {
                seg.ident == "GlobalCell" || seg.ident == "SharedCell"
            }),
            _ => false,
        };
        match emitter.cty(&s.file, &s.ty) {
            Some(c) => {
                let c = if cell { c } else { constify(c) };
                static_decls.push((s.name.clone(), format!("extern {};", decl(&c, &s.name))))
            }
            None => skipped_statics.push(s.name.clone()),
        }
    }
    static_decls.sort();
    emitter.run();

    // ---- constants
    let mut const_vals: BTreeMap<String, i128> = BTreeMap::new();
    let mut const_conflicts: Vec<String> = Vec::new();
    let mut const_uneval = 0usize;
    for (name, copies) in &world.consts {
        // reserved names are compiler/libc macros (__INT_MAX__, ...): the C
        // preprocessor would mangle them, and no spec can want them
        if name.starts_with("__") || world.deny.contains(name) {
            continue;
        }
        let mut vals = BTreeSet::new();
        for k in copies {
            if let Some(v) = eval_const(&world, &k.file, &k.expr, 0) {
                vals.insert(v);
            }
        }
        match vals.len() {
            0 => const_uneval += 1,
            1 => {
                let v = *vals.iter().next().unwrap();
                // LuaJIT `static const` handles up to 32-bit values.
                if v >= -(1i128 << 31) && v < (1i128 << 32) {
                    const_vals.insert(name.clone(), v);
                }
            }
            _ => const_conflicts.push(name.clone()),
        }
    }
    // Constants share the cdef namespace with types; drop collisions.
    for name in emitter.emitted.keys() {
        const_vals.remove(name);
    }
    for name in &emitter.opaque {
        const_vals.remove(name);
    }

    // ---- topological order over by-value deps
    let emitted = emitter.emitted.clone();
    let mut order: Vec<String> = Vec::new();
    let mut state: HashMap<String, u8> = HashMap::new(); // 0 unseen 1 visiting 2 done
    fn visit(
        name: &str,
        world: &World,
        emitted: &BTreeMap<String, Def>,
        state: &mut HashMap<String, u8>,
        order: &mut Vec<String>,
    ) {
        match state.get(name) {
            Some(2) => return,
            Some(1) => panic!("by-value type cycle through {}", name),
            _ => {}
        }
        let def = match emitted.get(name) {
            Some(d) => d,
            None => return,
        };
        state.insert(name.to_string(), 1);
        let mut deps = BTreeSet::new();
        value_deps(world, def, &mut deps);
        for d in deps {
            visit(&d, world, emitted, state, order);
        }
        state.insert(name.to_string(), 2);
        order.push(name.to_string());
    }
    for name in emitted.keys() {
        visit(name, &world, &emitted, &mut state, &mut order);
    }

    // ---- emit
    let mut chunk = String::new();
    let mut manifest_types: Vec<(String, String, Vec<String>)> = Vec::new(); // (name, kindword, fields)
    writeln!(chunk, "// Generated by tools/ffigen. Do not edit.").unwrap();
    writeln!(
        chunk,
        "// C declarations for the unit tests' ffi.cdef, derived from the Rust crate."
    )
    .unwrap();

    for name in &emitter.opaque {
        let c = c_name(name);
        writeln!(chunk, "typedef struct {} {};", c, c).unwrap();
    }
    // Denied names are defined by the system preamble the harness cdefs
    // before this chunk; give the tag a same-named typedef so field/proto
    // references by bare name resolve.
    for name in emitter.denied_refs.clone() {
        writeln!(chunk, "typedef struct {} {};", name, name).unwrap();
    }
    for name in &order {
        let def = &emitted[name];
        let c = c_name(name);
        match def.kind {
            Kind::Struct(_) => writeln!(chunk, "typedef struct {} {};", c, c).unwrap(),
            Kind::Union(_) => writeln!(chunk, "typedef union {} {};", c, c).unwrap(),
            _ => {}
        }
    }
    // Aliases before struct bodies, topo-sorted among themselves: a typedef
    // name must be declared before any use (even inside fn-pointer types),
    // and only struct/union tags have forward declarations above.
    fn alias_refs(world: &World, file: &str, ty: &syn::Type, out: &mut BTreeSet<String>) {
        match ty {
            syn::Type::Path(tp) => {
                if tp.path.segments.len() > 1
                    && tp
                        .path
                        .segments
                        .first()
                        .map_or(false, |s| s.ident == "libc")
                {
                    return;
                }
                if let Some(seg) = tp.path.segments.last() {
                    let name = seg.ident.to_string();
                    if let syn::PathArguments::AngleBracketed(ab) = &seg.arguments {
                        for a in &ab.args {
                            if let syn::GenericArgument::Type(t) = a {
                                alias_refs(world, file, t, out);
                            }
                        }
                        return;
                    }
                    if prim(&name).is_some() {
                        return;
                    }
                    if name.starts_with("C2Rust_Unnamed") {
                        if let Some(d) = world.by_file.get(&(file.to_string(), name)) {
                            let dfile = d.file.clone();
                            match &d.kind {
                                Kind::Struct(fs) | Kind::Union(fs) => {
                                    for f in fs.clone() {
                                        alias_refs(world, &dfile, &f.ty, out);
                                    }
                                }
                                Kind::Alias(t) => alias_refs(world, &dfile, &t.clone(), out),
                                Kind::Opaque => {}
                            }
                        }
                        return;
                    }
                    out.insert(name);
                }
            }
            syn::Type::Ptr(p) => alias_refs(world, file, &p.elem, out),
            syn::Type::Array(a) => alias_refs(world, file, &a.elem, out),
            syn::Type::BareFn(f) => {
                for a in &f.inputs {
                    alias_refs(world, file, &a.ty, out);
                }
                if let syn::ReturnType::Type(_, t) = &f.output {
                    alias_refs(world, file, t, out);
                }
            }
            syn::Type::Paren(p) => alias_refs(world, file, &p.elem, out),
            syn::Type::Group(g) => alias_refs(world, file, &g.elem, out),
            _ => {}
        }
    }
    fn avisit(
        name: &str,
        world: &World,
        emitted: &BTreeMap<String, Def>,
        state: &mut HashMap<String, u8>,
        aorder: &mut Vec<String>,
    ) {
        match state.get(name) {
            Some(_) => return, // done, or benign cycle through a pointer
            None => {}
        }
        let def = match emitted.get(name) {
            Some(d) => d,
            None => return,
        };
        let t = match &def.kind {
            Kind::Alias(t) => t.clone(),
            _ => return,
        };
        state.insert(name.to_string(), 1);
        let mut deps = BTreeSet::new();
        alias_refs(world, &def.file, &t, &mut deps);
        for d in deps {
            if d != name {
                avisit(&d, world, emitted, state, aorder);
            }
        }
        aorder.push(name.to_string());
    }
    let mut astate: HashMap<String, u8> = HashMap::new();
    let mut aorder: Vec<String> = Vec::new();
    for name in emitted.keys() {
        avisit(name, &world, &emitted, &mut astate, &mut aorder);
    }
    for name in &aorder {
        let def = emitted[name].clone();
        let c = c_name(name);
        if let Kind::Alias(t) = &def.kind {
            match emitter.cty(&def.file, t) {
                Some(CTy::Named { name: target, .. }) => {
                    // typedef to a struct/union tag or another typedef name
                    let kw = match emitted.get(&target).map(|d| &d.kind) {
                        Some(Kind::Struct(_)) => "struct ",
                        Some(Kind::Union(_)) => "union ",
                        _ => "",
                    };
                    if target != c {
                        writeln!(chunk, "typedef {}{} {};", kw, target, c).unwrap();
                    }
                }
                Some(cty) => writeln!(chunk, "typedef {};", decl(&cty, &c)).unwrap(),
                None => emitter.notes.push(format!("alias skipped: {}", name)),
            }
        }
    }
    for name in &order {
        let def = emitted[name].clone();
        let c = c_name(name);
        match &def.kind {
            Kind::Alias(_) => {} // emitted above
            Kind::Struct(fields) | Kind::Union(fields) => {
                let kw = if matches!(def.kind, Kind::Union(_)) {
                    "union"
                } else {
                    "struct"
                };
                let mut body: Vec<String> = Vec::new();
                let mut names: Vec<String> = Vec::new();
                let mut ok = true;
                for f in fields {
                    match emitter.field_lines(&def.file, f) {
                        Some(lines) => {
                            for l in lines {
                                body.push(format!("  {}", l));
                            }
                            if f.bits.is_empty() && !f.padding {
                                names.push(c_name(&f.name));
                            }
                        }
                        None => {
                            emitter
                                .notes
                                .push(format!("field skipped: {}.{}", name, f.name));
                            ok = false;
                        }
                    }
                }
                if !ok {
                    // an unrenderable field would corrupt the layout: leave
                    // the tag incomplete (pointer-only use keeps working).
                    emitter
                        .notes
                        .push(format!("type left incomplete: {}", name));
                    continue;
                }
                let align = def
                    .align
                    .map(|a| format!(" __attribute__((aligned({})))", a))
                    .unwrap_or_default();
                writeln!(chunk, "{}{} {} {{\n{}\n}};", kw, align, c, body.join("\n")).unwrap();
                manifest_types.push((c.clone(), kw.to_string(), names));
            }
            Kind::Opaque => {}
        }
    }
    writeln!(chunk).unwrap();
    for (name, v) in &const_vals {
        if *v >= -(1i128 << 31) && *v < (1i128 << 31) {
            writeln!(chunk, "static const int {} = {};", name, v).unwrap();
        } else {
            writeln!(chunk, "static const unsigned int {} = {}u;", name, v).unwrap();
        }
    }
    writeln!(chunk).unwrap();
    for (_, d) in &static_decls {
        writeln!(chunk, "{}", d).unwrap();
    }
    writeln!(chunk).unwrap();
    for (_, d) in &protos {
        writeln!(chunk, "{}", d).unwrap();
    }

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(&out_path, &chunk).expect("write chunk");

    // ---- manifest (drives scripts/check-unit-cdefs.py)
    if let Some(mp) = manifest_path {
        let mut m = String::from("{\n");
        m.push_str("  \"types\": {\n");
        let tlines: Vec<String> = manifest_types
            .iter()
            .map(|(n, kw, fs)| {
                let fl: Vec<String> = fs.iter().map(|f| format!("\"{}\"", f)).collect();
                format!(
                    "    \"{}\": {{\"kind\": \"{}\", \"fields\": [{}]}}",
                    n,
                    kw,
                    fl.join(", ")
                )
            })
            .collect();
        m.push_str(&tlines.join(",\n"));
        m.push_str("\n  },\n  \"consts\": {\n");
        let clines: Vec<String> = const_vals
            .iter()
            .map(|(n, v)| format!("    \"{}\": {}", n, v))
            .collect();
        m.push_str(&clines.join(",\n"));
        m.push_str("\n  },\n  \"protos\": {\n");
        let plines: Vec<String> = protos
            .iter()
            .map(|(n, d)| format!("    \"{}\": {:?}", n, d))
            .collect();
        m.push_str(&plines.join(",\n"));
        m.push_str("\n  },\n  \"statics\": {\n");
        let slines: Vec<String> = static_decls
            .iter()
            .map(|(n, d)| format!("    \"{}\": {:?}", n, d))
            .collect();
        m.push_str(&slines.join(",\n"));
        m.push_str("\n  }\n}\n");
        std::fs::write(mp, m).expect("write manifest");
    }

    eprintln!(
        "ffigen: emitted {} types ({} opaque, {} unknown->opaque), {} consts ({} conflict-dropped, {} unevaluated), {} protos ({} skipped), {} statics ({} skipped)",
        emitted.len(),
        emitter.opaque.len(),
        emitter.unknown.len(),
        const_vals.len(),
        const_conflicts.len(),
        const_uneval,
        protos.len(),
        skipped_fns.len(),
        static_decls.len(),
        skipped_statics.len()
    );
    if !const_conflicts.is_empty() {
        eprintln!(
            "ffigen: conflicting consts dropped: {}",
            const_conflicts.join(" ")
        );
    }
    if !skipped_fns.is_empty() {
        eprintln!("ffigen: skipped fns: {}", skipped_fns.join(" "));
    }
    if !skipped_statics.is_empty() {
        eprintln!("ffigen: skipped statics: {}", skipped_statics.join(" "));
    }
    if !emitter.unknown.is_empty() {
        eprintln!(
            "ffigen: unknown types: {}",
            emitter
                .unknown
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
    for n in &emitter.notes {
        eprintln!("ffigen: note: {}", n);
    }
}
