//! Phase 5a: unify the c2rust-duplicated type graph.
//!
//! c2rust emitted the full type graph into every translation unit: each module
//! has its own nominal `buf_T`, `typval_T`, ..., whose anonymous nested
//! members got per-module names (`C2Rust_Unnamed_10` vs `C2Rust_Unnamed_11`).
//! This tool:
//!
//!   1. inventories every top-level struct/union/type-alias/opaque-extern-type
//!      per module,
//!   2. proves structural equality by partition refinement (exact
//!      bisimulation: two defs are equivalent iff their layout-relevant
//!      skeletons match and every locally-named type they reference is
//!      equivalent, co-inductively — cycles like `buf_T -> *mut buf_T` are
//!      handled by the fixpoint),
//!   3. derives stable crate-wide names for anonymous types from their parent
//!      field (`file_buffer` + `b_wininfo` -> `file_buffer_b_wininfo`),
//!   4. extracts one canonical definition per group into `src/nvim/types/`
//!      (split by domain, each file under the 1k-line ratchet cap) and
//!      replaces every per-module copy with a `pub use` re-export aliased to
//!      the old local name, so no other line in the module changes,
//!   5. emits a temporary layout-parity test asserting
//!      size/align/offset-of-every-field of every copy against the canonical
//!      copy, to be run before and after the merge.
//!
//! Same-named types that are *not* structurally identical are never merged:
//! for each name only the largest equivalence class is canonicalized and the
//! outliers keep their module-local definitions (reported).

use quote::ToTokens;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;
use std::ops::Range;
use std::path::{Path, PathBuf};

fn span_range(node: &dyn ToTokens) -> Range<usize> {
    let mut iter = node.to_token_stream().into_iter();
    let Some(first) = iter.next() else {
        panic!("empty token stream for spanned node");
    };
    let mut range = first.span().byte_range();
    for tt in iter {
        let r = tt.span().byte_range();
        range.start = range.start.min(r.start);
        range.end = range.end.max(r.end);
    }
    range
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Struct,
    Union,
    Alias,
    Opaque,
}

struct FileData {
    rel: String,     // repo-relative path, e.g. "src/nvim/buffer.rs"
    modpath: String, // e.g. "src::nvim::buffer"
    src: String,
    ast: syn::File,
}

struct Def {
    file: usize,
    name: String,
    kind: Kind,
    range: Range<usize>,
    /// Layout-relevant skeleton with `\x00` placeholders where local nominal
    /// references (edges) sit. Derives excluded (unioned at merge time).
    skeleton: String,
    /// Sorted derive idents (behavior attrs, not layout).
    derives: BTreeSet<String>,
    edges: Vec<usize>,
    edge_labels: Vec<String>,
    fields: Vec<String>,
    dirty: Option<String>,
    /// item index in FileData.ast (for rename visiting)
    item_idx: usize,
    /// for opaque: index within the foreign mod
    sub_idx: Option<usize>,
}

fn is_anon(name: &str) -> bool {
    name.starts_with("C2Rust_Unnamed")
}

// ---------------------------------------------------------------------------
// Type serialization
// ---------------------------------------------------------------------------

struct Ser<'a> {
    env: &'a HashMap<String, usize>,
    edges: Vec<usize>,
    edge_labels: Vec<String>,
    label: String,
    out: String,
    dirty: Option<String>,
}

impl<'a> Ser<'a> {
    fn ty(&mut self, ty: &syn::Type) {
        match ty {
            syn::Type::Path(tp) => {
                if tp.qself.is_some() {
                    self.dirty = Some("qualified self type".into());
                    return;
                }
                let segs = &tp.path.segments;
                if tp.path.leading_colon.is_none() && segs.len() == 1 && segs[0].arguments.is_none()
                {
                    let ident = segs[0].ident.to_string();
                    if let Some(&target) = self.env.get(&ident) {
                        self.edges.push(target);
                        self.edge_labels.push(self.label.clone());
                        self.out.push('\x00');
                        return;
                    }
                    // primitive / not locally defined: literal
                    self.out.push_str(&ident);
                    return;
                }
                if tp.path.leading_colon.is_some() {
                    self.out.push_str("::");
                }
                for (i, seg) in segs.iter().enumerate() {
                    if i > 0 {
                        self.out.push_str("::");
                    }
                    self.out.push_str(&seg.ident.to_string());
                    match &seg.arguments {
                        syn::PathArguments::None => {}
                        syn::PathArguments::AngleBracketed(ab) => {
                            self.out.push('<');
                            for arg in &ab.args {
                                match arg {
                                    syn::GenericArgument::Type(t) => self.ty(t),
                                    other => {
                                        self.out.push_str(&other.to_token_stream().to_string())
                                    }
                                }
                                self.out.push(',');
                            }
                            self.out.push('>');
                        }
                        syn::PathArguments::Parenthesized(_) => {
                            self.dirty = Some("parenthesized path args".into());
                        }
                    }
                }
            }
            syn::Type::Ptr(p) => {
                self.out.push_str(if p.mutability.is_some() {
                    "*mut "
                } else {
                    "*const "
                });
                self.ty(&p.elem);
            }
            syn::Type::Array(a) => {
                self.out.push('[');
                self.ty(&a.elem);
                self.out.push(';');
                let len = a.len.to_token_stream().to_string();
                // A named constant in an array length would not resolve from
                // the canonical home; flag it for manual handling.
                for tok in a.len.to_token_stream() {
                    if let proc_macro2::TokenTree::Ident(id) = tok {
                        let s = id.to_string();
                        if s != "as" && s != "usize" && !s.chars().next().unwrap().is_ascii_digit()
                        {
                            self.dirty = Some(format!("array length names `{s}`"));
                        }
                    }
                }
                self.out.push_str(&len);
                self.out.push(']');
            }
            syn::Type::BareFn(f) => {
                if f.unsafety.is_some() {
                    self.out.push_str("unsafe ");
                }
                if let Some(abi) = &f.abi {
                    self.out.push_str(&abi.to_token_stream().to_string());
                }
                self.out.push_str("fn(");
                for input in &f.inputs {
                    self.ty(&input.ty);
                    self.out.push(',');
                }
                if f.variadic.is_some() {
                    self.out.push_str("...");
                }
                self.out.push(')');
                if let syn::ReturnType::Type(_, t) = &f.output {
                    self.out.push_str("->");
                    self.ty(t);
                }
            }
            syn::Type::Tuple(t) => {
                self.out.push('(');
                for elem in &t.elems {
                    self.ty(elem);
                    self.out.push(',');
                }
                self.out.push(')');
            }
            syn::Type::Paren(p) => self.ty(&p.elem),
            syn::Type::Group(g) => self.ty(&g.elem),
            other => {
                self.out.push_str(&other.to_token_stream().to_string());
            }
        }
    }
}

