# A/B benches

Five whole-binary timing canaries, each driving one family hard enough that a
few percent of its work is visible in a total. They are not part of any gate:
nothing runs them but a person asking whether a change cost something.

| bench        | what it drives                                                                    |
| ------------ | --------------------------------------------------------------------------------- |
| `evalbench`  | the eval substrate: typval allocation, copy, encode/decode, function calls        |
| `inbench`    | input and the command line: typeahead, mappings, termcodes, completion, messages  |
| `mlbench`    | the memline hot path: `ml_get`, `ml_append`, `ml_delete`, byte/line offsets       |
| `scrbench`   | the screen pipeline: grid diffing, `win_line`, syntax, statusline, the popup menu |
| `spellbench` | spell checking and suggestion: the word tries, `z=`, the SAL engine               |

Each is a `.lua` canary plus a `.sh` driver; the driver's two modes share
`common.sh`. The `.lua` files carry the phase map — which code each phase is
the canary for — and the noise each phase was measured to have. Read that
before quoting a phase.

`just benchmark` is a different thing: the busted suite in the parent
directory, which times library-level operations and runs against a **dev**
build. These drive a whole binary and are meant for two of them.

## Instructions retired — the answer that is not noise

```
test/benchmark/ab/scrbench.sh --cachegrind /path/to/nvim
```

One deterministic run, no repetition, printing the instructions the process
retired:

```
scrbench	Ir	28,018,423,927
```

Do that for each side and compare the two numbers by hand. **This is the
measurement that settles a perf question.** It is deterministic, so the same
binary answers the same number to a few parts per million; wall clock on this
machine drifts several percent between sessions and cannot resolve that. It
costs about 50x the wall-clock run.

It does not need both orders — there is no ordering effect in an instruction
count.

**It is deterministic to a few parts per million for four of the five benches,
and not for `evalbench`.** Measured: three runs of one binary, minutes apart,
spread 15 ppm on `mlbench` and 11 ppm on `spellbench` — and **3,626 ppm
(0.36 %) on `evalbench`**. Disabling ASLR (`setarch -R`) narrows that to
0.126 % without closing it. There is a cross-session component as well: an
`evalbench` floor recorded in one session came back 0.163 % different from the
same binary weeks later, where `mlbench`'s reproduced to 2 ppm. So: **measure
both sides in one sitting**, never against a stored number, and treat an
`evalbench` difference under about 0.4 % as no difference at all.

**It does need `codegen-units = 1` on both sides.** The default release
profile splits the crate into sixteen units, and which unit a function lands
in decides what gets inlined into it; adding a function to a module, or
changing the size of one three call levels away, repartitions the crate and
moves real instructions. Measured: two commits whose diff `scrbench` never
executes — the whole of it is in `diff/`, and no phase of the canary turns
diff mode on — came out 170 M Ir (0.60 %) apart at the default, all of it
inside `drawline::win_line`, and **0.01 % apart** with `codegen-units = 1`.
So a sub-percent difference at the default is not evidence of anything. Build
both sides with

```toml
[profile.release]
codegen-units = 1
```

before believing a number under a percent or two, and say in the writeup
which way the binaries were built.

When a number moves and the function is not obvious, ask callgrind:

```
valgrind --tool=callgrind --cache-sim=no --branch-sim=no \
  --callgrind-out-file=cg.out <the same nvim invocation>
callgrind_annotate cg.out | head -40
```

Read **self (exclusive)** Ir, not inclusive: LuaJIT's `lj_vm_return` /
`lj_BC_FUNCC` recursion cycles inflate inclusive totals into the millions of
percent, and most of the accessors in this tree are `#[inline(always)]`, so
their cost belongs to their callers anyway.

## Wall clock — when the question is time and not work

```
test/benchmark/ab/scrbench.sh <nvim-A> <nvim-B> [rounds]
```

The two binaries run alternately inside one session and the **minimum** per
phase is reported for each, with the percentage change. Cross-session minima
drift several percent here, so a stored number from last week is worthless and
only an interleaved run against a binary kept from the other commit means
anything.

Four rules, each of which has been paid for at least once:

1. **Build both sides `--release` with `codegen-units = 1`.** 16-CGU release
   timing is layout-sensitive and has produced a +21 % swing on a subsystem
   the change did not touch. Build each from a `git archive` of its commit
   into its own `CARGO_TARGET_DIR`:

   ```
   git archive <rev> | tar -x -C /tmp/side-a
   cd /tmp/side-a && CARGO_TARGET_DIR=/tmp/side-a/target \
     RUSTFLAGS="-C codegen-units=1" cargo build --release
   ```

2. **Take the same-binary floor first.** Pass one binary twice; a phase whose
   self-A/B swing exceeds the effect resolves nothing. The honest floor is
   not a byte-identical copy but a _dead-code padding_ build — the same source
   plus one never-called `#[inline(never)]` function — because two
   functionally identical binaries move more than most real changes do.

3. **Run both orders.** A same-sign-both-orders delta is a finding; a
   single-order 1–3 % is placement.

4. **Read TOTAL and the sign pattern, not one phase.** Every bench has a
   `ctl` phase that the families under test do not reach; it is the noise
   floor, and a phase whose change is not clearly larger than `ctl`'s says
   nothing.

## Arguments

`<nvim-A> <nvim-B> [rounds]`, and nothing else is positional. The runtime is
this tree's `runtime/`; set `VIMRUNTIME` to point somewhere else — both sides
must read the same one, which is why it is not per-binary. Per-bench knobs are
environment variables, documented in each driver's header
(`SCRBENCH_COLS`, `EVALBENCH_SCALE`, `INBENCH_WORK`, `MLBENCH_LINES`, …).

Rounds default to 7, 8 or 12 depending on the bench, chosen so the floor is
small enough to be useful; fewer is not.
