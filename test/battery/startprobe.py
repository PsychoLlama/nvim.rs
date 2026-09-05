#!/usr/bin/env python3
"""Startup differential probe: the process is the observable.

Promoted into test/battery/ at B16-5 from the phase-14 scratchpad
(`1785211338-phase-14-scripts/m/startup-probe.py`) and extended with the
`log-*` cases, which are the only view of log.rs anywhere in the toolbox
(`core/log_spec` is 122 lines and asserts almost nothing).

It is a PAIRED probe -- there is no stored baseline, only two binaries
compared.  B16 keeps one at 2ecc9b69f7 in ~/agents/scratch/b16/, with
`p-base.txt` next to it; run:

    startprobe.py ~/agents/scratch/b16/nvim-2ecc9b69f7 p-base.txt
    startprobe.py ./target/debug/nvim p-cur.txt
    diff p-base.txt p-cur.txt

**Copy `runtime/doc/tags` into any worktree build before comparing**: it
is generated and gitignored, so a worktree binary's `:help` silently
does nothing and every help case diverges for a reason that is not code
(B16-2's trap).

Runs one nvim process per case and records the exit status, stdout and
stderr, normalised so two binaries built from different worktrees compare
byte-for-byte.  Unlike the editor-state probes of the earlier slices this
one needs no restart framing: a case that aborts only kills its own
process, and the abort shows up as its exit status.

    startup-probe.py <nvim-binary> <out-file> [--only=substr]

The binary is copied to a fixed path first (`run/nvim`) so that anything
derived from argv[0] is identical between the two builds.
"""

import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
# test/battery/startprobe.py -> the checkout two levels up.  $REPO overrides.
REPO = Path(os.environ.get("REPO") or HERE.parent.parent)
RUNTIME = REPO / "runtime"
# Not under HERE: this lives in test/battery/, which is a source directory
# and not a scratch one.  Short and in /tmp on purpose, like every sweep's
# work directory: nvim elides a message wider than the (headless, 80
# column) screen, and a long fixture path is what turns an error text into
# `...<tail>`.  SUBS catches that spelling too, but not needing it is
# better.  $STARTPROBE_WORK overrides.
SCRATCH = Path(os.environ.get("STARTPROBE_WORK") or "/tmp/startprobe")
FIX = SCRATCH / "fix"
RUN = SCRATCH / "run"
# 20 s was enough on an idle machine and produced two load-sensitive
# TIMEOUTs (`headless-no-c`, `c-errors-only`) whenever it was not.  Those
# two are *meant* to time out -- they have no `qa!` -- so they carry
# their own short budget; everything else gets a generous one.
TIMEOUT = 45


# ---------------------------------------------------------------- fixtures


def fixtures() -> None:
    """Build a fixed scratch tree.  Everything here is content-addressed by
    hand: no timestamps, no random names, nothing that reaches a message."""
    if FIX.exists():
        shutil.rmtree(FIX)
    FIX.mkdir(parents=True)
    (FIX / "home").mkdir()
    (FIX / "cfg").mkdir()
    (FIX / "data").mkdir()
    (FIX / "state").mkdir()
    (FIX / "cache").mkdir()
    (FIX / "run").mkdir()
    (FIX / "xdgrun").mkdir()

    write = lambda name, text: (FIX / name).write_text(text)

    write("a.txt", "alpha one\nalpha two\nalpha three\n")
    write("b.txt", "beta one\nbeta two\n")
    write("c.txt", "gamma one\n")
    write("d.txt", "delta one\n")
    write("utf.txt", "\u00e4\u00f6\u00fc \u4e2d\u6587 \U0001f600\nsecond\n")
    write("empty.txt", "")
    write("noeol.txt", "no trailing newline")

    # A user config that says so, and one that fails.
    write(
        "vimrc.vim",
        'echomsg "VIMRC RAN"\nlet g:vimrc = 1\nset shiftwidth=3\n',
    )
    write("badrc.vim", 'echomsg "BADRC"\ncall Nope()\nechomsg "AFTER"\n')
    write("errrc.vim", "set nosuchoption\n")
    write("exrc.vim", 'echomsg "EXRC RAN"\n')

    # A session file.
    write(
        "sess.vim",
        "edit " + str(FIX / "a.txt") + '\nlet g:session = 1\nechomsg "SESSION"\n',
    )

    # Lua scripts for -l.
    write(
        "script.lua",
        "io.stdout:write('lua ran\\n')\n"
        "io.stdout:write('args ' .. vim.inspect(_G.arg) .. '\\n')\n",
    )
    write("errscript.lua", "error('boom')\n")
    write("exitscript.lua", "os.exit(7)\n")
    write("syntaxerr.lua", "this is not lua ((\n")
    write("apiscript.lua", "io.stdout:write(vim.api.nvim_get_mode().mode .. '\\n')\n")

    # Normal-mode script input for -s.
    write("in.vim", "ihello\x1b:wq! " + str(FIX / "out.txt") + "\n")
    # Ex commands for -es < file.
    write("ex.txt", 'echo "ex mode"\nqa!\n')

    # A tags file with a good, a duplicate and a dangling entry.
    write(
        "tagged.c",
        "int alpha;\nint beta;\nint gamma;\n",
    )
    write(
        "tags",
        "alpha\ttagged.c\t/^int alpha;$/\n"
        "beta\ttagged.c\t/^int beta;$/\n"
        "beta\ttagged.c\t/^int beta;$/\n"
        "dangling\tmissing.c\t/^nope$/\n",
    )

    # A quickfix errorfile.
    write("errors.txt", str(FIX / "a.txt") + ":2:1: warning: something\n")

    write("modeline.txt", "x\n# vim: sw=7 ts=7\n")

    # Config trees: user configs, system configs, and an $NVIM_APPNAME one.
    def cfg(path: str, text: str) -> None:
        target = FIX / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text)

    cfg("cfg1/nvim/init.lua", "vim.g.which = 'cfg1 init.lua'\n")
    cfg("cfg2/nvim/init.vim", "let g:which = 'cfg2 init.vim'\n")
    cfg("cfg3/nvim/init.lua", "vim.g.which = 'cfg3 init.lua'\n")
    cfg("cfg3/nvim/init.vim", "let g:which = 'cfg3 init.vim'\n")
    cfg("cfg4/nvim/init.vim", "let g:which = 'cfg4 init.vim'\nset exrc\n")
    cfg("cfgbad/nvim/init.lua", "error('init.lua exploded')\n")
    cfg("cfgapp/myapp/init.lua", "vim.g.which = 'appname init.lua'\n")
    cfg("sys1/nvim/sysinit.vim", "let g:sys = 'sys1'\n")
    cfg("sys2/nvim/sysinit.vim", "let g:sys = 'sys2'\n")
    cfg("exrcdir/.nvim.lua", "vim.g.exrc_ran = true\n")
    cfg("exrcdir/keep.txt", "x\n")

    # A file where a socket should be: `socket_watcher_start` logs
    # "Removing stale socket" and then fails to bind, which is three log
    # lines at three levels in one run.
    write("stale.sock", "")
    (FIX / "logs").mkdir(exist_ok=True)


