#!/usr/bin/env python3
"""Refuse a commit that carries generated output instead of source.

Build artifacts have reached this history more than once and had to be
rewritten back out of it before a push. Each arrived the same way -- a broad
`git add` in a dirty tree -- and none was visible in the commit's diffstat.

The rules below are about shape, not about the artifacts that happened:
enumerating last month's mistakes only catches last month's mistakes.
.gitignore is where a *specific* shape gets named, and this treats that list as
the statement of intent it is.

  force-added  the path is ignored yet staged anyway. Nothing in this tree is
               legitimately both (`git ls-files | git check-ignore` is empty),
               so every shape .gitignore already names is covered, and one
               learned later is covered the moment it is added there -- which
               is the fix for anything this guard lets through.
  oversize     the blob is over SIZE_LIMIT and the path was not already over it
               in HEAD. Grandfathering by path rather than by a committed
               baseline keeps this self-maintaining: the four files that are
               legitimately large (two 1.6 MB fixtures, the spell file, the
               icon) stay editable, and a *new* one has to be argued for. The
               largest blob written in the last 400 commits is 223 KB.
  bulk         the *newly added* paths total more than BULK_LIMIT. Modified
               paths are excluded, so a tree-wide sweep (1012 files in one
               commit, in this history) does not trip it, while dropping a
               build directory in does.

Size is the whole test for content, deliberately. Sniffing for executable magic
(ELF, ar, Mach-O) was tried and dropped: a compiled object big enough to matter
is already oversize, and one small enough to slip past that costs little if it
lands. Nothing here reads a blob's bytes, only its size.

    just artifact-guard                        # the index; the pre-commit hook
    just artifact-guard --range origin/main..HEAD   # every blob a push would carry

Range mode walks objects, not diff endpoints, so it still sees a blob that was
added and then deleted inside the range -- the state an artifact is usually
found in, already buried a few commits back. Deleting it never got it out of
the pack.
"""

import os
import subprocess
import sys

SIZE_LIMIT = 512 * 1024
BULK_LIMIT = 4 * 1024 * 1024
EMPTY_TREE = "4b825dc642cb6eb9a060e54bf8d69288fbee4904"
OVERRIDE = "ARTIFACT_GUARD"


def git(*args, **kwargs):
    return subprocess.run(
        ["git", *args], capture_output=True, text=True, check=True, **kwargs
    ).stdout


def human(size):
    for unit in ("B", "KB", "MB", "GB"):
        if size < 1024 or unit == "GB":
            return f"{size:.0f} {unit}" if unit == "B" else f"{size:.1f} {unit}"
        size /= 1024


