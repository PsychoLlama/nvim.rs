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

`TABLE` is a file of `old_name new_name` lines; `#` starts a comment and blank
lines are skipped. The script finds each `old_name`'s own declaration under
`crates/nvim/src` (`struct`/`enum`/`union`/`type`, and it must be unique), asks
rust-analyzer to rename the symbol at that position, and applies the returned
`WorkspaceEdit` to disk. `--dry-run` prints the edit counts per file and writes
nothing.

`--fold` is for an alias over a C tag struct -- `pub type win_T = window_S;`.
Two renames are needed, because the LSP renames the *alias*, not the tag it
points at: the tag is renamed to the new name first, then the alias onto the
same name, and the `type X = X;` line that leaves behind is deleted. Give the
table the *alias* name; the tag is read off its `type` line.

## What this does not reach, and the pitfalls behind each flag

* **Comments and doc links.** rust-analyzer renames code. A `///` mention or a
  `[`win_T`]` intra-doc link is left alone, so follow a batch with a
  word-boundary sweep over comments only (`xform.masked` inverted) -- never an
  unmasked tree-wide `s///`, which is the mistake this tool exists to avoid.
* **Non-Rust followers.** `tools/apigen/src` spells three type names,
  `test/unit/fixtures/{shim.h,vterm_test.c}` spell `schar_T`, and
  `scripts/ratchet.py`'s `RAW_WIN_BUF` needle names `win_T`/`buf_T`/
  `tabpage_T`. Fix them in the same commit; `tools/ffigen` reads the Rust and
  needs nothing.
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

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "crates" / "nvim" / "src"

# The declaration of a type, at the start of a line so a mention inside an
# expression or a doc comment is not mistaken for one.
DECL = r"^(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum|union|type)\s+{}\b"


class Client:
    """A minimal LSP client over rust-analyzer's stdio."""

    def __init__(self, root):
        self.root = root
        self.proc = subprocess.Popen(
            ["rust-analyzer"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            cwd=root,
        )
        self.next_id = 1
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
                    # Both off deliberately -- see the module docstring.
                    "cargo": {"buildScripts": {"enable": False}},
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


def declaration(name, root):
    """(path, 0-based line, 0-based column) of `name`'s own declaration.

    The caller never hand-computes a position: a table is a list of names, and
    a name that is declared twice (or not at all) is an error rather than a
    coin flip over which declaration the rename starts from.
    """
    needle = re.compile(DECL.format(re.escape(name)), re.MULTILINE)
    found = []
    for path in sorted((root / "crates" / "nvim" / "src").rglob("*.rs")):
        text = path.read_text()
        for match in needle.finditer(text):
            line = text.count("\n", 0, match.start())
            column = text.rindex(name, match.start(), match.end()) - (
                text.rfind("\n", 0, match.start()) + 1
            )
            found.append((path, line, column))
    if not found:
        sys.exit(f"ra-rename: no declaration of `{name}` under crates/nvim/src")
    if len(found) > 1:
        where = ", ".join(f"{p.relative_to(root)}:{ln + 1}" for p, ln, _ in found)
        sys.exit(f"ra-rename: `{name}` is declared more than once: {where}")
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


def collect(edit, out):
    """Fold a `WorkspaceEdit` into path -> [(start, end, text)] byte edits."""
    changes = edit.get("changes") or {}
    for document in edit.get("documentChanges") or []:
        if "textDocument" in document and "edits" in document:
            changes.setdefault(document["textDocument"]["uri"], []).extend(
                document["edits"]
            )
    for uri, edits in changes.items():
        path = Path(uri.removeprefix("file://"))
        for one in edits:
            out.setdefault(path, []).append(one)
    return out


def offsets(text):
    """Byte offset of the start of each line, for LSP position -> index."""
    starts = [0]
    for line in text.split("\n"):
        starts.append(starts[-1] + len(line) + 1)
    return starts


def apply(changes):
    """Write the collected edits. Bottom-up per file, so offsets stay valid."""
    for path, edits in changes.items():
        text = path.read_text()
        starts = offsets(text)

        def index(position):
            # UTF-16 code units in the protocol; this tree's identifiers are
            # ASCII, and an edit that is not would land in the wrong column
            # rather than silently corrupt -- assert instead of guessing.
            return starts[position["line"]] + position["character"]

        spans = sorted(
            (index(e["range"]["start"]), index(e["range"]["end"]), e["newText"])
            for e in edits
        )
        for start, end, _ in spans:
            assert text[start:end].isascii(), f"{path}: non-ASCII edit at {start}"
        for start, end, new in reversed(spans):
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


def report(changes, label):
    total = sum(len(edits) for edits in changes.values())
    print(f"{label}: {total} edits in {len(changes)} files")
    for path, edits in sorted(changes.items()):
        print(f"    {len(edits):5}  {path.relative_to(ROOT)}")


def read_table(path):
    pairs = []
    for raw in Path(path).read_text().split("\n"):
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        parts = line.split()
        if len(parts) != 2:
            sys.exit(f"ra-rename: not an `old new` pair: {raw!r}")
        pairs.append((parts[0], parts[1]))
    return pairs


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("table", help="file of `old_name new_name` lines")
    parser.add_argument(
        "--dry-run", action="store_true", help="print edit counts, write nothing"
    )
    parser.add_argument(
        "--fold",
        action="store_true",
        help="rename the aliased tag too and drop the `type X = X;` it leaves",
    )
    parser.add_argument("--root", default=str(ROOT), help="workspace root")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    pairs = read_table(args.table)
    client = Client(root)
    client.initialize()
    try:
        for old, new in pairs:
            path, line, column = declaration(old, root)
            changes = {}
            if args.fold:
                tag = alias_target(path, line, old)
                tag_path, tag_line, tag_column = declaration(tag, root)
                collect(
                    client.rename(tag_path, tag_line, tag_column, new) or {}, changes
                )
                if not args.dry_run:
                    apply(changes)
                    changes = {}
                    # The alias moved with the tag: re-find it before asking
                    # for the second rename.
                    path, line, column = declaration(old, root)
            collect(client.rename(path, line, column, new) or {}, changes)
            if args.dry_run:
                report(
                    changes,
                    f"{old} -> {new}" + (f" (fold of {tag})" if args.fold else ""),
                )
                continue
            apply(changes)
            if args.fold and not drop_self_alias(path, new):
                sys.exit(f"ra-rename: no `type {new} = {new};` left in {path}")
            print(f"{old} -> {new}: applied")
    finally:
        client.shutdown()


if __name__ == "__main__":
    main()
