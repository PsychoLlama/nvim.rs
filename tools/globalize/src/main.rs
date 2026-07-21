//! Mechanical `static mut` -> `GlobalCell` rewriter (migration phase 3a).
//!
//! For a batch of "target" modules, converts every top-level and
//! function-local `static mut` they define into `static NAME:
//! GlobalCell<T>`, then fixes up access sites crate-wide:
//!
//! - cross-module `extern "C" { static mut NAME: T; }` redeclarations of a
//!   converted exported global are retyped in place to `static NAME:
//!   GlobalCell<T>;` — same symbol, same layout (the cell is
//!   repr(transparent)), and the module keeps viewing the global at its own
//!   c2rust-duplicated copy of the type;
//! - `NAME = rhs` (whole-static assignment) becomes `NAME.set(rhs)`;
//! - `NAME` in a plain value position becomes `NAME.get()`;
//! - `&raw mut NAME` / `&raw const NAME` become `NAME.ptr()` /
//!   `(NAME.ptr() as *const _)` (`as_raw` in const positions);
//! - every other place expression (field/index projection, `&`/`&mut`,
//!   method receiver, compound assignment) routes through the raw-pointer
//!   escape hatch: `(*NAME.ptr())`.
//!
//! All rewrites preserve the transpiled evaluation order and copy
//! semantics; anything the tool cannot prove it understands (name shadowed
//! by a local binding, uses inside macro tokens, qualified paths) is left
//! untouched and reported — the crate's type checker then flags any such
//! site that actually needed conversion, so nothing is missed silently.
//!
//! Edits are applied textually via token byte offsets, so untouched code
//! keeps its exact formatting (rustfmt normalizes the rest).

use proc_macro2::TokenTree;
use quote::ToTokens;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::ops::Range;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};

/// A textual edit: replace `range` with `text` (insertion when empty range).
#[derive(Debug)]
struct Edit {
    range: Range<usize>,
    text: String,
}

/// Byte range of a node computed from its first and last token, avoiding
/// `Span::join` (whose fallback silently degrades to the first token).
fn span_range(node: &dyn ToTokens) -> Range<usize> {
    let mut start = usize::MAX;
    let mut end = 0;
    let mut stack: Vec<TokenTree> = node.to_token_stream().into_iter().collect();
    if stack.is_empty() {
        panic!("empty token stream for spanned node");
    }
    for tt in &stack {
        let r = tt.span().byte_range();
        start = start.min(r.start);
        end = end.max(r.end);
    }
    // Group spans already include their delimiters; no need to recurse.
    stack.clear();
    start..end
}

fn is_bare_path(expr: &syn::Expr) -> Option<String> {
    if let syn::Expr::Path(p) = expr {
        if p.qself.is_none()
            && p.path.leading_colon.is_none()
            && p.path.segments.len() == 1
            && p.path.segments[0].arguments.is_none()
        {
            return Some(p.path.segments[0].ident.to_string());
        }
    }
    None
}

fn is_assign_op(op: &syn::BinOp) -> bool {
    use syn::BinOp::*;
    matches!(
        op,
        AddAssign(_)
            | SubAssign(_)
            | MulAssign(_)
            | DivAssign(_)
            | RemAssign(_)
            | BitXorAssign(_)
            | BitAndAssign(_)
            | BitOrAssign(_)
            | ShlAssign(_)
            | ShrAssign(_)
    )
}

fn has_no_mangle(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| a.path().is_ident("no_mangle"))
}

// ---------------------------------------------------------------------------
// Module map: file path -> crate module path, from lib.rs's inline mod tree.
// ---------------------------------------------------------------------------