# ------------------------------------------------------------------- cases


def cases() -> list[tuple[str, dict]]:
    """(label, spec).  spec keys: args, stdin, env, timeout."""
    f = str(FIX)
    out: list[tuple[str, dict]] = []

    def add(label, args, stdin=None, env=None, timeout=TIMEOUT, logfile=False, runs=1):
        out.append(
            (
                label,
                {
                    "args": args,
                    "stdin": stdin,
                    "env": env or {},
                    "timeout": timeout,
                    # `logfile` gives the case its OWN $NVIM_LOG_FILE and
                    # appends the file's contents to the record.  Without
                    # it every case shares one log nobody ever reads.
                    "logfile": logfile,
                    "runs": runs,
                },
            )
        )

    hl = ["--headless", "-u", "NONE", "-i", "NONE"]

    # -- the null case and the informational exits ------------------------
    add("null", hl + ["+q"])
    add("null-qa", hl + ["-c", "qa!"])
    add("help", ["--help"])
    add("help-h", ["-h"])
    add("help-?", ["-?"])
    add("version", ["--version"])
    add("version-v", ["-v"])
    add("api-info", ["--api-info"])
    add("api-info-headless", ["--headless", "--api-info"])

    # -- argument errors: exact text and exit status ----------------------
    add("bad-long", hl + ["--nosuchoption", "+q"])
    add("bad-long-eq", hl + ["--nosuch=1", "+q"])
    add("bad-short", hl + ["-Y", "+q"])
    add("bad-short-cluster", hl + ["-nY", "+q"])
    add("cmd-noarg", ["--headless", "-u", "NONE", "-i", "NONE", "--cmd"])
    add("c-noarg", ["--headless", "-u", "NONE", "-i", "NONE", "-c"])
    add("u-noarg", ["--headless", "-i", "NONE", "-u"])
    add("i-noarg", ["--headless", "-u", "NONE", "-i"])
    add("s-noarg", ["--headless", "-u", "NONE", "-i", "NONE", "-s"])
    add("S-noarg", ["--headless", "-u", "NONE", "-i", "NONE", "-c", "qa!", "-S"])
    add("t-noarg", ["--headless", "-u", "NONE", "-i", "NONE", "-t"])
    add("q-noarg-ok", hl + ["-q", "+q"])
    add("w-noarg", ["--headless", "-u", "NONE", "-i", "NONE", "-w"])
    add("W-noarg", ["--headless", "-u", "NONE", "-i", "NONE", "-W"])
    add("listen-noarg", ["--headless", "-u", "NONE", "-i", "NONE", "--listen"])
    add("server-noarg", ["--headless", "-u", "NONE", "-i", "NONE", "--server"])
    add(
        "startuptime-noarg", ["--headless", "-u", "NONE", "-i", "NONE", "--startuptime"]
    )
    add("l-noarg", ["--headless", "-u", "NONE", "-i", "NONE", "-l"])
    add("too-many-files", hl + ["-t", "alpha", f + "/a.txt", "+q"])

    # The MAX_ARG_CMDS ceilings.  10 is the limit for both arrays.
    for n in (9, 10, 11, 12):
        add(
            f"cmd-x{n}",
            hl + sum([["--cmd", f"let g:n{i}=1"] for i in range(n)], []) + ["+q"],
        )
        add(
            f"c-x{n}",
            hl + sum([["-c", f"let g:n{i}=1"] for i in range(n)], []) + ["+q"],
        )
        add(f"plus-x{n}", hl + [f"+let g:n{i}=1" for i in range(n)] + ["-c", "qa!"])

    # -- ordering of --cmd / -c / +cmd / -S -------------------------------
    add(
        "order-cmd-c",
        hl
        + [
            "--cmd",
            'echomsg "1 precmd"',
            "--cmd",
            'echomsg "2 precmd"',
            "-c",
            'echomsg "3 cmd"',
            "-c",
            'echomsg "4 cmd"',
            '+echomsg "5 plus"',
            "-c",
            "qa!",
        ],
    )
    add(
        "order-S",
        hl + [f + "/b.txt", "-S", f + "/sess.vim", "-c", "echo bufname()", "-c", "qa!"],
    )
    add("plus-linenr", hl + [f + "/a.txt", "+2", "-c", "echo line('.')", "-c", "qa!"])
    add("plus-bare", hl + [f + "/a.txt", "+", "-c", "echo line('.')", "-c", "qa!"])
    add(
        "plus-huge",
        hl + [f + "/a.txt", "+99999999", "-c", "echo line('.')", "-c", "qa!"],
    )
    add(
        "plus-overflow",
        hl
        + [f + "/a.txt", "+99999999999999999999", "-c", "echo line('.')", "-c", "qa!"],
    )
    add(
        "plus-pattern",
        hl + [f + "/a.txt", "+/two", "-c", "echo line('.')", "-c", "qa!"],
    )

    # -- window / tab counts, including the silly ones --------------------
    for flag in ("-o", "-O", "-p"):
        for n in (
            "",
            "0",
            "1",
            "2",
            "9",
            "99",
            "999",
            "2147483647",
            "99999999999999999999",
        ):
            add(
                f"win{flag}{n or 'bare'}",
                hl
                + [
                    flag + n,
                    f + "/a.txt",
                    f + "/b.txt",
                    "-c",
                    "echo winnr('$') tabpagenr('$')",
                    "-c",
                    "qa!",
                ],
                timeout=40,
            )
    add("win-o-nofiles", hl + ["-o3", "-c", "echo winnr('$')", "-c", "qa!"])
    add(
        "win-diff",
        hl
        + [
            "-d",
            f + "/a.txt",
            f + "/b.txt",
            "-c",
            "echo winnr('$') &diff",
            "-c",
            "qa!",
        ],
    )
    add("win-diff-one", hl + ["-d", f + "/a.txt", "-c", "echo winnr('$')", "-c", "qa!"])
    add(
        "win-diff-four",
        hl
        + [
            "-d",
            f + "/a.txt",
            f + "/b.txt",
            f + "/c.txt",
            f + "/d.txt",
            "-c",
            "echo winnr('$')",
            "-c",
            "qa!",
        ],
    )

    # -- the vimrc paths --------------------------------------------------
    add(
        "u-NONE",
        [
            "--headless",
            "-i",
            "NONE",
            "-u",
            "NONE",
            "-c",
            "echo &shiftwidth",
            "-c",
            "qa!",
        ],
    )
    add(
        "u-NORC",
        [
            "--headless",
            "-i",
            "NONE",
            "-u",
            "NORC",
            "-c",
            "echo &shiftwidth",
            "-c",
            "qa!",
        ],
    )
    add(
        "u-DEFAULTS",
        [
            "--headless",
            "-i",
            "NONE",
            "-u",
            "DEFAULTS",
            "-c",
            "echo &shiftwidth",
            "-c",
            "qa!",
        ],
    )
    add(
        "u-file",
        [
            "--headless",
            "-i",
            "NONE",
            "-u",
            f + "/vimrc.vim",
            "-c",
            "echo g:vimrc",
            "-c",
            "qa!",
        ],
    )
    add(
        "u-missing",
        [
            "--headless",
            "-i",
            "NONE",
            "-u",
            f + "/nope.vim",
            "-c",
            "echo 1",
            "-c",
            "qa!",
        ],
    )
    add("u-bad", ["--headless", "-i", "NONE", "-u", f + "/badrc.vim", "-c", "qa!"])
    add("u-erropt", ["--headless", "-i", "NONE", "-u", f + "/errrc.vim", "-c", "qa!"])
    add("clean", ["--headless", "--clean", "-c", "echo &shiftwidth", "-c", "qa!"])
    add(
        "noplugin",
        ["--headless", "-i", "NONE", "--noplugin", "-c", "echo 1", "-c", "qa!"],
    )
    add(
        "home-vimrc",
        ["--headless", "-i", "NONE", "-c", "echo get(g:,'vimrc',0)", "-c", "qa!"],
        env={"XDG_CONFIG_HOME": f + "/cfg"},
    )
    add(
        "viminit",
        hl[:-2] + ["-i", "NONE", "-c", "echo get(g:,'viminit',0)", "-c", "qa!"],
        env={"VIMINIT": "let g:viminit = 1"},
    )
    add(
        "viminit-u-NONE",
        [
            "--headless",
            "-i",
            "NONE",
            "-u",
            "NONE",
            "-c",
            "echo get(g:,'viminit',0)",
            "-c",
            "qa!",
        ],
        env={"VIMINIT": "let g:viminit = 1"},
    )
    add(
        "exinit",
        ["--headless", "-i", "NONE", "-c", "echo get(g:,'exinit',0)", "-c", "qa!"],
        env={"EXINIT": "let g:exinit = 1"},
    )
    add("nvim-appname-bad", hl + ["+q"], env={"NVIM_APPNAME": "/absolute/bad"})
    add("nvim-appname-dotdot", hl + ["+q"], env={"NVIM_APPNAME": "../escape"})
    add("nvim-appname-ok", hl + ["+q"], env={"NVIM_APPNAME": "probeapp"})

    # -- exrc -------------------------------------------------------------
    add("exrc-off", hl + ["-c", "echo &exrc", "-c", "qa!"])
    add(
        "exrc-on",
        ["--headless", "-i", "NONE", "-c", "echo get(g:,'exrc_ran',0)", "-c", "qa!"],
        env={"XDG_CONFIG_HOME": f + "/cfg4", "PROBE_CWD": f + "/exrcdir"},
    )

    # -- where the config comes from --------------------------------------
    said = ["-c", "echo get(g:,'which','-') get(g:,'sys','-')", "-c", "qa!"]
    base = ["--headless", "-i", "NONE"]
    add("cfg-home-lua", base + said, env={"XDG_CONFIG_HOME": f + "/cfg1"})
    add("cfg-home-vim", base + said, env={"XDG_CONFIG_HOME": f + "/cfg2"})
    add("cfg-home-both", base + said, env={"XDG_CONFIG_HOME": f + "/cfg3"})
    add("cfg-home-badlua", base + said, env={"XDG_CONFIG_HOME": f + "/cfgbad"})
    add("cfg-home-missing", base + said, env={"XDG_CONFIG_HOME": f + "/nodir"})
    add(
        "cfg-appname",
        base + said,
        env={"XDG_CONFIG_HOME": f + "/cfgapp", "NVIM_APPNAME": "myapp"},
    )
    add(
        "cfg-dirs-user",
        base + said,
        env={"XDG_CONFIG_HOME": f + "/nodir", "XDG_CONFIG_DIRS": f + "/cfg1"},
    )
    add(
        "cfg-dirs-user-slash",
        base + said,
        env={"XDG_CONFIG_HOME": f + "/nodir", "XDG_CONFIG_DIRS": f + "/cfg1/"},
    )
    add(
        "cfg-dirs-user-second",
        base + said,
        env={
            "XDG_CONFIG_HOME": f + "/nodir",
            "XDG_CONFIG_DIRS": f + "/nodir:" + f + "/cfg2",
        },
    )
    add(
        "cfg-dirs-user-first-wins",
        base + said,
        env={
            "XDG_CONFIG_HOME": f + "/nodir",
            "XDG_CONFIG_DIRS": f + "/cfg1:" + f + "/cfg2",
        },
    )
    add(
        "cfg-sys",
        base + said,
        env={"XDG_CONFIG_HOME": f + "/nodir", "XDG_CONFIG_DIRS": f + "/sys1"},
    )
    add(
        "cfg-sys-slash",
        base + said,
        env={"XDG_CONFIG_HOME": f + "/nodir", "XDG_CONFIG_DIRS": f + "/sys1/"},
    )
    add(
        "cfg-sys-second",
        base + said,
        env={
            "XDG_CONFIG_HOME": f + "/nodir",
            "XDG_CONFIG_DIRS": f + "/nodir:" + f + "/sys2",
        },
    )
    add(
        "cfg-sys-first-wins",
        base + said,
        env={
            "XDG_CONFIG_HOME": f + "/nodir",
            "XDG_CONFIG_DIRS": f + "/sys1:" + f + "/sys2",
        },
    )
    add(
        "cfg-sys-and-user",
        base + said,
        env={"XDG_CONFIG_HOME": f + "/cfg1", "XDG_CONFIG_DIRS": f + "/sys1"},
    )
    add(
        "cfg-dirs-empty",
        base + said,
        env={"XDG_CONFIG_HOME": f + "/nodir", "XDG_CONFIG_DIRS": ""},
    )
    add(
        "cfg-dirs-colons",
        base + said,
        env={"XDG_CONFIG_HOME": f + "/nodir", "XDG_CONFIG_DIRS": "::" + f + "/sys1:"},
    )
    add(
        "cfg-u-NONE-beats-home",
        ["--headless", "-i", "NONE", "-u", "NONE"] + said,
        env={"XDG_CONFIG_HOME": f + "/cfg1", "XDG_CONFIG_DIRS": f + "/sys1"},
    )
    add(
        "cfg-clean-beats-home",
        ["--headless", "--clean"] + said,
        env={"XDG_CONFIG_HOME": f + "/cfg1", "XDG_CONFIG_DIRS": f + "/sys1"},
    )
    add(
        "cfg-viminit-beats-home",
        base + said,
        env={"XDG_CONFIG_HOME": f + "/cfg1", "VIMINIT": "let g:which = 'VIMINIT'"},
    )
    add(
        "cfg-exinit-when-nothing",
        base + said,
        env={"XDG_CONFIG_HOME": f + "/nodir", "EXINIT": "let g:which = 'EXINIT'"},
    )
    add(
        "cfg-es-skips",
        ["-es", "-i", "NONE"],
        stdin="echo get(g:,'which','-')\nqa!\n",
        env={"XDG_CONFIG_HOME": f + "/cfg1"},
    )

    # -- ex / silent modes ------------------------------------------------
    add("es", ["-es", "-u", "NONE", "-i", "NONE"], stdin='echo "es ran"\nqa!\n')
    add("Es", ["-Es", "-u", "NONE", "-i", "NONE"], stdin='echo "Es ran"\nqa!\n')
    add("es-mode", ["-es", "-u", "NONE", "-i", "NONE"], stdin="echo mode(1)\nqa!\n")
    add("Es-mode", ["-Es", "-u", "NONE", "-i", "NONE"], stdin="echo mode(1)\nqa!\n")
    add("es-err", ["-es", "-u", "NONE", "-i", "NONE"], stdin="call Nope()\nqa!\n")
    add("es-cq", ["-es", "-u", "NONE", "-i", "NONE"], stdin="cquit 3\n")
    add("e-dash", ["-e", "-u", "NONE", "-i", "NONE", "-s"], stdin='echo "e -s"\nqa!\n')
    add("E-dash", ["-E", "-u", "NONE", "-i", "NONE", "-s"], stdin='echo "E -s"\nqa!\n')
    add("es-file", ["-es", "-u", "NONE", "-i", "NONE", f + "/a.txt"], stdin="%p\nqa!\n")

    # -- -l lua -----------------------------------------------------------
    add("l-script", ["-u", "NONE", "-i", "NONE", "-l", f + "/script.lua"])
    add(
        "l-args",
        ["-u", "NONE", "-i", "NONE", "-l", f + "/script.lua", "one", "two", "--three"],
    )
    add("l-err", ["-u", "NONE", "-i", "NONE", "-l", f + "/errscript.lua"])
    add("l-exit", ["-u", "NONE", "-i", "NONE", "-l", f + "/exitscript.lua"])
    add("l-syntax", ["-u", "NONE", "-i", "NONE", "-l", f + "/syntaxerr.lua"])
    add("l-missing", ["-u", "NONE", "-i", "NONE", "-l", f + "/nope.lua"])
    add("l-api", ["-u", "NONE", "-i", "NONE", "-l", f + "/apiscript.lua"])
    add(
        "l-after-file",
        ["-u", "NONE", "-i", "NONE", f + "/a.txt", "-l", f + "/script.lua"],
    )

    # -- stdin as a buffer ------------------------------------------------
    add(
        "stdin-dash",
        hl + ["-", "-c", "%p", "-c", "qa!"],
        stdin="from stdin 1\nfrom stdin 2\n",
    )
    add("stdin-dash-empty", hl + ["-", "-c", "echo line('$')", "-c", "qa!"], stdin="")
    add(
        "stdin-dash-plus-file",
        hl + ["-", f + "/a.txt", "-c", "echo argc()", "-c", "qa!"],
        stdin="x\n",
    )
    add("stdin-dash-twice", hl + ["-", "-", "-c", "qa!"], stdin="x\n")

    # -- verbosity and startuptime shapes ---------------------------------
    for lvl in ("", "0", "1", "2", "9", "12", "16"):
        add(
            f"V{lvl or 'bare'}",
            ["--headless", "-u", "NONE", "-i", "NONE", "-V" + lvl, "+q"],
        )
    add(
        "V-file",
        ["--headless", "-u", "NONE", "-i", "NONE", "-V1" + f + "/verbose.log", "+q"],
    )
    add("startuptime", hl + ["--startuptime", f + "/st.log", "+q"])
    add("startuptime-dash", hl + ["--startuptime", "-", "+q"])
    add("startuptime-bad", hl + ["--startuptime", f + "/nodir/st.log", "+q"])

    # -- file arguments ---------------------------------------------------
    add(
        "file-one",
        hl + [f + "/a.txt", "-c", "echo expand('%:t') line('$')", "-c", "qa!"],
    )
    add(
        "file-many",
        hl
        + [
            f + "/a.txt",
            f + "/b.txt",
            f + "/c.txt",
            "-c",
            "echo argc() expand('%:t')",
            "-c",
            "qa!",
        ],
    )
    add(
        "file-missing",
        hl + [f + "/nope.txt", "-c", "echo line('$') &modified", "-c", "qa!"],
    )
    add(
        "file-utf8",
        hl + [f + "/utf.txt", "-c", "echo strchars(getline(1))", "-c", "qa!"],
    )
    add("file-noeol", hl + [f + "/noeol.txt", "-c", "echo &endofline", "-c", "qa!"])
    add("file-empty", hl + [f + "/empty.txt", "-c", "echo line('$')", "-c", "qa!"])
    add("file-modeline", hl + [f + "/modeline.txt", "-c", "echo &sw", "-c", "qa!"])
    add(
        "file-dashdash",
        hl + ["-c", "echo expand('%')", "-c", "qa!", "--", "-weird.txt"],
    )
    add(
        "file-dashdash-opt",
        hl + ["-c", "echo argc()", "-c", "qa!", "--", "--cmd", "-c"],
    )
    add(
        "file-literal",
        hl + ["--literal", f + "/a*.txt", "-c", "echo expand('%:t')", "-c", "qa!"],
    )
    add("file-wildcard", hl + [f + "/a*.txt", "-c", "echo expand('%:t')", "-c", "qa!"])

    # -- read-only, modifiable, binary, noswap ----------------------------
    add("R", hl + ["-R", f + "/a.txt", "-c", "echo &readonly &modifiable", "-c", "qa!"])
    add("m", hl + ["-m", f + "/a.txt", "-c", "echo &write", "-c", "qa!"])
    add("M", hl + ["-M", f + "/a.txt", "-c", "echo &write &modifiable", "-c", "qa!"])
    add("b", hl + ["-b", f + "/a.txt", "-c", "echo &binary", "-c", "qa!"])
    add("n", hl + ["-n", f + "/a.txt", "-c", "echo &swapfile", "-c", "qa!"])
    add("Z", hl + ["-Z", "-c", "echo 1", "-c", "qa!"])
    add("A", hl + ["-A", "-c", "echo &arabic", "-c", "qa!"])
    add("H", hl + ["-H", "-c", "echo &rightleft", "-c", "qa!"])

    # -- recovery ---------------------------------------------------------
    add("r-list", hl + ["-r"])
    add("L-list", hl + ["-L"])
    add("r-missing", hl + ["-r", f + "/nope.swp"])

    # -- tags and quickfix ------------------------------------------------
    add(
        "tag",
        hl + ["-t", "alpha", "-c", "echo expand('%:t') line('.')", "-c", "qa!"],
        env={"PROBE_CWD": f},
    )
    add(
        "tag-dup",
        hl + ["-t", "beta", "-c", "echo expand('%:t')", "-c", "qa!"],
        env={"PROBE_CWD": f},
    )
    add(
        "tag-dangling",
        hl + ["-t", "dangling", "-c", "echo expand('%:t')", "-c", "qa!"],
        env={"PROBE_CWD": f},
    )
    add(
        "tag-missing",
        hl + ["-t", "nosuchtag", "-c", "echo expand('%:t')", "-c", "qa!"],
        env={"PROBE_CWD": f},
    )
    add(
        "q-file",
        hl
        + ["-q", f + "/errors.txt", "-c", "echo expand('%:t') line('.')", "-c", "qa!"],
    )
    add("q-missing", hl + ["-q", f + "/nope.txt", "-c", "echo 1", "-c", "qa!"])
    add("q-dash", hl + ["-q", "-c", "echo 1", "-c", "qa!"])

    # -- shada ------------------------------------------------------------
    add(
        "i-NONE",
        [
            "--headless",
            "-u",
            "NONE",
            "-i",
            "NONE",
            "-c",
            "echo &shadafile",
            "-c",
            "qa!",
        ],
    )
    add(
        "i-file",
        [
            "--headless",
            "-u",
            "NONE",
            "-i",
            f + "/probe.shada",
            "-c",
            "echo &shadafile",
            "-c",
            "qa!",
        ],
    )

    # -- scriptin / scriptout ---------------------------------------------
    add(
        "s-script",
        hl + [f + "/c.txt", "-s", f + "/in.vim", "-c", "echo 'after'"],
        timeout=25,
    )
    add("s-missing", hl + ["-s", f + "/nope.vim", "+q"])
    add("w-out", hl + ["-w", f + "/w.log", "-c", "qa!"])
    add("W-out", hl + ["-W", f + "/W.log", "-c", "qa!"])

    # -- servers ----------------------------------------------------------
    add(
        "listen-ok",
        hl + ["--listen", f + "/run/sock", "-c", "echo serverlist()", "-c", "qa!"],
    )
    add("listen-bad", hl + ["--listen", f + "/nodir/sock", "-c", "qa!"])
    add("listen-empty", hl + ["--listen", "", "-c", "qa!"])
    add("server-noserver", hl + ["--server", f + "/run/nosock", "--remote-expr", "1"])
    add("remote-noserver", hl + ["-c", "qa!", "--remote", f + "/a.txt"])
    add("remote-expr-noserver", hl + ["--remote-expr", "1+1"])
    add("remote-bad-sub", hl + ["--remote-nosuch", f + "/a.txt"])

    # -- exit paths -------------------------------------------------------
    add("exit-cq", hl + ["-c", "cquit"])
    add("exit-cq-n", hl + ["-c", "cquit 42"])
    add("exit-cq-0", hl + ["-c", "cquit 0"])
    add("exit-qa-modified", hl + ["-c", "put ='x'", "-c", "qa", "-c", "qa!"])
    add("exit-vimleave", hl + ["-c", "au VimLeave * echomsg 'LEAVE'", "-c", "qa!"])
    add(
        "exit-vimleavepre",
        hl + ["-c", "au VimLeavePre * echomsg 'LEAVEPRE'", "-c", "qa!"],
    )
    add("exit-vimleave-err", hl + ["-c", "au VimLeave * call Nope()", "-c", "qa!"])
    add(
        "exit-defer",
        hl
        + [
            "-c",
            "func F()\ndefer execute('echomsg \"DEFER\"')\nendfunc",
            "-c",
            "call F()",
            "-c",
            "qa!",
        ],
    )
    add("exit-job", hl + ["-c", "call jobstart(['sleep','30'])", "-c", "qa!"])
    add("exit-two-buffers", hl + [f + "/a.txt", f + "/b.txt", "-c", "qa!"])
    add(
        "exit-swapexists",
        hl + ["-c", "au SwapExists * let v:swapchoice='e'", "-c", "qa!"],
    )

    # -- autocommands the startup sequence fires --------------------------
    add(
        "startup-autocmds",
        hl
        + [
            "--cmd",
            "au VimEnter * echomsg 'VimEnter'",
            "--cmd",
            "au UIEnter * echomsg 'UIEnter'",
            "--cmd",
            "au BufEnter * echomsg 'BufEnter ' .. expand('<afile>:t')",
            "--cmd",
            "au BufReadPost * echomsg 'BufReadPost'",
            "--cmd",
            "au SourcePre * echomsg 'SourcePre'",
            f + "/a.txt",
            "-c",
            "qa!",
        ],
    )
    add(
        "v-vim-did-enter",
        hl
        + ["--cmd", "echo v:vim_did_enter", "-c", "echo v:vim_did_enter", "-c", "qa!"],
    )
    add("startup-argv", hl + [f + "/a.txt", "-c", "echo v:argv[1:]", "-c", "qa!"])
    add("startup-progname", hl + ["-c", "echo v:progname", "-c", "qa!"])
    add("startup-servername", hl + ["-c", "echo v:servername != ''", "-c", "qa!"])
    add("startup-uis", hl + ["-c", "echo len(nvim_list_uis())", "-c", "qa!"])
    add("startup-lines-cols", hl + ["-c", "echo &lines &columns", "-c", "qa!"])
    add("startup-shada-opt", hl + ["-c", "echo &shada != ''", "-c", "qa!"])

    # -- combinations that historically diverge ---------------------------
    add(
        "headless-no-c", ["--headless", "-u", "NONE", "-i", "NONE"], stdin="", timeout=5
    )
    add(
        "headless-es-both",
        ["--headless", "-es", "-u", "NONE", "-i", "NONE"],
        stdin="qa!\n",
    )
    add(
        "embed-headless-eof",
        ["--embed", "--headless", "-u", "NONE", "-i", "NONE"],
        stdin="",
        timeout=15,
    )
    add("cmd-quits", hl + ["--cmd", "qa!"])
    add("cmd-cquit", hl + ["--cmd", "cquit 5"])
    add("cmd-errors", hl + ["--cmd", "call Nope()", "-c", "qa!"])
    add("c-errors", hl + ["-c", "call Nope()", "-c", "qa!"])
    add("c-errors-only", hl + ["-c", "call Nope()"], timeout=5)

    # -- $NVIM_LOG_FILE, read back (B16-5) --------------------------------
    #
    # log.rs has `core/log_spec` (122 lines) and nothing else behind it,
    # and the probe already pointed $NVIM_LOG_FILE at the fixture and
    # then never looked.  These cases READ THE FILE, which makes
    # `log_write_prefix`'s whole output -- level tag, ISO timestamp,
    # name.pid.channel, func:line, message -- an observable.  The
    # timestamp, pid and source line number are normalised away (the last
    # one because B16-8 and B16-13 renumber every caller of `logmsg`);
    # the level, the function name and the message are not.
    add("log-plain", hl + ["-c", "qa!"], logfile=True)
    add("log-exit-code", hl + ["-c", "cquit 3"], logfile=True)
    add("log-appends-across-runs", hl + ["-c", "qa!"], logfile=True, runs=2)
    # A stale socket file logs at three levels in one run: DBG for the
    # stream close, INF for the removal, WRN for the bind failure.
    add(
        "log-listen-stale",
        hl + ["--listen", f + "/stale.sock", "-c", "qa!"],
        logfile=True,
    )
    add("log-listen-dir", hl + ["--listen", f + "/home", "-c", "qa!"], logfile=True)
    add(
        "log-listen-ok",
        hl + ["--listen", f + "/run/logsock", "-c", "qa!"],
        logfile=True,
    )
    # `os_setenv` failing is the one `log_uv_failure` call site reachable
    # from a script.
    add(
        "log-uv-failure",
        hl + ["-c", "call setenv('', 'x')", "-c", "qa!"],
        logfile=True,
    )
    add(
        "log-embed-eof",
        ["--embed", "--headless", "-u", "NONE", "-i", "NONE"],
        stdin="",
        timeout=15,
        logfile=True,
    )
    add("log-v3", hl + ["-V3", "-c", "qa!"], logfile=True)
    # The fallback path in `open_log_file`: nvim cannot open the file, so
    # it writes the failure through the same prefix writer onto stderr.
    add(
        "log-unwritable",
        hl + ["-c", "qa!"],
        env={"NVIM_LOG_FILE": "/proc/nosuchdir/x.log"},
    )
    add("log-path-is-dir", hl + ["-c", "qa!"], env={"NVIM_LOG_FILE": f + "/home"})
    add("log-path-empty", hl + ["-c", "qa!"], env={"NVIM_LOG_FILE": ""})
    add("log-path-relative", hl + ["-c", "qa!"], env={"NVIM_LOG_FILE": "relative.log"})

    return out


