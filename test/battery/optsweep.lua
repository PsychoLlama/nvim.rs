-- optsweep -- the differential oracle for the option family.
--
-- Covers crates/nvim/src/nvim/options/{mod,flags,index,lookup,values,
-- table_1..table_5}.rs and optionstr.rs: the option TABLE itself (every
-- name, short name, type, scope, default and flag), `do_set` and its
-- `= += -= ^= & &vim < ! inv no` alphabet, `:set` / `:setlocal` /
-- `:setglobal` / `:set all&` / `:verbose set` / `:options`, the
-- global-local convention, what a new window and a new buffer inherit,
-- and every `E` code `do_set` raises.
--
-- Driven by optsweep.sh; see that file for the sandbox, the
-- artifacts and the scrub.
--
-- WHY IT IS A TABLE SWEEP.  `vimoption_T` holds its value behind a
-- `var: *mut c_void`, and the five generated `table_*.rs` files are the
-- only description of what that pointer means for each of the 374
-- options.  A rewrite that gets ONE entry's type, scope or default wrong
-- produces a plausible editor that is wrong about one option -- which no
-- functional test looks for, because no functional test enumerates the
-- table.  So o1 prints one row PER OPTION: its metadata AND its value.
--
-- DETERMINISM.  Option names are enumerated through a SORTED key list,
-- never `pairs()`.  No wall clock, no handle: the scope section names
-- windows and buffers by ordinal, not by id.  The sandbox's own settings
-- (`report`, `nomore`, `columns`, `lines`, ...) are re-applied by
-- `harness()` after every `&` or `all&`, because `&` restores the
-- COMPILED default and would otherwise leave the rest of the sweep
-- running under a different editor.

local api = vim.api

local ONLY = os.getenv("OPTSWEEP_ONLY") or ""
local WORK = os.getenv("OPT_WORK") or "/tmp"
local NVIM = os.getenv("OPT_NVIM") or "nvim"
local TIMEOUT = os.getenv("OPT_TIMEOUT") or "timeout"
local DUMP_OUT = os.getenv("OPT_DUMP") or (WORK .. "/opts")

local report = {}
local dump_lines = {}
local section_rows = {}
local section_order = {}
local total_rows = 0

