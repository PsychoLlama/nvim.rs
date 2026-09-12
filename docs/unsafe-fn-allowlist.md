# The `unsafe fn` allowlist

A quarter of the tree's `unsafe fn`s take no raw pointer at all. Their
parameters were retyped in an earlier phase, the keyword stayed, and a
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

The list prunes itself. `check_unsafe_fn_allowlist` fails the run when a row
excuses nothing: its function is gone, or it now carries a raw pointer or a C
ABI and the needle would skip it anyway. A row that stops being needed has to
leave in the same commit, the way a `PERIMETER` entry does.

The phase exit is `unsafe_fns_without_raw_params` = 0 with this list as the
floor. Adding a row lowers the count, so a row is a claim a reviewer reads:
the bar is the four classes above and nothing softer.

| item                     | file                                  | why the type system cannot hold it                                                                         |
| ------------------------ | ------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `Matches::with_capacity` | `crates/nvim/src/optionstr/expand.rs` | `cap:` the array it allocates is sized for `capacity` entries and [`Matches::push`] bounds-checks nothing. |