fn build_mod_map(repo: &Path) -> BTreeMap<PathBuf, String> {
    let lib_src = std::fs::read_to_string(repo.join("lib.rs")).expect("read lib.rs");
    let file = syn::parse_file(&lib_src).expect("parse lib.rs");
    let mut map = BTreeMap::new();
    fn walk(
        items: &[syn::Item],
        dir: PathBuf,
        mods: &mut Vec<String>,
        map: &mut BTreeMap<PathBuf, String>,
    ) {
        for item in items {
            let syn::Item::Mod(m) = item else { continue };
            let name = m.ident.to_string();
            // Raw identifiers (`mod r#loop`) name the file without the prefix.
            let fname = name.strip_prefix("r#").unwrap_or(&name).to_string();
            match &m.content {
                Some((_, children)) => {
                    mods.push(name.clone());
                    walk(children, dir.join(&fname), mods, map);
                    mods.pop();
                }
                None => {
                    let path_attr =
                        m.attrs
                            .iter()
                            .find(|a| a.path().is_ident("path"))
                            .map(|a| match &a.meta {
                                syn::Meta::NameValue(nv) => match &nv.value {
                                    syn::Expr::Lit(l) => match &l.lit {
                                        syn::Lit::Str(s) => s.value(),
                                        _ => panic!("non-str path attr"),
                                    },
                                    _ => panic!("non-lit path attr"),
                                },
                                _ => panic!("unexpected path attr shape"),
                            });
                    let file = dir.join(path_attr.unwrap_or(format!("{fname}.rs")));
                    let modpath = format!("crate::{}::{}", mods.join("::"), name);
                    map.insert(file, modpath);
                }
            }
        }
    }
    walk(&file.items, PathBuf::new(), &mut Vec::new(), &mut map);
    map
}

// ---------------------------------------------------------------------------
// Pass 1: collect every top-level static mut definition.
// ---------------------------------------------------------------------------

struct FileData {
    rel: PathBuf,
    src: String,
    ast: syn::File,
    modpath: String,
}

// ---------------------------------------------------------------------------
// Prescan of a fn: locally-bound identifiers and fn-local static muts.
// ---------------------------------------------------------------------------

#[derive(Default)]
struct Prescan {
    bound: HashSet<String>,
    local_statics: HashSet<String>,
}

impl<'ast> Visit<'ast> for Prescan {
    fn visit_pat_ident(&mut self, p: &'ast syn::PatIdent) {
        self.bound.insert(p.ident.to_string());
        visit::visit_pat_ident(self, p);
    }
    fn visit_item_static(&mut self, s: &'ast syn::ItemStatic) {
        if matches!(s.mutability, syn::StaticMutability::Mut(_)) {
            self.local_statics.insert(s.ident.to_string());
        }
        visit::visit_item_static(self, s);
    }
}

// ---------------------------------------------------------------------------
// Pass 2: the rewriter proper.
// ---------------------------------------------------------------------------

/// Extend a removal range over one trailing space so deleting a token does
/// not leave doubled whitespace behind.
fn pad_right(r: Range<usize>, src: &str) -> Range<usize> {
    if src.as_bytes().get(r.end) == Some(&b' ') {
        r.start..r.end + 1
    } else {
        r
    }
}

struct Rewriter<'a> {
    rel: &'a Path,
    src: &'a str,
    /// Converted names visible at file scope (this file's converted defs +
    /// extern redeclarations of converted exports).
    file_active: HashSet<String>,
    /// Converted exported name -> defining module path (crate-wide).
    exports: &'a HashMap<String, String>,
    /// This file defines targets (drives def rewriting + GlobalCell import).
    is_target: bool,
    shadowed: HashSet<String>,
    fn_extras: HashSet<String>,
    const_ctx: u32,
    edits: Vec<Edit>,
    warns: Vec<String>,
    converted_defs: usize,
    retyped_decls: usize,
}

impl<'a> Rewriter<'a> {
    fn active(&self, name: &str) -> bool {
        (self.file_active.contains(name) || self.fn_extras.contains(name))
            && !self.shadowed.contains(name)
    }

    fn warn(&mut self, span: proc_macro2::Span, msg: String) {
        let lc = span.start();
        self.warns.push(format!(
            "{}:{}:{}: {}",
            self.rel.display(),
            lc.line,
            lc.column + 1,
            msg
        ));
    }