/// Layout-relevant attributes (everything except derives and docs), as a
/// deterministic string. Derives are compared separately.
/// Iterate a `#[bitfield(...)]` attribute's tokens, calling `f` with each
/// string literal that is the value of `ty = "..."`. c2rust names the
/// bitfield's logical type there — as a *string* — so it references types
/// outside the syntactic type graph.
fn visit_bitfield_ty_lits(attr: &syn::Attribute, mut f: impl FnMut(&proc_macro2::Literal)) {
    if !attr.path().is_ident("bitfield") {
        return;
    }
    let syn::Meta::List(ml) = &attr.meta else { return };
    let mut state = 0u8; // 1 = saw `ty`, 2 = saw `ty =`
    for tt in ml.tokens.clone() {
        state = match (&tt, state) {
            (proc_macro2::TokenTree::Ident(id), _) if id == "ty" => 1,
            (proc_macro2::TokenTree::Punct(p), 1) if p.as_char() == '=' => 2,
            (proc_macro2::TokenTree::Literal(lit), 2) => {
                f(lit);
                0
            }
            _ => 0,
        };
    }
}

impl<'a> Ser<'a> {
    /// Field-attribute skeleton: like `attr_skeleton`, but a bitfield `ty`
    /// string naming a module-local type becomes an edge placeholder so the
    /// reference participates in equality, closure, and renaming.
    fn field_attrs(&mut self, attrs: &[syn::Attribute]) -> String {
        let mut skel = String::new();
        for attr in attrs {
            if attr.path().is_ident("doc") {
                continue;
            }
            if attr.path().is_ident("bitfield") {
                let mut tokens = attr.to_token_stream().to_string();
                let mut hits: Vec<String> = Vec::new();
                visit_bitfield_ty_lits(attr, |lit| {
                    let s = lit.to_string();
                    let inner = s.trim_matches('"');
                    if self.env.contains_key(inner) {
                        hits.push(s.clone());
                    }
                });
                for s in hits {
                    let inner = s.trim_matches('"');
                    let target = self.env[inner];
                    self.edges.push(target);
                    self.edge_labels.push(self.label.clone());
                    tokens = tokens.replacen(&s, "\"\u{0}\"", 1);
                }
                skel.push_str(&tokens);
                skel.push(';');
                continue;
            }
            skel.push_str(&attr.to_token_stream().to_string());
            skel.push(';');
        }
        skel
    }
}

fn attr_skeleton(attrs: &[syn::Attribute]) -> (String, BTreeSet<String>) {
    let mut skel = String::new();
    let mut derives = BTreeSet::new();
    for attr in attrs {
        if attr.path().is_ident("doc") {
            continue;
        }
        if attr.path().is_ident("derive") {
            let _ = attr.parse_nested_meta(|meta| {
                derives.insert(meta.path.to_token_stream().to_string());
                Ok(())
            });
            continue;
        }
        skel.push_str(&attr.to_token_stream().to_string());
        skel.push(';');
    }
    (skel, derives)
}

// ---------------------------------------------------------------------------
// Inventory
// ---------------------------------------------------------------------------

fn discover_files(repo: &Path) -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    let mut stack = vec![repo.join("src")];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let rel = path
                .strip_prefix(repo)
                .unwrap()
                .to_string_lossy()
                .to_string();
            if path.is_dir() {
                if rel == "src/bin" || rel == "src/nvim/types" {
                    continue;
                }
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push((rel, path));
            }
        }
    }
    out.sort();
    out
}

fn modpath_of(rel: &str) -> String {
    let stem = rel.trim_end_matches(".rs");
    let mut parts: Vec<&str> = stem.split('/').collect();
    // lib.rs: `#[path = "eval.rs"] pub mod eval_1;` (eval/ dir also exists)
    if stem == "src/nvim/eval" {
        *parts.last_mut().unwrap() = "eval_1";
    }
    parts
        .iter()
        .map(|p| match *p {
            // keyword module names are declared `pub mod r#move;` etc.
            "move" | "match" | "loop" => format!("r#{p}"),
            other => other.to_string(),
        })
        .collect::<Vec<_>>()
        .join("::")
}

