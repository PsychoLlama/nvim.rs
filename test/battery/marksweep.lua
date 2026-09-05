-- marksweep -- the differential oracle for the marktree family (batch B22).
--
-- Driven by marksweep.sh; see that file for the sandbox, the
-- artifacts and the scrub.  Everything here is deterministic by
-- construction: no wall clock, no `pairs()` over a hash table in an
-- answer, no process-global handle in a row, and every case builds its
-- own buffer and its own namespace so extmark ids restart at 1.
--
-- THE OBSERVABLE is `nvim__buf_debug_extmarks(buf, keys, dot)` -- the
-- node-by-node dump, with each node's intersection set -- PLUS
-- `nvim_buf_get_extmarks(.., {details = true})`.  Shape and behaviour
-- together: the dump is the only thing in a running editor that can see
-- a node boundary, a level, a `p_idx` or an intersection set, and the
-- details walk is what a user actually gets back.
--
-- NODE IDENTITY.  The `dot` rendering names each node by its PARENT
-- CHAIN (`MTNode`, `MTNode_a0`, `MTNode_b1_a3`) -- `'a' + level` and
-- `p_idx`, never an address -- so it is already stable.  This file
-- renumbers those names to emission-order ordinals `n0, n1, ...` anyway,
-- so that a shape line reads without decoding the chain and so that a
-- future rewrite that changes the naming scheme (but not the tree) does
-- not re-baseline every row.

local api = vim.api

local ONLY = os.getenv("MARKSWEEP_ONLY") or ""
local WORK = os.getenv("MARK_WORK") or "/tmp"
local NVIM = os.getenv("MARK_NVIM") or "nvim"
local TREE_OUT = os.getenv("MARK_TREE") or (WORK .. "/tree")
local MARKS_OUT = os.getenv("MARK_MARKS") or (WORK .. "/marks")

local report = {}
local tree_lines = {}
local mark_lines = {}
local section_rows = {}
local section_order = {}
local total_rows = 0