    fn edit(&mut self, range: Range<usize>, text: impl Into<String>) {
        self.edits.push(Edit {
            range,
            text: text.into(),
        });
    }

    /// `(*NAME.ptr())` place form (or as_raw in const positions, reported).
    fn place_edit(&mut self, expr: &syn::Expr, name: &str) {
        let r = span_range(expr);
        if self.const_ctx > 0 {
            self.warn(
                expr.span(),
                format!("place projection of {name} in const context — verify manually"),
            );
            self.edit(r, format!("(*{name}.as_raw())"));
        } else {
            self.edit(r, format!("(*{name}.ptr())"));
        }
    }

    fn get_edit(&mut self, expr: &syn::Expr, name: &str) {
        if self.const_ctx > 0 {
            // A value read of a static in const context was already illegal
            // before conversion; this indicates a tool bug.
            self.warn(
                expr.span(),
                format!("value read of {name} in const context"),
            );
            return;
        }
        self.edit(span_range(expr), format!("{name}.get()"));
    }

    /// Handle a child expression that may be a bare converted global used as
    /// a place. Returns true if it consumed the child.
    fn child_place(&mut self, child: &syn::Expr) -> bool {
        if let Some(name) = is_bare_path(child) {
            if self.active(&name) {
                self.place_edit(child, &name);
                return true;
            }
        }
        false
    }

    fn scan_macro_tokens(&mut self, tokens: &proc_macro2::TokenStream, span: proc_macro2::Span) {
        let mut stack: Vec<TokenTree> = tokens.clone().into_iter().collect();
        while let Some(tt) = stack.pop() {
            match tt {
                TokenTree::Ident(id) => {
                    let name = id.to_string();
                    if self.active(&name) {
                        self.warn(
                            span,
                            format!("converted global `{name}` referenced inside macro tokens — fix manually"),
                        );
                    }
                }
                TokenTree::Group(g) => stack.extend(g.stream()),
                _ => {}
            }
        }
    }

    fn convert_static_def(&mut self, s: &syn::ItemStatic) {
        let name = s.ident.to_string();
        if self.shadowed.contains(&name) {
            self.warn(
                s.ident.span(),
                format!(
                    "static mut `{name}` shares a name with a local binding — left unconverted"
                ),
            );
            return;
        }
        let syn::StaticMutability::Mut(m) = &s.mutability else { return };
        // `static mut N: T = init;` -> `static N: GlobalCell<T> = GlobalCell::new(init);`
        // Consume the following space too: rustfmt refuses to reformat items
        // whose giant literals exceed max_width, so residue would stick.
        self.edit(pad_right(m.span.byte_range(), self.src), "");
        let ty = span_range(&s.ty);
        self.edit(ty.start..ty.start, "GlobalCell<");
        self.edit(ty.end..ty.end, ">");
        let init = span_range(&s.expr);
        self.edit(init.start..init.start, "GlobalCell::new(");
        self.edit(init.end..init.end, ")");
        self.converted_defs += 1;
    }
}

impl<'a, 'ast> Visit<'ast> for Rewriter<'a> {
    fn visit_item_static(&mut self, s: &'ast syn::ItemStatic) {
        if self.is_target && matches!(s.mutability, syn::StaticMutability::Mut(_)) {
            self.convert_static_def(s);
        }
        self.const_ctx += 1;
        self.visit_expr(&s.expr);
        self.const_ctx -= 1;
    }

    fn visit_item_const(&mut self, c: &'ast syn::ItemConst) {
        self.const_ctx += 1;
        self.visit_expr(&c.expr);
        self.const_ctx -= 1;
    }

