-- synsweep -- every `syn match`/`syn region` pattern in runtime/syntax,
-- compiled and matched under all three 'regexpengine' settings.
--
-- Driven by synsweep.sh; see that file for the sandbox, the artifacts
-- and the scrub, and synverify.sh for what the row is for.
--
-- THE CORPUS IS THE RUNTIME.  Every `runtime/syntax/*.vim` file is read
-- as text (never sourced) and each `syn[tax] match` pattern and each
-- region's `start=`/`skip=`/`end=` pattern is cut out the way
-- `skip_regexp` finds its end: from the delimiter after the keyword to
-- the same delimiter unescaped and outside a `[]` collection.  Offsets
-- after the closing delimiter (`ms=s+1`) and the item's options are
-- dropped; line continuations are joined.  The cut is approximate -- a
-- pattern a syntax file builds with `:execute` is not seen, and a
-- mis-cut is still a pattern -- and the same text goes to both sides of
-- the differential, so an approximation costs coverage, never a false
-- DIFFERS.  Patterns are deduplicated by text; a row is named for the
-- first file and line that wrote it.
--
-- ONE ROW PER PATTERN.  Each engine answers a search against two
-- subjects -- the source line the pattern came from, and PROBE below --
-- as `start,end` column pairs (`-` for no match, `T` for a search that
-- ran out of time; see `answer`); an error answers `!E<n>`.  When all three settings agree
-- the row says `e012=` once; otherwise each engine is spelled out and
-- the row counts toward `syn/enginedelta`.  `\z(` and the other
-- syntax-only atoms are errors outside `:syntax`, and those are rows too:
-- what a rewritten parser refuses differently is the first thing to see.
--
-- DETERMINISM.  Files are read in sorted order; nothing iterates a hash
-- into an answer; the only clock is the search timeout, which separates
-- exponential patterns from the rest by orders of magnitude.

local RT = os.getenv("SYN_RT") or vim.env.VIMRUNTIME
local ERR_OUT = os.getenv("SYN_ERR")
-- A file the tag of each pattern is written to before it runs: where a
-- hang or a crash was.  Off by default.
local TRACE = os.getenv("SYN_TRACE") and io.open(os.getenv("SYN_TRACE"), "w")

local PROBE = table.concat({
  "  if (x == 0x1F) { return \"str\\n\" + 'c'; } // comment TODO: fix",
  "<div class=\"a\">&amp; text</div> # 3.14e-2 -42 $var @attr",
  "function! s:Foo(a, ...) abort | let l:b = [1, 2] | endfunction",
  "\tKEYWORD key_word Key-Word \195\169t\195\169 \228\184\173\230\150\135 :: -> => <= >= != ===",
}, " ")

local report = {}
local errs = {}

local function esc(s)
  return (tostring(s):gsub("[^\32-\126]", function(c)
    return string.format("\\x%02x", string.byte(c))
  end))
end

local function ecode(msg)
  msg = tostring(msg):gsub("[\r\n]+", " ")
  return msg:match("(E%d+):") or "ERR"
end

local function skip_pattern(s, i, delim)
  -- s:sub(i,i) is the first char after the opening delimiter; return index of closing delim
  local n = #s
  while i <= n do
    local c = s:sub(i, i)
    if c == delim then return i end
    if c == '\\' then
      i = i + 2
    elseif c == '[' then
      local j = i + 1
      if s:sub(j, j) == '^' then j = j + 1 end
      if s:sub(j, j) == ']' then j = j + 1 end
      local closed = nil
      while j <= n do
        local d = s:sub(j, j)
        if d == '\\' then j = j + 2
        elseif d == '[' and s:sub(j + 1, j + 1) == ':' then
          local e = s:find(':]', j + 2, true)
          j = e and e + 2 or j + 1
        elseif d == ']' then closed = j; break
        else j = j + 1 end
      end
      i = closed and closed + 1 or i + 1
    else
      i = i + 1
    end
  end
  return nil
