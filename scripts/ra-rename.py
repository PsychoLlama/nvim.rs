#!/usr/bin/env python3
"""Rename Rust types tree-wide by driving rust-analyzer's LSP rename.

The migration's remaining bulk renames are type names: the `_T` typedefs and
the `_S` tag structs c2rust inherited from the C. A textual sweep is the wrong
tool for them -- four tags are also field names, several targets collide with a
wrapper that already holds the plain name, and a rename has to follow `use`
paths -- so this drives the only thing in the toolchain that understands Rust
name resolution.

Usage:

    scripts/ra-rename.py TABLE [--dry-run] [--fold] [--root DIR]
    scripts/ra-rename.py TABLE --params [--report FILE] [--dry-run] [--root DIR]

`TABLE` is a file of `old_name new_name [declared_in]` lines; `#` starts a
comment and blank lines are skipped. Under `--params` the third field is a
regex the parameter's declared *type* must match instead (see below). The script finds each `old_name`'s own
declaration under `crates/nvim/src` (`struct`/`enum`/`union`/`type`), asks
rust-analyzer to rename the symbol at that position, and applies the returned
`WorkspaceEdit` to disk. `--dry-run` prints the edit counts per file and writes
nothing.

The declaration must be unique, so that a name declared twice is an error
rather than a coin flip over which one the rename starts from. `declared_in`
is how a collision batch says which one it means: a path the declaring file
must end with (`eval/list/mod.rs`), needed exactly when the tree already holds
two of a name -- which is the situation a collision rename exists to end.

## `--params` renames a *parameter*, once per signature that binds it

Phase 27's other half is the transpiler's parameter abbreviations -- `wp`,
`rettv`, `argvars`, `eap` -- and those are not one symbol but hundreds of
unrelated locals that happen to share a spelling. `--params` finds them the
way the ratchet counts them: mask the file, walk the `fn` *definitions*, and
match the binding inside the parameter list, so `old_buf` and a local of the
same name are not touched. `mut old` and the `_old` an unused parameter is
spelled with are both bindings; `_old` renames to `_new`, keeping the mark.

One spelling can mean two things, and then the type is what tells them apart:
`buf` is a buffer object in `buf: *mut Buffer` and an idiomatic byte buffer in
`buf: &mut [u8]`, and only the first is one of the transpiler's names. A row's
third field is a regex the declared type must match, so
`buf buffer \b(?:Buffer|BufferRef|BufferHandle|Buf)\b` renames the 627 that
name a buffer and leaves the 171 that name bytes -- the same split the
ratchet's `abbrev_params` counts.

Each position is then an ordinary LSP rename, which is what makes this worth
driving through rust-analyzer rather than `sed`: it follows the name into
closures, into `Foo { buf }` field-init shorthand (which it rewrites to
`Foo { buf: buffer }` rather than breaking it), and into the macro bodies it
can see.

**The pre-scan is the safety net the compiler does not provide.** Renaming a
parameter to a name the body already binds -- `let result`, a closure's
`|result|`, `Some(result)` -- compiles, because the new local simply shadows
the parameter from its `let` onwards, and every use below it silently changes
meaning. So the body and the rest of the signature are searched for the target
name first, and a signature that already holds it is *skipped* into
`--report`'s file rather than renamed. Occurrences after a `.` or a `::` are
field accesses and path segments, not bindings, and do not skip. The scan is
deliberately blunt in the other direction -- a struct-literal key `result:` or
a call to a function of that name skips too -- because a skip costs a line in
a report and a wrong rename costs a silent behaviour change.

`--fold` is for an alias over a C tag struct -- `pub type qf_info_T = qf_info_S;`.
Two renames are needed, because the LSP renames the *alias*, not the tag it
points at: both the tag and the alias are renamed to the new name, and the
`type X = X;` line that leaves behind is deleted. Give the table the *alias*
name; the tag is read off its `type` line.

## Every rename is computed against the pristine tree, then applied once

A whole table's edits are collected before a byte is written. That is not an
optimisation, it is what makes a multi-entry table correct: an applied edit
that rust-analyzer has not been told about leaves its index stale, and the
next rename in the same run answers from the stale index -- silently, with no
error and no empty-edit warning. Nothing is applied mid-run, so nothing goes
stale.

It also makes a table's entries *simultaneous* rather than chained, which is
what a collision batch wants: `Pos -> PosRef` and `pos_T -> Pos` in one table
rename the two distinct symbols they name, in either order, and the wrapper's
new name is never itself renamed by the entry below it. Two entries that
resolve to the same span are a conflict and abort; distinct identifiers never
do.

## `cfg` is the thing that actually under-renames, and it needs two passes

rust-analyzer analyses *one* `cfg` configuration, and code excluded by it is
not merely unindexed -- it is invisible to a rename, which returns a smaller
edit with no diagnostic. rust-analyzer's default `cargo.cfgs` is
`["debug_assertions", "miri"]`, i.e. **`miri` is on**, so every file this tree
gates with `#![cfg(not(miri))]` (32 of the unit specs) is skipped; the `p27-4`
batch that "under-renamed `tests/unit/{memline,indent}.rs`" was this, not
staleness. Turning `miri` off only trades the loss the other way, for the
`#[cfg(miri)]` shims in `os/cshim.rs` and `xdiff/ffi.rs`.

So the run asks a fresh server per configuration in `CFG_PROFILES` -- one
matching `cargo build`/`cargo test`, one matching `just miri` -- and unions the
edits. The servers are sequential, so the peak is one server's ~1.4 GB, and a
second pass costs a cache prime plus a request per name. Add a profile here
if a `cfg` ever hides code from both.

## What this still does not reach, and the pitfalls behind each flag

* **Comments and doc links.** rust-analyzer renames code. A `///` mention or a
  `[`vimoption_T`]` intra-doc link is left alone, so follow a batch with a
  word-boundary sweep over comments only (`xform.masked` inverted) -- never an
  unmasked tree-wide `s///`, which is the mistake this tool exists to avoid.
* **Non-Rust followers.** `tools/apigen/src` spells a few type names,
  `test/unit/fixtures/*.{c,h}` spell the ones the FFI fixtures use, and
  `scripts/ratchet.py`'s `RAW_WIN_BUF` needle names the window/buffer/tabpage
  structs. Fix them in the same commit; `tools/ffigen` reads the Rust and
  needs nothing. `scripts/ratchet.py` must be *excluded* from any sweep: its
  self-test fixtures spell `_T` names as data for the counts they assert.
* **Build scripts must stay off.** With `cargo.buildScripts.enable` on, the
  first rename request hung for six minutes and never answered. A rename needs
  no `OUT_DIR`, so both that and `procMacro.enable` are disabled below.
* **Cache priming is not optional.** The server answers a rename with an empty
  edit -- not an error -- if it is asked before the workspace is indexed. The
  client waits for the `rustAnalyzer/cachePriming` progress token to end, which
  is ~6 s warm on this tree.
* **Memory.** The server peaks around 1.4 GB RSS on this workspace.
"""