    // Extern-block redeclarations of converted globals are retyped in place:
    // `static mut NAME: T;` -> `static NAME: GlobalCell<T>;`. Same symbol,
    // transparent layout — and crucially the module keeps viewing the global
    // at its own duplicate of the struct type (c2rust re-emits type
    // definitions per module, so a `use` of the defining module's item would
    // change the nominal type at every access site).
    fn visit_item_foreign_mod(&mut self, fm: &'ast syn::ItemForeignMod) {
        for fi in &fm.items {
            if let syn::ForeignItem::Static(fs) = fi {
                let syn::StaticMutability::Mut(m) = &fs.mutability else {
                    continue;
                };
                let name = fs.ident.to_string();
                if self.exports.contains_key(&name) {
                    self.edit(pad_right(m.span.byte_range(), self.src), "");
                    let ty = span_range(&fs.ty);
                    self.edit(ty.start..ty.start, "GlobalCell<");
                    self.edit(ty.end..ty.end, ">");
                    self.file_active.insert(name);
                    self.retyped_decls += 1;
                }
            }
        }
    }

    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        let mut pre = Prescan::default();
        pre.visit_block(&f.block);
        let saved_shadow = self.shadowed.clone();
        let saved_extras = self.fn_extras.clone();
        for name in &pre.bound {
            if self.file_active.contains(name) || pre.local_statics.contains(name) {
                self.warn(
                    f.sig.ident.span(),
                    format!(
                        "fn {}: local binding shadows converted global `{name}` — accesses left as-is",
                        f.sig.ident
                    ),
                );
                self.shadowed.insert(name.clone());
            }
        }
        if self.is_target {
            for name in &pre.local_statics {
                if !self.shadowed.contains(name) {
                    self.fn_extras.insert(name.clone());
                }
            }
        }
        self.visit_block(&f.block);
        self.shadowed = saved_shadow;
        self.fn_extras = saved_extras;
    }

    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        let mut pre = Prescan::default();
        pre.visit_block(&f.block);
        let saved_shadow = self.shadowed.clone();
        let saved_extras = self.fn_extras.clone();
        for name in &pre.bound {
            if self.file_active.contains(name) || pre.local_statics.contains(name) {
                self.shadowed.insert(name.clone());
            }
        }
        if self.is_target {
            for name in &pre.local_statics {
                if !self.shadowed.contains(name) {
                    self.fn_extras.insert(name.clone());
                }
            }
        }
        self.visit_block(&f.block);
        self.shadowed = saved_shadow;
        self.fn_extras = saved_extras;
    }

    fn visit_expr(&mut self, e: &'ast syn::Expr) {
        match e {
            // NAME = rhs  ->  NAME.set(rhs)
            syn::Expr::Assign(a) => {
                if let Some(name) = is_bare_path(&a.left) {
                    if self.active(&name) {
                        let left = span_range(&a.left);
                        let right = span_range(&a.right);
                        self.edit(left.start..right.start, format!("{name}.set("));
                        self.edit(right.end..right.end, ")");
                        self.visit_expr(&a.right);
                        return;
                    }
                }
                self.visit_expr(&a.left);
                self.visit_expr(&a.right);
            }
            // NAME op= rhs  ->  (*NAME.ptr()) op= rhs
            syn::Expr::Binary(b) if is_assign_op(&b.op) => {
                if !self.child_place(&b.left) {
                    self.visit_expr(&b.left);
                }
                self.visit_expr(&b.right);
            }
            // &NAME / &mut NAME  ->  &(*NAME.ptr()) / &mut (*NAME.ptr())
            syn::Expr::Reference(r) => {
                if !self.child_place(&r.expr) {
                    self.visit_expr(&r.expr);
                }
            }
            // &raw mut NAME -> NAME.ptr();  &raw const NAME -> (NAME.ptr() as *const _)
            syn::Expr::RawAddr(r) => {
                if let Some(name) = is_bare_path(&r.expr) {
                    if self.active(&name) {
                        let accessor = if self.const_ctx > 0 { "as_raw" } else { "ptr" };
                        let text = match r.mutability {
                            syn::PointerMutability::Mut(_) => format!("{name}.{accessor}()"),
                            syn::PointerMutability::Const(_) => {
                                format!("({name}.{accessor}() as *const _)")
                            }
                        };
                        self.edit(span_range(e), text);
                        return;
                    }
                }
                self.visit_expr(&r.expr);
            }
            syn::Expr::Field(f) => {
                if !self.child_place(&f.base) {
                    self.visit_expr(&f.base);
                }
            }
            syn::Expr::Index(i) => {
                if !self.child_place(&i.expr) {
                    self.visit_expr(&i.expr);
                }
                self.visit_expr(&i.index);
            }
            syn::Expr::MethodCall(m) => {
                if !self.child_place(&m.receiver) {
                    self.visit_expr(&m.receiver);
                }
                for arg in &m.args {
                    self.visit_expr(arg);
                }
            }
            syn::Expr::Struct(s) => {
                for fv in &s.fields {
                    if fv.colon_token.is_none() {
                        // Shorthand `S { name }`: expand before rewriting.
                        if let Some(name) = is_bare_path(&fv.expr) {
                            if self.active(&name) {
                                self.edit(span_range(&fv.expr), format!("{name}: {name}.get()"));
                                continue;
                            }
                        }
                    }
                    self.visit_expr(&fv.expr);
                }
                if let Some(rest) = &s.rest {
                    self.visit_expr(rest);
                }
            }
            syn::Expr::Macro(m) => {
                self.scan_macro_tokens(&m.mac.tokens, e.span());
            }
            syn::Expr::Path(p) => {
                if let Some(name) = is_bare_path(e) {
                    if self.active(&name) {
                        self.get_edit(e, &name);
                        return;
                    }
                }
                if p.path.segments.len() > 1 {
                    let last = p.path.segments.last().unwrap().ident.to_string();
                    if self.active(&last) {
                        self.warn(
                            e.span(),
                            format!(
                                "qualified path ends in converted global `{last}` — check manually"
                            ),
                        );
                    }
                }
            }
            _ => visit::visit_expr(self, e),
        }
    }

    fn visit_stmt_macro(&mut self, m: &'ast syn::StmtMacro) {
        self.scan_macro_tokens(&m.mac.tokens, m.mac.path.span());
    }

    fn visit_item_macro(&mut self, m: &'ast syn::ItemMacro) {
        self.scan_macro_tokens(&m.mac.tokens, m.mac.path.span());
    }
}