# --------------------------------------------------------------- normalise

SUBS = [
    (re.compile(re.escape(str(FIX)).encode()), b"<FIX>"),
    (re.compile(re.escape(str(RUN)).encode()), b"<RUN>"),
    (re.compile(re.escape(str(RUNTIME)).encode()), b"<RT>"),
    (re.compile(re.escape(str(REPO)).encode()), b"<REPO>"),
    # A message wider than the screen reaches us elided to `...<tail>`, so
    # the escaped-prefix subs above cannot see it and the artifact keeps a
    # host path (`...d/agents/scratch/b16/startprobe/fix/...`, B16).  Anchor
    # on the LAST directory component instead: that survives the elision as
    # long as the tail does, which is exactly when the path is in the way.
    # Two elision markers, not one: a Lua chunk name comes back as
    # `...<tail>`, but `msg_outtrans_long` writes a leading `<` instead, and
    # `:w`'s "[New] 1L" line uses that one.  No trailing `/` either: `-S`'s
    # message names the directory itself.
    (
        re.compile(
            rb"(?:\.\.\.|<)[^\s'\"<>]*?/"
            + re.escape(FIX.name).encode()
            + rb"(?![\w.-])"
        ),
        b"<FIX>",
    ),
    (
        re.compile(
            rb"(?:\.\.\.|<)[^\s'\"<>]*?/"
            + re.escape(RUN.name).encode()
            + rb"(?![\w.-])"
        ),
        b"<RUN>",
    ),
    # Build identity and timings.  `nvim.rs dev-<rev>[-dirty]` is what
    # `:version`/`--version` print first, and it embeds the git rev the
    # binary was built from -- so a PAIRED run of two revisions differs on
    # the two `version*` rows for a reason that is not behaviour (B16-7).
    (re.compile(rb"nvim\.rs dev-[0-9a-f]+(-dirty)?"), b"nvim.rs dev-<REV>"),
    (re.compile(rb"NVIM v[^\n]*"), b"NVIM v<VER>"),
    (re.compile(rb"Build type: [^\n]*"), b"Build type: <BUILD>"),
    (re.compile(rb"LuaJIT [^\n]*"), b"LuaJIT <VER>"),
    (re.compile(rb'Run "nvim -V1 -v"[^\n]*'), b"<RUNV>"),
    (re.compile(rb"\d+\.\d{3}\s+\d+\.\d{3}"), b"<T> <T>"),
    (re.compile(rb"time in msec[^\n]*"), b"<TIMEHDR>"),
    (re.compile(rb"process \d+"), b"process <PID>"),
    (re.compile(rb"pid=\d+"), b"pid=<PID>"),
    (re.compile(rb"/tmp/nvim\.[^/\s]+"), b"<NVIMTMP>"),
    (re.compile(rb"nvim\.\d+\.\d+"), b"nvim.<PID>.<N>"),
    # --- the log prefix (B16-5) ---
    # "DBG 2026-08-09T12:38:03.012 ?.507603   vim_mktempdir:3323: ..."
    # The level tag, the logger name and the function name stay; the
    # clock, the pid/channel and the SOURCE LINE NUMBER go.  The last one
    # matters: B16-8 rewrites log.rs and B16-13 main.rs, and every
    # `logmsg` call site's line number moves with them without any
    # behaviour changing.
    (re.compile(rb"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}"), b"<TS>"),
    # The logger name is written with `%-10s`, so a shorter pid pads with
    # MORE spaces: the padding is a function of the pid's digit count and is
    # therefore not reproducible across runs (B16-18 — three `log-*` rows
    # drifted against a binary that had not changed).  Eat the run of spaces
    # with the name.
    (re.compile(rb"(?m)^(DBG|INF|WRN|ERR) <TS> \S+ +"), rb"\1 <TS> <WHO> "),
    (re.compile(rb"([a-zA-Z_][a-zA-Z0-9_]*):\d+: "), rb"\1:<L>: "),
    (re.compile(rb"0x[0-9a-f]{6,}"), b"<ADDR>"),
    (re.compile(rb"\?\.\d+"), b"?.<PID>"),
]