import argparse
import json
import os
import re
import subprocess
import sys
import threading
from pathlib import Path

# `--params` reuses the ratchet's masker and its `fn` span logic, so the
# parameters this renames are exactly the ones `abbrev_params` counts.
import ratchet

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "crates" / "nvim" / "src"

# Where a parameter rename looks, and the two subtrees the exit clause carves
# out of it -- ports whose parameter names are the upstream project's, same as
# the ratchet's `ABBREV_PARAM_EXEMPT`.
PARAM_ROOTS = ("crates/nvim/src", "crates/nvim/tests")
PARAM_EXEMPT = ("crates/nvim/src/lua/", "crates/nvim/src/vterm/")

# The declaration of a type, at the start of a line so a mention inside an
# expression or a doc comment is not mistaken for one.
DECL = r"^(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum|union|type)\s+{}\b"

# The `cfg` configurations a rename has to be answered under, because no single
# one sees the whole tree -- see the module docstring. `rust-analyzer.cargo.cfgs`
# replaces the server's default list wholesale.
CFG_PROFILES = (
    ("build", ["debug_assertions"]),
    ("miri", ["debug_assertions", "miri"]),
)


class Client:
    """A minimal LSP client over rust-analyzer's stdio."""

    def __init__(self, root, cfgs):
        self.root = root
        self.cfgs = cfgs
        self.proc = subprocess.Popen(
            ["rust-analyzer"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            cwd=root,
        )
        self.next_id = 1
        self.soft = False  # see `request`
        self.replies = {}
        self.primed = threading.Event()
        self.lock = threading.Lock()
        self.reader = threading.Thread(target=self._read_loop, daemon=True)
        self.reader.start()

    # -- transport ---------------------------------------------------------

    def _send(self, message):
        body = json.dumps(message).encode()
        header = f"Content-Length: {len(body)}\r\n\r\n".encode()
        self.proc.stdin.write(header + body)
        self.proc.stdin.flush()

    def _read_loop(self):
        out = self.proc.stdout
        while True:
            length = None
            while True:
                line = out.readline()
                if not line:
                    return
                if line in (b"\r\n", b"\n"):
                    break
                name, _, value = line.decode().partition(":")
                if name.strip().lower() == "content-length":
                    length = int(value.strip())
            if length is None:
                continue
            message = json.loads(out.read(length))
            if "id" in message and "method" not in message:
                with self.lock:
                    self.replies[message["id"]] = message
            elif message.get("method") == "$/progress":
                params = message.get("params", {})
                token = params.get("token")
                kind = params.get("value", {}).get("kind")
                if token == "rustAnalyzer/cachePriming" and kind == "end":
                    self.primed.set()
            elif message.get("method") == "window/workDoneProgress/create":
                self._send({"jsonrpc": "2.0", "id": message["id"], "result": None})

    def request(self, method, params, timeout=600.0):
        with self.lock:
            request_id = self.next_id
            self.next_id += 1
        self._send(
            {"jsonrpc": "2.0", "id": request_id, "method": method, "params": params}
        )
        waited = 0.0
        while waited < timeout:
            with self.lock:
                if request_id in self.replies:
                    reply = self.replies.pop(request_id)
                    if "error" in reply:
                        # A `--params` position can be real code the *other*
                        # `cfg` profile compiles, and this one answers "no
                        # references found". The union is the answer, so a
                        # soft client reports and carries on; a position no
                        # profile could rename is caught at the end.
                        if self.soft:
                            print(
                                f"ra-rename: {method}: {reply['error']}",
                                file=sys.stderr,
                            )
                            return None
                        sys.exit(f"ra-rename: {method}: {reply['error']}")
                    return reply.get("result")
            threading.Event().wait(0.02)
            waited += 0.02
        sys.exit(f"ra-rename: {method} timed out after {timeout:.0f}s")

    def notify(self, method, params):
        self._send({"jsonrpc": "2.0", "method": method, "params": params})

    # -- lifecycle ---------------------------------------------------------

    def initialize(self):
        uri = self.root.as_uri()
        self.request(
            "initialize",
            {
                "processId": os.getpid(),
                "rootUri": uri,
                "workspaceFolders": [{"uri": uri, "name": self.root.name}],
                "capabilities": {
                    "window": {"workDoneProgress": True},
                    "workspace": {
                        "workspaceEdit": {"documentChanges": True},
                        "applyEdit": True,
                    },
                },
                "initializationOptions": {
                    # Build scripts and proc macros off deliberately, `cfgs`
                    # pinned deliberately -- see the module docstring.
                    "cargo": {
                        "buildScripts": {"enable": False},
                        "cfgs": self.cfgs,
                    },
                    "procMacro": {"enable": False},
                },
            },
        )
        self.notify("initialized", {})
        # `waitForRustAnalyzerReady` does not exist as a request; the progress
        # token is the signal. Fall back to a plain wait if the server never
        # primes (an empty workspace, or a version that names the token
        # differently) rather than hanging forever.
        if not self.primed.wait(timeout=600):
            print("ra-rename: cache priming never ended; continuing", file=sys.stderr)

    def shutdown(self):
        try:
            self.request("shutdown", None, timeout=30)
            self.notify("exit", {})
        finally:
            self.proc.terminate()
            self.proc.wait(timeout=30)

    # -- the one request that matters --------------------------------------

    def rename(self, path, line, column, new_name):
        """The `WorkspaceEdit` for renaming the symbol at a position."""
        return self.request(
            "textDocument/rename",
            {
                "textDocument": {"uri": Path(path).as_uri()},
                "position": {"line": line, "character": column},
                "newName": new_name,
            },
        )


def declaration(name, root, declared_in=None):
    """(path, 0-based line, 0-based column) of `name`'s own declaration.

    The caller never hand-computes a position: a table is a list of names, and
    a name that is declared twice (or not at all) is an error rather than a
    coin flip over which declaration the rename starts from. `declared_in`
    narrows the search to files whose path ends with it, which is how a table
    names one of two same-named types.
    """
    needle = re.compile(DECL.format(re.escape(name)), re.MULTILINE)
    found = []
    for path in sorted((root / "crates" / "nvim" / "src").rglob("*.rs")):
        if declared_in and not path.as_posix().endswith(declared_in):
            continue
        text = path.read_text()
        for match in needle.finditer(text):
            line = text.count("\n", 0, match.start())
            column = text.rindex(name, match.start(), match.end()) - (
                text.rfind("\n", 0, match.start()) + 1
            )
            found.append((path, line, column))
    scope = f" in `{declared_in}`" if declared_in else ""
    if not found:
        sys.exit(f"ra-rename: no declaration of `{name}`{scope} under crates/nvim/src")
    if len(found) > 1:
        where = ", ".join(f"{p.relative_to(root)}:{ln + 1}" for p, ln, _ in found)
        sys.exit(f"ra-rename: `{name}` is declared more than once{scope}: {where}")
    return found[0]


def alias_target(path, line, name):
    """The tag a `type X = Tag;` alias points at, for `--fold`."""
    text = path.read_text().split("\n")[line]
    match = re.search(
        rf"\btype\s+{re.escape(name)}\s*=\s*(?:::)?(?:[A-Za-z0-9_]+::)*"
        r"([A-Za-z_][A-Za-z0-9_]*)\s*;",
        text,
    )
    if not match:
        sys.exit(f"ra-rename: `{name}` at {path}:{line + 1} is not a plain alias")
    return match.group(1)


def collect(edit, out, where):
    """Fold a `WorkspaceEdit` into path -> {range: newText}.

    Keyed by range so the same edit seen under two `cfg` profiles counts once,
    and so two profiles (or two table entries) that disagree about one span
    abort instead of racing to be applied last.
    """
    changes = dict(edit.get("changes") or {})
    for document in edit.get("documentChanges") or []:
        if "textDocument" in document and "edits" in document:
            changes.setdefault(document["textDocument"]["uri"], []).extend(
                document["edits"]
            )
    for uri, edits in changes.items():
        path = Path(uri.removeprefix("file://"))
        bucket = out.setdefault(path, {})
        for one in edits:
            span = one["range"]
            key = (
                span["start"]["line"],
                span["start"]["character"],
                span["end"]["line"],
                span["end"]["character"],
            )
            seen = bucket.get(key)
            if seen is not None and seen != one["newText"]:
                sys.exit(
                    f"ra-rename: {where}: conflicting edits at "
                    f"{path}:{key[0] + 1}: {seen!r} vs {one['newText']!r}"
                )
            bucket[key] = one["newText"]
    return out


def merge(into, changes, where):
    """Union one name's edits into the whole table's, rejecting conflicts."""
    for path, bucket in changes.items():
        target = into.setdefault(path, {})
        for key, text in bucket.items():
            seen = target.get(key)
            if seen is not None and seen != text:
                sys.exit(
                    f"ra-rename: {where}: conflicting edits at "
                    f"{path}:{key[0] + 1}: {seen!r} vs {text!r}"
                )
            target[key] = text
    return into


def offsets(text):
    """Byte offset of the start of each line, for LSP position -> index."""
    starts = [0]
    for line in text.split("\n"):
        starts.append(starts[-1] + len(line) + 1)
    return starts


def apply(changes):
    """Write the collected edits. Bottom-up per file, so offsets stay valid."""
    for path, bucket in changes.items():
        text = path.read_text()
        starts = offsets(text)

        def index(line, character):
            # UTF-16 code units in the protocol; this tree's identifiers are
            # ASCII, and an edit that is not would land in the wrong column
            # rather than silently corrupt -- assert instead of guessing.
            return starts[line] + character

        spans = sorted(
            (index(key[0], key[1]), index(key[2], key[3]), new)
            for key, new in bucket.items()
        )
        previous_start = len(text) + 1
        for start, end, _ in spans:
            assert text[start:end].isascii(), f"{path}: non-ASCII edit at {start}"
        for start, end, new in reversed(spans):
            assert end <= previous_start, f"{path}: overlapping edits at {start}"
            previous_start = start
            text = text[:start] + new + text[end:]
        path.write_text(text)


def drop_self_alias(path, name):
    """Delete the `type X = X;` a fold leaves behind. Answers whether it did."""
    text = path.read_text()
    needle = re.compile(
        rf"^[^\S\n]*(?:pub(?:\([^)]*\))?[^\S\n]+)?type[^\S\n]+{re.escape(name)}"
        rf"[^\S\n]*=[^\S\n]*{re.escape(name)}[^\S\n]*;[^\S\n]*\n",
        re.MULTILINE,
    )
    stripped, count = needle.subn("", text, count=1)
    if count:
        path.write_text(stripped)
    return bool(count)


def report_skips(skipped, path):
    """Write the signatures the pre-scan refused to rename."""
    print(f"skipped {len(skipped)} signatures (the body already binds the name)")
    body = "\n".join(skipped) + ("\n" if skipped else "")
    if path:
        Path(path).write_text(body)
    else:
        sys.stderr.write(body)


def report(changes, label):
    total = sum(len(bucket) for bucket in changes.values())
    print(f"{label}: {total} edits in {len(changes)} files")
    for path, bucket in sorted(changes.items()):
        try:
            shown = path.relative_to(ROOT)
        except ValueError:
            shown = path
        print(f"    {len(bucket):5}  {shown}")


def read_table(path):
    rows = []
    for raw in Path(path).read_text().split("\n"):
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        parts = line.split()
        if not 2 <= len(parts) <= 3:
            sys.exit(f"ra-rename: not an `old new [declared_in]` row: {raw!r}")
        rows.append((parts[0], parts[1], parts[2] if len(parts) == 3 else None))
    return rows


def plan(rows, root, fold):
    """Resolve every table entry to the positions a rename starts from.

    Done once, against the pristine tree, for every `cfg` profile: a position
    is a line and column, and applying an edit would move the ones below it.
    """
    entries = []
    for old, new, declared_in in rows:
        path, line, column = declaration(old, root, declared_in)
        positions = []
        tag = None
        if fold:
            tag = alias_target(path, line, old)
            positions.append((*declaration(tag, root), new))
        positions.append((path, line, column, new))
        entries.append(
            {"old": old, "new": new, "tag": tag, "alias": path, "positions": positions}
        )
    return entries


def fn_definitions(masked):
    """(name, parameter-list start, parameter-list end, body end) per `fn`.

    Offsets into the masked text, for `--params`. Only *definitions*, by the
    ratchet's own rule: a `fn` with no parameter list after the name is a
    function-pointer type or a trait bound, not a definition. A `fn` with no
    body (a trait method's declaration) reports its body as empty.
    """
    for match in ratchet.FN_NAME.finditer(masked):
        i = match.end()
        while i < len(masked) and masked[i].isspace():
            i += 1
        if i < len(masked) and masked[i] == "<":  # generic parameters
            i = ratchet.balanced(masked, i, "<", ">")
        while i < len(masked) and masked[i].isspace():
            i += 1
        if i >= len(masked) or masked[i] != "(":
            continue
        start = i
        end = ratchet.balanced(masked, i, "(", ")")
        yield match.group(1), start, end, body_end(masked, end)


def body_end(masked, after_params):
    """The index just past a `fn`'s body, or `after_params` when it has none.

    Walks the return type and any `where` clause at bracket depth zero. `->`
    is stepped over whole so its `>` does not read as a closing bracket.
    """
    i, depth = after_params, 0
    while i < len(masked):
        if masked[i : i + 2] == "->":
            i += 2
            continue
        char = masked[i]
        if char in "<([":
            depth += 1
        elif char in ">)]":
            depth -= 1
        elif depth <= 0 and char == "{":
            return ratchet.balanced(masked, i, "{", "}")
        elif depth <= 0 and char == ";":
            return after_params
        i += 1
    return after_params


def line_column(text, offset):
    """(0-based line, 0-based column) of a byte offset. ASCII identifiers."""
    line = text.count("\n", 0, offset)
    return line, offset - (text.rfind("\n", 0, offset) + 1)


def binds(name):
    """A parameter binding of `name`, with `mut` and the unused-`_` spelling.

    The `\b` in front is what keeps this off `old_buf` and `bufp`: a name with
    a qualifier already says what the abbreviation does not.
    """
    return re.compile(rf"\b(?:mut\s+)?(_?{re.escape(name)})\s*:")


# The pattern of a `let` statement, from the `let` up to the binding this scan
# found: only names, `mut`, and the punctuation a tuple or reference pattern
# spells. What it proves is that the mention is a *binding* in a `let`, which
# is the one shape whose scope starts at a place this scanner can find.
LET_PATTERN = re.compile(r"\s*let\s+(?:mut\s+)?[\w(),\s&]*$")


def statement_end(masked, at, end):
    """The index just past the statement *starting* at the offset `at`.

    `at` must be the statement's first character, not a position inside it:
    the walk counts brackets from zero, so a group opened before `at` would
    close it early.
    """
    depth = 0
    i = at
    while i < end:
        char = masked[i]
        if char in "([{":
            depth += 1
        elif char in ")]}":
            if depth == 0:
                return i
            depth -= 1
        elif char == ";" and depth == 0:
            return i + 1
        i += 1
    return end


def mentions(masked, start, end, name):
    """Every mention of `name` in a span that could be a binding of it.

    A mention after a `.` is a field or a method and after a `::` a path
    segment; neither can shadow a parameter, so neither counts.
    """
    for match in re.finditer(rf"\b{re.escape(name)}\b", masked[start:end]):
        before = masked[start : start + match.start()].rstrip()
        if before.endswith(".") or before.endswith(":"):
            continue
        after = masked[start + match.end() :]
        if after.startswith("::") or after.startswith("!"):
            continue
        yield start + match.start()


def conflicting(masked, start, end, old, new):
    """The mention of `new` that makes renaming `old` to it unsafe, or `None`.

    A `let` that binds `new` is only a conflict when `old` is still *used*
    after that statement: `let (args, rettv) = frame!(argvars, rettv);` shadows
    the parameter on the very line that consumes it for the last time, and
    renaming `argvars` to `args` there is the idiomatic raw-to-safe shadow, not
    a hazard. Every other shape of mention -- a closure parameter, a `for`
    binding, a call, a struct-literal key -- is a conflict outright, because
    the scope it opens is not one a regex can bound.
    """
    for at in mentions(masked, start, end, new):
        line_start = masked.rfind("\n", 0, at) + 1
        line_stop = masked.find("\n", at)
        line = masked[line_start : line_stop if line_stop != -1 else len(masked)]
        pattern = LET_PATTERN.match(masked[line_start:at])
        if not pattern:
            return at, line
        late = re.compile(rf"\b{re.escape(old)}\b").search(
            masked, statement_end(masked, line_start + pattern.start(), end), end
        )
        # Past this `let`, every mention of `new` *is* the local it bound, so
        # the scan is finished either way: the only question was whether `old`
        # outlives the shadow.
        return (late.start(), line) if late else None
    return None


def param_files(root):
    for where in PARAM_ROOTS:
        for path in sorted((root / where).rglob("*.rs")):
            relative = path.relative_to(root).as_posix()
            if not any(relative.startswith(skip) for skip in PARAM_EXEMPT):
                yield path, relative


def declared_type(masked, at, end):
    """The type a parameter's `:` at `at` introduces, up to the next `,`."""
    depth, i = 0, at
    while i < end:
        char = masked[i]
        if char in "([<{":
            depth += 1
        elif char in ")]>}":
            if depth == 0:
                break
            depth -= 1
        elif char == "," and depth == 0:
            break
        i += 1
    return masked[at:i]


def param_positions(old, new, root, type_filter=None):
    """Every `fn` parameter binding `old`, split into renames and skips.

    A rename is `(path, line, column, new_name)`; a skip is a line of report
    text naming the signature and the mention that blocked it. `type_filter`
    is a regex the declared type must match, which is how one spelling that
    means two things -- `buf` the buffer object, `buf` the byte buffer -- is
    renamed on one of them only.
    """
    needle = binds(old)
    kind = re.compile(type_filter) if type_filter else None
    positions, skipped = [], []
    for path, relative in param_files(root):
        masked = ratchet.mask(path.read_text())
        for name, start, end, stop in fn_definitions(masked):
            for match in needle.finditer(masked, start, end):
                bound = match.group(1)
                if kind and not kind.search(declared_type(masked, match.end(), end)):
                    continue
                target = f"_{new}" if bound.startswith("_") else new
                line, column = line_column(masked, match.start(1))
                clash = conflicting(masked, start, max(stop, end), bound, target)
                if clash is None:
                    positions.append((path, line, column, target))
                    continue
                at, text = clash
                clash_line, _ = line_column(masked, at)
                skipped.append(
                    f"{relative}:{line + 1}  fn {name}({bound}) -> {target}"
                    f"  blocked at :{clash_line + 1}  {text.strip()}"
                )
    return positions, skipped


def plan_params(rows, root):
    """`plan`'s counterpart for `--params`: one entry per name, many positions."""
    entries, skipped = [], []
    for old, new, type_filter in rows:
        positions, skips = param_positions(old, new, root, type_filter)
        if not positions:
            sys.exit(f"ra-rename: no parameter named `{old}` under {PARAM_ROOTS}")
        entries.append(
            {"old": old, "new": new, "tag": None, "alias": None, "positions": positions}
        )
        skipped.extend(skips)
    return entries, skipped


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("table", help="file of `old_name new_name [declared_in]` lines")
    parser.add_argument(
        "--dry-run", action="store_true", help="print edit counts, write nothing"
    )
    parser.add_argument(
        "--fold",
        action="store_true",
        help="rename the aliased tag too and drop the `type X = X;` it leaves",
    )
    parser.add_argument(
        "--params",
        action="store_true",
        help="rename a `fn` parameter everywhere it is bound, not a type",
    )
    parser.add_argument(
        "--report",
        help="where --params writes the signatures it skipped (default: stderr)",
    )
    parser.add_argument("--root", default=str(ROOT), help="workspace root")
    args = parser.parse_args()
    if args.params and args.fold:
        parser.error("--params and --fold rename different things")

    root = Path(args.root).resolve()
    rows = read_table(args.table)
    if args.params:
        entries, skipped = plan_params(rows, root)
        report_skips(skipped, args.report)
    else:
        entries = plan(rows, root, args.fold)

    for entry in entries:
        entry["changes"] = {}
        entry["answered"] = set()
    for profile, cfgs in CFG_PROFILES:
        client = Client(root, cfgs)
        client.soft = args.params
        try:
            client.initialize()
            for entry in entries:
                where = f"{entry['old']} -> {entry['new']} (cfg {profile})"
                for index, (path, line, column, name) in enumerate(entry["positions"]):
                    edit = client.rename(path, line, column, name) or {}
                    if edit:
                        entry["answered"].add(index)
                    collect(edit, entry["changes"], where)
        finally:
            client.shutdown()
    unanswered = [
        f"{path}:{line + 1}:{column + 1}  {entry['old']} -> {name}"
        for entry in entries
        for index, (path, line, column, name) in enumerate(entry["positions"])
        if index not in entry["answered"]
    ]
    if unanswered:
        sys.exit(
            "ra-rename: no profile could rename these positions:\n  "
            + "\n  ".join(unanswered)
        )

    changes = {}
    for entry in entries:
        label = f"{entry['old']} -> {entry['new']}"
        if entry["tag"]:
            label += f" (fold of {entry['tag']})"
        if args.dry_run:
            report(entry["changes"], label)
        merge(changes, entry["changes"], label)
    if args.dry_run:
        report(changes, f"total ({len(entries)} names)")
        return

    apply(changes)
    for entry in entries:
        if args.params:
            print(f"{entry['old']} -> {entry['new']}: applied")
            continue
        if args.fold and not drop_self_alias(entry["alias"], entry["new"]):
            sys.exit(
                f"ra-rename: no `type {entry['new']} = {entry['new']};` left in "
                f"{entry['alias']}"
            )
        print(f"{entry['old']} -> {entry['new']}: applied")


if __name__ == "__main__":
    main()