// ---------------------------------------------------------------------------

fn apply_edits(src: &str, mut edits: Vec<Edit>) -> String {
    // Descending by start; at equal starts, replacements before insertions so
    // an insertion at the same offset ends up textually first.
    edits.sort_by(|a, b| {
        b.range
            .start
            .cmp(&a.range.start)
            .then(b.range.end.cmp(&a.range.end))
    });
    for pair in edits.windows(2) {
        let (later, earlier) = (&pair[0], &pair[1]);
        assert!(
            earlier.range.end <= later.range.start || earlier.range == later.range,
            "overlapping edits: {earlier:?} vs {later:?}"
        );
    }
    let mut out = src.to_string();
    for e in &edits {
        out.replace_range(e.range.clone(), &e.text);
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut repo = PathBuf::from(".");
    let mut targets: Vec<String> = Vec::new();
    let mut all = false;
    let mut list = false;
    let mut dry_run = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--repo" => {
                i += 1;
                repo = PathBuf::from(&args[i]);
            }
            "--files" => {
                i += 1;
                targets.extend(args[i].split(',').map(str::to_string));
            }
            "--all" => all = true,
            "--list" => list = true,
            "--dry-run" => dry_run = true,
            other => panic!("unknown arg: {other}"),
        }
        i += 1;
    }

    let mod_map = build_mod_map(&repo);
    let mut files: Vec<FileData> = Vec::new();
    for (rel, modpath) in &mod_map {
        let path = repo.join(rel);
        let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {rel:?}: {e}"));
        let ast = syn::parse_file(&src).unwrap_or_else(|e| panic!("parse {rel:?}: {e}"));
        files.push(FileData {
            rel: rel.clone(),
            src,
            ast,
            modpath: modpath.clone(),
        });
    }

    // Pass 1: top-level static mut defs everywhere.
    let mut defs: BTreeMap<PathBuf, Vec<(String, bool)>> = BTreeMap::new();
    for (idx, fd) in files.iter().enumerate() {
        let _ = idx;
        for item in &fd.ast.items {
            if let syn::Item::Static(s) = item {
                if matches!(s.mutability, syn::StaticMutability::Mut(_)) {
                    defs.entry(fd.rel.clone())
                        .or_default()
                        .push((s.ident.to_string(), has_no_mangle(&s.attrs)));
                }
            }
        }
    }

    if list {
        let mut rows: Vec<(usize, String)> = defs
            .iter()
            .map(|(rel, v)| (v.len(), rel.display().to_string()))
            .collect();
        rows.sort();
        for (n, rel) in rows {
            println!("{n:5} {rel}");
        }
        return;
    }

    let target_files: HashSet<PathBuf> = if all {
        defs.keys().cloned().collect()
    } else {
        let set: HashSet<PathBuf> = targets.iter().map(PathBuf::from).collect();
        for t in &set {
            assert!(
                mod_map.contains_key(t),
                "{} is not a module of the crate",
                t.display()
            );
        }
        set
    };

    // Exported (no_mangle) converted names -> defining module path.
    let mut exports: HashMap<String, String> = HashMap::new();
    for fd in &files {
        if !target_files.contains(&fd.rel) {
            continue;
        }
        for (name, no_mangle) in defs.get(&fd.rel).into_iter().flatten() {
            if *no_mangle {
                let prev = exports.insert(name.clone(), fd.modpath.clone());
                assert!(prev.is_none(), "duplicate exported static {name}");
            }
        }
    }

    let mut total_edits = 0;
    let mut total_defs = 0;
    let mut all_warns = Vec::new();
    for fd in &files {
        let is_target = target_files.contains(&fd.rel);
        let file_active: HashSet<String> = if is_target {
            defs.get(&fd.rel)
                .into_iter()
                .flatten()
                .map(|(n, _)| n.clone())
                .collect()
        } else {
            HashSet::new()
        };
        let mut rw = Rewriter {
            rel: &fd.rel,
            src: &fd.src,
            file_active,
            exports: &exports,
            is_target,
            shadowed: HashSet::new(),
            fn_extras: HashSet::new(),
            const_ctx: 0,
            edits: Vec::new(),
            warns: Vec::new(),
            converted_defs: 0,
            retyped_decls: 0,
        };
        rw.visit_file(&fd.ast);

        if rw.edits.is_empty() {
            continue;
        }

        // The GlobalCell type import goes in front of the first item.
        let mut use_lines = String::new();
        if (rw.converted_defs > 0 || rw.retyped_decls > 0)
            && !fd.src.contains("global_cell::GlobalCell")
        {
            use_lines.push_str("use crate::src::nvim::global_cell::GlobalCell;\n");
        }
        if !use_lines.is_empty() {
            let first_item_start = fd
                .ast
                .items
                .iter()
                .map(|it| span_range(it).start)
                .min()
                .expect("file has items");
            rw.edits.push(Edit {
                range: first_item_start..first_item_start,
                text: use_lines,
            });
        }

        total_edits += rw.edits.len();
        total_defs += rw.converted_defs;
        println!(
            "{}: {} edits, {} defs converted, {} decls retyped",
            fd.rel.display(),
            rw.edits.len(),
            rw.converted_defs,
            rw.retyped_decls
        );
        all_warns.extend(rw.warns);

        if !dry_run {
            let out = apply_edits(&fd.src, rw.edits);
            std::fs::write(repo.join(&fd.rel), out).expect("write file");
        }
    }

    println!("---");
    println!("{total_defs} defs converted, {total_edits} edits total");
    if !all_warns.is_empty() {
        println!("--- {} warnings:", all_warns.len());
        for w in &all_warns {
            println!("WARN {w}");
        }
    }
}