end
local function patterns_of(line)
  local out = {}
  local rest = line:match('^%s*:?%s*sy[ntax]*%s+(.*)$')
  if not rest then return out end
  local kind, after = rest:match('^(%a+)%s+(.*)$')
  if kind ~= 'match' and kind ~= 'region' then return out end
  -- group name
  local s = after:gsub('^%S+', '', 1)
  local i = 1
  while i <= #s do
    local c = s:sub(i, i)
    if c:match('%s') then
      i = i + 1
    else
      local key = s:match('^(%a+)=', i)
      if key == 'start' or key == 'skip' or key == 'end' then
        local d = i + #key + 1
        local delim = s:sub(d, d)
        local e = skip_pattern(s, d + 1, delim)
        if not e then break end
        out[#out + 1] = s:sub(d + 1, e - 1)
        i = e + 1
        -- offsets like ms=s+1 follow without space: skip to space
        while i <= #s and not s:sub(i, i):match('%s') do i = i + 1 end
      elseif kind == 'match' and not c:match('[%w_]') then
        local e = skip_pattern(s, i + 1, c)
        if not e then break end
        out[#out + 1] = s:sub(i + 1, e - 1)
        i = e + 1
        while i <= #s and not s:sub(i, i):match('%s') do i = i + 1 end
      else
        while i <= #s and not s:sub(i, i):match('%s') do i = i + 1 end
      end
    end
  end
  return out
end
local function patterns_of(line)
  local out = {}
  local rest = line:match("^%s*:?%s*sy[ntax]*%s+(.*)$")
  if not rest then
    return out
  end
  local kind, after = rest:match("^(%a+)%s+(.*)$")
  if kind ~= "match" and kind ~= "region" then
    return out
  end
  -- The group name, then options and patterns in any order.
  local s = after:gsub("^%S+", "", 1)
  local i = 1
  local function past_token()
    while i <= #s and not s:sub(i, i):match("%s") do
      i = i + 1
    end
  end
  while i <= #s do
    local c = s:sub(i, i)
    if c:match("%s") then
      i = i + 1
    else
      local key = s:match("^(%a+)=", i)
      if key == "start" or key == "skip" or key == "end" then
        local d = i + #key + 1
        local e = skip_pattern(s, d + 1, s:sub(d, d))
        if not e then
          break
        end
        out[#out + 1] = s:sub(d + 1, e - 1)
        i = e + 1
        past_token()
      elseif kind == "match" and not c:match("[%w_]") then
        local e = skip_pattern(s, i + 1, c)
        if not e then
          break
        end
        out[#out + 1] = s:sub(i + 1, e - 1)
        i = e + 1
        past_token()
      else
        past_token()
      end
    end
  end
  return out
end

-- One engine's answer for one pattern: where `searchpos()` finds its
-- first match and where that match ends, on a one-line buffer holding the
-- subject, cursor at its start.  Through the buffer, not `match()`,
-- because only a search takes a timeout: a few syntax patterns backtrack
-- exponentially on some text (`\k\%(\k\+\s*\)*$` on a run of words), and
-- the row must finish.  A search that used up TIMEOUT answers `T` --
-- such a pattern is exponential, so it is either microseconds or never,
-- and the limit is far from both.
local TIMEOUT = 1000
local scratch = vim.api.nvim_create_buf(false, true)
vim.api.nvim_win_set_buf(0, scratch)
local function answer(pat, line)
  local parts = {}
  for _, subj in ipairs({ line, PROBE }) do
    vim.api.nvim_buf_set_lines(scratch, 0, -1, false, { subj })
    vim.api.nvim_win_set_cursor(0, { 1, 0 })
    local t0 = vim.uv.hrtime()
    local ok, s = pcall(vim.fn.searchpos, pat, "cnW", 0, TIMEOUT)
    if not ok then
      return "!" .. ecode(s), s
    end
    if (vim.uv.hrtime() - t0) / 1e6 >= TIMEOUT then
      parts[#parts + 1] = "T"
    elseif s[1] == 0 then
      parts[#parts + 1] = "-"
    else
      local e = vim.fn.searchpos(pat, "cnWe", 0, TIMEOUT)
      parts[#parts + 1] = string.format("%d,%d", s[2], e[2])
    end
  end
  return table.concat(parts, "/")
end

local files = vim.fn.glob(RT .. "/syntax/*.vim", false, true)
table.sort(files)
local seen, total, uniq, delta, errors = {}, 0, 0, 0, { 0, 0, 0 }
local delta_rows = {}
for _, f in ipairs(files) do
  local name = vim.fn.fnamemodify(f, ":t:r")
  local lines = vim.fn.readfile(f)
  local k = 1
  while k <= #lines do
    local l = lines[k]
    local at = k
    while lines[k + 1] and lines[k + 1]:match("^%s*\\") do
      k = k + 1
      l = l .. lines[k]:gsub("^%s*\\", "")
    end
    for _, pat in ipairs(patterns_of(l)) do
      total = total + 1
      if not seen[pat] then
        seen[pat] = true
        uniq = uniq + 1
        local tag = string.format("syn/%s:%d", name, at)
        if TRACE then
          TRACE:write(tag, " ", pat, "\n")
          TRACE:flush()
        end
        local a = {}
        for e = 0, 2 do
          vim.o.regexpengine = e
          local res, msg = answer(pat, l)
          a[e + 1] = res
          if msg then
            errors[e + 1] = errors[e + 1] + 1
            errs[#errs + 1] = string.format("%s e%d %s", tag, e, esc(tostring(msg):gsub("[\r\n]+", " "):match("(E%d+:.*)$") or msg))
          end
        end
        vim.o.regexpengine = 0
        if a[1] == a[2] and a[2] == a[3] then
          report[#report + 1] = string.format("%s pat=%s e012=%s", tag, esc(pat), a[1])
        else
          report[#report + 1] = string.format("%s pat=%s e0=%s e1=%s e2=%s", tag, esc(pat), a[1], a[2], a[3])
        end
        if a[2] ~= a[3] then
          delta = delta + 1
          delta_rows[#delta_rows + 1] = tag
        end
      end
    end
    k = k + 1
  end
end
report[#report + 1] = string.format("syn/files files=%d patterns=%d unique=%d", #files, total, uniq)
report[#report + 1] = string.format("syn/errors e0=%d e1=%d e2=%d", errors[1], errors[2], errors[3])
report[#report + 1] = string.format("syn/enginedelta rows=%d", delta)
report[#report + 1] = "syn/deltalist " .. table.concat(delta_rows, " ")

io.stdout:write(table.concat(report, "\n"), "\n")
if ERR_OUT then
  local fh = io.open(ERR_OUT, "w")
  fh:write(table.concat(errs, "\n"), "\n")
  fh:close()
end