def normalise(raw: bytes) -> str:
    text = raw
    for pattern, repl in SUBS:
        text = pattern.sub(repl, text)
    return (
        text.decode("utf-8", "backslashreplace")
        .replace("\\", "\\\\")
        .replace("\t", "\\t")
        .replace("\r", "\\r")
        .replace("\n", "\\n")
    )


# -------------------------------------------------------------------- main


def main() -> int:
    binary, out_path = Path(sys.argv[1]).resolve(), Path(sys.argv[2])
    only = next(
        (a.split("=", 1)[1] for a in sys.argv[3:] if a.startswith("--only=")), None
    )

    fixtures()
    RUN.mkdir(exist_ok=True)
    shutil.copy2(binary, RUN / "nvim")
    os.chmod(RUN / "nvim", 0o755)

    base_env = {
        "PATH": "/usr/bin:/bin",
        "HOME": str(FIX / "home"),
        "XDG_CONFIG_HOME": str(FIX / "empty-cfg"),
        "XDG_DATA_HOME": str(FIX / "data"),
        "XDG_STATE_HOME": str(FIX / "state"),
        "XDG_CACHE_HOME": str(FIX / "cache"),
        "XDG_RUNTIME_DIR": str(FIX / "xdgrun"),
        "VIMRUNTIME": str(RUNTIME),
        "TERM": "dumb",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "SHELL": "/bin/sh",
        "NVIM_LOG_FILE": str(FIX / "nvim.log"),
    }

    written = []
    for index, (label, spec) in enumerate(cases(), start=1):
        if only and only not in label:
            continue
        env = dict(base_env)
        env.update(spec["env"])
        cwd = env.pop("PROBE_CWD", str(FIX))
        # A log case gets its own file, removed first: the shared one in
        # base_env accumulates across all 200-odd cases and reading it
        # back would make every answer a function of the whole run.
        logpath = None
        if spec.get("logfile"):
            logpath = FIX / "logs" / f"{label}.log"
            logpath.parent.mkdir(parents=True, exist_ok=True)
            logpath.unlink(missing_ok=True)
            env["NVIM_LOG_FILE"] = str(logpath)
        # `run/nvim` also has to be removed for `--listen` cases whose
        # socket the previous run left behind.
        for _ in range(spec.get("runs", 1)):
            try:
                done = subprocess.run(
                    [str(RUN / "nvim"), *spec["args"]],
                    input=(spec["stdin"] or "").encode(),
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    env=env,
                    cwd=cwd,
                    timeout=spec["timeout"],
                )
                rc, sout, serr = done.returncode, done.stdout, done.stderr
            except subprocess.TimeoutExpired as exc:
                rc, sout, serr = "TIMEOUT", exc.stdout or b"", exc.stderr or b""
        logtext = b""
        if logpath is not None and logpath.exists():
            logtext = logpath.read_bytes()
        written.append(
            f"{index}\t{label}\t{rc}\t{normalise(sout)}\t{normalise(serr)}"
            f"\t{normalise(logtext)}"
        )
    out_path.write_text("\n".join(written) + "\n")
    print(f"{out_path}: {len(written)} cases")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