fn collect_defs(files: &[FileData]) -> Vec<Def> {
    // Pass A: names
    let mut defs: Vec<Def> = Vec::new();
    let mut pending: Vec<(usize, usize, Option<usize>)> = Vec::new(); // (file, item, sub)
    for (fi, file) in files.iter().enumerate() {
        for (ii, item) in file.ast.items.iter().enumerate() {
            match item {
                syn::Item::Struct(s) if s.generics.params.is_empty() => {
                    defs.push(def_stub(
                        fi,
                        s.ident.to_string(),
                        Kind::Struct,
                        span_range(s),
                        ii,
                        None,
                    ));
                    pending.push((fi, ii, None));
                }
                syn::Item::Union(u) if u.generics.params.is_empty() => {
                    defs.push(def_stub(
                        fi,
                        u.ident.to_string(),
                        Kind::Union,
                        span_range(u),
                        ii,
                        None,
                    ));
                    pending.push((fi, ii, None));
                }
                syn::Item::Type(t) if t.generics.params.is_empty() => {
                    defs.push(def_stub(
                        fi,
                        t.ident.to_string(),
                        Kind::Alias,
                        span_range(t),
                        ii,
                        None,
                    ));
                    pending.push((fi, ii, None));
                }
                syn::Item::ForeignMod(fm) => {
                    for (si, fitem) in fm.items.iter().enumerate() {
                        if let syn::ForeignItem::Type(ft) = fitem {
                            defs.push(def_stub(
                                fi,
                                ft.ident.to_string(),
                                Kind::Opaque,
                                span_range(ft),
                                ii,
                                Some(si),
                            ));
                            pending.push((fi, ii, Some(si)));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // env per file: name -> def index
    let mut envs: Vec<HashMap<String, usize>> = vec![HashMap::new(); files.len()];
    for (di, def) in defs.iter().enumerate() {
        envs[def.file].insert(def.name.clone(), di);
    }

    // Pass B: skeletons
    for (di, (fi, ii, sub)) in pending.iter().enumerate() {
        let env = &envs[*fi];
        let item = &files[*fi].ast.items[*ii];
        let def = &mut defs[di];
        match (item, sub) {
            (syn::Item::Struct(s), None) => {
                let (askel, derives) = attr_skeleton(&s.attrs);
                let mut ser = Ser {
                    env,
                    edges: vec![],
                    edge_labels: vec![],
                    label: String::new(),
                    out: String::new(),
                    dirty: None,
                };
                let _ = write!(ser.out, "struct;{};{};", s.vis.to_token_stream(), askel);
                for f in &s.fields {
                    let Some(id) = &f.ident else {
                        ser.dirty = Some("tuple struct".into());
                        continue;
                    };
                    ser.label = id.to_string();
                    let fa = ser.field_attrs(&f.attrs);
                    let _ = write!(ser.out, "{}:{}:{}:", id, f.vis.to_token_stream(), fa);
                    ser.ty(&f.ty);
                    ser.out.push(';');
                    def.fields.push(id.to_string());
                }
                def.skeleton = ser.out;
                def.derives = derives;
                def.edges = ser.edges;
                def.edge_labels = ser.edge_labels;
                def.dirty = ser.dirty;
            }
            (syn::Item::Union(u), None) => {
                let (askel, derives) = attr_skeleton(&u.attrs);
                let mut ser = Ser {
                    env,
                    edges: vec![],
                    edge_labels: vec![],
                    label: String::new(),
                    out: String::new(),
                    dirty: None,
                };
                let _ = write!(ser.out, "union;{};{};", u.vis.to_token_stream(), askel);
                for f in &u.fields.named {
                    let id = f.ident.as_ref().unwrap();
                    ser.label = id.to_string();
                    let fa = ser.field_attrs(&f.attrs);
                    let _ = write!(ser.out, "{}:{}:{}:", id, f.vis.to_token_stream(), fa);
                    ser.ty(&f.ty);
                    ser.out.push(';');
                    def.fields.push(id.to_string());
                }
                def.skeleton = ser.out;
                def.derives = derives;
                def.edges = ser.edges;
                def.edge_labels = ser.edge_labels;
                def.dirty = ser.dirty;
            }
            (syn::Item::Type(t), None) => {
                let (askel, derives) = attr_skeleton(&t.attrs);
                let mut ser = Ser {
                    env,
                    edges: vec![],
                    edge_labels: vec![],
                    label: "=".into(),
                    out: String::new(),
                    dirty: None,
                };
                let _ = write!(ser.out, "alias;{};{};", t.vis.to_token_stream(), askel);
                ser.ty(&t.ty);
                def.skeleton = ser.out;
                def.derives = derives;
                def.edges = ser.edges;
                def.edge_labels = ser.edge_labels;
                def.dirty = ser.dirty;
            }
            (syn::Item::ForeignMod(_), Some(_)) => {
                // Opaque C types are equal iff they have the same tag name.
                def.skeleton = format!("opaque;{};", def.name);
            }
            _ => unreachable!(),
        }
    }
    defs
}

fn def_stub(
    file: usize,
    name: String,
    kind: Kind,
    range: Range<usize>,
    item_idx: usize,
    sub_idx: Option<usize>,
) -> Def {
    Def {
        file,
        name,
        kind,
        range,
        skeleton: String::new(),
        derives: BTreeSet::new(),
        edges: vec![],
        edge_labels: vec![],
        fields: vec![],
        dirty: None,
        item_idx,
        sub_idx,
    }
}

// ---------------------------------------------------------------------------
// Partition refinement
// ---------------------------------------------------------------------------

fn refine(defs: &[Def]) -> Vec<u32> {
    let mut interner: HashMap<(String, Option<usize>), u32> = HashMap::new();
    let mut class: Vec<u32> = defs
        .iter()
        .enumerate()
        .map(|(di, d)| {
            // dirty defs are poisoned into singleton classes
            let key = (d.skeleton.clone(), d.dirty.as_ref().map(|_| di));
            let next = interner.len() as u32;
            *interner.entry(key).or_insert(next)
        })
        .collect();
    loop {
        let mut interner: HashMap<(u32, Vec<u32>), u32> = HashMap::new();
        let next: Vec<u32> = defs
            .iter()
            .enumerate()
            .map(|(di, d)| {
                let key = (class[di], d.edges.iter().map(|&e| class[e]).collect());
                let next = interner.len() as u32;
                *interner.entry(key).or_insert(next)
            })
            .collect();
        let old_count = class.iter().collect::<HashSet<_>>().len();
        if interner.len() == old_count {
            return next;
        }
        class = next;
    }
}

// ---------------------------------------------------------------------------
// Grouping
// ---------------------------------------------------------------------------

fn group_keys(defs: &[Def], class: &[u32]) -> Vec<String> {
    // contexts[d] = (parent def, label) for every edge parent -> d
    let mut contexts: Vec<Vec<(usize, String)>> = vec![Vec::new(); defs.len()];
    for (di, d) in defs.iter().enumerate() {
        for (e, l) in d.edges.iter().zip(&d.edge_labels) {
            contexts[*e].push((di, l.clone()));
        }
    }
    let mut keys: Vec<Option<String>> = vec![None; defs.len()];
    fn key_of(
        di: usize,
        defs: &[Def],
        class: &[u32],
        contexts: &[Vec<(usize, String)>],
        keys: &mut Vec<Option<String>>,
        visiting: &mut HashSet<usize>,
    ) -> String {
        if let Some(k) = &keys[di] {
            return k.clone();
        }
        if !visiting.insert(di) {
            return format!("L:{di}"); // parent cycle: give up, stays local
        }
        let d = &defs[di];
        let k = if !is_anon(&d.name) {
            format!("N:{}:{}", d.name, class[di])
        } else {
            let mut ctxs: Vec<(String, String)> = contexts[di]
                .iter()
                .map(|(p, l)| (key_of(*p, defs, class, contexts, keys, visiting), l.clone()))
                .collect();
            ctxs.sort();
            ctxs.dedup();
            match ctxs.first() {
                None => format!("L:{di}"),
                Some((pk, l)) if pk.starts_with("L:") => {
                    let _ = (pk, l);
                    format!("L:{di}") // parent is local-only: stay local
                }
                Some((pk, l)) => format!("A:{}:{}:{}", class[di], pk, l),
            }
        };
        visiting.remove(&di);
        keys[di] = Some(k.clone());
        k
    }
    let mut visiting = HashSet::new();
    (0..defs.len())
        .map(|di| key_of(di, defs, class, &contexts, &mut keys, &mut visiting))
        .collect()
}

struct Group {
    key: String,
    members: Vec<usize>,
    canonical_name: String,
    domain: String,
    canonical_member: usize, // def index
    merged: bool,
}

// ---------------------------------------------------------------------------
// Rename visitor: single-segment path idents (and the item's own ident)
// ---------------------------------------------------------------------------

struct RenameVisitor<'a> {
    map: &'a HashMap<String, String>,
    edits: Vec<(Range<usize>, String)>,
}

impl<'a, 'ast> syn::visit::Visit<'ast> for RenameVisitor<'a> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        if path.leading_colon.is_none() && path.segments.len() == 1 {
            let seg = &path.segments[0];
            if let Some(new) = self.map.get(&seg.ident.to_string()) {
                self.edits
                    .push((seg.ident.span().byte_range(), new.clone()));
            }
        }
        syn::visit::visit_path(self, path);
    }

    fn visit_attribute(&mut self, attr: &'ast syn::Attribute) {
        // bitfield `ty = "..."` strings name types too.
        visit_bitfield_ty_lits(attr, |lit| {
            let s = lit.to_string();
            if let Some(new) = self.map.get(s.trim_matches('"')) {
                self.edits
                    .push((lit.span().byte_range(), format!("\"{new}\"")));
            }
        });
        syn::visit::visit_attribute(self, attr);
    }
}

fn rename_edits(
    item: &syn::Item,
    sub: Option<usize>,
    map: &HashMap<String, String>,
) -> Vec<(Range<usize>, String)> {
    use syn::visit::Visit;
    let mut v = RenameVisitor { map, edits: vec![] };
    match (item, sub) {
        (syn::Item::ForeignMod(fm), Some(si)) => {
            if let syn::ForeignItem::Type(ft) = &fm.items[si] {
                if let Some(new) = map.get(&ft.ident.to_string()) {
                    v.edits.push((ft.ident.span().byte_range(), new.clone()));
                }
            }
        }
        (item, None) => {
            v.visit_item(item);
            // visit_item covers the ident via visit_ident? No: cover explicitly.
            let ident = match item {
                syn::Item::Struct(s) => Some(&s.ident),
                syn::Item::Union(u) => Some(&u.ident),
                syn::Item::Type(t) => Some(&t.ident),
                _ => None,
            };
            if let Some(id) = ident {
                if let Some(new) = map.get(&id.to_string()) {
                    let r = id.span().byte_range();
                    if !v.edits.iter().any(|(er, _)| er == &r) {
                        v.edits.push((r, new.clone()));
                    }
                }
            }
        }
        _ => unreachable!(),
    }
    v.edits
}

fn splice(src: &str, mut edits: Vec<(Range<usize>, String)>) -> String {
    edits.sort_by_key(|(r, _)| r.start);
    for pair in edits.windows(2) {
        assert!(pair[0].0.end <= pair[1].0.start, "overlapping edits");
    }
    let mut out = String::with_capacity(src.len());
    let mut pos = 0;
    for (r, text) in edits {
        out.push_str(&src[pos..r.start]);
        out.push_str(&text);
        pos = r.end;
    }
    out.push_str(&src[pos..]);
    out
}

/// Expand a byte range to whole lines (start of first line .. past the
/// newline of the last line).
fn line_expand(src: &str, r: &Range<usize>) -> Range<usize> {
    let start = src[..r.start].rfind('\n').map_or(0, |i| i + 1);
    let end = src[r.end..].find('\n').map_or(src.len(), |i| r.end + i + 1);
    start..end
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut repo = PathBuf::from(".");
    let mut domains_file: Option<PathBuf> = None;
    let mut only: Option<Vec<String>> = None;
    let mut skip: HashSet<String> = HashSet::new();
    let mut emit_tests: Option<PathBuf> = None;
    let mut apply = false;
    let mut verbose = false;
    let mut budget = 900usize;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--repo" => {
                repo = PathBuf::from(&args[i + 1]);
                i += 1;
            }
            "--domains" => {
                domains_file = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            "--only" => {
                only = Some(args[i + 1].split(',').map(str::to_string).collect());
                i += 1;
            }
            "--skip" => {
                skip.extend(args[i + 1].split(',').map(str::to_string));
                i += 1;
            }
            "--emit-tests" => {
                emit_tests = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            "--apply" => apply = true,
            "--verbose" => verbose = true,
            "--budget-lines" => {
                budget = args[i + 1].parse().unwrap();
                i += 1;
            }
            other => panic!("unknown arg: {other}"),
        }
        i += 1;
    }

    // ---- load & parse ----
    let files: Vec<FileData> = discover_files(&repo)
        .into_iter()
        .map(|(rel, path)| {
            let src = std::fs::read_to_string(&path).unwrap();
            let ast = syn::parse_file(&src).unwrap_or_else(|e| panic!("parse {rel}: {e}"));
            let modpath = modpath_of(&rel);
            FileData {
                rel,
                modpath,
                src,
                ast,
            }
        })
        .collect();
    eprintln!("parsed {} files", files.len());

    let mut defs = collect_defs(&files);
    eprintln!("collected {} type defs", defs.len());
    for d in &defs {
        if let Some(reason) = &d.dirty {
            eprintln!("  dirty: {}::{} — {}", files[d.file].rel, d.name, reason);
        }
    }

    // C forward declarations became per-module opaque `extern type`s while
    // modules with the full header got the real definition. In the original C
    // program a tag names one type globally, so an opaque `X` folds into the
    // concrete `X` — provided the concrete copies are structurally unambiguous
    // (exactly one equivalence class). Redirect every edge that lands on a
    // foldable opaque decl to a representative concrete def and re-refine, to
    // fixpoint: parents that only differed by opaqueness then merge too.
    //
    // If a fold ultimately causes a demotion — e.g. the concrete def lives in
    // one module and references that module's private singleton types, which
    // have no canonical home — the fold was a net loss: blacklist the name and
    // redo everything. A blacklisted name keeps the pre-fold behavior (its
    // opaque decls merge into a canonical `extern type`), which is exactly the
    // view those modules had all along.
    let pristine_edges: Vec<Vec<usize>> = defs.iter().map(|d| d.edges.clone()).collect();
    let mut fold_blacklist: HashSet<String> = HashSet::new();
    let (keys, mut groups, def_group, folded) = loop {
        for (d, e) in defs.iter_mut().zip(&pristine_edges) {
            d.edges = e.clone();
        }
        let mut folded: Vec<Option<usize>> = vec![None; defs.len()];
        let class = loop {
            let class = refine(&defs);
            struct NameInfo {
                opaques: Vec<usize>,
                concrete_classes: BTreeSet<u32>,
                rep: usize,
            }
            let mut by_name: HashMap<&str, NameInfo> = HashMap::new();
            for (di, d) in defs.iter().enumerate() {
                let e = by_name.entry(d.name.as_str()).or_insert(NameInfo {
                    opaques: vec![],
                    concrete_classes: BTreeSet::new(),
                    rep: usize::MAX,
                });
                if d.kind == Kind::Opaque {
                    if folded[di].is_none() {
                        e.opaques.push(di);
                    }
                } else {
                    e.concrete_classes.insert(class[di]);
                    e.rep = e.rep.min(di);
                }
            }
            let mut redirect: HashMap<usize, usize> = HashMap::new();
            for (name, info) in by_name {
                if info.opaques.is_empty()
                    || info.concrete_classes.is_empty()
                    || fold_blacklist.contains(name)
                {
                    continue;
                }
                if info.concrete_classes.len() > 1 {
                    eprintln!(
                        "  note: opaque `{name}` not folded: {} concrete classes",
                        info.concrete_classes.len()
                    );
                    continue;
                }
                for o in info.opaques {
                    redirect.insert(o, info.rep);
                    folded[o] = Some(info.rep);
                }
            }
            if redirect.is_empty() {
                break class;
            }
            eprintln!(
                "  folded {} opaque forward-decls; re-refining",
                redirect.len()
            );
            for d in defs.iter_mut() {
                for e in d.edges.iter_mut() {
                    if let Some(&r) = redirect.get(e) {
                        *e = r;
                    }
                }
            }
        };
        let mut keys = group_keys(&defs, &class);
        for di in 0..defs.len() {
            if let Some(rep) = folded[di] {
                keys[di] = keys[rep].clone();
            }
        }

        // ---- build groups ----
        let mut by_key: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (di, k) in keys.iter().enumerate() {
            by_key.entry(k.clone()).or_default().push(di);
        }

        // For each named type name, only the largest class merges.
        let mut largest_for_name: HashMap<String, (usize, String)> = HashMap::new();
        for (k, members) in &by_key {
            if let Some(rest) = k.strip_prefix("N:") {
                let name = rest.rsplit_once(':').unwrap().0;
                let entry = largest_for_name
                    .entry(name.to_string())
                    .or_insert((0, String::new()));
                if members.len() > entry.0 || (members.len() == entry.0 && k < &entry.1) {
                    *entry = (members.len(), k.clone());
                }
            }
        }

        let mut groups: Vec<Group> = Vec::new();
        let mut def_group: Vec<usize> = vec![usize::MAX; defs.len()];
        for (k, members) in &by_key {
            let gi = groups.len();
            for &m in members {
                def_group[m] = gi;
            }
            let d0 = &defs[members[0]];
            let mut merged = members.len() >= 2 && d0.dirty.is_none();
            if let Some(name) = k.strip_prefix("N:").map(|r| r.rsplit_once(':').unwrap().0) {
                if largest_for_name[name].1 != *k {
                    merged = false;
                }
                if skip.contains(name) {
                    merged = false;
                }
            }
            // distinct files sanity
            let mut seen_files = HashSet::new();
            if !members.iter().all(|&m| seen_files.insert(defs[m].file)) {
                eprintln!("  note: group {k} has two members in one module; not merging");
                merged = false;
            }
            groups.push(Group {
                key: k.clone(),
                members: members.clone(),
                canonical_name: if is_anon(&d0.name) {
                    String::new()
                } else {
                    d0.name.clone()
                },
                domain: String::new(),
                canonical_member: members[0],
                merged,
            });
        }

        // ---- closure: a merged group may only reference merged groups ----
        loop {
            let mut changed = false;
            for gi in 0..groups.len() {
                if !groups[gi].merged {
                    continue;
                }
                let bad = groups[gi]
                    .members
                    .iter()
                    .any(|&m| defs[m].edges.iter().any(|&e| !groups[def_group[e]].merged));
                if bad {
                    let blocker = groups[gi]
                        .members
                        .iter()
                        .flat_map(|&m| defs[m].edges.iter())
                        .find(|&&e| !groups[def_group[e]].merged)
                        .map(|&e| format!("{}::{}", files[defs[e].file].rel, defs[e].name))
                        .unwrap();
                    eprintln!(
                        "  demoted (references unmerged {blocker}): {}",
                        groups[gi].key
                    );
                    groups[gi].merged = false;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        // ---- canonical member: maximal derive set (must be a superset of all) ----
        for g in &mut groups {
            if !g.merged {
                continue;
            }
            let best = *g
                .members
                .iter()
                .max_by_key(|&&m| {
                    (
                        defs[m].kind != Kind::Opaque, // never pick a folded fwd-decl
                        defs[m].derives.len(),
                        std::cmp::Reverse(files[defs[m].file].rel.clone()),
                    )
                })
                .unwrap();
            for &m in &g.members {
                if !defs[m].derives.is_subset(&defs[best].derives) {
                    eprintln!(
                        "  demoted (incompatible derives {:?} vs {:?}): {}",
                        defs[m].derives, defs[best].derives, g.key
                    );
                    g.merged = false;
                }
            }
            g.canonical_member = best;
        }
        // re-run closure after any derive demotions
        loop {
            let mut changed = false;
            for gi in 0..groups.len() {
                if groups[gi].merged
                    && groups[gi]
                        .members
                        .iter()
                        .any(|&m| defs[m].edges.iter().any(|&e| !groups[def_group[e]].merged))
                {
                    eprintln!(
                        "  demoted (closure after derive demotion): {}",
                        groups[gi].key
                    );
                    groups[gi].merged = false;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        // ---- retry with a smaller fold set if a fold got demoted ----
        let newly: BTreeSet<String> = groups
            .iter()
            .filter(|g| !g.merged && g.members.iter().any(|&m| folded[m].is_some()))
            .map(|g| defs[g.members[0]].name.clone())
            .filter(|n| !fold_blacklist.contains(n))
            .collect();
        if newly.is_empty() {
            break (keys, groups, def_group, folded);
        }
        eprintln!("  fold retry: blacklisting {newly:?}");
        fold_blacklist.extend(newly);
    };
    let _ = &keys;

    // ---- restrict to --only closure ----
    if let Some(names) = &only {
        let roots: Vec<usize> = groups
            .iter()
            .enumerate()
            .filter(|(_, g)| {
                g.merged
                    && g.key
                        .strip_prefix("N:")
                        .is_some_and(|r| names.contains(&r.rsplit_once(':').unwrap().0.to_string()))
            })
            .map(|(gi, _)| gi)
            .collect();
        let mut keep: HashSet<usize> = HashSet::new();
        let mut stack = roots;
        while let Some(gi) = stack.pop() {
            if !keep.insert(gi) {
                continue;
            }
            for &m in &groups[gi].members {
                for &e in &defs[m].edges {
                    stack.push(def_group[e]);
                }
            }
        }
        for (gi, g) in groups.iter_mut().enumerate() {
            if g.merged && !keep.contains(&gi) {
                g.merged = false;
            }
        }
    }

    // ---- drop merged anon groups not referenced by any merged group ----
    loop {
        let mut referenced: HashSet<usize> = HashSet::new();
        for g in &groups {
            if g.merged {
                for &m in &g.members {
                    for &e in &defs[m].edges {
                        referenced.insert(def_group[e]);
                    }
                }
            }
        }
        let mut changed = false;
        for (gi, g) in groups.iter_mut().enumerate() {
            if g.merged && g.key.starts_with("A:") && !referenced.contains(&gi) {
                g.merged = false;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    // ---- anonymous naming ----
    // parent group of an anon group: from its key "A:class:<parentkey>:label"
    let key_to_gi: HashMap<String, usize> = groups
        .iter()
        .enumerate()
        .map(|(gi, g)| (g.key.clone(), gi))
        .collect();
    fn canon_name(
        gi: usize,
        groups: &mut Vec<Group>,
        key_to_gi: &HashMap<String, usize>,
    ) -> String {
        if !groups[gi].canonical_name.is_empty() {
            return groups[gi].canonical_name.clone();
        }
        let key = groups[gi].key.clone();
        // A:{class}:{parentkey}:{label} — parentkey itself contains ':'s; label
        // is the final segment.
        let rest = key.strip_prefix("A:").unwrap();
        let (_cls, rest) = rest.split_once(':').unwrap();
        let (parent_key, label) = rest.rsplit_once(':').unwrap();
        let pgi = key_to_gi[parent_key];
        let pname = canon_name(pgi, groups, key_to_gi);
        let name = if label == "=" {
            format!("{pname}_ty")
        } else {
            format!("{pname}_{label}")
        };
        groups[gi].canonical_name = name.clone();
        name
    }
    let merged_gis: Vec<usize> = groups
        .iter()
        .enumerate()
        .filter(|(_, g)| g.merged)
        .map(|(gi, _)| gi)
        .collect();
    for &gi in &merged_gis {
        canon_name(gi, &mut groups, &key_to_gi);
    }
    // uniquify canonical names among merged groups
    {
        let mut used: HashMap<String, usize> = HashMap::new();
        for &gi in &merged_gis {
            let name = groups[gi].canonical_name.clone();
            let n = used.entry(name.clone()).or_insert(0);
            *n += 1;
            if *n > 1 {
                let new = format!("{}_{}", name, n);
                eprintln!("  renamed for uniqueness: {} -> {}", name, new);
                groups[gi].canonical_name = new;
            }
        }
    }

    // ---- domains ----
    let domain_map: HashMap<String, String> = domains_file
        .map(|p| {
            std::fs::read_to_string(p)
                .unwrap()
                .lines()
                .filter_map(|l| {
                    let (name, domain) = l.split_once('\t')?;
                    Some((name.to_string(), domain.to_string()))
                })
                .collect()
        })
        .unwrap_or_default();
    fn domain_of(
        gi: usize,
        groups: &Vec<Group>,
        defs: &[Def],
        key_to_gi: &HashMap<String, usize>,
        map: &HashMap<String, String>,
        depth: usize,
    ) -> String {
        let g = &groups[gi];
        let d0 = &defs[g.members[0]];
        if !is_anon(&d0.name) {
            if let Some(dom) = map.get(&d0.name) {
                return dom.clone();
            }
            let n = &d0.name;
            if n.starts_with("uv_")
                || n.starts_with("uv__")
                || n.starts_with("sockaddr")
                || n.starts_with("addrinfo")
                || n == "queue"
            {
                return "uv".into();
            }
            if n.starts_with('_')
                || n.starts_with("__")
                || n.contains("va_list")
                || n == "FILE"
                || n == "fd_set"
                || n.starts_with("sig")
                || n.starts_with("pthread")
                || n.starts_with("timespec")
            {
                return "libc".into();
            }
            if depth < 8 {
                // alias: inherit from its target
                if d0.kind == Kind::Alias && d0.edges.len() == 1 {
                    // find target's group via key: recompute directly
                    return "misc".into();
                }
            }
            return "misc".into();
        }
        // anon: inherit parent's domain
        let rest = g.key.strip_prefix("A:").unwrap();
        let (_cls, rest) = rest.split_once(':').unwrap();
        let (parent_key, _label) = rest.rsplit_once(':').unwrap();
        if depth < 32 {
            if let Some(&pgi) = key_to_gi.get(parent_key) {
                return domain_of(pgi, groups, defs, key_to_gi, map, depth + 1);
            }
        }
        "misc".into()
    }
    for gi in 0..groups.len() {
        if groups[gi].merged {
            groups[gi].domain = domain_of(gi, &groups, &defs, &key_to_gi, &domain_map, 0);
        }
    }
    // Domain names become module names under types/. They must be valid,
    // non-keyword identifiers (upstream headers like termkey-internal.h
    // contain hyphens), and must not collide with any canonical type name:
    // `pub mod object;` would shadow the canonical `struct object` in every
    // `pub use crate::src::nvim::types::object` re-export (explicit mod beats
    // glob re-export in the type namespace).
    {
        let canonical_names: HashSet<String> = groups
            .iter()
            .filter(|g| g.merged)
            .map(|g| g.canonical_name.clone())
            .collect();
        const KEYWORDS: &[&str] = &[
            "as", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "false",
            "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
            "ref", "return", "self", "static", "struct", "super", "trait", "true", "type",
            "unsafe", "use", "where", "while", "async", "await", "try", "macro", "union",
        ];
        for g in &mut groups {
            if !g.merged {
                continue;
            }
            let mut dom: String = g
                .domain
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                .collect();
            if dom.chars().next().is_none_or(|c| c.is_ascii_digit()) {
                dom.insert(0, 'd');
            }
            while canonical_names.contains(&dom) || KEYWORDS.contains(&dom.as_str()) {
                dom.push_str("_defs");
            }
            g.domain = dom;
        }
    }

    // ---- summary ----
    let merged: Vec<usize> = groups
        .iter()
        .enumerate()
        .filter(|(_, g)| g.merged)
        .map(|(gi, _)| gi)
        .collect();
    let member_count: usize = merged.iter().map(|&gi| groups[gi].members.len()).sum();
    let dup_named = groups
        .iter()
        .filter(|g| !g.merged && g.members.len() >= 2 && g.key.starts_with("N:"))
        .count();
    eprintln!(
        "merged groups: {} covering {} defs ({} deleted); unmerged multi-copy named groups: {}",
        merged.len(),
        member_count,
        member_count - merged.len(),
        dup_named
    );
    if verbose {
        for &gi in &merged {
            let g = &groups[gi];
            eprintln!(
                "  {} [{} copies] -> types/{}.rs  ({})",
                g.canonical_name,
                g.members.len(),
                g.domain,
                g.key
            );
        }
        for g in groups.iter().filter(|g| !g.merged && g.members.len() >= 2) {
            eprintln!("  UNMERGED {} [{} copies]", g.key, g.members.len());
        }
    }

    // ---- emit layout-parity tests ----
    if let Some(test_path) = &emit_tests {
        let mut out = String::new();
        out.push_str(
            "// GENERATED by tools/unify (phase 5a) — temporary layout-parity proof.\n\
             // Asserts size/align/offset-of-every-field of every per-module copy of\n\
             // every merged type against the canonical copy. Run before and after the\n\
             // merge; not committed.\n\
             #![allow(non_snake_case)]\n\
             use core::mem::{size_of, align_of, MaybeUninit};\n\n\
             macro_rules! layout {\n\
                 ($t:path { $($f:ident),* $(,)? }) => {{\n\
                     let u = MaybeUninit::<$t>::uninit();\n\
                     let b = u.as_ptr();\n\
                     let mut v: Vec<usize> = vec![size_of::<$t>(), align_of::<$t>()];\n\
                     $( v.push(unsafe { (core::ptr::addr_of!((*b).$f) as *const u8 as usize) - (b as usize) }); )*\n\
                     v\n\
                 }};\n\
             }\n\n",
        );
        // Unsized detection for aliases in the PRE-merge tree (via pristine,
        // un-redirected edges): an alias to a module-local opaque fwd-decl is
        // unsized today even when the fold makes it sized post-merge, so it
        // cannot be measured before the merge.
        fn unsized_def(di: usize, defs: &[Def], pristine: &[Vec<usize>], depth: usize) -> bool {
            let d = &defs[di];
            match d.kind {
                Kind::Opaque => true,
                Kind::Alias if depth < 16 => {
                    // unsized only if the alias RHS is exactly one bare edge
                    d.skeleton.ends_with('\x00')
                        && pristine[di].len() == 1
                        && unsized_def(pristine[di][0], defs, pristine, depth + 1)
                }
                _ => false,
            }
        }
        let mut ntests = 0;
        for &gi in &merged {
            let g = &groups[gi];
            let d0 = &defs[g.canonical_member];
            let tname = &g.canonical_name;
            match d0.kind {
                Kind::Opaque => continue,
                Kind::Alias => {
                    let measurable: Vec<usize> = g
                        .members
                        .iter()
                        .copied()
                        .filter(|&m| {
                            defs[m].kind != Kind::Opaque
                                && !unsized_def(m, &defs, &pristine_edges, 0)
                        })
                        .collect();
                    let Some(&c) = measurable.first() else { continue };
                    let dc = &defs[c];
                    let _ = write!(out, "#[test]\nfn parity_{tname}() {{\n");
                    let canon_path =
                        format!("::c2rust_neovim::{}::{}", files[dc.file].modpath, dc.name);
                    let _ = write!(
                        out,
                        "    let canon = (size_of::<{canon_path}>(), align_of::<{canon_path}>());\n"
                    );
                    for &m in &measurable {
                        let d = &defs[m];
                        let p = format!("::c2rust_neovim::{}::{}", files[d.file].modpath, d.name);
                        let _ = write!(
                            out,
                            "    assert_eq!(canon, (size_of::<{p}>(), align_of::<{p}>()), {:?});\n",
                            files[d.file].rel
                        );
                    }
                    out.push_str("}\n\n");
                    ntests += 1;
                }
                Kind::Struct | Kind::Union => {
                    let fields = d0.fields.join(", ");
                    let _ = write!(
                        out,
                        "macro_rules! m_{tname} {{ ($t:path) => {{ layout!($t {{ {fields} }}) }} }}\n"
                    );
                    let _ = write!(out, "#[test]\nfn parity_{tname}() {{\n");
                    let canon_path =
                        format!("::c2rust_neovim::{}::{}", files[d0.file].modpath, d0.name);
                    let _ = write!(out, "    let canon = m_{tname}!({canon_path});\n");
                    for &m in &g.members {
                        let d = &defs[m];
                        if d.kind == Kind::Opaque {
                            continue; // folded fwd-decl: nothing to measure
                        }
                        let p = format!("::c2rust_neovim::{}::{}", files[d.file].modpath, d.name);
                        let _ = write!(
                            out,
                            "    assert_eq!(canon, m_{tname}!({p}), {:?});\n",
                            files[d.file].rel
                        );
                    }
                    out.push_str("}\n\n");
                    ntests += 1;
                }
            }
        }
        std::fs::write(test_path, &out).unwrap();
        eprintln!("wrote {} tests to {}", ntests, test_path.display());
    }

    if !apply {
        return;
    }

    // ---- emit src/nvim/types/ ----
    // Per-file rename maps (local anon/outdated name -> canonical name), for
    // splicing canonical texts.
    let mut rename_maps: Vec<HashMap<String, String>> = vec![HashMap::new(); files.len()];
    for &gi in &merged {
        let g = &groups[gi];
        for &m in &g.members {
            let d = &defs[m];
            if d.name != g.canonical_name {
                rename_maps[d.file].insert(d.name.clone(), g.canonical_name.clone());
            }
        }
    }

    // Pack domains into files under the line budget.
    let mut domain_items: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for &gi in &merged {
        domain_items
            .entry(groups[gi].domain.clone())
            .or_default()
            .push(gi);
    }
    let types_dir = repo.join("src/nvim/types");
    std::fs::create_dir_all(&types_dir).unwrap();
    let mut mod_names: Vec<String> = Vec::new();
    for (domain, gis) in &domain_items {
        let mut gis = gis.clone();
        gis.sort_by_key(|&gi| groups[gi].canonical_name.clone());
        // canonical texts
        let texts: Vec<(usize, String, bool)> = gis
            .iter()
            .map(|&gi| {
                let g = &groups[gi];
                let d = &defs[g.canonical_member];
                let file = &files[d.file];
                let item = &file.ast.items[d.item_idx];
                let edits: Vec<(Range<usize>, String)> =
                    rename_edits(item, d.sub_idx, &rename_maps[d.file])
                        .into_iter()
                        .filter(|(r, _)| r.start >= d.range.start && r.end <= d.range.end)
                        .map(|(r, t)| (r.start - d.range.start..r.end - d.range.start, t))
                        .collect();
                let text = splice(&file.src[d.range.clone()], edits);
                (gi, text, d.kind == Kind::Opaque)
            })
            .collect();
        let mut chunk_idx = 0usize;
        let mut chunk_opaque: Vec<String> = Vec::new();
        let mut chunk_items: Vec<String> = Vec::new();
        let mut chunk_lines = 0usize;
        let mut flush = |idx: &mut usize,
                         opaque: &mut Vec<String>,
                         items: &mut Vec<String>,
                         mod_names: &mut Vec<String>| {
            if opaque.is_empty() && items.is_empty() {
                return;
            }
            let name = if *idx == 0 {
                domain.clone()
            } else {
                format!("{}_{}", domain, *idx + 1)
            };
            let mut body = String::new();
            body.push_str("// Canonical type definitions extracted by tools/unify (phase 5a).\n");
            body.push_str(
                "// One definition per logical type; every module re-exports from here.\n",
            );
            body.push_str("use super::*;\n\n");
            if !opaque.is_empty() {
                body.push_str("extern \"C\" {\n");
                for o in opaque.iter() {
                    let _ = writeln!(body, "    {o}");
                }
                body.push_str("}\n\n");
            }
            for it in items.iter() {
                body.push_str(it);
                body.push_str("\n");
            }
            std::fs::write(types_dir.join(format!("{name}.rs")), &body).unwrap();
            mod_names.push(name);
            *idx += 1;
            opaque.clear();
            items.clear();
        };
        for (gi, text, opaque) in texts {
            let g = &groups[gi];
            let lines = text.lines().count() + 1;
            if chunk_lines + lines > budget && (!chunk_items.is_empty() || !chunk_opaque.is_empty())
            {
                flush(
                    &mut chunk_idx,
                    &mut chunk_opaque,
                    &mut chunk_items,
                    &mut mod_names,
                );
                chunk_lines = 0;
            }
            if opaque {
                chunk_opaque.push(format!("pub type {};", g.canonical_name));
            } else {
                chunk_items.push(text);
            }
            chunk_lines += lines;
        }
        flush(
            &mut chunk_idx,
            &mut chunk_opaque,
            &mut chunk_items,
            &mut mod_names,
        );
    }
    // mod.rs
    {
        let mut body = String::new();
        body.push_str(
            "// Canonical homes for the types c2rust duplicated into every module\n\
             // (phase 5a; generated by tools/unify, then maintained by hand).\n\
             // Every name is unique crate-wide and glob-re-exported so sibling\n\
             // files can `use super::*`.\n",
        );
        mod_names.sort();
        for m in &mod_names {
            let _ = writeln!(body, "pub mod {m};");
        }
        body.push('\n');
        for m in &mod_names {
            let _ = writeln!(body, "pub use self::{m}::*;");
        }
        std::fs::write(types_dir.join("mod.rs"), &body).unwrap();
        eprintln!("wrote src/nvim/types/ ({} domain files)", mod_names.len());
    }

    // ---- rewrite modules ----
    for (fi, file) in files.iter().enumerate() {
        let mut deletions: Vec<Range<usize>> = Vec::new();
        let mut reexports: Vec<String> = Vec::new();
        for &gi in &merged {
            let g = &groups[gi];
            for &m in &g.members {
                let d = &defs[m];
                if d.file != fi {
                    continue;
                }
                deletions.push(line_expand(&file.src, &d.range));
                if d.name == g.canonical_name {
                    reexports.push(g.canonical_name.clone());
                } else {
                    reexports.push(format!("{} as {}", g.canonical_name, d.name));
                }
            }
        }
        if deletions.is_empty() {
            continue;
        }
        reexports.sort();
        let mut edits: Vec<(Range<usize>, String)> =
            deletions.into_iter().map(|r| (r, String::new())).collect();
        // Insert before the first top-level item, after any `//!` module docs
        // and inner attributes (which must precede all items).
        let insert_at = file
            .ast
            .items
            .first()
            .map(|item| line_expand(&file.src, &span_range(item)).start)
            .unwrap_or(file.src.len());
        edits.push((
            insert_at..insert_at,
            format!(
                "pub use crate::src::nvim::types::{{{}}};\n",
                reexports.join(", ")
            ),
        ));
        let new_src = splice(&file.src, edits);
        std::fs::write(repo.join(&file.rel), new_src).unwrap();
    }
    eprintln!("rewrote modules; remember to add `pub mod types;` to lib.rs and run `just fmt`");
}