local function say(s)
  report[#report + 1] = s
end

local RT = os.getenv("VIMRUNTIME") or ""

-- The driver scrubs the ARTIFACTS, but a digest is computed in here, over
-- text that still carries the sandbox path -- `:set all` prints
-- `runtimepath`, `backupdir`, `shada` and `helpfile`.  Normalise first or
-- every digest row is a fresh random number per run.
local function norm(s)
  s = tostring(s):gsub(vim.pesc(WORK), "<WORK>")
  if RT ~= "" then
    s = s:gsub(vim.pesc(RT), "<RT>")
  end
  return s
end

local function digest(s)
  return vim.fn.sha256(norm(s)):sub(1, 12)
end

local cur_section = nil

local function section(name)
  cur_section = name
  section_order[#section_order + 1] = name
  section_rows[name] = 0
end

local function want(name)
  return ONLY == "" or name:match(ONLY) ~= nil
end

local function row(tag, extra)
  section_rows[cur_section] = section_rows[cur_section] + 1
  total_rows = total_rows + 1
  say(string.format("%-40s %s", tag, extra))
end

local function esc(s)
  s = tostring(s)
  return (s:gsub("[^\32-\126]", function(c)
    return string.format("\\x%02x", string.byte(c))
  end))
end

local function ser(v)
  local t = type(v)
  if t == "table" then
    local parts = {}
    for i = 1, #v do
      parts[#parts + 1] = ser(v[i])
    end
    return "{" .. table.concat(parts, "|") .. "}"
  elseif t == "string" then
    return esc(v)
  else
    return tostring(v)
  end
end

local function ecode(msg)
  msg = tostring(msg):gsub("[\r\n]+", " ")
  local tail = msg:match("(E%d+:.*)$")
  if tail then
    return esc(tail)
  end
  tail = msg:match("Vim%b():%s*(.*)$") or msg:match("Vim:%s*(.*)$")
  return esc(tail or msg)
end

-- Run an ex command, capture its output, answer `ok, text`.  `silent`
-- keeps `smsg`-class chatter off the prompt (a headless run writes it to
-- stderr) while still letting a real error throw.
local function ex(cmd)
  local ok, res = pcall(vim.fn.execute, cmd, "silent")
  if not ok then
    return false, ecode(res)
  end
  return true, (esc(tostring(res)):gsub("^\\x0a", ""):gsub("\\x0a", "|"))
end

local function exq(cmd) -- answer just the text, `!<Ecode>` on failure
  local ok, t = ex(cmd)
  if ok then
    return (t:gsub("^%s+", ""))
  end
  return "!" .. (t:match("^(E%d+)") or "ERR")
end

-- The sandbox's own settings.  `:set {o}&` and `:set all&` restore the
-- COMPILED default, so anything the harness relies on must be put back
-- or every row after the first `&` is measured under a different editor.
local function harness()
  vim.cmd(
    "silent! set columns=80 lines=24 laststatus=0 showtabline=0 ruler noshowcmd "
      .. "report=9999 nomore shortmess=aoOtTIcCF noswapfile undolevels=1000 shell=/bin/sh"
  )
end

local INFO = api.nvim_get_all_options_info()
local NAMES = {}
for k in pairs(INFO) do
  NAMES[#NAMES + 1] = k
end
table.sort(NAMES)

-- Reading a window- or buffer-scoped option through `nvim_get_option_value`
-- with no `{win=}`/`{buf=}` answers the current one, which is what `:set
-- {o}?` shows.
local function getval(name)
  local ok, v = pcall(api.nvim_get_option_value, name, {})
  if not ok then
    return "!" .. (ecode(v):match("^(E%d+)") or "ERR")
  end
  return ser(v)
end

-- =====================================================================
section("o0-canary")

if want("o0/count") then
  local scopes, types, gl, cl, fl, dup = {}, {}, 0, 0, 0, 0
  for _, n in ipairs(NAMES) do
    local i = INFO[n]
    scopes[i.scope] = (scopes[i.scope] or 0) + 1
    types[i.type] = (types[i.type] or 0) + 1
    gl = gl + (i.global_local and 1 or 0)
    cl = cl + (i.commalist and 1 or 0)
    fl = fl + (i.flaglist and 1 or 0)
    dup = dup + (i.allows_duplicates and 1 or 0)
  end
  row(
    "o0/count",
    string.format(
      "options=%d global=%d win=%d buf=%d bool=%d num=%d str=%d global_local=%d commalist=%d flaglist=%d dup=%d",
      #NAMES,
      scopes.global or 0,
      scopes.win or 0,
      scopes.buf or 0,
      types.boolean or 0,
      types.number or 0,
      types.string or 0,
      gl,
      cl,
      fl,
      dup
    )
  )
end

if want("o0/shortnames") then
  -- Every short name must be unique, must resolve, and must resolve to
  -- ITS OWN option.  `:set {abbr}?` prints the FULL name, so the mapping
  -- is observable: the digest below is `find_option_index`'s abbreviation
  -- half, all 341 entries of it, and nothing else in the tree reads it.
  local seen, dupes, bad, wrong = {}, 0, 0, 0
  local map = {}
  for _, n in ipairs(NAMES) do
    local sn = INFO[n].shortname
    if sn ~= "" then
      if seen[sn] then
        dupes = dupes + 1
      end
      seen[sn] = n
      local ok, out = pcall(vim.fn.execute, "silent set " .. sn .. "?", "silent")
      if not ok then
        bad = bad + 1
        map[#map + 1] = string.format("%-8s -> !%s", sn, ecode(out))
      else
        local shown = tostring(out):gsub("^[\r\n%s]+", ""):gsub("[\r\n]+$", "")
        local resolved = shown:match("^no(%a+)$") or shown:match("^(%a+)") or ""
        if resolved ~= n then
          wrong = wrong + 1
        end
        map[#map + 1] = string.format("%-8s -> %-24s %s", sn, n, esc(shown))
      end
    end
  end
  local ct = 0
  for _ in pairs(seen) do
    ct = ct + 1
  end
  table.sort(map)
  dump_lines[#dump_lines + 1] = "== shortname-map"
  for _, l in ipairs(map) do
    dump_lines[#dump_lines + 1] = "   " .. l
  end
  row(
    "o0/shortnames",
    string.format(
      "shortnames=%d dupes=%d unresolvable=%d misresolved=%d d=%s",
      ct,
      dupes,
      bad,
      wrong,
      digest(table.concat(map, "\n"))
    )
  )
end

if want("o0/stable") then
  local a = getval("shiftwidth") .. "/" .. exq("set backspace?")
  local b = getval("shiftwidth") .. "/" .. exq("set backspace?")
  row("o0/stable", string.format("same=%s a=%s", tostring(a == b), a))
end

if want("o0/allamp") then
  -- `:set all&` twice in a row must be idempotent, and must not disturb
  -- an option set AFTER it.
  vim.cmd("silent! set all&")
  harness()
  local d1 = digest(vim.fn.execute("silent set all", "silent"))
  vim.cmd("silent! set all&")
  harness()
  local d2 = digest(vim.fn.execute("silent set all", "silent"))
  row("o0/allamp", string.format("idempotent=%s d=%s", tostring(d1 == d2), d1))
end

if want("o0/wasset") then
  -- `was_set` is the table's own memory of whether the user touched an
  -- option; the harness has touched exactly the ones `harness()` names.
  local n = 0
  local which = {}
  local fresh = api.nvim_get_all_options_info()
  for _, name in ipairs(NAMES) do
    if fresh[name].was_set then
      n = n + 1
      which[#which + 1] = name
    end
  end
  row("o0/wasset", string.format("n=%d [%s]", n, table.concat(which, " ")))
end

-- =====================================================================
section("o1-table")
-- ONE ROW PER OPTION: everything the table knows about it, plus its
-- value.  This is the section a `vimoption_T` rewrite breaks.
for _, name in ipairs(NAMES) do
  local tag = "o1/" .. name
  if want(tag) then
    local i = INFO[name]
    row(
      tag,
      string.format(
        "short=%-6s type=%-7s scope=%-6s gl=%-5s cl=%-5s fl=%-5s dup=%-5s def=%-30s val=%s",
        i.shortname == "" and "-" or i.shortname,
        i.type,
        i.scope,
        tostring(i.global_local),
        tostring(i.commalist),
        tostring(i.flaglist),
        tostring(i.allows_duplicates),
        ser(i.default),
        getval(name)
      )
    )
  end
end

-- =====================================================================
section("o2-show")
-- `:set {o}?` / `:setlocal {o}?` / `:setglobal {o}?` for every option.
-- The three answers differ exactly where the scope rules say they should,
-- and the SHOW path is its own code (`showoneopt` / `option_value2string`).
for _, name in ipairs(NAMES) do
  local tag = "o2/" .. name
  if want(tag) then
    row(
      tag,
      string.format(
        "set=%-46s local=%-46s global=%s",
        exq("set " .. name .. "?"),
        exq("setlocal " .. name .. "?"),
        exq("setglobal " .. name .. "?")
      )
    )
  end
end

-- =====================================================================
section("o3-defaults")
-- `&` versus `&vim`, per option.  Neovim carries ONE default per option
-- where Vim carries two, so these agreeing everywhere is a fact worth
-- pinning: the day a rewrite reintroduces a second default table, this
-- section says so.
local ampdiff = 0
for _, name in ipairs(NAMES) do
  local tag = "o3/" .. name
  if want(tag) then
    local okv, ev = pcall(vim.cmd, "silent! set " .. name .. "&vim")
    local vimval = getval(name)
    local oka, ea = pcall(vim.cmd, "silent! set " .. name .. "&")
    local defval = getval(name)
    if vimval ~= defval then
      ampdiff = ampdiff + 1
    end
    row(
      tag,
      string.format(
        "amp_vim=%-40s amp=%-40s same=%-5s okv=%-5s oka=%s",
        vimval,
        defval,
        tostring(vimval == defval),
        tostring(okv),
        tostring(oka)
      )
    )
    if not okv then
      dump_lines[#dump_lines + 1] = string.format("o3-err %-24s &vim %s", name, ecode(ev))
    end
    if not oka then
      dump_lines[#dump_lines + 1] = string.format("o3-err %-24s &    %s", name, ecode(ea))
    end
  end
end
vim.cmd("silent! set all&")
harness()

if want("o3/ampdiff") then
  row("o3/ampdiff", string.format("differ=%d of=%d", ampdiff, #NAMES))
end

-- =====================================================================
section("o4-verbose")
-- `:verbose set {o}?` answers WHERE an option was last set, which is the
-- only user-visible use of `last_set_sid`/`last_set_linenr`.  A script
-- writes the answers; the path is scrubbed by the driver.
local VSET = {
  "shiftwidth=3",
  "textwidth=44",
  "number",
  "wrap",
  "backspace=eol",
  "scrolloff=5",
  "path=/x,/y",
  "formatoptions=cq",
  "undolevels=77",
  "statusline=%f",
  "listchars=eol:$",
  "cpoptions=aA",
}
do
  local f = WORK .. "/optset.vim"
  local fh = io.open(f, "w")
  fh:write('" optsweep\n')
  for _, s in ipairs(VSET) do
    fh:write("set " .. s .. "\n")
  end
  fh:close()
  vim.cmd("silent! source " .. f)
  for _, s in ipairs(VSET) do
    local name = s:match("^(%a+)")
    local tag = "o4/" .. name
    if want(tag) then
      local i = api.nvim_get_all_options_info()[name]
      row(
        tag,
        string.format(
          "verbose=%-70s sid_set=%-5s linenr=%-3d chan=%d was_set=%s",
          exq("verbose set " .. name .. "?"),
          tostring(i.last_set_sid ~= 0),
          i.last_set_linenr,
          i.last_set_chan,
          tostring(i.was_set)
        )
      )
    end
  end
  -- ... and from the API, which books a CHANNEL rather than a script.
  if want("o4/api-source") then
    api.nvim_set_option_value("tabstop", 6, {})
    local i = api.nvim_get_all_options_info().tabstop
    row(
      "o4/api-source",
      string.format(
        "verbose=%-40s sid=%d linenr=%d chan=%d",
        exq("verbose set tabstop?"),
        i.last_set_sid,
        i.last_set_linenr,
        i.last_set_chan
      )
    )
  end
  if want("o4/verbose-all") then
    local out = vim.fn.execute("silent verbose set", "silent")
    dump_lines[#dump_lines + 1] = "== verbose-set-changed"
    for line in (out .. "\n"):gmatch("([^\n]*)\n") do
      dump_lines[#dump_lines + 1] = "   " .. esc(line)
    end
    row("o4/verbose-all", string.format("bytes=%d d=%s", #out, digest(out)))
  end
end
vim.cmd("silent! set all&")
harness()

-- =====================================================================
section("o5-poke")
-- Set every option to a type-appropriate value and record what happened:
-- accepted with a read-back, or refused with an `E` code.  This is the
-- widest statement the sweep makes about the table -- 374 independent
-- assertions that the entry's TYPE is what the table says it is.
--
-- THE DENYLIST IS DECLARED, not silent.  These options either run
-- something (a provider, a shell, a spell download) or reconfigure the
-- harness in a way `set {o}&` cannot undo, and each one is still a ROW,
-- marked SKIPPED with its reason, so the section's row count stays equal
-- to the option count.
local DENY = {
  clipboard = "loads the clipboard provider, which runs a shell command",
  keymap = "sources a keymap script",
  spellfile = "writes and may create a spell file",
  spelllang = "loads (and offers to download) spell files",
  shell = "the abort probes fork a shell",
  shellcmdflag = "the abort probes fork a shell",
  shellredir = "the abort probes fork a shell",
  shellpipe = "the abort probes fork a shell",
  shellxquote = "the abort probes fork a shell",
  shellxescape = "the abort probes fork a shell",
  verbose = "floods every later row with trace output",
  verbosefile = "redirects the trace into the artifacts",
  debug = "turns later errors into throws",
  writedelay = "sleeps per character drawn",
  redrawdebug = "forces redraws with timing in them",
  shada = "rewrites the sandbox shada file",
  shadafile = "rewrites the sandbox shada file",
}

local POKE = { boolean = true, number = 7, string = "zz" }

for _, name in ipairs(NAMES) do
  local tag = "o5/" .. name
  if want(tag) then
    if DENY[name] then
      row(tag, string.format("SKIPPED reason=%s", DENY[name]))
    else
      local i = INFO[name]
      local before = getval(name)
      local ok, err = pcall(api.nvim_set_option_value, name, POKE[i.type], {})
      local after = getval(name)
      -- ... and the same poke through `:set`, which is a different
      -- parser reaching the same table.
      local lit = i.type == "boolean" and name or (name .. "=" .. tostring(POKE[i.type]))
      local okex, errex = pcall(vim.cmd, "silent! set " .. lit)
      local afterex = getval(name)
      row(
        tag,
        string.format(
          "api_ok=%-5s api_val=%-30s ex_ok=%-5s ex_val=%-30s was=%-24s err=%s",
          tostring(ok),
          after,
          tostring(okex),
          afterex,
          before,
          ok and "-" or (ecode(err):match("^(E%d+)") or "ERR")
        )
      )
      if not ok then
        dump_lines[#dump_lines + 1] = string.format("o5-err %-24s api  %s", name, ecode(err))
      end
      if not okex then
        dump_lines[#dump_lines + 1] = string.format("o5-err %-24s ex   %s", name, ecode(errex))
      end
      vim.cmd("silent! set all&")
      harness()
    end
  end
end
vim.cmd("silent! set all&")
harness()

-- =====================================================================
section("o6-scope")
-- The global-local convention and what a fresh window / fresh buffer
-- inherits.  Every read goes through BOTH doors -- `:set{local,global}`
-- and `nvim_get_option_value` with an explicit `{scope=}`/`{win=}`/
-- `{buf=}` -- because they are different code paths onto the same table
-- entry and a rewrite can move one without the other.
local SCOPED = {
  -- name, a value, a second value
  { "shiftwidth", 3, 9 },
  { "tabstop", 4, 12 },
  { "expandtab", true, false },
  { "textwidth", 40, 66 },
  { "formatoptions", "cq", "tj" },
  { "iskeyword", "a-z,_", "a-z,-" },
  { "commentstring", "#%s", "//%s" },
  { "modifiable", true, false },
  { "undolevels", 55, 66 },
  { "path", "/g1", "/g2" },
  { "tags", "/t1", "/t2" },
  { "define", "d1", "d2" },
  { "equalprg", "e1", "e2" },
  { "makeprg", "m1", "m2" },
  { "errorformat", "%f", "%l" },
  -- `number` is local-TRUE / global-false on purpose: `copy_winopt` copies
  -- `wo_nu` and `wo_rnu` adjacently, and a swap between them is invisible
  -- while the two agree -- which they do at every default.
  { "number", false, true },
  { "relativenumber", true, false },
  { "wrap", true, false },
  { "list", true, false },
  { "foldcolumn", "3", "0" },
  { "signcolumn", "yes", "no" },
  { "conceallevel", 2, 0 },
  { "cursorline", true, false },
  { "winfixheight", true, false },
  { "colorcolumn", "80", "10" },
  { "winhighlight", "Normal:ErrorMsg", "Normal:WarningMsg" },
  { "scrolloff", 5, 9 },
  { "sidescrolloff", 5, 9 },
  { "statusline", "%f", "%F" },
  { "winbar", "%f", "%F" },
  { "virtualedit", "all", "block" },
  { "hlsearch", true, false },
  { "ignorecase", true, false },
}

do
  vim.cmd("silent! only")
  local b1 = api.nvim_create_buf(true, false)
  api.nvim_buf_set_lines(b1, 0, -1, false, { "one", "two" })
  api.nvim_win_set_buf(0, b1)
  local w1 = api.nvim_get_current_win()
  vim.cmd("silent! split")
  local w2 = api.nvim_get_current_win()
  local b2 = api.nvim_create_buf(true, false)
  api.nvim_buf_set_lines(b2, 0, -1, false, { "three" })
  api.nvim_win_set_buf(w2, b2)
  api.nvim_set_current_win(w1)

  local function readall(name)
    local function g(opts)
      local ok, v = pcall(api.nvim_get_option_value, name, opts)
      return ok and ser(v) or ("!" .. (ecode(v):match("^(E%d+)") or "ERR"))
    end
    return string.format(
      "cur=%-18s loc=%-18s glo=%-18s w2=%-18s b2=%-18s ?set=%-26s ?loc=%-26s ?glo=%s",
      g({}),
      g({ scope = "local" }),
      g({ scope = "global" }),
      g({ win = w2 }),
      g({ buf = b2 }),
      exq("set " .. name .. "?"),
      exq("setlocal " .. name .. "?"),
      exq("setglobal " .. name .. "?")
    )
  end

  for _, c in ipairs(SCOPED) do
    local name, v1, v2 = c[1], c[2], c[3]
    local i = INFO[name]
    for _, step in ipairs({ "setglobal", "setlocal", "revert" }) do
      local tag = string.format("o6/%s/%s", name, step)
      if want(tag) then
        if step == "setglobal" then
          vim.cmd("silent! set " .. name .. "&")
          harness()
          local lit = type(v1) == "boolean" and (v1 and name or "no" .. name) or (name .. "=" .. tostring(v1))
          pcall(vim.cmd, "silent! setglobal " .. lit)
        elseif step == "setlocal" then
          local lit = type(v2) == "boolean" and (v2 and name or "no" .. name) or (name .. "=" .. tostring(v2))
          pcall(vim.cmd, "silent! setlocal " .. lit)
        else
          pcall(vim.cmd, "silent! set " .. name .. "<")
        end
        row(tag, string.format("scope=%-6s gl=%-5s %s", i.scope, tostring(i.global_local), readall(name)))
      end
    end
    -- What a FRESH window and a FRESH buffer inherit from that state.
    local tag = "o6/" .. name .. "/inherit"
    if want(tag) then
      local lit = type(v2) == "boolean" and (v2 and name or "no" .. name) or (name .. "=" .. tostring(v2))
      pcall(vim.cmd, "silent! setlocal " .. lit)
      vim.cmd("silent! split")
      local newwin = exq("setlocal " .. name .. "?")
      vim.cmd("silent! enew")
      local newbuf = exq("setlocal " .. name .. "?")
      vim.cmd("silent! close")
      api.nvim_set_current_win(w1)
      row(tag, string.format("newwin=%-30s newbuf=%s", newwin, newbuf))
    end
    vim.cmd("silent! set " .. name .. "&")
    harness()
  end
  vim.cmd("silent! only")
end
vim.cmd("silent! set all&")
harness()

-- =====================================================================
section("o7-dumps")
-- The bulk layer.  `:set all` / `:setglobal all` / `:setlocal all` and
-- the `:options` window are the whole table rendered by three different
-- printers; the `.opts` artifact carries them verbatim and the row
-- carries their size and digest.
do
  local function dump(tag, text)
    dump_lines[#dump_lines + 1] = "== " .. tag
    for line in (text .. "\n"):gmatch("([^\n]*)\n") do
      dump_lines[#dump_lines + 1] = "   " .. esc(line)
    end
    return string.format("bytes=%d lines=%d d=%s", #text, select(2, text:gsub("\n", "\n")) + 1, digest(text))
  end

  for _, c in ipairs({
    { "set-all", "silent set all" },
    { "setglobal-all", "silent setglobal all" },
    { "setlocal-all", "silent setlocal all" },
    { "set-termcap", "silent set termcap" },
  }) do
    local tag = "o7/" .. c[1]
    if want(tag) then
      local ok, out = pcall(vim.fn.execute, c[2], "silent")
      row(tag, ok and dump(c[1], out) or ("!" .. ecode(out)))
    end
  end

  if want("o7/optwin") then
    local ok, err = pcall(vim.cmd, "silent! options")
    if ok then
      local lines = api.nvim_buf_get_lines(0, 0, -1, false)
      local text = table.concat(lines, "\n")
      dump_lines[#dump_lines + 1] = "== optwin"
      for _, l in ipairs(lines) do
        dump_lines[#dump_lines + 1] = "   " .. esc(l)
      end
      row("o7/optwin", string.format("lines=%d bytes=%d d=%s", #lines, #text, digest(text)))
      vim.cmd("silent! close")
    else
      row("o7/optwin", "!" .. ecode(err))
    end
  end

  if want("o7/infodump") then
    -- The info dict for every option, fixed field order.
    local out = {}
    for _, name in ipairs(NAMES) do
      local i = INFO[name]
      out[#out + 1] = string.format(
        "%-24s %-6s %-7s %-6s gl=%-5s cl=%-5s fl=%-5s dup=%-5s def=%s",
        i.name,
        i.shortname,
        i.type,
        i.scope,
        tostring(i.global_local),
        tostring(i.commalist),
        tostring(i.flaglist),
        tostring(i.allows_duplicates),
        ser(i.default)
      )
    end
    local text = table.concat(out, "\n")
    dump_lines[#dump_lines + 1] = "== optioninfo"
    for _, l in ipairs(out) do
      dump_lines[#dump_lines + 1] = "   " .. l
    end
    row("o7/infodump", string.format("options=%d bytes=%d d=%s", #out, #text, digest(text)))
  end
end
vim.cmd("silent! set all&")
harness()

-- =====================================================================
section("o8-errors")
-- The `E` alphabet of `do_set`.  Every one of these is an OBSERVABLE
-- ROW: the sweep records the code, it does not avoid the input.
local BADSET = {
  { "unknown", "set nosuchoption" },
  { "unknown-q", "set nosuchoption?" },
  { "unknown-set", "set nosuchoption=1" },
  { "unknown-no", "set nonosuchoption" },
  { "unknown-inv", "set invnosuchoption" },
  { "num-nan", "set shiftwidth=abc" },
  { "num-empty", "set shiftwidth=" },
  { "num-neg", "set shiftwidth=-1" },
  { "num-huge", "set shiftwidth=999999999999" },
  { "num-hex", "set shiftwidth=0x10" },
  { "num-oct", "set shiftwidth=010" },
  { "bool-value", "set number=3" },
  { "bool-plus", "set number+=1" },
  { "no-on-string", "set nobackspace" },
  { "no-on-number", "set noshiftwidth" },
  { "inv-on-string", "set invbackspace" },
  { "inv-on-number", "set invshiftwidth" },
  { "bang-on-string", "set backspace!" },
  { "bang-on-bool", "set number!" },
  { "str-bad-flag", "set cpoptions=Q" },
  { "str-bad-value", "set background=purple" },
  { "str-bad-enum", "set backspace=nosuch" },
  { "str-bad-fillchars", "set fillchars=nosuch:x" },
  { "str-bad-listchars", "set listchars=nosuch:x" },
  { "str-bad-guicursor", "set guicursor=nosuch" },
  { "str-bad-virtualedit", "set virtualedit=nosuch" },
  { "str-bad-mouse", "set mouse=q" },
  { "str-bad-selection", "set selection=nosuch" },
  { "str-bad-clipboard", "set clipboard=nosuch" },
  { "str-bad-signcolumn", "set signcolumn=nosuch" },
  { "str-bad-foldmethod", "set foldmethod=nosuch" },
  { "str-bad-encoding", "set encoding=nosuch" },
  { "str-bad-casemap", "set casemap=nosuch" },
  { "str-bad-display", "set display=nosuch" },
  { "str-bad-wildmode", "set wildmode=nosuch" },
  { "str-bad-completeopt", "set completeopt=nosuch" },
  { "str-bad-sessionoptions", "set sessionoptions=nosuch" },
  { "str-bad-switchbuf", "set switchbuf=nosuch" },
  { "str-bad-jumpoptions", "set jumpoptions=nosuch" },
  { "str-bad-eventignore", "set eventignore=NoSuchEvent" },
  { "str-bad-winborder", "set winborder=nosuch" },
  { "str-bad-belloff", "set belloff=nosuch" },
  { "str-bad-messagesopt", "set messagesopt=nosuch" },
  { "num-range-cmdheight", "set cmdheight=-1" },
  { "num-range-laststatus", "set laststatus=9" },
  { "num-range-conceallevel", "set conceallevel=9" },
  { "num-range-pumblend", "set pumblend=200" },
  { "num-range-history", "set history=-1" },
  { "num-range-foldcolumn", "set foldcolumn=99" },
  { "trailing", "set shiftwidth=3 zz" },
  { "trailing-bang", "set foldopen!" },
  { "sep-comma", "set shiftwidth=3,tabstop=4" },
  { "local-on-global", "setlocal hidden" },
  { "global-on-window", "setglobal number" },
  { "local-only-nonlocal", "setlocal shiftwidth<" },
  { "lt-on-global", "set hidden<" },
  { "sub-on-number", "set shiftwidth-=1" },
  { "add-on-number", "set shiftwidth+=1" },
  { "pre-on-number", "set shiftwidth^=1" },
  { "add-on-bool", "set hidden+=1" },
  { "sub-missing", "set path-=/nosuchentry" },
  { "add-dup", "set path+=." },
  { "empty-name", "set =" },
  { "just-equals", "set ?" },
  { "secure-modeline", "setlocal modeline&" },
  { "readonly-channel", "set channel=3" },
  { "readonly-busy", "set busy=1" },
  { "compatible", "set compatible" },
  { "edcompatible", "set edcompatible" },
  { "termencoding", "set termencoding=latin1" },
  { "guifont", "set guifont=nosuch" },
  { "pastetoggle", "set pastetoggle=<F1>" },
}

for _, c in ipairs(BADSET) do
  local tag = "o8/" .. c[1]
  if want(tag) then
    vim.v.errmsg = ""
    -- CAPTURED, never bare `vim.cmd`: `set {stringopt}!` and `set {o}?`
    -- PRINT, a headless run sends that to stdout, and stdout is the
    -- report -- the editor's buffer and Lua's are flushed independently,
    -- so a bare command lands at a random place in the artifact.
    local ok, res = ex(c[2])
    local sil = pcall(vim.cmd, "silent! " .. c[2])
    row(
      tag,
      string.format(
        "cmd=%-34s ok=%-5s out=%-30s err=%-52s silent_ok=%-5s errmsg=%s",
        c[2],
        tostring(ok),
        ok and res:gsub("^%s+", "") or "-",
        ok and "-" or res,
        tostring(sil),
        esc(vim.v.errmsg)
      )
    )
    vim.cmd("silent! set all&")
    harness()
  end
end
vim.v.errmsg = ""

-- =====================================================================
section("o9-modify")
-- `+= -= ^= < ! inv no &` over one option of each shape.  These are five
-- separate paths through `do_set`'s argument parser and every one of
-- them writes the table entry a different way.
local MODIFY = {
  { "commalist", "path", { "+=/a", "+=/b", "-=/a", "^=/c", "+=/a,/b", "-=/nosuch", "&" } },
  { "flaglist", "cpoptions", { "+=q", "-=a", "^=z", "+=aq", "-=nosuchflag", "&" } },
  { "flaglist-fo", "formatoptions", { "+=t", "-=q", "^=n", "+=cro", "&" } },
  -- `+=`/`-=`/`^=` on a NUMBER is arithmetic, not a list splice, and it is
  -- a different arm of `get_option_newval`: the row must read the VALUE
  -- back, because "the command was accepted" is true either way.
  { "number", "shiftwidth", { "=5", "+=3", "-=2", "^=4", "&", "=0", "=1" } },
  { "boolean", "wrapscan", { "", "!", "inv", "no", "&" } },
  { "string-plain", "statusline", { "=%f", "=%F%=%l", "&" } },
  { "commalist-dup", "wildignore", { "+=*.o", "+=*.o", "-=*.o", "&" } },
  { "colon-list", "listchars", { "=eol:$", "+=tab:>-", "-=eol:$", "&" } },
  { "colon-list-fc", "fillchars", { "=vert:|", "+=fold:-", "-=vert:|", "&" } },
  { "commalist-num", "vartabstop", { "=4,8", "+=12", "-=4", "&" } },
  { "gl-number", "undolevels", { "=44", "<", "&" } },
  { "gl-string", "tags", { "+=/t", "-=/t", "<", "&" } },
  { "win-bool", "number", { "", "!", "inv", "no", "<", "&" } },
  { "buf-number", "tabstop", { "=3", "<", "&" } },
}

for _, m in ipairs(MODIFY) do
  local shape, name, steps = m[1], m[2], m[3]
  for si, step in ipairs(steps) do
    local tag = string.format("o9/%s-%s/%d", shape, name, si)
    if want(tag) then
      vim.v.errmsg = ""
      local cmd = "set " .. (step:match("^inv") and (step .. name) or (step == "no" and ("no" .. name) or (name .. step)))
      if step == "" then
        cmd = "set " .. name
      elseif step == "inv" then
        cmd = "set inv" .. name
      elseif step == "no" then
        cmd = "set no" .. name
      end
      local ok, out = ex(cmd)
      row(
        tag,
        string.format(
          "cmd=%-30s ok=%-5s out=%-24s val=%-46s show=%-46s err=%s",
          cmd,
          tostring(ok),
          ok and out:gsub("^%s+", "") or "-",
          getval(name),
          exq("set " .. name .. "?"),
          ok and "-" or out
        )
      )
    end
  end
  vim.cmd("silent! set " .. name .. "&")
  harness()
end
vim.cmd("silent! set all&")
harness()
vim.v.errmsg = ""

-- =====================================================================
section("o91-abortprobe")
-- Extreme and self-referential option values, in FRESH CHILDREN.
local PROBES = {
  {
    "geometry-min",
    [[
      pcall(vim.cmd, 'set columns=1 lines=1')
      pcall(vim.cmd, 'redraw')
      io.stdout:write('co=', vim.o.columns, ' li=', vim.o.lines, '\n')
    ]],
  },
  {
    "winmin-over",
    [[
      pcall(vim.cmd, 'set columns=20 lines=10')
      pcall(vim.cmd, 'set winminwidth=40 winminheight=40')
      pcall(vim.cmd, 'split')
      pcall(vim.cmd, 'vsplit')
      pcall(vim.cmd, 'redraw')
      io.stdout:write('wins=', #vim.api.nvim_list_wins(), '\n')
    ]],
  },
  {
    "foldcolumn-max",
    [[
      pcall(vim.cmd, 'set foldcolumn=9 numberwidth=20 signcolumn=yes:9 columns=25')
      pcall(vim.cmd, 'redraw')
      io.stdout:write('ok\n')
    ]],
  },
  {
    "statusline-recursive",
    [[
      pcall(vim.cmd, [==[set laststatus=2 statusline=%!string(&statusline)]==])
      pcall(vim.cmd, 'redraw')
      io.stdout:write('ok\n')
    ]],
  },
  {
    "tabline-error",
    [[
      pcall(vim.cmd, [==[set showtabline=2 tabline=%!nosuchfunction()]==])
      pcall(vim.cmd, 'redraw')
      io.stdout:write('tal=', vim.o.tabline, '\n')
    ]],
  },
  {
    "all-flags-on",
    [[
      pcall(vim.cmd, 'set cpoptions=aAbBcCdDeEfFgHiIjJkKlLmMnoOpPqrRsStuvwWxXyZ$!%*-+<>;#{|.~')
      pcall(vim.cmd, 'set formatoptions=tcroq2vblmMB1jp]')
      pcall(vim.cmd, 'set shortmess=filnxtToOFswaqIcCS')
      pcall(vim.cmd, 'set guioptions=!aPFcefgimrLtTk')
      io.stdout:write('cpo=', vim.o.cpoptions, ' fo=', vim.o.formatoptions, '\n')
    ]],
  },
  {
    "listchars-multibyte",
    [[
      pcall(vim.cmd, [==[set list listchars=eol:¶,tab:»·,trail:·,extends:→]==])
      vim.api.nvim_buf_set_lines(0, 0, -1, false, {'a\tb  ', string.rep('x', 300)})
      pcall(vim.cmd, 'redraw')
      io.stdout:write('lcs=', vim.o.listchars, '\n')
    ]],
  },
  {
    "fillchars-all",
    [[
      pcall(vim.cmd, [==[set fillchars=stl:^,stlnc:=,vert:│,fold:-,foldopen:-,foldclose:+,foldsep:│,diff:x,eob:~,lastline:@,msgsep:-]==])
      pcall(vim.cmd, 'split')
      pcall(vim.cmd, 'redraw')
      io.stdout:write('fcs=', vim.o.fillchars, '\n')
    ]],
  },
  {
    "maxmempattern-tiny",
    [[
      pcall(vim.cmd, 'set maxmempattern=1')
      local l = {}
      for i = 1, 400 do l[i] = string.rep('ab', 60) end
      vim.api.nvim_buf_set_lines(0, 0, -1, false, l)
      local ok = pcall(vim.cmd, [==[%s/\(a*\)*b/x/g]==])
      io.stdout:write('ok=', tostring(ok), '\n')
    ]],
  },
  {
    "undolevels-extremes",
    [[
      for _, v in ipairs({-1, 0, 1, 100000}) do
        pcall(vim.cmd, 'set undolevels=' .. v)
        vim.api.nvim_buf_set_lines(0, 0, -1, false, {'a'})
        vim.api.nvim_buf_set_lines(0, 0, -1, false, {'b'})
        pcall(vim.cmd, 'silent undo')
        io.stdout:write(v, '=', table.concat(vim.api.nvim_buf_get_lines(0,0,-1,false), ''), ' ')
      end
      io.stdout:write('\n')
    ]],
  },
  {
    "textwidth-huge",
    [[
      pcall(vim.cmd, 'set textwidth=100000 wrapmargin=100000')
      vim.api.nvim_buf_set_lines(0, 0, -1, false, {string.rep('word ', 200)})
      pcall(vim.cmd, 'normal! gqq')
      io.stdout:write('n=', vim.api.nvim_buf_line_count(0), '\n')
    ]],
  },
  {
    "modeline-storm",
    [[
      pcall(vim.cmd, 'set modeline modelines=5')
      local f = os.getenv('OPT_WORK') .. '/ml.txt'
      local fh = io.open(f, 'w')
      fh:write('vim: sw=3 ts=9 tw=44\n')
      fh:write('vim: set nosuchoption=1 :\n')
      fh:write('vim: set columns=999 :\n')
      fh:write('vim: set shell=/bin/evil :\n')
      fh:write('vim: set statusline=%!nosuch() :\n')
      fh:close()
      pcall(vim.cmd, 'edit ' .. f)
      io.stdout:write('sw=', vim.bo.shiftwidth, ' ts=', vim.bo.tabstop, ' tw=', vim.bo.textwidth,
        ' co=', vim.o.columns, ' sh=', vim.o.shell, '\n')
    ]],
  },
  {
    "setall-loop",
    [[
      for _ = 1, 20 do pcall(vim.cmd, 'set all&') end
      io.stdout:write('sw=', vim.o.shiftwidth, ' bs=', vim.o.backspace, '\n')
    ]],
  },
  {
    "window-option-churn",
    [[
      for i = 1, 30 do
        pcall(vim.cmd, 'split')
        pcall(vim.cmd, 'setlocal number foldcolumn=2 winhighlight=Normal:ErrorMsg')
      end
      pcall(vim.cmd, 'only')
      io.stdout:write('wins=', #vim.api.nvim_list_wins(), ' nu=', tostring(vim.wo.number), '\n')
    ]],
  },
  {
    "buffer-option-churn",
    [[
      for i = 1, 30 do
        local b = vim.api.nvim_create_buf(true, false)
        vim.api.nvim_set_option_value('shiftwidth', i, {buf = b})
        vim.api.nvim_set_option_value('filetype', 'ft' .. i, {buf = b})
      end
      io.stdout:write('bufs=', #vim.api.nvim_list_bufs(), '\n')
    ]],
  },
  {
    "optwin-twice",
    [[
      pcall(vim.cmd, 'options')
      pcall(vim.cmd, 'close')
      pcall(vim.cmd, 'options')
      io.stdout:write('n=', vim.api.nvim_buf_line_count(0), '\n')
    ]],
  },
}

local aborted = 0
for i, p in ipairs(PROBES) do
  local tag = "o91/" .. p[1]
  if want(tag) then
    local file = string.format("%s/optprobe-%02d.lua", WORK, i)
    local fh = io.open(file, "w")
    fh:write(p[2], "\n")
    fh:close()
    local cmd = string.format(
      "%s -k 2 30 %s --headless -u NONE -i NONE "
        .. "--cmd 'set noswapfile nomore report=9999 shortmess=aoOtTIcCF' "
        .. "-c 'luafile %s' -c 'qa!' 2>&1",
      TIMEOUT,
      NVIM,
      file
    )
    local out = vim.fn.system(cmd)
    local rc = vim.v.shell_error
    out = out
      :gsub("[\r\n]+", " ")
      :gsub("0x%x+", "<ADDR>")
      :gsub("%.rs:%d+:%d+", ".rs:<LINE>")
      :gsub("%.rs:%d+", ".rs:<LINE>")
      :gsub(vim.pesc(WORK), "<WORK>")
      :gsub("%s+$", "")
    if rc ~= 0 then
      aborted = aborted + 1
    end
    section_rows[cur_section] = section_rows[cur_section] + 1
    total_rows = total_rows + 1
    say(string.format("%-40s rc=%-4d alive=%-5s said=%s", tag, rc, tostring(rc == 0), esc(out:sub(1, 220))))
  end
end

if want("o91/groups") then
  section_rows[cur_section] = section_rows[cur_section] + 1
  total_rows = total_rows + 1
  say(string.format("%-40s cases=%d aborted=%d", "o91/groups", #PROBES, aborted))
end

-- ---------------------------------------------------------------- output
local tail = {}
for _, s in ipairs(section_order) do
  tail[#tail + 1] = string.format("## %s rows=%d", s, section_rows[s] or 0)
end
tail[#tail + 1] = string.format("## TOTAL rows=%d", total_rows)

io.stdout:write(table.concat(report, "\n"), "\n", table.concat(tail, "\n"), "\n")

local fh = io.open(DUMP_OUT, "w")
fh:write(table.concat(dump_lines, "\n"), "\n")
fh:close()
