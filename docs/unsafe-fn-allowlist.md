# The `unsafe fn` allowlist

When this list was opened, a quarter of the tree's `unsafe fn`s took no raw
pointer at all. Their parameters had been retyped, the keyword stayed, and a
`# Safety` section was written to satisfy the ratchet — "the window must be
live" on a `Win`, "`tv` must point at an initialized typval" on a `&TypVal`,
"main thread only" on editor code that is main-thread everywhere. None of
those is a contract: they name what the type already holds. The fix is to
delete the keyword, never to keep it and write the section.

`unsafe_fns_without_raw_params` counts what is left of that class, and this
file is its floor. A row here says the signature shows no raw pointer **and
the `unsafe` is still earned** — that the obligation is real and lives
somewhere the type system cannot reach.

## What a row has to say

Every row opens with one of four reason classes, and nothing else qualifies:

- **`alloc:`** — it returns raw memory, or hands ownership of an allocation
  across the call, and the caller is the one who has to free it.
- **`cap:`** — a capacity the caller must honour: a buffer of at least _n_
  bytes, an index the callee will not bounds-check, a length in elements
  rather than bytes.
- **`ffi:`** — it is an FFI callback or runs on a foreign frame: the C
  library, the `lua_State` stack, or a signal/thread context whose invariants
  no Rust type describes.
- **`field:`** — a raw-pointer _field_ reached through a borrowed struct. The
  parameter is a reference, so the needle sees nothing, but the body walks a
  pointer the struct holds. This is the string and state phases' debt: the row
  leaves when the field does, and it names the field so a reader can tell.

A `NonNull<T>` parameter is a raw pointer the needle deliberately does not
match, so it takes a row (`alloc:` or `field:`, whichever the call is doing).

## How the ratchet enforces it

`unsafe_fn_allowlist` in `scripts/ratchet.py` reads the table below. The key
is the item: `Owner::name` for a method — a bare name does not excuse a
method, or the row would quietly cover every type that grows one — and `name`
for a free function.

The file column is checked too: a row excuses its item only in the file it
names (a private helper defined in two files lists both, comma-separated), and
a row naming a file that does not define the item fails the run.

The list prunes itself. `check_unsafe_fn_allowlist` fails the run when a row
excuses nothing: its function is gone, it moved out of the file the row names,
or it now carries a raw pointer or a C ABI and the needle would skip it anyway.
A row that stops being needed has to leave in the same commit, the way a
`PERIMETER` entry does.

`unsafe_fns_without_raw_params` is 0 with this list as the floor, and every
`unsafe fn` left in the tree — 240, all on the perimeter
([docs/unsafe-audit.md](unsafe-audit.md)) — has a `# Safety` section naming
what its caller must hold. Adding a row lowers the count, so a row is a claim
a reviewer reads: the bar is the four classes above and nothing softer.

| item                          | file                                    | why the type system cannot hold it                                                                                                                  |
| ----------------------------- | --------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------- |
| `CollectedMark::is_collected` | `crates/nvim/src/lua/stack/userdata.rs` | `field:` reads the collected record through the mark's pointer into a userdata block, which only the Lua collector's reachability keeps allocated.  |
| `TypVal::bit_copy`            | `crates/nvim/src/eval/typval/handle.rs` | `alloc:` duplicates the value's bits without touching its refcount, so exactly one of the two may ever be released.                                 |
| `try_to_free_memory`          | `crates/nvim/src/memory/mod.rs`         | `alloc:` drops the scrollback and the memfile caches from inside an allocation.                                                                     |
| `towlower`                    | `crates/nvim/src/os/cshim.rs`           | `ffi:` stands in for the libc declaration under `cfg(miri)` and keeps its `unsafe`, or every caller's block becomes `unused_unsafe` in that lane.   |
| `towupper`                    | `crates/nvim/src/os/cshim.rs`           | `ffi:` as [`towlower`].                                                                                                                             |
| `Buf::ml_unlent`              | `crates/nvim/src/winlayer/deref.rs`     | `field:` lends the memline through the handle's slot pointer for any lifetime; the caller ends the borrow before anything else reaches the memline. |
| `Buf::ml_unlent_mut`          | `crates/nvim/src/winlayer/deref.rs`     | `field:` as [`Buf::ml_unlent`], mutably, for one write.                                                                                             |
| `Child::pty`                  | `crates/nvim/src/event/proc.rs`         | `field:` lends the child's `PtyProc` through the handle's `Proc` pointer as `&mut`, which no other borrow may overlap.                              |
| `Watcher::address`            | `crates/nvim/src/event/socket/mod.rs`   | `field:` lends the watcher's address through the handle's pointer, valid only while the watcher lives and nothing rewrites the address.             |
| `ReadStream::open_fd`         | `crates/nvim/src/event/rstream.rs`      | `ffi:` registers the stream with libuv by address: it must stay put and alive until the handle's close callback, which outlasts every borrow.       |
| `ReadStream::open`            | `crates/nvim/src/event/rstream.rs`      | `ffi:` as [`ReadStream::open_fd`], for a stream whose common half the socket layer connected.                                                       |
| `Incoming::accept`            | `crates/nvim/src/event/socket/mod.rs`   | `ffi:` registers the caller's stream with libuv by address, as [`ReadStream::open_fd`].                                                             |
| `socket_connect`              | `crates/nvim/src/event/socket/mod.rs`   | `ffi:` as [`Incoming::accept`]; a failed connect has closed the handle before it returns.                                                           |
| `SignalWatcher::open`         | `crates/nvim/src/event/signal.rs`       | `ffi:` registers the watcher with libuv by address, as [`ReadStream::open_fd`].                                                                     |