local function say(s)
  report[#report + 1] = s
end

local function digest(s)
  return vim.fn.sha256(s):sub(1, 12)
end

-- ---------------------------------------------------------------- shape
-- Parse the Graphviz rendering into an ordinal-numbered node list.  Node
-- blocks are emitted depth-first (child 0, then key 0, then child 1 ...),
-- which is what makes the ordinals stable.
local function parse_dot(dot)
  local nodes, byname, parent_of = {}, {}, {}
  local cur = nil
  for line in (dot .. "\n"):gmatch("([^\n]*)\n") do
    local name = line:match("^%s*(%S+)%[shape=plaintext")
    local from, to = line:match("^%s*(%S+) %-> (%S+)%s*$")
    local row = line:match("^%s*<tr><td>(.*)</td></tr>%s*$")
    if name then
      cur = { name = name, rows = {} }
      nodes[#nodes + 1] = cur
      byname[name] = cur
    elseif from then
      parent_of[to] = from
    elseif row and cur then
      cur.rows[#cur.rows + 1] = row
    end
  end
  for i, n in ipairs(nodes) do
    n.ord = i - 1
    -- Two rows means the first is the intersection set; one means the
    -- node intersects nothing.  An empty node has one empty row.
    if #n.rows >= 2 then
      n.ix = n.rows[1]
      n.keys = n.rows[2]
    else
      n.ix = ""
      n.keys = n.rows[1] or ""
    end
    local lvlchar, pidx = n.name:match("_([a-z])(%d+)$")
    n.lvl = lvlchar and (string.byte(lvlchar) - string.byte("a")) or nil
    n.pidx = pidx and tonumber(pidx) or -1
    local p = parent_of[n.name]
    n.par = p and byname[p] and byname[p].ord or -1
  end
  -- The root's level is one more than its first child's; a lone root is
  -- level 0.  (`dot` only spells a level into a CHILD's name.)
  for i = #nodes, 1, -1 do
    local n = nodes[i]
    if n.lvl == nil then
      local lvl = 0
      for _, m in ipairs(nodes) do
        if m.par == n.ord and m.lvl then
          lvl = m.lvl + 1
        end
      end
      n.lvl = lvl
    end
  end
  return nodes
end

local function count_keys(keystr)
  if keystr == "" then
    return 0
  end
  local n = 1
  for _ in keystr:gmatch(",") do
    n = n + 1
  end
  return n
end

local function count_ix(ixstr)
  return count_keys(ixstr)
end

-- One `.tree` block per case: the normalised per-node shape, then the
-- plain (positional) dump verbatim, wrapped.  Big trees are digested --
-- the churn sections would otherwise bury every other row -- but their
-- per-level aggregate is still printed, so a level that gains or loses a
-- node is visible without the bytes.
local MAX_NODES = 80
local MAX_PLAIN = 4000

local function emit_tree(tag, buf)
  local plain = api.nvim__buf_debug_extmarks(buf, true, false)
  local dot = api.nvim__buf_debug_extmarks(buf, true, true)
  local nodes = parse_dot(dot)
  local shape = {}
  local lvlcount, lvlkeys, lvlix, maxlvl = {}, {}, {}, 0
  local nkeys, nix, nixnodes = 0, 0, 0
  for _, n in ipairs(nodes) do
    local k, x = count_keys(n.keys), count_ix(n.ix)
    nkeys = nkeys + k
    nix = nix + x
    if x > 0 then
      nixnodes = nixnodes + 1
    end
    lvlcount[n.lvl] = (lvlcount[n.lvl] or 0) + 1
    lvlkeys[n.lvl] = (lvlkeys[n.lvl] or 0) + k
    lvlix[n.lvl] = (lvlix[n.lvl] or 0) + x
    maxlvl = math.max(maxlvl, n.lvl)
    shape[#shape + 1] = string.format(
      "n%-3d par=%-4s lvl=%d pidx=%-2d nk=%-2d ix=[%s] k=[%s]",
      n.ord,
      n.par >= 0 and ("n" .. n.par) or "-",
      n.lvl,
      n.pidx,
      k,
      n.ix,
      n.keys
    )
  end
  local shapetext = table.concat(shape, "\n")

  tree_lines[#tree_lines + 1] = string.format(
    "== %s nodes=%d lvls=%d keys=%d ix=%d ixnodes=%d",
    tag,
    #nodes,
    maxlvl + 1,
    nkeys,
    nix,
    nixnodes
  )
  for lvl = maxlvl, 0, -1 do
    tree_lines[#tree_lines + 1] = string.format(
      "   lvl%d nodes=%d keys=%d ix=%d",
      lvl,
      lvlcount[lvl] or 0,
      lvlkeys[lvl] or 0,
      lvlix[lvl] or 0
    )
  end
  if #nodes <= MAX_NODES then
    for _, l in ipairs(shape) do
      tree_lines[#tree_lines + 1] = "   " .. l
    end
  else
    tree_lines[#tree_lines + 1] = string.format("   shape digested %s", digest(shapetext))
  end
  if #plain <= MAX_PLAIN then
    local i = 1
    while i <= #plain do
      tree_lines[#tree_lines + 1] = "   | " .. plain:sub(i, i + 119)
      i = i + 120
    end
    if #plain == 0 then
      tree_lines[#tree_lines + 1] = "   | <empty>"
    end
  else
    tree_lines[#tree_lines + 1] = string.format("   | digested len=%d %s", #plain, digest(plain))
  end

  return {
    nodes = #nodes,
    lvls = maxlvl + 1,
    keys = nkeys,
    ix = nix,
    ixnodes = nixnodes,
    plainlen = #plain,
    shape5 = digest(shapetext),
    plain5 = digest(plain),
  }
end

-- ---------------------------------------------------------------- marks
-- A FIXED field order: the details dict is a Lua hash and iterating it
-- would answer a different order run to run.
local DETAIL_KEYS = {
  "end_row",
  "end_col",
  "right_gravity",
  "end_right_gravity",
  "priority",
  "hl_group",
  "hl_eol",
  "hl_mode",
  "invalid",
  "invalidate",
  "undo_restore",
  "virt_text",
  "virt_text_pos",
  "virt_text_win_col",
  "virt_text_hide",
  "virt_text_repeat_linebreak",
  "virt_lines",
  "virt_lines_above",
  "virt_lines_leftcol",
  "sign_text",
  "sign_hl_group",
  "number_hl_group",
  "line_hl_group",
  "cursorline_hl_group",
  "conceal",
  "conceal_lines",
  "spell",
  "url",
  "ephemeral",
  "scoped",
}

local function ser(v)
  local t = type(v)
  if t == "table" then
    local parts = {}
    for i = 1, #v do
      parts[#parts + 1] = ser(v[i])
    end
    return "{" .. table.concat(parts, "|") .. "}"
  elseif t == "string" then
    return (v:gsub("[^%w%p ]", function(c)
      return string.format("\\x%02x", string.byte(c))
    end))
  else
    return tostring(v)
  end
end

local function render_marks(marks)
  local out = {}
  for _, m in ipairs(marks) do
    local fields = { string.format("id=%d %d/%d", m[1], m[2], m[3]) }
    local d = m[4]
    if d then
      for _, k in ipairs(DETAIL_KEYS) do
        if d[k] ~= nil then
          fields[#fields + 1] = k .. "=" .. ser(d[k])
        end
      end
    end
    out[#out + 1] = table.concat(fields, " ")
  end
  return out
end

local MAX_MARKS = 120

local function emit_marks(tag, buf, ns)
  local marks = api.nvim_buf_get_extmarks(buf, ns, 0, -1, { details = true })
  local rendered = render_marks(marks)
  local text = table.concat(rendered, "\n")
  mark_lines[#mark_lines + 1] = string.format("== %s marks=%d", tag, #marks)
  if #rendered <= MAX_MARKS then
    for _, l in ipairs(rendered) do
      mark_lines[#mark_lines + 1] = "   " .. l
    end
  else
    for i = 1, 3 do
      mark_lines[#mark_lines + 1] = "   " .. rendered[i]
    end
    mark_lines[#mark_lines + 1] = string.format("   ... digested %s", digest(text))
    for i = #rendered - 2, #rendered do
      mark_lines[#mark_lines + 1] = "   " .. rendered[i]
    end
  end
  return { n = #marks, d5 = digest(text) }
end

-- ---------------------------------------------------------------- cases
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
  say(string.format("%-34s %s", tag, extra))
end

-- Build a buffer of `n` numbered lines, put it in the current window
-- (some observables -- `nvim_win_text_height`, `getwininfo()` -- need a
-- window) and answer buf + a fresh namespace.
local function fixture(nlines, width)
  local buf = api.nvim_create_buf(false, true)
  local lines = {}
  for i = 1, (nlines or 20) do
    lines[i] = string.format("l%04d %s", i, string.rep("x", width or 10))
  end
  api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  api.nvim_win_set_buf(0, buf)
  local ns = api.nvim_create_namespace("")
  return buf, ns
end

-- Run one case.  The body may return a string of extra fields; anything
-- it throws is recorded (deterministically -- an API error message has
-- no path or address in it) instead of killing the sweep.
local function case(tag, nlines, body, opts)
  opts = opts or {}
  if not want(tag) then
    return
  end
  local buf, ns = fixture(nlines, opts.width)
  local ok, extra = pcall(body, buf, ns)
  if not ok then
    extra = "ERR=" .. tostring(extra):gsub("[\n\r]+", " "):gsub("%s+", " ")
  end
  local t = emit_tree(tag, buf)
  local m = emit_marks(tag, buf, opts.allns and -1 or ns)
  row(
    tag,
    string.format(
      "nodes=%d lvls=%d keys=%d ix=%d ixn=%d marks=%d dumplen=%d shape=%s plain5=%s mk=%s%s",
      t.nodes,
      t.lvls,
      t.keys,
      t.ix,
      t.ixnodes,
      m.n,
      t.plainlen,
      t.shape5,
      t.plain5,
      m.d5,
      (extra and extra ~= "" and (" " .. extra) or "")
    )
  )
  if not opts.keep then
    pcall(api.nvim_buf_delete, buf, { force = true })
  end
  return buf, ns
end

-- A deterministic LCG: every side must see the identical operation order.
local seed = 20220802
local function rnd(n)
  seed = (seed * 1103515245 + 12345) % 2147483648
  return seed % n
end
local function reseed(s)
  seed = s
end

-- ==========================================================================
-- m0 -- the shape canary.  These rows are this sweep's own contracts; if
-- one of them moves, every row below it is measuring something else.
-- ==========================================================================
section("m0-canary")

case("m0/empty", 20, function(buf, ns)
  local d = api.nvim__buf_debug_extmarks(buf, true, false)
  return string.format("dumplen=%d ns_empty=%s", #d, tostring(d == ""))
end)

case("m0/one", 20, function(buf, ns)
  local id = api.nvim_buf_set_extmark(buf, ns, 3, 2, {})
  return "id1=" .. id
end)

-- MT_BRANCH_FACTOR is 10, so a node holds 2*10-1 = 19 keys and the
-- twentieth splits the root.  These two rows ARE that constant.
case("m0/fill-19", 40, function(buf, ns)
  for i = 1, 19 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
end)

case("m0/fill-20", 40, function(buf, ns)
  for i = 1, 20 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
end)

case("m0/fill-21", 40, function(buf, ns)
  for i = 1, 21 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
end)

-- Three levels: 19 keys per leaf and 19 separators in the root gives at
-- most 399 keys at two levels.
case("m0/level3", 500, function(buf, ns)
  for i = 1, 420 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
end)

case("m0/stable", 60, function(buf, ns)
  for i = 1, 40 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
  local a = api.nvim__buf_debug_extmarks(buf, true, true)
  local b = api.nvim__buf_debug_extmarks(buf, true, true)
  local c = api.nvim__buf_debug_extmarks(buf, true, false)
  local d = api.nvim__buf_debug_extmarks(buf, false, false)
  return string.format(
    "dot_stable=%s keys_live=%s dotlen=%d",
    tostring(a == b),
    tostring(c ~= d),
    #a
  )
end)

-- The intersection lever.  Only a range covering the WHOLE of a node
-- lands in that node's set, so a long pair populates several sets and a
-- short one populates none.  If this row stops distinguishing them, the
-- whole intersect half of the sweep is measuring nothing.
case("m0/ix-lever", 200, function(buf, ns)
  for i = 1, 100 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
  local before = api.nvim__buf_debug_extmarks(buf, true, false)
  local short = api.nvim_buf_set_extmark(buf, ns, 10, 0, { end_row = 11, end_col = 0 })
  local mid = api.nvim__buf_debug_extmarks(buf, true, false)
  local long = api.nvim_buf_set_extmark(buf, ns, 1, 0, { end_row = 99, end_col = 0 })
  local after = api.nvim__buf_debug_extmarks(buf, true, false)
  local function ixcount(d)
    local n = 0
    for _ in d:gmatch("{") do
      n = n + 1
    end
    return n
  end
  return string.format(
    "ix_before=%d ix_short=%d ix_long=%d short_id=%d long_id=%d",
    ixcount(before),
    ixcount(mid),
    ixcount(after),
    short,
    long
  )
end)

case("m0/shrink", 60, function(buf, ns)
  local ids = {}
  for i = 1, 45 do
    ids[#ids + 1] = api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
  local grown = api.nvim__buf_debug_extmarks(buf, true, true)
  for i = 1, 40 do
    api.nvim_buf_del_extmark(buf, ns, ids[i])
  end
  local n = 0
  for _ in grown:gmatch("shape=plaintext") do
    n = n + 1
  end
  return "nodes_at_45=" .. n
end)

case("m0/id-base", 20, function(buf, ns)
  local a = api.nvim_buf_set_extmark(buf, ns, 0, 0, {})
  local b = api.nvim_buf_set_extmark(buf, ns, 1, 0, {})
  local c = api.nvim_buf_set_extmark(buf, ns, 2, 0, { id = 99 })
  local d = api.nvim_buf_set_extmark(buf, ns, 3, 0, {})
  return string.format("ids=%d,%d,%d,%d", a, b, c, d)
end)

case("m0/badbuf", 20, function(buf, ns)
  local ok, err = pcall(api.nvim__buf_debug_extmarks, 999999, true, false)
  -- Strip only the `<chunk>:<line>: ` prefix Lua prepends; the API's own
  -- message is the assertion.
  return "ok=" .. tostring(ok) .. " err=" .. (tostring(err):gsub("^[^ ]-:%d+: ", ""))
end)

case("m0/dot-vs-plain", 80, function(buf, ns)
  for i = 1, 60 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
  local plain = api.nvim__buf_debug_extmarks(buf, true, false)
  local dot = api.nvim__buf_debug_extmarks(buf, true, true)
  local pn = 0
  for _ in plain:gmatch("%[") do
    pn = pn + 1
  end
  local dn = 0
  for _ in dot:gmatch("shape=plaintext") do
    dn = dn + 1
  end
  local pk = 0
  for _ in plain:gmatch("%d+/%d+") do
    pk = pk + 1
  end
  return string.format("plain_nodes=%d dot_nodes=%d plain_keys=%d agree=%s", pn, dn, pk, tostring(pn == dn))
end)

-- ==========================================================================
-- m1 -- put/del x gravity x pairs.
-- ==========================================================================
section("m1-putdel")

local POSITIONS = {
  { "bol", 0, 0 },
  { "mid", 3, 4 },
  { "eol", 3, 16 },
  { "past-eol", 3, 15 },
  { "lastline", 19, 0 },
  { "emptyline", 5, 0 },
}

for _, p in ipairs(POSITIONS) do
  case("m1/put/" .. p[1], 20, function(buf, ns)
    api.nvim_buf_set_lines(buf, 5, 6, false, { "" })
    local id = api.nvim_buf_set_extmark(buf, ns, p[2], p[3], {})
    return "id=" .. id
  end)
end

for _, g in ipairs({ true, false }) do
  local gname = g and "rg" or "lg"
  -- Insert exactly AT the mark: right gravity keeps the mark left of the
  -- insertion, left gravity carries it along.
  case("m1/gravity/" .. gname .. "/insert-at", 20, function(buf, ns)
    local id = api.nvim_buf_set_extmark(buf, ns, 3, 4, { right_gravity = g })
    api.nvim_buf_set_text(buf, 3, 4, 3, 4, { "ZZ" })
    local pos = api.nvim_buf_get_extmark_by_id(buf, ns, id, {})
    return string.format("pos=%d/%d", pos[1], pos[2])
  end)
  case("m1/gravity/" .. gname .. "/insert-before", 20, function(buf, ns)
    local id = api.nvim_buf_set_extmark(buf, ns, 3, 4, { right_gravity = g })
    api.nvim_buf_set_text(buf, 3, 2, 3, 2, { "ZZ" })
    local pos = api.nvim_buf_get_extmark_by_id(buf, ns, id, {})
    return string.format("pos=%d/%d", pos[1], pos[2])
  end)
  case("m1/gravity/" .. gname .. "/insert-lines", 20, function(buf, ns)
    local id = api.nvim_buf_set_extmark(buf, ns, 3, 4, { right_gravity = g })
    api.nvim_buf_set_lines(buf, 3, 3, false, { "new" })
    local pos = api.nvim_buf_get_extmark_by_id(buf, ns, id, {})
    return string.format("pos=%d/%d", pos[1], pos[2])
  end)
  case("m1/gravity/" .. gname .. "/split-line", 20, function(buf, ns)
    local id = api.nvim_buf_set_extmark(buf, ns, 3, 4, { right_gravity = g })
    api.nvim_buf_set_text(buf, 3, 4, 3, 4, { "", "" })
    local pos = api.nvim_buf_get_extmark_by_id(buf, ns, id, {})
    return string.format("pos=%d/%d", pos[1], pos[2])
  end)
end

for _, erg in ipairs({ true, false }) do
  case("m1/pair/erg-" .. tostring(erg), 20, function(buf, ns)
    local id = api.nvim_buf_set_extmark(buf, ns, 2, 1, {
      end_row = 6,
      end_col = 3,
      end_right_gravity = erg,
    })
    api.nvim_buf_set_text(buf, 6, 3, 6, 3, { "QQ" })
    local pos = api.nvim_buf_get_extmark_by_id(buf, ns, id, { details = true })
    return string.format("end=%d/%d", pos[3].end_row, pos[3].end_col)
  end)
end

case("m1/pair/zero-width", 20, function(buf, ns)
  local id = api.nvim_buf_set_extmark(buf, ns, 4, 2, { end_row = 4, end_col = 2 })
  return "id=" .. id
end)

case("m1/pair/same-line", 20, function(buf, ns)
  api.nvim_buf_set_extmark(buf, ns, 4, 2, { end_row = 4, end_col = 8 })
end)

case("m1/pair/nested", 20, function(buf, ns)
  api.nvim_buf_set_extmark(buf, ns, 1, 0, { end_row = 18, end_col = 0 })
  api.nvim_buf_set_extmark(buf, ns, 4, 0, { end_row = 12, end_col = 0 })
  api.nvim_buf_set_extmark(buf, ns, 6, 0, { end_row = 8, end_col = 0 })
end)

case("m1/pair/interleaved", 20, function(buf, ns)
  api.nvim_buf_set_extmark(buf, ns, 1, 0, { end_row = 10, end_col = 0 })
  api.nvim_buf_set_extmark(buf, ns, 5, 0, { end_row = 15, end_col = 0 })
  api.nvim_buf_set_extmark(buf, ns, 8, 0, { end_row = 18, end_col = 0 })
end)

case("m1/pair/end-before-start", 20, function(buf, ns)
  local ok, err = pcall(api.nvim_buf_set_extmark, buf, ns, 6, 0, { end_row = 2, end_col = 0 })
  return "ok=" .. tostring(ok) .. " res=" .. (tostring(err):gsub("^[^ ]-:%d+: ", ""))
end)

case("m1/pair/covering-node", 200, function(buf, ns)
  for i = 1, 100 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
  api.nvim_buf_set_extmark(buf, ns, 0, 0, { end_row = 150, end_col = 0 })
end)

case("m1/pair/covering-many", 300, function(buf, ns)
  for i = 1, 220 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
  for i = 1, 6 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, { end_row = 200 + i, end_col = 0 })
  end
end)

case("m1/del/by-id", 20, function(buf, ns)
  local ids = {}
  for i = 1, 6 do
    ids[i] = api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
  local a = api.nvim_buf_del_extmark(buf, ns, ids[3])
  local b = api.nvim_buf_del_extmark(buf, ns, 4242)
  return string.format("del=%s missing=%s", tostring(a), tostring(b))
end)

case("m1/del/pair", 20, function(buf, ns)
  local id = api.nvim_buf_set_extmark(buf, ns, 2, 0, { end_row = 9, end_col = 0 })
  api.nvim_buf_set_extmark(buf, ns, 5, 0, {})
  api.nvim_buf_del_extmark(buf, ns, id)
end)

case("m1/del/pair-in-tree", 200, function(buf, ns)
  for i = 1, 100 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
  local id = api.nvim_buf_set_extmark(buf, ns, 2, 0, { end_row = 150, end_col = 0 })
  api.nvim_buf_del_extmark(buf, ns, id)
end)

case("m1/del/shuffled-half", 200, function(buf, ns)
  reseed(4242)
  local ids = {}
  for i = 1, 150 do
    ids[i] = api.nvim_buf_set_extmark(buf, ns, rnd(190), rnd(5), {})
  end
  for i = #ids, 2, -1 do
    local j = rnd(i) + 1
    ids[i], ids[j] = ids[j], ids[i]
  end
  for i = 1, 75 do
    api.nvim_buf_del_extmark(buf, ns, ids[i])
  end
end)

case("m1/del/drain", 120, function(buf, ns)
  reseed(99)
  local ids = {}
  for i = 1, 90 do
    ids[i] = api.nvim_buf_set_extmark(buf, ns, rnd(110), 0, {})
  end
  for i = #ids, 2, -1 do
    local j = rnd(i) + 1
    ids[i], ids[j] = ids[j], ids[i]
  end
  for i = 1, #ids do
    api.nvim_buf_del_extmark(buf, ns, ids[i])
  end
end)

case("m1/del/clear-range", 60, function(buf, ns)
  for i = 1, 50 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
  api.nvim_buf_clear_namespace(buf, ns, 10, 30)
end)

case("m1/del/clear-mid-pair", 60, function(buf, ns)
  for i = 1, 40 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
  api.nvim_buf_set_extmark(buf, ns, 5, 0, { end_row = 35, end_col = 0 })
  api.nvim_buf_clear_namespace(buf, ns, 15, 25)
end)

case("m1/put/same-pos", 20, function(buf, ns)
  for _ = 1, 8 do
    api.nvim_buf_set_extmark(buf, ns, 4, 2, {})
  end
end)

case("m1/put/reuse-id", 20, function(buf, ns)
  local id = api.nvim_buf_set_extmark(buf, ns, 2, 0, {})
  api.nvim_buf_set_extmark(buf, ns, 8, 3, { id = id })
  api.nvim_buf_set_extmark(buf, ns, 8, 3, { id = id, end_row = 12, end_col = 1 })
  api.nvim_buf_set_extmark(buf, ns, 1, 1, { id = id })
end)

case("m1/put/descending", 40, function(buf, ns)
  for i = 30, 1, -1 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
end)

case("m1/put/all-same-row", 40, function(buf, ns)
  for i = 0, 24 do
    api.nvim_buf_set_extmark(buf, ns, 3, i % 11, {})
  end
end)

case("m1/put/two-ns", 40, function(buf, ns)
  local ns2 = api.nvim_create_namespace("")
  for i = 1, 12 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
    api.nvim_buf_set_extmark(buf, ns2, i, 1, {})
  end
  api.nvim_buf_clear_namespace(buf, ns2, 0, -1)
end, { allns = true })

-- ==========================================================================
-- m2 -- splice at every relation to a node boundary.  A boundary key is a
-- key held in an INTERNAL node; everything the splice arithmetic can get
-- wrong is about a key on one side of one of those.
-- ==========================================================================
section("m2-splice")

-- Build a two-level tree over `rows` lines with a mark on every fourth
-- line, then answer where the internal node's separator keys live.
local function boundary_rows(buf, ns, nmarks, step)
  local rows = {}
  for i = 1, nmarks do
    rows[i] = i * step
    -- BOTH gravities at the same position, deliberately: with only the
    -- default (right) gravity an insertion AT the mark and one BEFORE it
    -- answer the same tree, and half of the fifteen gestures below would
    -- be duplicates of each other.
    api.nvim_buf_set_extmark(buf, ns, rows[i], 2, { right_gravity = true })
    api.nvim_buf_set_extmark(buf, ns, rows[i], 2, { right_gravity = false })
  end
  -- The root's keys, in the plain dump, are the ones NOT inside a nested
  -- bracket pair; parse them out so a case can aim at a real boundary.
  local plain = api.nvim__buf_debug_extmarks(buf, false, false)
  local depth, bounds, cur = 0, {}, ""
  local function flush()
    local r, col = cur:match("^(%d+)/(%d+)$")
    if r then
      bounds[#bounds + 1] = { tonumber(r), tonumber(col) }
    end
    cur = ""
  end
  for i = 1, #plain do
    local c = plain:sub(i, i)
    -- An INTERNAL node's key is followed by its right child's `[`, not by
    -- a comma (only a leaf comma-terminates), so `[` and `]` flush too --
    -- without that this parser answers zero boundaries and every m2 case
    -- silently falls back to a fixed row in the middle of a leaf.
    if c == "[" then
      if depth == 1 then
        flush()
      end
      depth = depth + 1
      cur = ""
    elseif c == "]" then
      if depth == 1 then
        flush()
      end
      depth = depth - 1
      cur = ""
    elseif depth == 1 then
      if c == "," then
        flush()
      else
        cur = cur .. c
      end
    end
  end
  return bounds
end

local SPLICES = {
  ["ins-before"] = function(buf, r, c)
    api.nvim_buf_set_text(buf, r, math.max(c - 1, 0), r, math.max(c - 1, 0), { "AB" })
  end,
  ["ins-at"] = function(buf, r, c)
    api.nvim_buf_set_text(buf, r, c, r, c, { "AB" })
  end,
  ["ins-after"] = function(buf, r, c)
    api.nvim_buf_set_text(buf, r, c + 1, r, c + 1, { "AB" })
  end,
  ["del-before"] = function(buf, r, c)
    api.nvim_buf_set_text(buf, r, 0, r, math.max(c - 1, 0), {})
  end,
  ["del-over"] = function(buf, r, c)
    api.nvim_buf_set_text(buf, r, math.max(c - 1, 0), r, c + 2, {})
  end,
  ["del-after"] = function(buf, r, c)
    api.nvim_buf_set_text(buf, r, c + 1, r, c + 3, {})
  end,
  ["split-at"] = function(buf, r, c)
    api.nvim_buf_set_text(buf, r, c, r, c, { "", "" })
  end,
  ["join-next"] = function(buf, r, c)
    api.nvim_buf_set_text(buf, r, 12, r + 1, 0, {})
  end,
  ["join-prev"] = function(buf, r, c)
    api.nvim_buf_set_text(buf, r - 1, 12, r, 0, {})
  end,
  ["line-ins-before"] = function(buf, r, c)
    api.nvim_buf_set_lines(buf, r, r, false, { "inserted" })
  end,
  ["line-ins-after"] = function(buf, r, c)
    api.nvim_buf_set_lines(buf, r + 1, r + 1, false, { "inserted" })
  end,
  ["line-del-at"] = function(buf, r, c)
    api.nvim_buf_set_lines(buf, r, r + 1, false, {})
  end,
  ["line-del-span"] = function(buf, r, c)
    api.nvim_buf_set_lines(buf, math.max(r - 2, 0), r + 3, false, {})
  end,
  ["line-del-many"] = function(buf, r, c)
    api.nvim_buf_set_lines(buf, math.max(r - 20, 0), r + 20, false, { "one" })
  end,
  ["replace-block"] = function(buf, r, c)
    api.nvim_buf_set_lines(buf, math.max(r - 3, 0), r + 3, false, { "a", "b", "c" })
  end,
}

local SPLICE_ORDER = {
  "ins-before",
  "ins-at",
  "ins-after",
  "del-before",
  "del-over",
  "del-after",
  "split-at",
  "join-next",
  "join-prev",
  "line-ins-before",
  "line-ins-after",
  "line-del-at",
  "line-del-span",
  "line-del-many",
  "replace-block",
}

for _, name in ipairs(SPLICE_ORDER) do
  case("m2/bound/" .. name, 400, function(buf, ns)
    local bounds = boundary_rows(buf, ns, 60, 5)
    local b = bounds[2] or { 20, 2 }
    SPLICES[name](buf, b[1], b[2])
    return string.format("bounds=%d at=%d/%d", #bounds, b[1], b[2])
  end)
end

-- The same fifteen gestures aimed one line PAST the boundary, where the
-- key belongs to the following leaf instead.
for _, name in ipairs(SPLICE_ORDER) do
  case("m2/leaf/" .. name, 400, function(buf, ns)
    local bounds = boundary_rows(buf, ns, 60, 5)
    local b = bounds[2] or { 20, 2 }
    SPLICES[name](buf, b[1] + 5, b[2])
    return string.format("bounds=%d at=%d/%d", #bounds, b[1] + 5, b[2])
  end)
end

case("m2/multi/every-boundary", 400, function(buf, ns)
  local bounds = boundary_rows(buf, ns, 60, 5)
  for i = #bounds, 1, -1 do
    api.nvim_buf_set_text(buf, bounds[i][1], bounds[i][2], bounds[i][1], bounds[i][2], { "II" })
  end
  return "bounds=" .. #bounds
end)

case("m2/multi/delete-every-boundary-line", 400, function(buf, ns)
  local bounds = boundary_rows(buf, ns, 60, 5)
  for i = #bounds, 1, -1 do
    api.nvim_buf_set_lines(buf, bounds[i][1], bounds[i][1] + 1, false, {})
  end
  return "bounds=" .. #bounds
end)

case("m2/multi/collapse-all", 400, function(buf, ns)
  boundary_rows(buf, ns, 60, 5)
  api.nvim_buf_set_lines(buf, 0, -1, false, { "collapsed" })
end)

case("m2/pair/across-boundary", 400, function(buf, ns)
  local bounds = boundary_rows(buf, ns, 60, 5)
  local b = bounds[2] or { 20, 2 }
  api.nvim_buf_set_extmark(buf, ns, b[1] - 3, 0, { end_row = b[1] + 3, end_col = 0 })
  api.nvim_buf_set_text(buf, b[1], b[2], b[1], b[2], { "", "" })
  return string.format("at=%d/%d", b[1], b[2])
end)

case("m2/pair/spanning-node-splice", 400, function(buf, ns)
  local bounds = boundary_rows(buf, ns, 60, 5)
  api.nvim_buf_set_extmark(buf, ns, 1, 0, { end_row = 290, end_col = 0 })
  api.nvim_buf_set_lines(buf, 100, 140, false, {})
  return "bounds=" .. #bounds
end)

case("m2/col/maxcol", 40, function(buf, ns)
  local id = api.nvim_buf_set_extmark(buf, ns, 3, 0, { end_row = 5, end_col = 0 })
  api.nvim_buf_set_lines(buf, 3, 6, false, { "short" })
  local d = api.nvim_buf_get_extmark_by_id(buf, ns, id, { details = true })
  return string.format("pos=%d/%d end=%d/%d", d[1], d[2], d[3].end_row, d[3].end_col)
end)

case("m2/utf/multibyte", 20, function(buf, ns)
  api.nvim_buf_set_lines(buf, 0, 3, false, { "αβγδε ζηθι", "日本語テキスト", "abc" })
  local a = api.nvim_buf_set_extmark(buf, ns, 0, 4, {})
  local b = api.nvim_buf_set_extmark(buf, ns, 1, 6, {})
  api.nvim_buf_set_text(buf, 0, 0, 0, 2, {})
  local pa = api.nvim_buf_get_extmark_by_id(buf, ns, a, {})
  local pb = api.nvim_buf_get_extmark_by_id(buf, ns, b, {})
  return string.format("a=%d/%d b=%d/%d", pa[1], pa[2], pb[1], pb[2])
end)

-- ==========================================================================
-- m3 -- range collapse, swap and damage.  A splice that deletes a pair's
-- whole range collapses it onto one position; one that deletes only the
-- text BETWEEN the ends leaves them adjacent; and a delete that spans a
-- pair's start can leave the end BEFORE the start, which is the damage
-- path (`marktree_move_pair` / the damage map).
-- ==========================================================================
section("m3-collapse")

case("m3/collapse/whole", 40, function(buf, ns)
  local id = api.nvim_buf_set_extmark(buf, ns, 4, 1, { end_row = 12, end_col = 4 })
  api.nvim_buf_set_lines(buf, 3, 14, false, {})
  local d = api.nvim_buf_get_extmark_by_id(buf, ns, id, { details = true })
  return string.format("pos=%d/%d end=%s/%s", d[1], d[2], d[3].end_row, d[3].end_col)
end)

case("m3/collapse/inner", 40, function(buf, ns)
  local id = api.nvim_buf_set_extmark(buf, ns, 4, 1, { end_row = 12, end_col = 4 })
  api.nvim_buf_set_lines(buf, 5, 12, false, {})
  local d = api.nvim_buf_get_extmark_by_id(buf, ns, id, { details = true })
  return string.format("pos=%d/%d end=%s/%s", d[1], d[2], d[3].end_row, d[3].end_col)
end)

case("m3/collapse/start-only", 40, function(buf, ns)
  local id = api.nvim_buf_set_extmark(buf, ns, 4, 1, { end_row = 12, end_col = 4 })
  api.nvim_buf_set_lines(buf, 2, 7, false, {})
  local d = api.nvim_buf_get_extmark_by_id(buf, ns, id, { details = true })
  return string.format("pos=%d/%d end=%s/%s", d[1], d[2], d[3].end_row, d[3].end_col)
end)

case("m3/collapse/end-only", 40, function(buf, ns)
  local id = api.nvim_buf_set_extmark(buf, ns, 4, 1, { end_row = 12, end_col = 4 })
  api.nvim_buf_set_lines(buf, 10, 16, false, {})
  local d = api.nvim_buf_get_extmark_by_id(buf, ns, id, { details = true })
  return string.format("pos=%d/%d end=%s/%s", d[1], d[2], d[3].end_row, d[3].end_col)
end)

case("m3/collapse/same-col", 40, function(buf, ns)
  local id = api.nvim_buf_set_extmark(buf, ns, 4, 2, { end_row = 4, end_col = 8 })
  api.nvim_buf_set_text(buf, 4, 2, 4, 8, {})
  local d = api.nvim_buf_get_extmark_by_id(buf, ns, id, { details = true })
  return string.format("pos=%d/%d end=%s/%s", d[1], d[2], d[3].end_row, d[3].end_col)
end)

case("m3/collapse/many-pairs", 200, function(buf, ns)
  for i = 1, 40 do
    api.nvim_buf_set_extmark(buf, ns, i * 4, 0, { end_row = i * 4 + 3, end_col = 2 })
  end
  api.nvim_buf_set_lines(buf, 20, 120, false, { "gone" })
end)

case("m3/collapse/nested-pairs", 200, function(buf, ns)
  for i = 1, 20 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, { end_row = 190 - i, end_col = 0 })
  end
  api.nvim_buf_set_lines(buf, 10, 180, false, {})
end)

case("m3/gravity/pair-both", 40, function(buf, ns)
  local id = api.nvim_buf_set_extmark(buf, ns, 4, 2, {
    end_row = 8,
    end_col = 2,
    right_gravity = false,
    end_right_gravity = true,
  })
  api.nvim_buf_set_text(buf, 4, 2, 4, 2, { "L" })
  api.nvim_buf_set_text(buf, 8, 2, 8, 2, { "R" })
  local d = api.nvim_buf_get_extmark_by_id(buf, ns, id, { details = true })
  return string.format("pos=%d/%d end=%d/%d", d[1], d[2], d[3].end_row, d[3].end_col)
end)

case("m3/swap/end-into-start", 40, function(buf, ns)
  local id = api.nvim_buf_set_extmark(buf, ns, 4, 6, {
    end_row = 4,
    end_col = 12,
    right_gravity = true,
    end_right_gravity = false,
  })
  api.nvim_buf_set_text(buf, 4, 4, 4, 14, {})
  local d = api.nvim_buf_get_extmark_by_id(buf, ns, id, { details = true })
  return string.format("pos=%d/%d end=%s/%s", d[1], d[2], d[3].end_row, d[3].end_col)
end)

case("m3/move/pair-across-tree", 300, function(buf, ns)
  for i = 1, 120 do
    api.nvim_buf_set_extmark(buf, ns, i * 2, 0, {})
  end
  local id = api.nvim_buf_set_extmark(buf, ns, 10, 0, { end_row = 250, end_col = 0 })
  api.nvim_buf_set_extmark(buf, ns, 200, 0, { id = id, end_row = 260, end_col = 0 })
  return "moved=" .. id
end)

-- `marktree_move_region` is a whole function of splice.rs that NOTHING else
-- in this sweep enters: `:move` is its only door (ex_cmds/lines.rs ->
-- `extmark_move_region`). It lifts the marks inside the moved region out of
-- the tree, splices twice, and puts them back at the destination.
local function moveregion(buf, cmd)
  api.nvim_buf_call(buf, function()
    vim.cmd(cmd)
  end)
end

case("m3/moveregion/down", 40, function(buf, ns)
  for i = 4, 10 do
    api.nvim_buf_set_extmark(buf, ns, i, 2, {})
  end
  moveregion(buf, "silent 5,11move 30")
end)

case("m3/moveregion/up", 40, function(buf, ns)
  for i = 20, 26 do
    api.nvim_buf_set_extmark(buf, ns, i, 2, {})
  end
  moveregion(buf, "silent 21,27move 3")
end)

-- The gravity comparator of the lift: a mark sitting exactly at the region's
-- END is taken with the region only if it is LEFT-gravity.
case("m3/moveregion/edge-gravity", 40, function(buf, ns)
  for _, row in ipairs({ 4, 10 }) do
    for _, col in ipairs({ 0, 5 }) do
      api.nvim_buf_set_extmark(buf, ns, row, col, { right_gravity = true })
      api.nvim_buf_set_extmark(buf, ns, row, col, { right_gravity = false })
    end
  end
  moveregion(buf, "silent 5,11move 30")
end)

case("m3/moveregion/pairs", 60, function(buf, ns)
  api.nvim_buf_set_extmark(buf, ns, 5, 0, { end_row = 9, end_col = 2 })
  api.nvim_buf_set_extmark(buf, ns, 2, 0, { end_row = 20, end_col = 2 })
  api.nvim_buf_set_extmark(buf, ns, 8, 0, { end_row = 40, end_col = 2 })
  moveregion(buf, "silent 6,10move 45")
end)

case("m3/moveregion/tree", 400, function(buf, ns)
  for i = 1, 190 do
    api.nvim_buf_set_extmark(buf, ns, i * 2, i % 3, {})
  end
  for i = 1, 8 do
    api.nvim_buf_set_extmark(buf, ns, i * 20, 0, { end_row = i * 20 + 90, end_col = 1 })
  end
  moveregion(buf, "silent 100,180move 300")
end)

case("m3/moveregion/backwards-tree", 400, function(buf, ns)
  for i = 1, 190 do
    api.nvim_buf_set_extmark(buf, ns, i * 2, i % 3, {})
  end
  for i = 1, 8 do
    api.nvim_buf_set_extmark(buf, ns, i * 20, 0, { end_row = i * 20 + 90, end_col = 1 })
  end
  moveregion(buf, "silent 300,380move 100")
end)

case("m3/damage/delete-through-pairs", 300, function(buf, ns)
  for i = 1, 60 do
    api.nvim_buf_set_extmark(buf, ns, i * 4, 0, { end_row = i * 4 + 10, end_col = 1 })
  end
  api.nvim_buf_set_lines(buf, 40, 200, false, {})
end)

case("m3/damage/split-through-pairs", 300, function(buf, ns)
  for i = 1, 60 do
    api.nvim_buf_set_extmark(buf, ns, i * 4, 0, { end_row = i * 4 + 10, end_col = 1 })
  end
  for i = 1, 20 do
    api.nvim_buf_set_text(buf, i * 10, 3, i * 10, 3, { "", "" })
  end
end)

-- ==========================================================================
-- m4 -- `invalidate` / `undo_restore` x undo / redo.
-- The buffer must be a real, undoable one, so these cases build their own
-- (a scratch buffer still has undo, but `nvim_buf_set_lines` from Lua
-- writes one undo block per call, which is what we want to step).
-- ==========================================================================
section("m4-undo")

local UNDO_MATRIX = {
  { "plain", {} },
  { "invalidate", { invalidate = true } },
  { "no-undo", { undo_restore = false } },
  { "inval-no-undo", { invalidate = true, undo_restore = false } },
}

for _, m in ipairs(UNDO_MATRIX) do
  case("m4/point/" .. m[1], 20, function(buf, ns)
    local o = vim.tbl_extend("force", { hl_group = "Comment" }, m[2])
    local id = api.nvim_buf_set_extmark(buf, ns, 6, 2, o)
    api.nvim_buf_call(buf, function()
      vim.cmd("normal! 7Gdd")
    end)
    local mid = api.nvim_buf_get_extmarks(buf, ns, 0, -1, { details = true })
    api.nvim_buf_call(buf, function()
      vim.cmd("silent undo")
    end)
    local after = api.nvim_buf_get_extmarks(buf, ns, 0, -1, { details = true })
    return string.format(
      "after_del=%d after_undo=%d id=%d",
      #mid,
      #after,
      id
    )
  end)

  case("m4/pair/" .. m[1], 20, function(buf, ns)
    local o = vim.tbl_extend("force", {
      end_row = 9,
      end_col = 3,
      hl_group = "Comment",
    }, m[2])
    local id = api.nvim_buf_set_extmark(buf, ns, 5, 1, o)
    api.nvim_buf_call(buf, function()
      vim.cmd("normal! 6G5dd")
    end)
    local mid = api.nvim_buf_get_extmarks(buf, ns, 0, -1, { details = true })
    local inval = mid[1] and mid[1][4] and mid[1][4].invalid
    api.nvim_buf_call(buf, function()
      vim.cmd("silent undo")
    end)
    local after = api.nvim_buf_get_extmarks(buf, ns, 0, -1, { details = true })
    return string.format(
      "after_del=%d invalid=%s after_undo=%d id=%d",
      #mid,
      tostring(inval),
      #after,
      id
    )
  end)
end

case("m4/undo/redo", 20, function(buf, ns)
  api.nvim_buf_set_extmark(buf, ns, 5, 1, { end_row = 9, end_col = 3, invalidate = true })
  api.nvim_buf_call(buf, function()
    vim.cmd("normal! 6G5dd")
    vim.cmd("silent undo")
    vim.cmd("silent redo")
  end)
end)

case("m4/undo/twice", 20, function(buf, ns)
  api.nvim_buf_set_extmark(buf, ns, 5, 1, { end_row = 9, end_col = 3, invalidate = true })
  api.nvim_buf_call(buf, function()
    vim.cmd("normal! 6Gdd")
    vim.cmd("normal! 6Gdd")
    vim.cmd("silent undo")
    vim.cmd("silent undo")
  end)
end)

case("m4/undo/tree", 100, function(buf, ns)
  for i = 1, 60 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, { invalidate = true })
  end
  api.nvim_buf_call(buf, function()
    vim.cmd("normal! 10G30dd")
    vim.cmd("silent undo")
  end)
end)

case("m4/undo/tree-noundo", 100, function(buf, ns)
  for i = 1, 60 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, { undo_restore = false })
  end
  api.nvim_buf_call(buf, function()
    vim.cmd("normal! 10G30dd")
    vim.cmd("silent undo")
  end)
end)

case("m4/undo/paired-tree", 200, function(buf, ns)
  for i = 1, 40 do
    api.nvim_buf_set_extmark(buf, ns, i * 4, 0, { end_row = i * 4 + 2, end_col = 1, invalidate = true })
  end
  api.nvim_buf_call(buf, function()
    vim.cmd("normal! 20G100dd")
    vim.cmd("silent undo")
  end)
end)

case("m4/undo/invalid-then-valid", 30, function(buf, ns)
  local id = api.nvim_buf_set_extmark(buf, ns, 5, 1, { end_row = 8, end_col = 3, invalidate = true })
  api.nvim_buf_call(buf, function()
    vim.cmd("normal! 6G4dd")
  end)
  local a = api.nvim_buf_get_extmark_by_id(buf, ns, id, { details = true })
  api.nvim_buf_call(buf, function()
    vim.cmd("silent undo")
  end)
  local b = api.nvim_buf_get_extmark_by_id(buf, ns, id, { details = true })
  return string.format(
    "invalid_after=%s invalid_undone=%s",
    tostring(a[3] and a[3].invalid),
    tostring(b[3] and b[3].invalid)
  )
end)

-- ==========================================================================
-- m5 -- `set_text` churn until the root splits and shrinks.  Each row is a
-- CHECKPOINT of one long-running tree, so the section reads as a trace:
-- the node and level counts must move, and move back.
-- ==========================================================================
section("m5-churn")

do
  local buf, ns = fixture(600, 12)
  local ids = {}
  local function checkpoint(tag, extra)
    if not want(tag) then
      return
    end
    local t = emit_tree(tag, buf)
    local m = emit_marks(tag, buf, ns)
    row(
      tag,
      string.format(
        "nodes=%d lvls=%d keys=%d ix=%d ixn=%d marks=%d dumplen=%d shape=%s plain5=%s mk=%s%s",
        t.nodes,
        t.lvls,
        t.keys,
        t.ix,
        t.ixnodes,
        m.n,
        t.plainlen,
        t.shape5,
        t.plain5,
        m.d5,
        extra and (" " .. extra) or ""
      )
    )
  end

  reseed(777)
  checkpoint("m5/00-empty")
  for i = 1, 15 do
    ids[#ids + 1] = api.nvim_buf_set_extmark(buf, ns, i * 3, 1, {})
  end
  checkpoint("m5/01-onenode")
  for i = 16, 60 do
    ids[#ids + 1] = api.nvim_buf_set_extmark(buf, ns, i * 3, 1, {})
  end
  checkpoint("m5/02-twolevel")
  for i = 61, 190 do
    ids[#ids + 1] = api.nvim_buf_set_extmark(buf, ns, (i * 3) % 590, i % 7, {})
  end
  checkpoint("m5/03-grown")
  for i = 1, 12 do
    ids[#ids + 1] = api.nvim_buf_set_extmark(buf, ns, i * 40, 0, { end_row = i * 40 + 60, end_col = 2 })
  end
  checkpoint("m5/04-pairs")
  -- The churn edits lines it has already cut up, so every column has to
  -- be clamped to the line it lands on -- an out-of-range column is an
  -- API error, not a splice.
  local function col_at(r, c)
    local l = api.nvim_buf_get_lines(buf, r, r + 1, true)[1]
    return math.min(c, #l)
  end
  for i = 1, 120 do
    local r = rnd(560)
    api.nvim_buf_set_text(buf, r, col_at(r, 0), r, col_at(r, 0), { "II" })
  end
  checkpoint("m5/05-col-churn")
  for i = 1, 60 do
    local r = rnd(500)
    local c = col_at(r, 2)
    api.nvim_buf_set_text(buf, r, c, r, c, { "", "" })
  end
  checkpoint("m5/06-split-churn")
  for i = 1, 60 do
    local r = rnd(400)
    if r + 1 < api.nvim_buf_line_count(buf) then
      api.nvim_buf_set_text(buf, r, col_at(r, 3), r + 1, 0, {})
    end
  end
  checkpoint("m5/07-join-churn")
  for i = 1, 40 do
    local r = rnd(300)
    api.nvim_buf_set_lines(buf, r, r + 2, false, { "one" })
  end
  checkpoint("m5/08-line-churn")
  for i = #ids, 2, -1 do
    local j = rnd(i) + 1
    ids[i], ids[j] = ids[j], ids[i]
  end
  for i = 1, math.floor(#ids / 3) do
    api.nvim_buf_del_extmark(buf, ns, ids[i])
  end
  checkpoint("m5/09-del-third")
  for i = math.floor(#ids / 3) + 1, math.floor(#ids * 2 / 3) do
    api.nvim_buf_del_extmark(buf, ns, ids[i])
  end
  checkpoint("m5/10-del-two-thirds")
  api.nvim_buf_set_lines(buf, 0, 100, false, {})
  checkpoint("m5/11-cut-head")
  api.nvim_buf_set_lines(buf, -50, -1, false, {})
  checkpoint("m5/12-cut-tail")
  for i = math.floor(#ids * 2 / 3) + 1, #ids do
    api.nvim_buf_del_extmark(buf, ns, ids[i])
  end
  checkpoint("m5/13-drained")
  api.nvim_buf_clear_namespace(buf, ns, 0, -1)
  checkpoint("m5/14-cleared")
  pcall(api.nvim_buf_delete, buf, { force = true })
end

-- ==========================================================================
-- m6 -- the filtered (meta-count) walk.  `nvim_buf_get_extmarks`'s `type`
-- is a POST-filter and sees nothing of the tree; the real meta-count walk
-- is `marktree_itr_get_filter` / `_next_filter` / `_step_out_filter`,
-- reached from plines.rs (inline virtual text), decoration/signs.rs and
-- decoration/query.rs (conceal lines).  `nvim_win_text_height` walks the
-- first two without a redraw; the sign column needs one.
-- ==========================================================================
section("m6-filter")

local function height(buf)
  local h = api.nvim_win_text_height(0, {})
  return string.format("all=%d fill=%d", h.all, h.fill)
end

case("m6/height/plain", 40, function(buf, ns)
  return height(buf)
end)

case("m6/height/inline-one", 40, function(buf, ns)
  api.nvim_buf_set_extmark(buf, ns, 5, 2, {
    virt_text = { { string.rep("V", 200), "Comment" } },
    virt_text_pos = "inline",
  })
  return height(buf)
end)

-- The point of the section: ONE inline mark buried in a multi-level tree.
-- A wrong meta count makes the filtered walk skip the node that holds it
-- and the height answers as if the virtual text were not there.
case("m6/height/inline-deep", 400, function(buf, ns)
  for i = 1, 300 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
  api.nvim_buf_set_extmark(buf, ns, 220, 2, {
    virt_text = { { string.rep("V", 300), "Comment" } },
    virt_text_pos = "inline",
  })
  return height(buf)
end)

case("m6/height/inline-many", 400, function(buf, ns)
  for i = 1, 300 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
  for i = 1, 20 do
    api.nvim_buf_set_extmark(buf, ns, i * 15, 2, {
      virt_text = { { string.rep("V", 90), "Comment" } },
      virt_text_pos = "inline",
    })
  end
  return height(buf)
end)

case("m6/height/virt-lines-one", 40, function(buf, ns)
  api.nvim_buf_set_extmark(buf, ns, 5, 0, {
    virt_lines = { { { "a", "Comment" } }, { { "b", "Comment" } } },
  })
  return height(buf)
end)

case("m6/height/virt-lines-deep", 400, function(buf, ns)
  for i = 1, 300 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
  api.nvim_buf_set_extmark(buf, ns, 250, 0, {
    virt_lines = { { { "x", "Comment" } }, { { "y", "Comment" } }, { { "z", "Comment" } } },
  })
  return height(buf)
end)

case("m6/height/conceal-lines", 40, function(buf, ns)
  vim.wo[0].conceallevel = 3
  api.nvim_buf_set_extmark(buf, ns, 5, 0, { conceal_lines = "" })
  api.nvim_buf_set_extmark(buf, ns, 6, 0, { conceal_lines = "" })
  local h = height(buf)
  vim.wo[0].conceallevel = 0
  return h
end)

case("m6/height/conceal-deep", 400, function(buf, ns)
  vim.wo[0].conceallevel = 3
  for i = 1, 300 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
  api.nvim_buf_set_extmark(buf, ns, 240, 0, { conceal_lines = "" })
  local h = height(buf)
  vim.wo[0].conceallevel = 0
  return h
end)

case("m6/height/range", 400, function(buf, ns)
  for i = 1, 300 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
  api.nvim_buf_set_extmark(buf, ns, 100, 2, {
    virt_text = { { string.rep("V", 300), "Comment" } },
    virt_text_pos = "inline",
  })
  local a = api.nvim_win_text_height(0, { start_row = 0, end_row = 50 })
  local b = api.nvim_win_text_height(0, { start_row = 90, end_row = 110 })
  local c = api.nvim_win_text_height(0, { start_row = 150, end_row = 200 })
  return string.format("head=%d mid=%d tail=%d", a.all, b.all, c.all)
end)

local function textoff()
  vim.cmd("redraw")
  local wi = vim.fn.getwininfo(api.nvim_get_current_win())[1]
  return string.format("textoff=%d", wi.textoff)
end

case("m6/signs/none", 40, function(buf, ns)
  vim.wo[0].signcolumn = "auto"
  return textoff()
end)

case("m6/signs/one", 40, function(buf, ns)
  vim.wo[0].signcolumn = "auto"
  api.nvim_buf_set_extmark(buf, ns, 3, 0, { sign_text = "S1" })
  return textoff()
end)

case("m6/signs/deep", 400, function(buf, ns)
  vim.wo[0].signcolumn = "auto"
  for i = 1, 300 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
  api.nvim_buf_set_extmark(buf, ns, 260, 0, { sign_text = "S2" })
  return textoff()
end)

case("m6/signs/stacked", 40, function(buf, ns)
  vim.wo[0].signcolumn = "auto"
  api.nvim_buf_set_extmark(buf, ns, 3, 0, { sign_text = "S1" })
  api.nvim_buf_set_extmark(buf, ns, 3, 0, { sign_text = "S2" })
  api.nvim_buf_set_extmark(buf, ns, 3, 0, { sign_text = "S3" })
  return textoff()
end)

case("m6/signs/removed", 40, function(buf, ns)
  vim.wo[0].signcolumn = "auto"
  local id = api.nvim_buf_set_extmark(buf, ns, 3, 0, { sign_text = "S1" })
  local a = textoff()
  api.nvim_buf_del_extmark(buf, ns, id)
  local b = textoff()
  return a .. " after_del_" .. b
end)

-- The meta counts are maintained BY the rebalance: a split, a merge and a
-- pivot each have to carry their children's counts with them, and nothing
-- else in this sweep deletes marks out from under a DECORATED tree. An
-- undercount makes the filtered walk skip the node holding the virtual
-- text and the height comes back as if it were not there.
case("m6/height/after-rebalance", 500, function(buf, ns)
  reseed(31337)
  local ids = {}
  for i = 1, 400 do
    ids[#ids + 1] = api.nvim_buf_set_extmark(buf, ns, rnd(480), 0, {})
  end
  for i = 1, 12 do
    api.nvim_buf_set_extmark(buf, ns, i * 37, 2, {
      virt_text = { { string.rep("V", 200), "Comment" } },
      virt_text_pos = "inline",
    })
  end
  local before = api.nvim_win_text_height(0, {}).all
  for i = #ids, 2, -1 do
    local j = rnd(i) + 1
    ids[i], ids[j] = ids[j], ids[i]
  end
  for i = 1, 340 do
    api.nvim_buf_del_extmark(buf, ns, ids[i])
  end
  return string.format("before=%d %s", before, height(buf))
end)

case("m6/signs/after-rebalance", 500, function(buf, ns)
  vim.wo[0].signcolumn = "auto"
  reseed(4711)
  local ids = {}
  for i = 1, 400 do
    ids[#ids + 1] = api.nvim_buf_set_extmark(buf, ns, rnd(480), 0, {})
  end
  api.nvim_buf_set_extmark(buf, ns, 240, 0, { sign_text = "S9" })
  local before = textoff()
  for i = #ids, 2, -1 do
    local j = rnd(i) + 1
    ids[i], ids[j] = ids[j], ids[i]
  end
  for i = 1, 380 do
    api.nvim_buf_del_extmark(buf, ns, ids[i])
  end
  return string.format("before_%s after_%s", before, textoff())
end)

case("m6/signhl/line", 40, function(buf, ns)
  vim.wo[0].signcolumn = "auto"
  api.nvim_buf_set_extmark(buf, ns, 3, 0, { line_hl_group = "Comment" })
  api.nvim_buf_set_extmark(buf, ns, 4, 0, { number_hl_group = "Comment" })
  api.nvim_buf_set_extmark(buf, ns, 5, 0, { cursorline_hl_group = "Comment" })
  return textoff()
end)

-- The recipe a gesture GRID found (B20-1: build the mutant once, diff a grid,
-- never hunt with build-and-sweep cycles) for the meta bookkeeping a PIVOT
-- does. It needs three things at once, and the earlier after-rebalance cases
-- had none of them together: every fifth mark decorated (so a pivot's two
-- keys usually differ in meta), enough deletion to pivot and merge repeatedly,
-- and the height read PER RANGE -- the whole-buffer number survives errors
-- that a sixty-line window does not.
local function heightgrid(buf, nlines)
  local parts = {}
  for r = 0, nlines - 60, 60 do
    parts[#parts + 1] = api.nvim_win_text_height(0, { start_row = r, end_row = r + 59 }).all
  end
  return table.concat(parts, ",")
end

for _, spec in ipairs({
  { "inline-del50", "inline", 50 },
  { "inline-del80", "inline", 80 },
  { "conceal-del50", "conceal", 50 },
  { "vlines-del80", "vlines", 80 },
}) do
  case("m6/pivot/" .. spec[1], 700, function(buf, ns)
    vim.wo[0].conceallevel = 3
    reseed(4242)
    local decor = ({
      inline = { virt_text = { { string.rep("V", 200), "Comment" } }, virt_text_pos = "inline" },
      conceal = { conceal_lines = "" },
      vlines = { virt_lines = { { { "a", "Comment" } }, { { "b", "Comment" } } } },
    })[spec[2]]
    local ids = {}
    for i = 1, 400 do
      local o = (i % 5 == 0) and vim.deepcopy(decor) or {}
      ids[#ids + 1] = api.nvim_buf_set_extmark(buf, ns, (i * 7) % 690, 0, o)
    end
    local h0 = api.nvim_win_text_height(0, {})
    for i = #ids, 2, -1 do
      local j = rnd(i) + 1
      ids[i], ids[j] = ids[j], ids[i]
    end
    for i = 1, math.floor(#ids * spec[3] / 100) do
      api.nvim_buf_del_extmark(buf, ns, ids[i])
    end
    local h1 = api.nvim_win_text_height(0, {})
    local grid = heightgrid(buf, 700)
    vim.wo[0].conceallevel = 0
    return string.format("h0=%d/%d h1=%d/%d grid=%s", h0.all, h0.fill, h1.all, h1.fill, grid)
  end)
end

case("m6/type-filter", 60, function(buf, ns)
  for i = 1, 20 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
  api.nvim_buf_set_extmark(buf, ns, 5, 0, { sign_text = "S1" })
  api.nvim_buf_set_extmark(buf, ns, 7, 0, { virt_text = { { "v", "Comment" } } })
  api.nvim_buf_set_extmark(buf, ns, 9, 0, { virt_lines = { { { "l", "Comment" } } } })
  api.nvim_buf_set_extmark(buf, ns, 11, 0, { end_row = 12, end_col = 0, hl_group = "Comment" })
  local function n(t)
    return #api.nvim_buf_get_extmarks(buf, ns, 0, -1, { type = t })
  end
  return string.format(
    "sign=%d virt_text=%d virt_lines=%d highlight=%d",
    n("sign"),
    n("virt_text"),
    n("virt_lines"),
    n("highlight")
  )
end)

-- ==========================================================================
-- m7 -- the overlap walk (`marktree_itr_get_overlap` /
-- `marktree_itr_step_overlap`), which is the ONLY reader of a node's
-- intersection set.  Every row prints both counts: the difference between
-- the overlap walk and the plain walk over the same range IS the
-- intersection set read back.
-- ==========================================================================
section("m7-overlap")

-- A fixture with three regimes of pair: node-covering (which populate the
-- sets), short (which do not) and nested.
local function overlap_fixture(buf, ns)
  for i = 1, 200 do
    api.nvim_buf_set_extmark(buf, ns, i * 2, 0, {})
  end
  for i = 1, 10 do
    api.nvim_buf_set_extmark(buf, ns, i * 3, 0, { end_row = 380 - i * 3, end_col = 0 })
  end
  for i = 1, 30 do
    api.nvim_buf_set_extmark(buf, ns, i * 12, 1, { end_row = i * 12 + 4, end_col = 2 })
  end
end

-- 3 and 12 are a long and a short pair's own START row; 16 and 350 are
-- their END rows. Those four are the overlap contract's boundary, and
-- the rest sample the interior.
-- The overlap walk's ANSWER, not just its cardinality: a count alone
-- cannot tell "found the wrong pair" from "found the right one".
local function idlist(marks)
  local ids = {}
  for _, m in ipairs(marks) do
    ids[#ids + 1] = m[1]
  end
  table.sort(ids)
  local joined = table.concat(ids, ",")
  if #ids > 12 then
    return string.format("n%d/%s", #ids, digest(joined))
  end
  return "[" .. joined .. "]"
end

local OVERLAP_POINTS =
  { 0, 1, 3, 5, 12, 16, 25, 50, 100, 150, 199, 200, 250, 300, 350, 370, 377, 395 }

for _, r in ipairs(OVERLAP_POINTS) do
  case("m7/at/" .. string.format("%03d", r), 400, function(buf, ns)
    overlap_fixture(buf, ns)
    local ov = api.nvim_buf_get_extmarks(buf, ns, { r, 0 }, { r, 0 }, { overlap = true, details = true })
    local pl = api.nvim_buf_get_extmarks(buf, ns, { r, 0 }, { r, 0 }, { details = true })
    return string.format(
      "overlap=%d plain=%d delta=%d ids=%s",
      #ov,
      #pl,
      #ov - #pl,
      idlist(ov)
    )
  end)
end

for _, span in ipairs({ { 0, 20 }, { 40, 60 }, { 100, 130 }, { 190, 210 }, { 300, 399 } }) do
  case(string.format("m7/span/%03d-%03d", span[1], span[2]), 400, function(buf, ns)
    overlap_fixture(buf, ns)
    local ov =
      api.nvim_buf_get_extmarks(buf, ns, { span[1], 0 }, { span[2], 0 }, { overlap = true, details = true })
    local pl = api.nvim_buf_get_extmarks(buf, ns, { span[1], 0 }, { span[2], 0 }, { details = true })
    return string.format(
      "overlap=%d plain=%d delta=%d ids=%s",
      #ov,
      #pl,
      #ov - #pl,
      idlist(ov)
    )
  end)
end

-- Column-precise, because the overlap tests are `pos_leq` / `pos_less`
-- over a whole MTPos: the short pairs run from `i*12`/1 to `i*12+4`/2, so
-- these ask one column either side of a pair's own end and start.
case("m7/at/pair-end-col", 400, function(buf, ns)
  overlap_fixture(buf, ns)
  local function n(r, c)
    return idlist(api.nvim_buf_get_extmarks(buf, ns, { r, c }, { r, c }, { overlap = true }))
  end
  return string.format("at=%s before=%s after=%s", n(16, 2), n(16, 1), n(16, 3))
end)

case("m7/at/pair-start-col", 400, function(buf, ns)
  overlap_fixture(buf, ns)
  local function n(r, c)
    return idlist(api.nvim_buf_get_extmarks(buf, ns, { r, c }, { r, c }, { overlap = true }))
  end
  return string.format("at=%s before=%s after=%s", n(12, 1), n(12, 0), n(12, 2))
end)

case("m7/limit", 400, function(buf, ns)
  overlap_fixture(buf, ns)
  local a = api.nvim_buf_get_extmarks(buf, ns, { 100, 0 }, { 200, 0 }, { overlap = true, limit = 5 })
  local b = api.nvim_buf_get_extmarks(buf, ns, { 100, 0 }, { 200, 0 }, { overlap = true, limit = 0 })
  return string.format("limit5=%d limit0=%d", #a, #b)
end)

case("m7/reverse", 400, function(buf, ns)
  overlap_fixture(buf, ns)
  local a = api.nvim_buf_get_extmarks(buf, ns, { 200, 0 }, { 100, 0 }, { overlap = true })
  local b = api.nvim_buf_get_extmarks(buf, ns, { 100, 0 }, { 200, 0 }, { overlap = true })
  return string.format("rev=%d fwd=%d", #a, #b)
end)

case("m7/after-splice", 400, function(buf, ns)
  overlap_fixture(buf, ns)
  api.nvim_buf_set_lines(buf, 100, 160, false, {})
  local ov = api.nvim_buf_get_extmarks(buf, ns, { 120, 0 }, { 120, 0 }, { overlap = true })
  local pl = api.nvim_buf_get_extmarks(buf, ns, { 120, 0 }, { 120, 0 }, {})
  return string.format("overlap=%d plain=%d", #ov, #pl)
end)

case("m7/after-del", 400, function(buf, ns)
  overlap_fixture(buf, ns)
  local marks = api.nvim_buf_get_extmarks(buf, ns, 0, -1, { details = true })
  local killed = 0
  for _, m in ipairs(marks) do
    if m[4] and m[4].end_row and m[4].end_row - m[2] > 100 then
      api.nvim_buf_del_extmark(buf, ns, m[1])
      killed = killed + 1
    end
  end
  local ov = api.nvim_buf_get_extmarks(buf, ns, { 150, 0 }, { 150, 0 }, { overlap = true })
  return string.format("killed=%d overlap=%d", killed, #ov)
end)

case("m7/empty-tree", 40, function(buf, ns)
  local ov = api.nvim_buf_get_extmarks(buf, ns, { 5, 0 }, { 5, 0 }, { overlap = true })
  return "overlap=" .. #ov
end)

case("m7/point-only", 40, function(buf, ns)
  for i = 1, 10 do
    api.nvim_buf_set_extmark(buf, ns, i, 0, {})
  end
  local ov = api.nvim_buf_get_extmarks(buf, ns, { 5, 0 }, { 5, 0 }, { overlap = true })
  local pl = api.nvim_buf_get_extmarks(buf, ns, { 5, 0 }, { 5, 0 }, {})
  return string.format("overlap=%d plain=%d", #ov, #pl)
end)

-- ==========================================================================
-- m91 -- the abort probe.  Each gesture runs in its own child, so an abort
-- costs one row instead of the sweep.  `aborted=0` IS the baseline: any
-- abort at all is a regression by construction.
-- ==========================================================================
section("m91-abortprobe")

local PROBES = {
  {
    "huge-row",
    [[
      local ns = vim.api.nvim_create_namespace('p')
      vim.api.nvim_buf_set_lines(0, 0, -1, false, {'a','b','c'})
      pcall(vim.api.nvim_buf_set_extmark, 0, ns, 2147483646, 0, {})
      pcall(vim.api.nvim_buf_set_extmark, 0, ns, 0, 0, { end_row = 2147483646, end_col = 0 })
      vim.api.nvim_buf_set_lines(0, 0, -1, false, {'x'})
    ]],
  },
  {
    "maxcol",
    [[
      local ns = vim.api.nvim_create_namespace('p')
      vim.api.nvim_buf_set_lines(0, 0, -1, false, {'abcdef','ghijkl'})
      pcall(vim.api.nvim_buf_set_extmark, 0, ns, 0, 2147483646, {})
      pcall(vim.api.nvim_buf_set_extmark, 0, ns, 0, 0, { end_row = 1, end_col = 2147483646 })
      vim.api.nvim_buf_set_text(0, 0, 0, 1, 6, {'z'})
    ]],
  },
  {
    "del-under-pairs",
    [[
      local ns = vim.api.nvim_create_namespace('p')
      local l = {}
      for i = 1, 2000 do l[i] = 'line ' .. i end
      vim.api.nvim_buf_set_lines(0, 0, -1, false, l)
      for i = 1, 400 do
        vim.api.nvim_buf_set_extmark(0, ns, i * 4, 0, { end_row = i * 4 + 40, end_col = 1 })
      end
      vim.api.nvim_buf_set_lines(0, 0, -1, false, {'gone'})
    ]],
  },
  {
    "clear-mid-pairs",
    [[
      local ns = vim.api.nvim_create_namespace('p')
      local l = {}
      for i = 1, 1000 do l[i] = 'line ' .. i end
      vim.api.nvim_buf_set_lines(0, 0, -1, false, l)
      for i = 1, 300 do
        vim.api.nvim_buf_set_extmark(0, ns, i, 0, { end_row = 990 - i, end_col = 0 })
      end
      vim.api.nvim_buf_clear_namespace(0, ns, 300, 700)
    ]],
  },
  {
    "zero-width-storm",
    [[
      local ns = vim.api.nvim_create_namespace('p')
      local l = {}
      for i = 1, 500 do l[i] = 'line ' .. i end
      vim.api.nvim_buf_set_lines(0, 0, -1, false, l)
      for i = 1, 400 do
        vim.api.nvim_buf_set_extmark(0, ns, i, 2, { end_row = i, end_col = 2 })
      end
      vim.api.nvim_buf_set_lines(0, 100, 300, false, {})
    ]],
  },
  {
    "same-pos-storm",
    [[
      local ns = vim.api.nvim_create_namespace('p')
      vim.api.nvim_buf_set_lines(0, 0, -1, false, {'abcdefghij'})
      for i = 1, 600 do vim.api.nvim_buf_set_extmark(0, ns, 0, 5, {}) end
      vim.api.nvim_buf_set_text(0, 0, 0, 0, 10, {})
    ]],
  },
  {
    "reuse-id-storm",
    [[
      local ns = vim.api.nvim_create_namespace('p')
      local l = {}
      for i = 1, 400 do l[i] = 'line ' .. i end
      vim.api.nvim_buf_set_lines(0, 0, -1, false, l)
      local id = vim.api.nvim_buf_set_extmark(0, ns, 0, 0, {})
      for i = 1, 400 do
        vim.api.nvim_buf_set_extmark(0, ns, (i * 7) % 399, 0, { id = id, end_row = (i * 7) % 399 + 1, end_col = 0 })
      end
    ]],
  },
  {
    "undo-storm",
    [[
      local ns = vim.api.nvim_create_namespace('p')
      local l = {}
      for i = 1, 400 do l[i] = 'line ' .. i end
      vim.api.nvim_buf_set_lines(0, 0, -1, false, l)
      for i = 1, 200 do
        vim.api.nvim_buf_set_extmark(0, ns, i, 0, { end_row = i + 20, end_col = 0, invalidate = true })
      end
      for i = 1, 20 do vim.cmd('normal! 10G10dd') end
      for i = 1, 30 do vim.cmd('silent! undo') end
      for i = 1, 30 do vim.cmd('silent! redo') end
    ]],
  },
  {
    "split-storm",
    [[
      local ns = vim.api.nvim_create_namespace('p')
      local l = {}
      for i = 1, 800 do l[i] = 'line ' .. i .. ' padding' end
      vim.api.nvim_buf_set_lines(0, 0, -1, false, l)
      for i = 1, 600 do vim.api.nvim_buf_set_extmark(0, ns, i, 4, {}) end
      for i = 1, 200 do vim.api.nvim_buf_set_text(0, i * 3, 4, i * 3, 4, {'', ''}) end
    ]],
  },
  {
    "join-storm",
    [[
      local ns = vim.api.nvim_create_namespace('p')
      local l = {}
      for i = 1, 800 do l[i] = 'line ' .. i .. ' padding' end
      vim.api.nvim_buf_set_lines(0, 0, -1, false, l)
      for i = 1, 600 do vim.api.nvim_buf_set_extmark(0, ns, i, 4, {}) end
      for i = 1, 300 do
        local n = vim.api.nvim_buf_line_count(0)
        if n > 2 then vim.api.nvim_buf_set_text(0, 0, 4, 1, 0, {}) end
      end
    ]],
  },
  {
    "wipe-with-marks",
    [[
      local ns = vim.api.nvim_create_namespace('p')
      local b = vim.api.nvim_create_buf(false, true)
      local l = {}
      for i = 1, 500 do l[i] = 'line ' .. i end
      vim.api.nvim_buf_set_lines(b, 0, -1, false, l)
      for i = 1, 400 do
        vim.api.nvim_buf_set_extmark(b, ns, i, 0, { end_row = i + 50, end_col = 0 })
      end
      vim.api.nvim_buf_delete(b, { force = true })
    ]],
  },
  {
    "overlap-everywhere",
    [[
      local ns = vim.api.nvim_create_namespace('p')
      local l = {}
      for i = 1, 600 do l[i] = 'line ' .. i end
      vim.api.nvim_buf_set_lines(0, 0, -1, false, l)
      for i = 1, 200 do
        vim.api.nvim_buf_set_extmark(0, ns, i, 0, { end_row = 590 - i, end_col = 0 })
      end
      for r = 0, 599, 7 do
        vim.api.nvim_buf_get_extmarks(0, ns, {r, 0}, {r, 0}, { overlap = true })
      end
    ]],
  },
  {
    "dot-deep",
    [[
      local ns = vim.api.nvim_create_namespace('p')
      local l = {}
      for i = 1, 5000 do l[i] = 'line ' .. i end
      vim.api.nvim_buf_set_lines(0, 0, -1, false, l)
      for i = 1, 4000 do vim.api.nvim_buf_set_extmark(0, ns, i, 0, {}) end
      for i = 1, 40 do
        vim.api.nvim_buf_set_extmark(0, ns, i * 20, 0, { end_row = 4900, end_col = 0 })
      end
      local d = vim.api.nvim__buf_debug_extmarks(0, true, true)
      local p = vim.api.nvim__buf_debug_extmarks(0, true, false)
      io.stdout:write('dot=', #d, ' plain=', #p, '\n')
    ]],
  },
  {
    "invalid-then-splice",
    [[
      local ns = vim.api.nvim_create_namespace('p')
      local l = {}
      for i = 1, 400 do l[i] = 'line ' .. i end
      vim.api.nvim_buf_set_lines(0, 0, -1, false, l)
      for i = 1, 200 do
        vim.api.nvim_buf_set_extmark(0, ns, i, 0, { end_row = i + 5, end_col = 0, invalidate = true })
      end
      vim.api.nvim_buf_set_lines(0, 10, 300, false, {})
      for i = 1, 50 do vim.api.nvim_buf_set_text(0, 0, 0, 0, 0, {'x'}) end
      vim.cmd('silent! undo')
    ]],
  },
  {
    "empty-buffer-splice",
    [[
      local ns = vim.api.nvim_create_namespace('p')
      vim.api.nvim_buf_set_lines(0, 0, -1, false, {})
      for i = 1, 50 do pcall(vim.api.nvim_buf_set_extmark, 0, ns, 0, 0, {}) end
      vim.api.nvim_buf_set_lines(0, 0, -1, false, {'a'})
      vim.api.nvim_buf_set_lines(0, 0, -1, false, {})
    ]],
  },
  {
    "pair-at-eof",
    [[
      local ns = vim.api.nvim_create_namespace('p')
      local l = {}
      for i = 1, 200 do l[i] = 'line ' .. i end
      vim.api.nvim_buf_set_lines(0, 0, -1, false, l)
      for i = 1, 100 do
        vim.api.nvim_buf_set_extmark(0, ns, 199, 0, { end_row = 199, end_col = 8 })
      end
      vim.api.nvim_buf_set_lines(0, 150, -1, false, {})
    ]],
  },
  {
    "rebalance-drain",
    [[
      local ns = vim.api.nvim_create_namespace('p')
      local l = {}
      for i = 1, 3000 do l[i] = 'line ' .. i end
      vim.api.nvim_buf_set_lines(0, 0, -1, false, l)
      local ids = {}
      local seed = 12345
      local function rnd(n) seed = (seed * 1103515245 + 12345) % 2147483648 return seed % n end
      for i = 1, 2500 do ids[i] = vim.api.nvim_buf_set_extmark(0, ns, rnd(2900), rnd(4), {}) end
      for i = #ids, 2, -1 do local j = rnd(i) + 1 ids[i], ids[j] = ids[j], ids[i] end
      for i = 1, #ids do vim.api.nvim_buf_del_extmark(0, ns, ids[i]) end
      io.stdout:write('left=', #vim.api.nvim_buf_get_extmarks(0, ns, 0, -1, {}), '\n')
    ]],
  },
  {
    "meta-filter-drain",
    [[
      local ns = vim.api.nvim_create_namespace('p')
      local l = {}
      for i = 1, 1000 do l[i] = 'line ' .. i end
      vim.api.nvim_buf_set_lines(0, 0, -1, false, l)
      local ids = {}
      for i = 1, 500 do
        ids[i] = vim.api.nvim_buf_set_extmark(0, ns, i, 0, {
          virt_text = { { 'v', 'Comment' } }, virt_text_pos = 'inline',
          sign_text = 'S1',
        })
      end
      vim.cmd('redraw')
      for i = 1, #ids do vim.api.nvim_buf_del_extmark(0, ns, ids[i]) end
      vim.cmd('redraw')
      io.stdout:write('h=', vim.api.nvim_win_text_height(0, {}).all, '\n')
    ]],
  },
  {
    "two-buffer-churn",
    [[
      local ns = vim.api.nvim_create_namespace('p')
      local bufs = {}
      for b = 1, 4 do
        bufs[b] = vim.api.nvim_create_buf(false, true)
        local l = {}
        for i = 1, 300 do l[i] = 'line ' .. i end
        vim.api.nvim_buf_set_lines(bufs[b], 0, -1, false, l)
        for i = 1, 200 do
          vim.api.nvim_buf_set_extmark(bufs[b], ns, i, 0, { end_row = i + 30, end_col = 0 })
        end
      end
      for b = 1, 4 do vim.api.nvim_buf_set_lines(bufs[b], 50, 250, false, {}) end
      for b = 1, 4 do vim.api.nvim_buf_delete(bufs[b], { force = true }) end
    ]],
  },
  {
    "namespace-storm",
    [[
      local l = {}
      for i = 1, 500 do l[i] = 'line ' .. i end
      vim.api.nvim_buf_set_lines(0, 0, -1, false, l)
      local nss = {}
      for n = 1, 20 do
        nss[n] = vim.api.nvim_create_namespace('p' .. n)
        for i = 1, 60 do
          vim.api.nvim_buf_set_extmark(0, nss[n], i * 8, 0, { end_row = i * 8 + 5, end_col = 0 })
        end
      end
      vim.api.nvim_buf_set_lines(0, 100, 400, false, {})
      for n = 1, 20 do vim.api.nvim_buf_clear_namespace(0, nss[n], 0, -1) end
    ]],
  },
}

local aborted = 0
for i, p in ipairs(PROBES) do
  local tag = "m91/" .. p[1]
  if want(tag) then
    local file = string.format("%s/probe-%02d.lua", WORK, i)
    local fh = io.open(file, "w")
    fh:write(p[2], "\n")
    fh:close()
    local cmd = string.format(
      "%s --headless -u NONE -i NONE --cmd 'set noswapfile nomore report=9999 shortmess=aoOtTIcCF' "
        .. "-c 'luafile %s' -c 'qa!' 2>&1",
      NVIM,
      file
    )
    local out = vim.fn.system(cmd)
    local rc = vim.v.shell_error
    -- Scrub what a message can carry that a rebuild would move: a source
    -- line number, an address, the work directory.
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
    say(string.format("%-34s rc=%-4d alive=%-5s said=%s", tag, rc, tostring(rc == 0), out:sub(1, 200)))
  end
end

if want("m91/groups") then
  section_rows[cur_section] = section_rows[cur_section] + 1
  total_rows = total_rows + 1
  say(string.format("%-34s cases=%d aborted=%d", "m91/groups", #PROBES, aborted))
end

-- ---------------------------------------------------------------- output
local tail = {}
for _, s in ipairs(section_order) do
  tail[#tail + 1] = string.format("## %s rows=%d", s, section_rows[s] or 0)
end
tail[#tail + 1] = string.format("## TOTAL rows=%d", total_rows)

io.stdout:write(table.concat(report, "\n"), "\n", table.concat(tail, "\n"), "\n")

local fh = io.open(TREE_OUT, "w")
fh:write(table.concat(tree_lines, "\n"), "\n")
fh:close()
fh = io.open(MARKS_OUT, "w")
fh:write(table.concat(mark_lines, "\n"), "\n")
fh:close()