class Sizes:
    """One `git cat-file --batch-check` for every object we have to measure.

    Sizes come from the object store, never the worktree: the commit is made
    from the index, and a `git add`ed file may have changed or gone since.
    `--batch-check` answers with a header line and no payload, so measuring a
    48 MB blob costs a line of text.
    """

    def __init__(self):
        self.proc = subprocess.Popen(
            ["git", "cat-file", "--batch-check"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True,
        )

    def of(self, rev):
        """Size of `rev` in bytes, or None if it does not resolve."""
        self.proc.stdin.write(rev + "\n")
        self.proc.stdin.flush()
        fields = self.proc.stdout.readline().split()
        # "<rev> missing" -- and the rev is echoed verbatim, so a path with a
        # space in it puts more than two fields on that line.
        if fields[-1] == "missing":
            return None
        return int(fields[2])

    def close(self):
        self.proc.stdin.close()
        self.proc.stdout.close()
        self.proc.wait()


def staged():
    """[(path, sha, is_new)] for everything the next commit would write."""
    head = subprocess.run(
        ["git", "rev-parse", "--verify", "-q", "HEAD"], capture_output=True
    )
    base = "HEAD" if head.returncode == 0 else EMPTY_TREE
    fields = git("diff-index", "--cached", "--raw", "-z", base).split("\0")
    entries = []
    i = 0
    while i < len(fields) and fields[i]:
        meta = fields[i].split(" ")
        dst_mode, dst_sha, status = meta[1], meta[3], meta[4]
        # A rename or a copy names both paths; the new one is what we keep.
        paths = 2 if status[0] in "RC" else 1
        path = fields[i + paths]
        i += 1 + paths
        if status[0] == "D" or dst_mode in ("120000", "160000"):
            continue  # deletions carry nothing; symlinks and submodules aren't blobs
        entries.append((path, dst_sha, status[0] in "AC"))
    return entries


def in_range(rev_range):
    """[(path, sha, is_new)] for every blob the range introduces.

    `--objects` enumerates what a push would have to send, so a blob that was
    committed and then deleted a commit later is still here -- deleting it
    does not get it out of the pack.
    """
    entries = []
    seen = set()
    objects = git(
        # Unquoted paths: rev-list has no -z, and a quoted path would reach the
        # report (and check-ignore) with its escapes still in it.
        "-c",
        "core.quotePath=false",
        "rev-list",
        "--objects",
        "--filter=object:type=blob",
        rev_range,
    )
    for line in objects.splitlines():
        sha, _, path = line.partition(" ")
        if path and sha not in seen:
            seen.add(sha)
            entries.append((path, sha, True))
    return entries


def force_added(paths):
    """The staged paths .gitignore names -- i.e. those added with `git add -f`.

    `--no-index` is the whole point: without it check-ignore reports nothing
    for a path that is already in the index, which is every path here.
    """
    if not paths:
        return set()
    out = subprocess.run(
        ["git", "check-ignore", "--no-index", "--stdin", "-z"],
        input="\0".join(paths) + "\0",
        capture_output=True,
        text=True,
    ).stdout
    return {path for path in out.split("\0") if path}


def main():
    args = sys.argv[1:]
    if os.environ.get(OVERRIDE) == "off":
        return
    if "--range" in args:
        rest = args[args.index("--range") + 1 :]
        if not rest:
            sys.exit("artifact-guard: --range wants a revision range")
        rev_range = rest[0]
        entries, mode = in_range(rev_range), rev_range
    else:
        entries, mode = staged(), "the index"

    ignored = force_added([path for path, _, _ in entries])
    sizes = Sizes()
    forced = []
    oversize = []
    added_bytes = 0
    for path, sha, is_new in entries:
        size = sizes.of(sha)
        if size is None:
            continue
        if is_new:
            added_bytes += size
        if path in ignored:
            forced.append((path, size))
        elif size > SIZE_LIMIT:
            # Grandfathered by path: a file already this large in HEAD is one
            # somebody chose, and it stays editable. A newline in a path would
            # desynchronise --batch-check's line protocol, and such a path has
            # no grandfathered version worth the risk.
            was = None if "\n" in path else sizes.of(f"HEAD:{path}")
            if was is None or was <= SIZE_LIMIT:
                oversize.append((path, size))
    sizes.close()

    problems = []
    if forced:
        problems.append("gitignored, yet staged:")
        problems += [f"  {path}: {human(size)}" for path, size in sorted(forced)]
    if oversize:
        problems.append(f"blobs over {human(SIZE_LIMIT)}:")
        problems += [
            f"  {path}: {human(size)}" for path, size in sorted(oversize, reverse=True)
        ]
    if added_bytes > BULK_LIMIT:
        problems.append(
            f"new files total {human(added_bytes)}, over the "
            f"{human(BULK_LIMIT)} a single commit may add"
        )

    if problems:
        print(f"artifact-guard: unexpected content in {mode}:", file=sys.stderr)
        print("\n".join(problems), file=sys.stderr)
        sys.exit(
            "artifact-guard: committing generated output is strongly "
            f"discouraged; override with `{OVERRIDE}=off <command>`"
        )


if __name__ == "__main__":
    main()
