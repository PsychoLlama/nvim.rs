-- Driver for the variable-layer differential sweep; see
-- varsweep.sh.
--
-- Covers eval/vars.rs -- the half of the eval substrate `evalsweep`
-- deliberately does not reach.  evalsweep asks what a *value* renders
-- as; this asks what `:let`, `:unlet`, `:const`, `:lockvar` and the
-- scope dictionaries *do*:
--
--   s1  :let in all its forms -- every target kind (g:/b:/w:/t:/v:/l:/
--       s:/$ENV/&opt/@reg), every compound operator, indexing, slicing,
--       list unpack with and without a rest target, curly-brace names,
--       and the listing forms (`:let`, `:let g:`, `:let name`) whose
--       *message column* is part of the contract (list_one_var_a pads
--       to column 22).
--   s2  :unlet / :unlet! / :const -- including nonexistent, locked,
--       multiple arguments, subscripted targets and the const arms.
--   s3  :lockvar N / :unlockvar N over nested containers, islocked()
--       over every arm it has, and lock visibility through aliases,
--       copy() and deepcopy().
--   s4  the scope dictionaries themselves -- reads, writes, has_key,
--       filtered iteration, remove()/extend() on a scope dict, and the
--       buffer/window/tab accessors (getbufvar/setbufvar/getwinvar/
--       setwinvar/gettabvar/settabvar/gettabwinvar/settabwinvar) with
--       their missing-variable, default-argument and whole-dict arms.
--       b:changedtick is asked separately: it is the one scope entry
--       with a hand-written read-only path.
--   s5  function scopes -- l:, a:, a:000/a:0/a:firstline, and what
--       assignment, unlet and islocked answer inside a function.
--   s6  v: -- the readonly (E46) / fixed (E795) / wrong-type (E963)
--       arms, per variable, plus what unlet and lockvar say.
--   s7  the error-text corpus: one direct trigger per documented code.
--   s8  heredoc assignment (`=<< [trim] [eval] MARKER`) and its four
--       error arms, plus {expr} interpolation and {{ }} escaping.
--   s9  :redir => / =>> into a variable, a list element and a dict key.
--   s10 exists() / var_exists() over every spelling of a name.
--   s11 the same listings and errors again, *uncaptured*, so the .stderr
--       artifact carries the real message path -- msg_advance(22)'s
--       padding and msg_outtrans's rendering of unprintable bytes.
--       Without it that artifact is empty and gates nothing.
--   s12 the four options that hold Vimscript the editor calls out to:
--       'charconvert', 'diffexpr', 'patchexpr', 'spellsuggest'.
--
-- s13 onward are eval/userfunc.rs -- the function layer, which no
-- differential reached before B14-13:
--
--   s13 :function definitions in every shape (arguments, defaults,
--       varargs, the four attributes, every name spelling, the rejected
--       forms) and what calling each one answers; then :return.
--   s14 the :function listings, `:function /pat/`, and :delfunction.
--   s15 function()/funcref(), partials, closures, lambdas, call(),
--       and 'maxfuncdepth'.
--   s16 autoload: four fixture packages under $WORK/rtp.
--   s17 one direct trigger per function-layer error code.
--   s18 the same listings and errors *uncaptured*, for .stderr.
--
-- s19 is the reference-counting surface, added by P23-14:
--
--   s19 dictwatcheradd()/dictwatcherdel() -- which appeared in no
--       oracle at all -- and what a forced collection
--       (test_garbagecollect_now()) leaves behind: reference cycles,
--       shared subtrees, partials, closures, blobs, a funcref whose
--       function was deleted, and the `change` dict a watcher keeps.
--
-- Everything printed has to be reproducible across two builds run
-- minutes apart and from two working directories, so the report carries
-- no address, pid, wall-clock time or path outside the work directory.
-- Three artifacts: the readable report on stdout, a canonical
-- (sorted-key) JSON dump on $VARS_STRUCT of every structured answer,
-- and stderr -- which in a headless process is where nvim's own
-- messages go, and is the only view of some of them.
--
-- Buffer, window and tabpage handles are one monotonic counter for the
-- whole run, so a section inserted above s4 renumbers every handle it
-- records.  APPEND SECTIONS AT THE BOTTOM, and record a handle only
-- where the handle *is* the answer.
--
-- VARSWEEP_ONLY is a Lua pattern matched against each section name; it
-- exists for iterating on one section, not for gating.

local work = assert(os.getenv('VARS_WORK'), 'VARS_WORK unset')
local structpath = assert(os.getenv('VARS_STRUCT'), 'VARS_STRUCT unset')
local structfd = assert(io.open(structpath, 'w'))
local only = os.getenv('VARSWEEP_ONLY')
if only == '' then
  only = nil
end
local trace = os.getenv('VARSWEEP_TRACE') == '1'

-- Unbuffered: nvim's own messages go to stderr, but a Lua error would
-- otherwise lose the tail of the report.
io.stdout:setvbuf('line')

local function emit(...)
  io.write(table.concat({ ... }, ' '), '\n')
end

local runtime = os.getenv('VIMRUNTIME') or ''
local script = debug.getinfo(1, 'S').source:sub(2)

--- Strip the bits of an answer that name where -- or when -- the run
--- happened.  Sorting happens after this, never before: a path is the
--- classic answer that differs between two working directories and
--- survives the three-runs rule.
local function scrub(text)
  text = tostring(text)
  text = text:gsub(vim.pesc(script), '<SCRIPT>')
  text = text:gsub(vim.pesc(work), '<WORK>')
  text = text:gsub(vim.pesc(work:sub(2)), '<WORK>')
  if runtime ~= '' then
    text = text:gsub(vim.pesc(runtime), '<RUNTIME>')
  end
  text = text:gsub('0x%x+', '<ADDR>')
  text = text:gsub('<lambda>%d+', '<lambda>')
  -- A script id is one monotonic counter for the whole run, so a
  -- <SNR>N_ prefix is a function of every section above the one that
  -- printed it.  The mangling itself is the question, not the number:
  -- a rewrite that drops the underscore leaves <SNR>2Foo, which does
  -- not match this pattern and so still shows in the diff.
  text = text:gsub('<SNR>%d+_', '<SNR>_')
  -- `:function` prints where the function was defined.  For the sweep's
  -- own definitions that is this file, at whatever line the nvim_exec2
  -- call happens to sit on -- which would re-baseline the artifact on
  -- any edit above it.  Fixture files keep their real line numbers.
  text = text:gsub('(Last set from <SCRIPT>) line %d+', '%1 line N')
  -- v:progpath and friends name the binary under test, which is a
  -- different path on each side of the diff by construction.
  text = text:gsub('%S*/target/debug/nvim', '<NVIM>')
  text = text:gsub('%S*/nvim%-%x+', '<NVIM>')
  return text
end

--- Escape to one printable line, so a byte difference shows in the diff
--- and a report line stays a report line.
local function esc(bytes)
  return (tostring(bytes):gsub('[^\32-\126]', function(c)
    return string.format('\\x%02x', c:byte())
  end))
end

-- ---------------------------------------------------------------------
-- Canonical dump.  Verbatim from evalsweep.lua: the two artifacts are
-- read side by side often enough that they must escape and sort the
-- same way.
-- ---------------------------------------------------------------------
local canon

local function canon_number(value)
  if value ~= value then
    return '"nan"'
  elseif value == math.huge then
    return '"inf"'
  elseif value == -math.huge then
    return '"-inf"'
  elseif value == math.floor(value) and math.abs(value) < 2 ^ 53 then
    return string.format('%d', value)
  end
  return string.format('%.17g', value)
end

canon = function(value)
  local kind = type(value)
  if value == vim.NIL then
    return 'null'
  elseif kind == 'nil' then
    return 'nil'
  elseif kind == 'number' then
    return canon_number(value)
  elseif kind == 'boolean' then
    return tostring(value)
  elseif kind == 'string' then
    return '"' .. esc(scrub(value)):gsub('"', '\\"') .. '"'
  elseif kind ~= 'table' then
    return '"<' .. kind .. '>"'
  end
  if next(value) == nil then
    return getmetatable(value) and '{}' or '[]'
  end
  if vim.islist(value) then
    local parts = {}
    for _, item in ipairs(value) do
      parts[#parts + 1] = canon(item)
    end
    return '[' .. table.concat(parts, ',') .. ']'
  end
  local keys, byname = {}, {}
  for key in pairs(value) do
    local name = type(key) == 'string' and esc(key)
      or ('<' .. (type(key) == 'table' and canon(key) or scrub(tostring(key))) .. '>')
    keys[#keys + 1] = name
    byname[name] = key
  end
  table.sort(keys)
  local parts = {}
  for _, name in ipairs(keys) do
    parts[#parts + 1] = '"' .. name .. '":' .. canon(value[byname[name]])
  end
  return '{' .. table.concat(parts, ',') .. '}'
end

local function struct(label, value)
  structfd:write(label, ' ', canon(value), '\n')
end

-- ---------------------------------------------------------------------
-- Asking things.  An error is observable behaviour -- for most of :let
-- and :unlet it is the *only* observable behaviour -- so every failure
-- is reported, never swallowed.
-- ---------------------------------------------------------------------

--- Normalise an error to its message: a pcall against vim.fn or
--- vim.api prefixes the Lua source position, which is a line number in
--- this file and would re-baseline the artifact on any edit.
local function errtext(res)
  local text = scrub(tostring(res))
  text = text:gsub('^.-:%d+: ', '')
  -- nvim_exec2 prefixes the line number *inside the source it was
  -- given*, so a case that grows a setup line re-baselines its own
  -- answer.  The line number is never the question here.
  text = text:gsub('^nvim_exec2%(%), line %d+: ', '')
  text = text:gsub('^Vim:', '')
  text = text:gsub('^Vim%b():', '')
  return text
end

--- Evaluate a Vimscript expression that answers a *string*.  Kept
--- inside Vimscript on purpose: routing an answer through a Lua value
--- would mix evalsweep's converters into this sweep's question.
local function veval(label, expr)
  local ok, res = pcall(vim.fn.eval, expr)
  if not ok then
    emit(label, '!', esc(errtext(res)))
    struct(label, { err = errtext(res) })
    return nil
  end
  emit(label, '=', esc(scrub(res)))
  struct(label, res)
  return res
end

--- Run ex commands and report *everything they said*: the messages they
--- printed (nvim_exec2's captured output, which is the only view of the
--- `:let` listing's column alignment) or the error they raised.
local function exec(label, src)
  local ok, res = pcall(vim.api.nvim_exec2, src, { output = true })
  if not ok then
    emit(label, '!', esc(errtext(res)))
    struct(label, { err = errtext(res) })
    return nil
  end
  local out = res.output or ''
  emit(label, '=', esc(scrub(out)))
  struct(label, { out = out })
  return out
end

--- Run one ex command and then ask one expression about the state it
--- left behind.  Two report lines per case: what the command said, and
--- what the world looks like afterwards.  A rewrite that reports the
--- right error and performs half the assignment anyway is only visible
--- in the second line.
local function exec_then(label, src, after)
  exec(label, src)
  if after then
    veval(label .. ' after', after)
  end
end

--- Quietly run one or more ex commands whose failure is not the
--- question (fixture setup and teardown).  Returns the error text, or
--- nil.  Nothing is reported: a teardown that answers is a teardown
--- that re-baselines the artifact when a case above it changes.
local function quiet(src)
  local ok, res = pcall(vim.api.nvim_exec2, src, { output = false })
  if not ok then
    return errtext(res)
  end
  return nil
end

local SECTIONS = {}
local function section(name, fn)
  SECTIONS[#SECTIONS + 1] = { name = name, fn = fn }
end

--- Vimscript source for a value, by name.  One table so that every
--- section that needs "one of each type" spells the same set.
local VALUES = {
  { 'num', '7' },
  { 'negnum', '-3' },
  { 'flt', '1.5' },
  { 'str', "'ab'" },
  { 'estr', "''" },
  { 'list', '[1, 2]' },
  { 'dict', "{'a': 1}" },
  { 'blob', '0z00ff' },
  { 'null', 'v:null' },
  { 'true', 'v:true' },
  { 'func', "function('tr')" },
  { 'lambda', '{ x -> x }' },
}

-- ---------------------------------------------------------------------
-- s1 -- :let
-- ---------------------------------------------------------------------

-- Every kind of thing `:let` can assign to, with the command that
-- restores it and the expression that says what it now holds.  The
-- reads are deliberately not uniform: an option answers its own type,
-- an environment variable is always a string, and a register carries
-- its type separately.
local TARGETS = {
  { 'g', 'g:P', 'silent! unlet! g:P', "exists('g:P') ? string(g:P) . ' t' . type(g:P) : 'GONE'" },
  { 'b', 'b:P', 'silent! unlet! b:P', "exists('b:P') ? string(b:P) . ' t' . type(b:P) : 'GONE'" },
  { 'w', 'w:P', 'silent! unlet! w:P', "exists('w:P') ? string(w:P) . ' t' . type(w:P) : 'GONE'" },
  { 't', 't:P', 'silent! unlet! t:P', "exists('t:P') ? string(t:P) . ' t' . type(t:P) : 'GONE'" },
  { 'l', 'l:P', 'silent! unlet! l:P', "exists('l:P') ? string(l:P) . ' t' . type(l:P) : 'GONE'" },
  { 's', 's:P', 'silent! unlet! s:P', "exists('s:P') ? string(s:P) . ' t' . type(s:P) : 'GONE'" },
  { 'v', 'v:errmsg', "let v:errmsg = ''", 'string(v:errmsg)' },
  { 'env', '$VARSP', 'silent! unlet! $VARSP', "string($VARSP) . ' e' . exists('$VARSP')" },
  { 'optnum', '&textwidth', 'set textwidth=0', 'string(&textwidth)' },
  { 'optlocal', '&l:textwidth', 'setlocal textwidth=0', 'string(&l:textwidth)' },
  { 'optglobal', '&g:textwidth', 'setglobal textwidth=0', 'string(&g:textwidth)' },
  { 'optbool', '&ignorecase', 'set noignorecase', 'string(&ignorecase)' },
  { 'optstr', '&spelllang', 'set spelllang=en', 'string(&spelllang)' },
  { 'reg', '@a', "call setreg('a', '')", "string(@a) . ' ' . getregtype('a')" },
}

section('s1-let-target', function()
  -- The cross product of "what can be assigned to" and "what can be
  -- assigned".  Most of the interesting answers are error texts:
  -- ex_let_env, ex_let_option and ex_let_register each reject a
  -- different set of value types, and each with its own message.
  for _, target in ipairs(TARGETS) do
    local tag, lhs, reset, read = target[1], target[2], target[3], target[4]
    for _, value in ipairs(VALUES) do
      local label = string.format('lt %s %s', tag, value[1])
      quiet(reset)
      exec_then(label, string.format('let %s = %s', lhs, value[2]), read)
      quiet(reset)
    end
  end
end)

section('s1-let-op', function()
  -- The compound operators.  `.=` and `..=` differ (one is the
  -- deprecated spelling and both are string concatenation), `+=` is
  -- overloaded on List and Blob, and every other pairing is a type
  -- error whose text is the answer.
  local OPS = { '=', '+=', '-=', '*=', '/=', '%=', '.=', '..=' }
  local RHS = { { 'n', '2' }, { 'f', '0.5' }, { 's', "'x'" }, { 'l', '[3]' }, { 'd', "{'b': 2}" } }
  for _, seed in ipairs(VALUES) do
    for _, op in ipairs(OPS) do
      for _, rhs in ipairs(RHS) do
        local label = string.format('lo %s %s %s', seed[1], op:gsub('%p', { ['='] = 'e', ['+'] = 'p', ['-'] = 'm', ['*'] = 'x', ['/'] = 'd', ['%'] = 'r', ['.'] = 'c' }), rhs[1])
        quiet('silent! unlet! g:P')
        quiet('let g:P = ' .. seed[2])
        exec_then(
          label,
          string.format('let g:P %s %s', op, rhs[2]),
          "exists('g:P') ? string(g:P) . ' t' . type(g:P) : 'GONE'"
        )
      end
    end
  end
  quiet('silent! unlet! g:P')
  -- The same operators against the targets that never reach
  -- set_var_lval: ex_let_env, ex_let_option and ex_let_register each
  -- implement the compound forms themselves, and ex_let_option's
  -- five-way arithmetic switch is reachable from nothing else at all.
  local SIDE = {
    { 'optnum', '&textwidth', 'set textwidth=8', 'string(&textwidth)' },
    { 'optbool', '&ignorecase', 'set ignorecase', 'string(&ignorecase)' },
    { 'optstr', '&spelllang', 'set spelllang=en', 'string(&spelllang)' },
    { 'optlocal', '&l:textwidth', 'setlocal textwidth=8', 'string(&l:textwidth)' },
    { 'env', '$VARSP', "let $VARSP = '4'", 'string($VARSP)' },
    { 'reg', '@a', "call setreg('a', '4')", 'string(@a)' },
  }
  for _, target in ipairs(SIDE) do
    for _, op in ipairs(OPS) do
      for _, rhs in ipairs({ { 'n', '2' }, { 's', "'x'" }, { 'l', '[3]' } }) do
        local label = string.format(
          'lz %s %s %s',
          target[1],
          op:gsub('%p', { ['='] = 'e', ['+'] = 'p', ['-'] = 'm', ['*'] = 'x', ['/'] = 'd', ['%'] = 'r', ['.'] = 'c' }),
          rhs[1]
        )
        quiet(target[3])
        exec_then(label, string.format('let %s %s %s', target[2], op, rhs[2]), target[4])
      end
    end
    quiet(target[3])
  end
  quiet('set textwidth=0 noignorecase spelllang=en')
  quiet('silent! unlet! $VARSP')
  quiet("call setreg('a', '')")
end)

section('s1-let-index', function()
  -- Subscripted targets.  get_lval decides what a subscript means and
  -- ex_let_one applies it; every out-of-range, wrong-type and
  -- wrong-length arm has its own message, and the state afterwards is
  -- the half of the answer that says whether the assignment was
  -- partially performed.
  local SHAPES = {
    { 'list', '[1, 2, 3]' },
    { 'nested', '[[1, 2], [3, 4]]' },
    { 'dict', "{'a': 1, 'b': 2}" },
    { 'dictnest', "{'a': {'b': 1}}" },
    { 'str', "'abcd'" },
    { 'blob', '0z00112233' },
    { 'num', '7' },
  }
  local LHS = {
    { 'i0', '[0]' },
    { 'ineg', '[-1]' },
    { 'ibig', '[9]' },
    { 'istr', "['a']" },
    { 'sl01', '[0:1]' },
    { 'slopen', '[1:]' },
    { 'slhead', '[:1]' },
    { 'slneg', '[-2:-1]' },
    { 'slrev', '[2:0]' },
    { 'dot', '.a' },
    { 'dotmiss', '.zz' },
    { 'deep', '[0][0]' },
    { 'deepk', ".a.b" },
  }
  local RHS = { { 'n', '99' }, { 'l', '[8, 9]' }, { 'b', '0zaa' } }
  for _, shape in ipairs(SHAPES) do
    for _, lhs in ipairs(LHS) do
      for _, rhs in ipairs(RHS) do
        local label = string.format('li %s %s %s', shape[1], lhs[1], rhs[1])
        quiet('silent! unlet! g:P')
        quiet('let g:P = ' .. shape[2])
        exec_then(
          label,
          string.format('let g:P%s = %s', lhs[2], rhs[2]),
          "exists('g:P') ? string(g:P) : 'GONE'"
        )
      end
    end
  end
  quiet('silent! unlet! g:P')
end)

section('s1-let-unpack', function()
  -- ex_let_vars: the [a, b] and [a, b; rest] forms, their two count
  -- errors (E687 fewer targets, E688 more) and skip_var_list's own
  -- parse errors.
  local CASES = {
    { 'pair', 'let [g:A, g:B] = [1, 2]' },
    { 'few', 'let [g:A, g:B] = [1]' },
    { 'many', 'let [g:A, g:B] = [1, 2, 3]' },
    { 'one', 'let [g:A] = [1]' },
    { 'rest', 'let [g:A; g:B] = [1, 2, 3]' },
    { 'restempty', 'let [g:A; g:B] = [1]' },
    { 'restfew', 'let [g:A, g:B; g:C] = [1]' },
    { 'restonly', 'let [; g:A] = [1, 2]' },
    { 'doublesemi', 'let [g:A; g:B; g:C] = [1, 2, 3]' },
    { 'notlist', "let [g:A, g:B] = 'ab'" },
    { 'notlistnum', 'let [g:A, g:B] = 7' },
    { 'nested', 'let [g:A, [g:B, g:C]] = [1, [2, 3]]' },
    { 'unclosed', 'let [g:A, g:B = [1, 2]' },
    { 'empty', 'let [] = []' },
    { 'trailingcomma', 'let [g:A, ] = [1]' },
    { 'mixedscope', 'let [g:A, b:B, $VARSP, &textwidth, @a] = [1, 2, "e", 5, "r"]' },
    { 'index', 'let g:L = [0, 0] | let [g:L[0], g:L[1]] = [7, 8]' },
    { 'dup', 'let [g:A, g:A] = [1, 2]' },
    { 'op', 'let [g:A, g:B] += [1, 2]' },
    { 'const', 'const [g:A, g:B] = [1, 2]' },
    { 'blob', 'let [g:A, g:B] = 0z0102' },
  }
  for _, case in ipairs(CASES) do
    quiet('silent! unlet! g:A g:B g:C g:L')
    quiet('silent! unlet! $VARSP')
    quiet('set textwidth=0')
    exec_then(
      'up ' .. case[1],
      case[2],
      "join(map(['g:A', 'g:B', 'g:C', 'g:L'], "
        .. "'v:val . \"=\" . (exists(v:val) ? string(eval(v:val)) : \"GONE\")'), ' ')"
    )
    veval('up ' .. case[1] .. ' side', "string([$VARSP, &textwidth, @a])")
  end
  quiet('silent! unlet! g:A g:B g:C g:L')
  quiet('silent! unlet! $VARSP')
  quiet('set textwidth=0')
  quiet("call setreg('a', '')")
end)

section('s1-let-curly', function()
  -- Curly-brace names.  get_name_len() expands them before anything
  -- else looks at the name, so a failure here is reported by
  -- ex_let/list_arg_vars rather than by the assignment.
  local CASES = {
    { 'simple', "let g:base = 'x' | let g:{g:base}1 = 5" },
    { 'whole', "let g:base = 'g:zz' | let {g:base} = 6" },
    { 'concat', "let g:base = 'a' | let g:{g:base . 'b'} = 7" },
    { 'nested', "let g:base = 'a' | let g:i = 'base' | let g:{g:{g:i}} = 8" },
    { 'unclosed', "let g:base = 'x' | let g:{g:base = 9" },
    { 'stray', "let g:x} = 10" },
    { 'emptyexpr', 'let g:{} = 11' },
    { 'badexpr', 'let g:{nosuchvar} = 12' },
    { 'numexpr', 'let g:{1 + 1} = 13' },
    { 'scopeexpr', "let {'b:'}c = 14" },
    { 'envcurly', "let g:base = 'VARSP' | let ${g:base} = 'ev'" },
    { 'envcurlybad', 'let ${nosuchvar} = 15' },
    { 'read', "let g:base = 'x' | let g:{g:base}1 = 16 | echo g:{g:base}1" },
    { 'unlet', "let g:base = 'x' | let g:{g:base}1 = 17 | unlet g:{g:base}1" },
    { 'funcname', "let g:base = 'F' | let g:{g:base} = function('tr')" },
  }
  for _, case in ipairs(CASES) do
    quiet('silent! unlet! g:base g:i g:x g:x1 g:ab g:zz g:2 g:F g:c b:c')
    quiet('silent! unlet! $VARSP')
    exec('cb ' .. case[1], case[2])
    veval(
      'cb ' .. case[1] .. ' after',
      "join(map(['g:x1', 'g:zz', 'g:ab', 'g:2', 'g:F', 'b:c', '$VARSP'], "
        .. "'v:val . \"=\" . (exists(v:val) ? string(eval(v:val)) : \"GONE\")'), ' ')"
    )
  end
  quiet('silent! unlet! g:base g:i g:x g:x1 g:ab g:zz g:2 g:F g:c b:c')
  quiet('silent! unlet! $VARSP')
end)

section('s1-let-list', function()
  -- The listing forms.  list_one_var_a pads the name to column 22 and
  -- prefixes the value with one of `#`, `*`, `[`, `{` or a space
  -- depending on the type -- so the *message column* is the contract
  -- here, and `execute()` is the only way to see it.
  quiet('silent! unlet! g:ZZnum g:ZZstr g:ZZlist g:ZZdict g:ZZfn g:ZZflt g:ZZblob g:ZZlong')
  quiet('let g:ZZnum = 42')
  quiet("let g:ZZstr = 'hello world'")
  quiet('let g:ZZlist = [1, 2, 3]')
  quiet("let g:ZZdict = {'a': 1}")
  quiet("let g:ZZfn = function('tr')")
  quiet('let g:ZZflt = 1.5')
  quiet('let g:ZZblob = 0z00ff')
  quiet("let g:ZZlong = repeat('averylongvariablevalue', 4)")
  quiet("let g:ZZlongname_that_exceeds_the_column = 1")
  local NAMES = {
    'g:ZZnum',
    'g:ZZstr',
    'g:ZZlist',
    'g:ZZdict',
    'g:ZZfn',
    'g:ZZflt',
    'g:ZZblob',
    'g:ZZlong',
    'g:ZZlongname_that_exceeds_the_column',
  }
  for _, name in ipairs(NAMES) do
    exec('li1 ' .. name, 'let ' .. name)
  end
  exec('li1 multiple', 'let g:ZZnum g:ZZstr g:ZZlist')
  exec('li1 subscript', 'let g:ZZlist[0]')
  exec('li1 key', 'let g:ZZdict.a')
  exec('li1 missing', 'let g:ZZnosuch')
  exec('li1 mixed', 'let g:ZZnum g:ZZnosuch g:ZZstr')
  exec('li1 trailing', 'let g:ZZnum g:ZZstr trailing!garbage')
  exec('li1 curly', "let g:base = 'ZZ' | let g:{g:base}num")
  exec('li1 opt', 'let &textwidth')
  exec('li1 env', 'let $VARSP')
  exec('li1 reg', 'let @a')

  -- The whole-scope listings.  Filtered to this section's own prefix,
  -- because `let g:` answers every global the run has created so far
  -- and that would make the artifact a function of every section above.
  local function filtered(label, cmd, pat)
    local ok, res = pcall(vim.api.nvim_exec2, cmd, { output = true })
    if not ok then
      emit(label, '!', esc(errtext(res)))
      struct(label, { err = errtext(res) })
      return
    end
    local kept = {}
    for line in ((res.output or '') .. '\n'):gmatch('([^\n]*)\n') do
      if line:match(pat) then
        kept[#kept + 1] = line
      end
    end
    emit(label, '=', esc(scrub(table.concat(kept, '|'))))
    struct(label, kept)
  end
  quiet('let b:ZZb = 1')
  quiet('let w:ZZw = 2')
  quiet('let t:ZZt = 3')
  filtered('li1 scope-g', 'let g:', '^ZZ')
  -- `:let` with no argument lists g:, b:, w:, t: and v: in that order,
  -- and only the non-global ones carry their scope prefix -- so the
  -- filter has to allow both spellings or the answer is scope-g again.
  filtered('li1 scope-all', 'let', '^%a?:?ZZ')
  filtered('li1 scope-b', 'let b:', '^b:ZZ')
  filtered('li1 scope-w', 'let w:', '^w:ZZ')
  filtered('li1 scope-t', 'let t:', '^t:ZZ')
  filtered('li1 scope-v', 'let v:', '^v:t_')
  filtered('li1 scope-s', 'let s:', '.')
  filtered('li1 scope-l', 'let l:', '.')
  filtered('li1 scope-a', 'let a:', '.')
  exec('li1 scope-bad', 'let x:')
  exec('li1 scope-dict', 'let g:.a')
  quiet('silent! unlet! b:ZZb w:ZZw t:ZZt g:base')
  quiet('silent! unlet! g:ZZnum g:ZZstr g:ZZlist g:ZZdict g:ZZfn g:ZZflt g:ZZblob g:ZZlong')
  quiet('silent! unlet! g:ZZlongname_that_exceeds_the_column')
end)

-- ---------------------------------------------------------------------
-- s2 -- :unlet and :const
-- ---------------------------------------------------------------------

section('s2-unlet', function()
  -- do_unlet_var / do_unlet, and ex_unletlock's argument walk.  The
  -- second report line for each case is what survived: `:unlet a b c`
  -- stops at the first failure, and only the state says where.
  local SETUP = table.concat({
    'silent! unlet! g:A g:B g:L g:D g:C',
    'let g:A = 1',
    'let g:B = 2',
    'let g:L = [1, 2, 3]',
    "let g:D = {'a': 1, 'b': 2}",
    "let $VARSP = 'x'",
  }, '\n')
  local STATE = "join(map(['g:A', 'g:B', 'g:L', 'g:D'], "
    .. "'v:val . \"=\" . (exists(v:val) ? string(eval(v:val)) : \"GONE\")'), ' ') "
    .. ".. ' env=' . exists('$VARSP')"
  local CASES = {
    { 'one', 'unlet g:A' },
    { 'missing', 'unlet g:NOSUCH' },
    { 'missingbang', 'unlet! g:NOSUCH' },
    { 'two', 'unlet g:A g:B' },
    { 'stopsatfirst', 'unlet g:NOSUCH g:A' },
    { 'bangcontinues', 'unlet! g:NOSUCH g:A' },
    { 'index', 'unlet g:L[0]' },
    { 'indexneg', 'unlet g:L[-1]' },
    { 'indexbig', 'unlet g:L[9]' },
    { 'slice', 'unlet g:L[0:1]' },
    { 'sliceopen', 'unlet g:L[1:]' },
    { 'slicehead', 'unlet g:L[:1]' },
    { 'slicerev', 'unlet g:L[2:0]' },
    { 'sliceneg', 'unlet g:L[-2:-1]' },
    { 'key', 'unlet g:D.a' },
    { 'keybracket', "unlet g:D['b']" },
    { 'keymissing', 'unlet g:D.zz' },
    { 'keymissingbang', 'unlet! g:D.zz' },
    { 'wholedict', 'unlet g:D' },
    { 'env', 'unlet $VARSP' },
    { 'envmissing', 'unlet $VARSNOSUCH' },
    { 'opt', 'unlet &textwidth' },
    { 'reg', 'unlet @a' },
    { 'scope', 'unlet g:' },
    { 'vvar', 'unlet v:count' },
    { 'vvarerrmsg', 'unlet v:errmsg' },
    { 'locked', 'lockvar g:A | unlet g:A' },
    { 'lockedbang', 'lockvar g:A | unlet! g:A' },
    { 'lockedinner', 'lockvar! g:L | unlet g:L[0]' },
    { 'lockedcontainer', 'lockvar 1 g:L | unlet g:L[0]' },
    { 'noargs', 'unlet' },
    { 'curly', "let g:base = 'A' | unlet g:{g:base}" },
    { 'trailing', 'unlet g:A garbage!' },
    { 'func', "let g:F = function('tr') | unlet g:F" },
    { 'selfref', 'let g:C = [1] | call add(g:C, g:C) | unlet g:C' },
    { 'strindex', "let g:S = 'abc' | unlet g:S[0]" },
  }
  for _, case in ipairs(CASES) do
    quiet('silent! unlockvar! g:A')
    quiet('silent! unlockvar! g:L')
    quiet(SETUP)
    exec_then('ul ' .. case[1], case[2], STATE)
    quiet('silent! unlockvar! g:A')
    quiet('silent! unlockvar! g:L')
  end
  quiet('silent! unlet! g:A g:B g:L g:D g:C g:S g:F g:base')
  quiet('silent! unlet! $VARSP')
  quiet('set textwidth=0')
end)

section('s2-const', function()
  -- set_var_const's is_const arm: what a :const declaration locks, and
  -- what the second write to it says.
  local CASES = {
    { 'plain', 'const g:K = 1', 'let g:K = 2' },
    { 'redeclare', 'const g:K = 1', 'const g:K = 2' },
    { 'letafter', 'const g:K = 1', 'unlet g:K' },
    { 'listinner', 'const g:K = [1, 2]', 'let g:K[0] = 9' },
    { 'listadd', 'const g:K = [1, 2]', 'call add(g:K, 3)' },
    { 'dictinner', "const g:K = {'a': 1}", 'let g:K.a = 9' },
    { 'dictnew', "const g:K = {'a': 1}", 'let g:K.b = 9' },
    { 'dictnested', "const g:K = {'a': {'b': 1}}", 'let g:K.a.b = 9' },
    { 'nestedlist', 'const g:K = [[1]]', 'let g:K[0][0] = 9' },
    { 'unpack', 'const [g:K, g:K2] = [1, 2]', 'let g:K = 3' },
    { 'op', 'const g:K = 1', 'let g:K += 1' },
    { 'novalue', 'const g:K', 'echo 1' },
    { 'letfirst', 'let g:K = 1', 'const g:K = 2' },
    { 'islocked', 'const g:K = [1]', "echo islocked('g:K') . islocked('g:K[0]')" },
    { 'env', "const $VARSP = 'x'", "let $VARSP = 'y'" },
    { 'opt', 'const &textwidth = 5', 'let &textwidth = 6' },
    { 'reg', "const @a = 'x'", "let @a = 'y'" },
    { 'index', 'let g:L = [1] | const g:L[0] = 2', 'echo 1' },
    { 'unlockvar', 'const g:K = 1', 'unlockvar g:K | let g:K = 2' },
    { 'bang', 'const! g:K = 1', 'echo 1' },
  }
  for _, case in ipairs(CASES) do
    quiet('silent! unlockvar! g:K')
    quiet('silent! unlockvar! g:K2')
    quiet('silent! unlockvar! g:L')
    quiet('silent! unlet! g:K g:K2 g:L')
    quiet('silent! unlet! $VARSP')
    quiet('set textwidth=0')
    exec('co ' .. case[1] .. ' decl', case[2])
    exec_then(
      'co ' .. case[1] .. ' write',
      case[3],
      "join(map(['g:K', 'g:K2', 'g:L'], "
        .. "'v:val . \"=\" . (exists(v:val) ? string(eval(v:val)) : \"GONE\")'), ' ')"
    )
  end
  quiet('silent! unlockvar! g:K')
  quiet('silent! unlockvar! g:K2')
  quiet('silent! unlockvar! g:L')
  quiet('silent! unlet! g:K g:K2 g:L')
  quiet('silent! unlet! $VARSP')
  quiet('set textwidth=0')
  quiet("call setreg('a', '')")
end)

-- ---------------------------------------------------------------------
-- s3 -- lockvar / unlockvar / islocked
-- ---------------------------------------------------------------------

section('s3-lockdepth', function()
  -- do_lock_var walks `deep` levels down and stops; unlockvar walks the
  -- same way with the flag cleared.  evalsweep's lock section asks what
  -- a *write* to a locked container says; this one asks what the lock
  -- *is*, at every reachable path, which is the only view of the walk
  -- itself.
  local SHAPES = {
    { 'list', '[1, [2, [3]]]', { '', '[0]', '[1]', '[1][0]', '[1][1]', '[1][1][0]' } },
    { 'dict', "{'a': {'b': {'c': 1}}}", { '', '.a', '.a.b', '.a.b.c' } },
    { 'mixed', "{'a': [1, {'b': 2}]}", { '', '.a', '.a[0]', '.a[1]', '.a[1].b' } },
    { 'blob', '0z001122', { '', '[0]' } },
    { 'str', "'abc'", { '' } },
    { 'num', '7', { '' } },
  }
  local LOCKS = {
    { 'none', '' },
    { 'plain', 'lockvar g:L' },
    { 'bang', 'lockvar! g:L' },
    { 'd0', 'lockvar 0 g:L' },
    { 'd1', 'lockvar 1 g:L' },
    { 'd2', 'lockvar 2 g:L' },
    { 'd3', 'lockvar 3 g:L' },
    { 'd99', 'lockvar 99 g:L' },
    { 'unbang', 'lockvar! g:L | unlockvar g:L' },
    { 'unbangbang', 'lockvar! g:L | unlockvar! g:L' },
    { 'un1', 'lockvar! g:L | unlockvar 1 g:L' },
    { 'un2', 'lockvar! g:L | unlockvar 2 g:L' },
    { 'unonly', 'unlockvar g:L' },
    { 'relock', 'lockvar 1 g:L | lockvar! g:L' },
  }
  for _, shape in ipairs(SHAPES) do
    for _, lock in ipairs(LOCKS) do
      quiet('silent! unlockvar! g:L')
      quiet('silent! unlet! g:L')
      quiet('let g:L = ' .. shape[2])
      if lock[2] ~= '' then
        exec(string.format('ld %s %s cmd', shape[1], lock[1]), lock[2])
      end
      local parts = {}
      for _, path in ipairs(shape[3]) do
        parts[#parts + 1] = string.format("'%s:' . islocked('g:L%s')", path == '' and '.' or path, path)
      end
      veval(
        string.format('ld %s %s locked', shape[1], lock[1]),
        'join([' .. table.concat(parts, ', ') .. "], ' ')"
      )
      -- One write per level, so the lock's *effect* is recorded beside
      -- what islocked() claims about it.  They can disagree: the flag
      -- lives on the container, the check on the item.
      exec(
        string.format('ld %s %s write', shape[1], lock[1]),
        "try | let g:L = 0 | catch | echo 'top:' . v:exception | endtry"
      )
      quiet('silent! unlockvar! g:L')
      quiet('silent! unlet! g:L')
      quiet('let g:L = ' .. shape[2])
      if lock[2] ~= '' then
        quiet('silent! ' .. lock[2])
      end
      exec(
        string.format('ld %s %s writein', shape[1], lock[1]),
        "try | call extend(g:L, type(g:L) == v:t_dict ? {'zz': 1} : (type(g:L) == v:t_list ? [9] : 0)) "
          .. "| catch | echo 'in:' . v:exception | endtry\n"
          .. 'echo "state:" . string(g:L)'
      )
      quiet('silent! unlockvar! g:L')
    end
  end
  quiet('silent! unlet! g:L')
end)

section('s3-islocked', function()
  -- islocked()'s own arms, over names that are not a plain global.  The
  -- error texts are the answer for most of them.
  quiet('silent! unlet! g:L g:D g:F g:S')
  quiet('let g:L = [1, 2]')
  quiet("let g:D = {'a': 1}")
  quiet("let g:F = function('tr')")
  quiet("let g:S = 'abc'")
  quiet('lockvar g:S')
  local NAMES = {
    'g:L',
    'g:S',
    'g:D',
    'g:F',
    'g:L[0]',
    'g:L[9]',
    'g:D.a',
    "g:D['a']",
    'g:D.zz',
    'g:NOSUCH',
    'g:',
    'b:',
    'v:',
    'v:count',
    'v:true',
    'v:errmsg',
    'v:t_number',
    'l:',
    'l:x',
    'a:',
    'a:x',
    's:x',
    '&textwidth',
    '$HOME',
    '@a',
    '',
    '1bad',
    'nosuchscope:x',
    'g:L[0][0]',
    'g:S[0]',
  }
  for _, name in ipairs(NAMES) do
    veval(
      'il ' .. (name == '' and '<empty>' or name),
      string.format(
        "execute('try | echo islocked(%s) | catch | echo v:exception | endtry')",
        vim.fn.string(name):gsub("'", "''")
      )
    )
  end
  quiet('silent! unlockvar! g:S')
  quiet('silent! unlet! g:L g:D g:F g:S')
end)

section('s3-lockalias', function()
  -- A lock lives on the container, not on the name, so two names for
  -- one list are both locked -- but `let g:B = copy(g:A)` goes through
  -- set_var, which builds its own dictitem, and the copy is not.
  local CASES = {
    { 'alias', 'let g:A = [1, [2]]', 'let g:B = g:A', 'lockvar g:A' },
    { 'aliasdeep', 'let g:A = [1, [2]]', 'let g:B = g:A', 'lockvar! g:A' },
    { 'copy', 'let g:A = [1, [2]]', 'let g:B = copy(g:A)', 'lockvar! g:A' },
    { 'deepcopy', 'let g:A = [1, [2]]', 'let g:B = deepcopy(g:A)', 'lockvar! g:A' },
    { 'inner', 'let g:A = [1, [2]]', 'let g:B = g:A[1]', 'lockvar! g:A' },
    { 'innercopy', 'let g:A = [1, [2]]', 'let g:B = copy(g:A)', 'lockvar 2 g:A' },
    { 'dictalias', "let g:A = {'a': [1]}", 'let g:B = g:A', 'lockvar! g:A' },
    { 'dictcopy', "let g:A = {'a': [1]}", 'let g:B = copy(g:A)', 'lockvar! g:A' },
    { 'nested', "let g:A = {'a': [1]}", "let g:B = {'x': g:A}", 'lockvar! g:B' },
    { 'unlockalias', 'let g:A = [1, [2]]', 'let g:B = g:A | lockvar! g:A', 'unlockvar! g:B' },
    { 'reassign', 'let g:A = [1, [2]]', 'lockvar g:A', 'let g:B = g:A' },
    { 'scopealias', 'let g:A = [1]', "let g:B = g:['A']", 'lockvar! g:A' },
  }
  for _, case in ipairs(CASES) do
    quiet('silent! unlockvar! g:A')
    quiet('silent! unlockvar! g:B')
    quiet('silent! unlet! g:A g:B')
    exec('la ' .. case[1] .. ' setup', case[2] .. '\n' .. case[3] .. '\n' .. case[4])
    veval(
      'la ' .. case[1] .. ' locked',
      "join(map(['g:A', 'g:B', 'g:A[0]', 'g:B[0]'], "
        .. "'v:val . \":\" . string(islocked(v:val))'), ' ')"
    )
    exec(
      'la ' .. case[1] .. ' write',
      "try | let g:B[0] = 99 | catch | echo 'B0:' . v:exception | endtry\n"
        .. "try | call add(g:B, 5) | catch | echo 'addB:' . v:exception | endtry\n"
        .. 'echo "A=" . string(g:A) . " B=" . string(g:B)'
    )
    quiet('silent! unlockvar! g:A')
    quiet('silent! unlockvar! g:B')
  end
  quiet('silent! unlet! g:A g:B')
end)

-- ---------------------------------------------------------------------
-- s4 -- the scope dictionaries
-- ---------------------------------------------------------------------

section('s4-scopedict', function()
  -- A scope is a real Dict behind a ScopeDictDictItem, so every dict
  -- builtin reaches it -- and init_var_dict() marks the entry itself
  -- fixed and read-only, which is what most of these answer about.
  -- Every read is filtered to this section's own prefix: `keys(g:)`
  -- answers every global the run has created so far, and recording that
  -- would make the artifact a function of every section above.
  local SCOPES = { 'g:', 'b:', 'w:', 't:', 'v:', 'l:', 'a:', 's:' }
  for _, scope in ipairs(SCOPES) do
    local tag = 'sd ' .. scope
    quiet(string.format('silent! unlet! %sQQa %sQQb', scope, scope))
    quiet(string.format('silent! let %sQQa = 1', scope))
    quiet(string.format('silent! let %sQQb = [2]', scope))
    veval(tag .. ' type', string.format("string(type(%s)) . ' ' . string(typename(%s))", scope, scope))
    veval(
      tag .. ' keys',
      string.format("string(sort(filter(keys(%s), 'v:val =~# \"^QQ\"')))", scope)
    )
    veval(tag .. ' haskey', string.format("string(has_key(%s, 'QQa')) . string(has_key(%s, 'zz'))", scope, scope))
    veval(tag .. ' index', string.format("string(get(%s, 'QQa', 'DEF'))", scope))
    veval(tag .. ' bracket', string.format("execute('try | echo %s[\"QQa\"] | catch | echo v:exception | endtry')", scope))
    veval(tag .. ' empty', string.format('string(empty(%s))', scope))
    veval(tag .. ' locked', string.format("string(islocked('%s'))", scope))
    exec(tag .. ' setitem', string.format("try | let %s['QQc'] = 3 | catch | echo v:exception | endtry\necho string(get(%s, 'QQc', 'GONE'))", scope, scope))
    exec(tag .. ' remove', string.format("try | call remove(%s, 'QQa') | catch | echo v:exception | endtry\necho string(get(%s, 'QQa', 'GONE'))", scope, scope))
    exec(tag .. ' extend', string.format("try | call extend(%s, {'QQd': 4}) | catch | echo v:exception | endtry\necho string(get(%s, 'QQd', 'GONE'))", scope, scope))
    exec(tag .. ' assign', string.format("try | let %s = {} | catch | echo v:exception | endtry", scope))
    exec(tag .. ' unletself', string.format("try | unlet %s | catch | echo v:exception | endtry", scope))
    exec(tag .. ' lock', string.format("try | lockvar %s | catch | echo v:exception | endtry", scope))
    -- MUST be undone here.  `:lockvar g:` locks two levels down, i.e.
    -- every global and every v: variable, and nothing below would ever
    -- unlock them: the first draft of this section left v:oldfiles and
    -- v:completed_item locked, which made s6-vvars answer E741 to every
    -- write and hid before_set_vvar's E963 arm entirely (measured: a
    -- mutation on it was NOT CAUGHT).
    quiet(string.format('silent! unlockvar! %s', scope))
    exec(tag .. ' filter', string.format("try | call filter(%s, 'v:key !~# \"^QQ\"') | catch | echo v:exception | endtry\necho string(sort(filter(keys(%s), 'v:val =~# \"^QQ\"')))", scope, scope))
    quiet(string.format('silent! unlet! %sQQa %sQQb %sQQc %sQQd', scope, scope, scope, scope))
  end
  -- The dict-valued forms of a scope reference.  `g:['x']` and `g:x`
  -- take different paths through find_var_ht_dict.
  quiet('let g:QQz = 5')
  veval('sd deref bracket', "string(g:['QQz'])")
  veval('sd deref get', "string(get(g:, 'QQz'))")
  exec('sd deref set', "let g:['QQy'] = 6\necho string(g:QQy)")
  exec('sd deref unlet', "unlet g:['QQy']\necho string(exists('g:QQy'))")
  veval('sd deref nested', "string(has_key(g:, 'QQz'))")
  exec('sd bad scope', "try | echo x:foo | catch | echo v:exception | endtry")
  exec('sd bad let', "try | let x:foo = 1 | catch | echo v:exception | endtry")
  quiet('silent! unlet! g:QQz g:QQy')
end)

section('s4-bufwinvar', function()
  -- get_var_from / getwinvar / setwinvar: the missing-variable arm, the
  -- default argument, the whole-dictionary arm (an empty name), the
  -- `&option` arm and every invalid handle.
  quiet('silent! unlet! g:VB')
  quiet("let g:VB = bufadd('varsbufA')")
  quiet('call bufload(g:VB)')
  quiet('split')
  quiet('tabnew')
  quiet('tabprevious')
  quiet('call setbufvar(g:VB, "bv", 11)')
  quiet('call setwinvar(1, "wv", 12)')
  quiet('call setwinvar(2, "wv", 13)')
  quiet('call settabvar(1, "tv", 14)')
  quiet('call settabvar(2, "tv", 15)')
  quiet('call settabwinvar(2, 1, "twv", 16)')

  local GETS = {
    { 'buf-present', "getbufvar(g:VB, 'bv')" },
    { 'buf-missing', "getbufvar(g:VB, 'nosuch')" },
    { 'buf-default', "getbufvar(g:VB, 'nosuch', 'DEF')" },
    { 'buf-alldict', "sort(filter(keys(getbufvar(g:VB, '')), 'v:val =~# \"^bv\\\\|^changedtick\"'))" },
    { 'buf-alldict-def', "type(getbufvar(g:VB, '', 'DEF'))" },
    { 'buf-opt', "getbufvar(g:VB, '&textwidth')" },
    { 'buf-opt-missing', "getbufvar(g:VB, '&nosuchopt')" },
    { 'buf-opt-missing-def', "getbufvar(g:VB, '&nosuchopt', 'DEF')" },
    { 'buf-optamp', "type(getbufvar(g:VB, '&'))" },
    { 'buf-badhandle', "getbufvar(99999, 'bv')" },
    { 'buf-badhandle-def', "getbufvar(99999, 'bv', 'DEF')" },
    { 'buf-zero', "getbufvar(0, 'bv', 'DEF')" },
    { 'buf-byname', "getbufvar('varsbufA', 'bv')" },
    { 'buf-changedtick', "getbufvar(g:VB, 'changedtick')" },
    { 'win-present', "getwinvar(1, 'wv')" },
    { 'win-other', "getwinvar(2, 'wv')" },
    { 'win-missing', "getwinvar(1, 'nosuch')" },
    { 'win-default', "getwinvar(1, 'nosuch', 'DEF')" },
    { 'win-alldict', "sort(filter(keys(getwinvar(1, '')), 'v:val =~# \"^wv\"'))" },
    { 'win-opt', "getwinvar(1, '&number')" },
    { 'win-opt-missing-def', "getwinvar(1, '&nosuchopt', 'DEF')" },
    { 'win-zero', "getwinvar(0, 'wv')" },
    { 'win-bad', "getwinvar(99, 'wv')" },
    { 'win-bad-def', "getwinvar(99, 'wv', 'DEF')" },
    { 'tab-present', "gettabvar(1, 'tv')" },
    { 'tab-other', "gettabvar(2, 'tv')" },
    { 'tab-missing', "gettabvar(1, 'nosuch')" },
    { 'tab-default', "gettabvar(1, 'nosuch', 'DEF')" },
    { 'tab-bad', "gettabvar(99, 'tv', 'DEF')" },
    { 'tab-alldict', "sort(filter(keys(gettabvar(1, '')), 'v:val =~# \"^tv\"'))" },
    { 'tabwin-present', "gettabwinvar(2, 1, 'twv')" },
    { 'tabwin-missing', "gettabwinvar(2, 1, 'nosuch')" },
    { 'tabwin-default', "gettabwinvar(2, 1, 'nosuch', 'DEF')" },
    { 'tabwin-opt', "gettabwinvar(2, 1, '&number')" },
    { 'tabwin-badtab', "gettabwinvar(99, 1, 'twv', 'DEF')" },
    { 'tabwin-badwin', "gettabwinvar(2, 99, 'twv', 'DEF')" },
  }
  for _, case in ipairs(GETS) do
    veval(
      'bw ' .. case[1],
      string.format(
        "execute('try | echo string(%s) | catch | echo v:exception | endtry')",
        case[2]:gsub("'", "''")
      )
    )
  end

  local SETS = {
    { 'buf-new', "call setbufvar(g:VB, 'nv', 21)", "getbufvar(g:VB, 'nv')" },
    { 'buf-opt', "call setbufvar(g:VB, '&textwidth', 22)", "getbufvar(g:VB, '&textwidth')" },
    { 'buf-opt-bad', "call setbufvar(g:VB, '&nosuchopt', 1)", "1" },
    { 'buf-opt-list', "call setbufvar(g:VB, '&textwidth', [1])", "getbufvar(g:VB, '&textwidth')" },
    { 'buf-badhandle', "call setbufvar(99999, 'nv', 23)", "1" },
    { 'buf-changedtick', "call setbufvar(g:VB, 'changedtick', 99)", "getbufvar(g:VB, 'changedtick')" },
    { 'buf-emptyname', "call setbufvar(g:VB, '', 24)", "1" },
    { 'buf-listval', "call setbufvar(g:VB, 'lv', [1, 2])", "getbufvar(g:VB, 'lv')" },
    { 'win-new', "call setwinvar(1, 'nv', 25)", "getwinvar(1, 'nv')" },
    { 'win-opt', "call setwinvar(1, '&number', 1)", "getwinvar(1, '&number')" },
    { 'win-bad', "call setwinvar(99, 'nv', 26)", "1" },
    { 'win-emptyname', "call setwinvar(1, '', 27)", "1" },
    { 'tab-new', "call settabvar(1, 'nv', 28)", "gettabvar(1, 'nv')" },
    { 'tab-bad', "call settabvar(99, 'nv', 29)", "1" },
    { 'tab-emptyname', "call settabvar(1, '', 30)", "1" },
    { 'tab-scoped', "call settabvar(1, 't:nv2', 31)", "gettabvar(1, 'nv2')" },
    { 'tabwin-new', "call settabwinvar(2, 1, 'nv', 32)", "gettabwinvar(2, 1, 'nv')" },
    { 'tabwin-opt', "call settabwinvar(2, 1, '&number', 0)", "gettabwinvar(2, 1, '&number')" },
    { 'tabwin-badtab', "call settabwinvar(99, 1, 'nv', 33)", "1" },
  }
  for _, case in ipairs(SETS) do
    exec('bw set-' .. case[1], string.format('try | %s | catch | echo v:exception | endtry', case[2]))
    veval(
      'bw set-' .. case[1] .. ' after',
      string.format(
        "execute('try | echo string(%s) | catch | echo v:exception | endtry')",
        case[3]:gsub("'", "''")
      )
    )
  end

  -- b:changedtick: the one scope entry with a hand-written read-only
  -- path in set_var_const, plus its DI_FLAGS_FIX arm.
  local TICK = {
    { 'read', "echo type(b:changedtick)" },
    { 'assign', 'let b:changedtick = 5' },
    { 'unlet', 'unlet b:changedtick' },
    { 'unletbang', 'unlet! b:changedtick' },
    { 'lock', 'lockvar b:changedtick' },
    { 'unlock', 'unlockvar b:changedtick' },
    { 'islocked', "echo islocked('b:changedtick')" },
    { 'remove', "call remove(b:, 'changedtick')" },
    { 'extend', "call extend(b:, {'changedtick': 7})" },
    { 'haskey', "echo has_key(b:, 'changedtick')" },
    { 'inkeys', "echo index(keys(b:), 'changedtick') >= 0" },
    { 'bump', 'let g:t0 = b:changedtick | call setline(1, "x") | echo b:changedtick > g:t0' },
  }
  for _, case in ipairs(TICK) do
    exec('bw tick-' .. case[1], string.format('try | %s | catch | echo v:exception | endtry', case[2]))
  end

  quiet('silent! unlet! g:t0')
  quiet('silent! tabonly!')
  quiet('silent! only!')
  quiet('silent! bwipeout! varsbufA')
  quiet('silent! unlet! g:VB w:wv w:nv t:tv t:nv t:nv2')
end)

-- ---------------------------------------------------------------------
-- s5 -- function scopes
-- ---------------------------------------------------------------------

section('s5-funcscope', function()
  -- l: and a: live in a funccall_T, so every question about them has to
  -- be asked from inside a function.  find_var_in_scoped_ht() and the
  -- get_funccal_* accessors are the vars.rs side of that, and nothing
  -- outside a function reaches them at all.
  local BODY = {
    { 'l-set', 'let l:x = 1', "string(l:x) . string(exists('l:x')) . string(exists('x'))" },
    { 'l-bare', 'let x = 2', "string(l:x) . string(exists('l:x'))" },
    { 'l-unlet', 'let l:x = 3 | unlet l:x', "string(exists('l:x'))" },
    { 'l-unletbare', 'let x = 4 | unlet x', "string(exists('l:x'))" },
    { 'l-keys', "let l:x = 5 | let l:y = 6", "string(sort(keys(l:)))" },
    { 'l-dict', 'let l:x = 7', "string(type(l:)) . string(get(l:, 'x'))" },
    { 'l-locked', 'let l:x = 8 | lockvar l:x', "string(islocked('l:x')) . execute('try | let l:x = 9 | catch | echo v:exception | endtry')" },
    { 'l-shadow', 'let l:g = 10', "string(l:g) . string(exists('g:g'))" },
    { 'l-const', 'const l:c = 11', "execute('try | let l:c = 12 | catch | echo v:exception | endtry')" },
    { 'a-read', 'let l:_ = 0', "string(a:one) . string(a:two)" },
    { 'a-count', 'let l:_ = 0', 'string(a:0) . string(a:000)' },
    { 'a-keys', 'let l:_ = 0', 'string(sort(keys(a:)))' },
    { 'a-dict', 'let l:_ = 0', "string(type(a:)) . string(get(a:, 'one'))" },
    { 'a-assign', 'let l:_ = 0', "execute('try | let a:one = 99 | catch | echo v:exception | endtry')" },
    { 'a-unlet', 'let l:_ = 0', "execute('try | unlet a:one | catch | echo v:exception | endtry')" },
    { 'a-islocked', 'let l:_ = 0', "string(islocked('a:one')) . string(islocked('a:000'))" },
    { 'a-000-write', 'let l:_ = 0', "execute('try | let a:000[0] = 1 | catch | echo v:exception | endtry')" },
    { 'a-missing', 'let l:_ = 0', "execute('try | echo a:nosuch | catch | echo v:exception | endtry')" },
    { 'a-scopeset', 'let l:_ = 0', "execute('try | let a:zz = 1 | catch | echo v:exception | endtry')" },
    { 'a-lock', 'let l:_ = 0', "execute('try | lockvar a: | catch | echo v:exception | endtry')" },
    { 'l-list', 'let l:x = 13', "execute('let l:')" },
    { 'a-list', 'let l:_ = 0', "execute('let a:')" },
    { 'nested', 'let l:x = 14', 'string(VarsInner())' },
    { 'lambda', 'let l:x = 15 | let l:F = { -> l:x }', 'string(l:F())' },
    { 's-from-fn', 'let l:_ = 0', "execute('try | let s:sv = 1 | echo s:sv | catch | echo v:exception | endtry')" },
    { 'g-from-fn', 'let g:gv = 16', "string(g:gv) . string(exists('gv'))" },
  }
  quiet(table.concat({
    'function! VarsInner() abort',
    "  return [exists('l:x'), exists('a:one'), get(l:, 'x', 'NOL')]",
    'endfunction',
  }, '\n'))
  for _, case in ipairs(BODY) do
    quiet('silent! unlet! g:gv')
    local src = table.concat({
      'function! VarsProbe(one, two, ...) abort',
      '  ' .. case[2],
      '  return ' .. case[3],
      'endfunction',
    }, '\n')
    quiet('silent! delfunction! VarsProbe')
    local err = quiet(src)
    if err then
      emit('fn ' .. case[1] .. ' define', '!', esc(err))
      struct('fn ' .. case[1] .. ' define', { err = err })
    else
      veval(
        'fn ' .. case[1],
        "execute('try | echo VarsProbe(\"A\", \"B\", \"C\") | catch | echo v:exception | endtry')"
      )
    end
  end
  -- Range and dict functions: a:firstline/a:lastline exist only for the
  -- first, `self` only for the second.
  quiet(table.concat({
    'function! VarsRange() range abort',
    '  return [a:firstline, a:lastline, sort(keys(a:))]',
    'endfunction',
    "let g:VD = {'k': 1}",
    'function! g:VD.m() abort',
    "  return [self.k, sort(keys(l:)), exists('l:self')]",
    'endfunction',
  }, '\n'))
  veval('fn range', "execute('try | echo VarsRange() | catch | echo v:exception | endtry')")
  veval('fn dict', "execute('try | echo g:VD.m() | catch | echo v:exception | endtry')")
  veval('fn range-outside', "execute('try | echo string(a:firstline) | catch | echo v:exception | endtry')")
  veval('fn l-outside', "execute('try | echo string(l:) | catch | echo v:exception | endtry')")
  veval('fn a-outside', "execute('try | echo string(a:) | catch | echo v:exception | endtry')")
  veval('fn l-let-outside', "execute('try | let l:oz = 1 | echo l:oz | catch | echo v:exception | endtry')")
  quiet('silent! delfunction! VarsProbe')
  quiet('silent! delfunction! VarsInner')
  quiet('silent! delfunction! VarsRange')
  quiet('silent! unlet! g:VD g:gv g:oz')
end)

-- ---------------------------------------------------------------------
-- s6 -- v:
-- ---------------------------------------------------------------------

section('s6-vvars', function()
  -- Every v: variable carries vv_flags (VV_RO / VV_RO_SBX / VV_COMPAT)
  -- and a fixed type; before_set_vvar and var_check_ro/_fixed decide
  -- what a write, a delete and a lock answer.  The set below covers
  -- every flag combination and every v_type the table declares.
  local NAMES = {
    'count',
    'count1',
    'prevcount',
    'errmsg',
    'statusmsg',
    'warningmsg',
    'shell_error',
    'this_session',
    'version',
    'lnum',
    'register',
    'hlsearch',
    'oldfiles',
    'errors',
    'completed_item',
    'progpath',
    'servername',
    'exception',
    'throwpoint',
    'char',
    'val',
    'key',
    'fname_in',
    'cmdarg',
    'swapname',
    'searchforward',
    'scrollstart',
    'testing',
    'event',
    'msgpack_types',
    'true',
    'false',
    'null',
    't_number',
    't_string',
    'numbermax',
    'numbermin',
    'numbersize',
    'argv',
    'option_new',
    'option_type',
    'nosuchvvar',
  }
  -- Every write reads the value back in the same expression: what a
  -- rejected write *says* is only half the contract, and the arms of
  -- before_set_vvar that convert rather than refuse (the VAR_STRING
  -- and VAR_NUMBER ones, which never reach the type check at all) are
  -- invisible without the read.  `%s` is substituted everywhere, so a
  -- probe may name the variable as often as it likes.
  local PROBES = {
    { 'type', 'string(type(v:%s))' },
    { 'exists', "string(exists('v:%s'))" },
    { 'islocked', "string(islocked('v:%s'))" },
    { 'setnum', "execute('try | let v:%s = 3 | echo \"->\" . string(v:%s) | catch | echo v:exception | endtry')" },
    { 'setstr', "execute('try | let v:%s = \"s\" | echo \"->\" . string(v:%s) | catch | echo v:exception | endtry')" },
    { 'setlist', "execute('try | let v:%s = [1] | echo \"->\" . string(v:%s) | catch | echo v:exception | endtry')" },
    { 'setdict', "execute('try | let v:%s = {} | echo \"->\" . string(v:%s) | catch | echo v:exception | endtry')" },
    { 'unlet', "execute('try | unlet v:%s | catch | echo v:exception | endtry')" },
    { 'lock', "execute('try | lockvar v:%s | catch | echo v:exception | endtry')" },
    { 'unlock', "execute('try | unlockvar v:%s | catch | echo v:exception | endtry')" },
    { 'scopeset', "execute('try | let v:[\"%s\"] = 4 | catch | echo v:exception | endtry')" },
    -- The same write, read back: `scopeset` records what the write
    -- *said*, this what it left behind.  Writing a v: variable through
    -- the scope dictionary is the one path that can change its declared
    -- type (O-B14-10), and only the read shows it.
    { 'scopetype', "execute('try | let v:[\"%s\"] = 4 | echo \"->\" . string(type(v:%s)) | catch | echo v:exception | endtry')" },
    { 'remove', "execute('try | call remove(v:, \"%s\") | catch | echo v:exception | endtry')" },
  }
  -- A zero value per v:t_* code, used to put a variable back after the
  -- `scopeset` probe -- which is a divergence, not a synonym for
  -- `:let`: `let v:['errmsg'] = 4` goes through the scope *dictionary*
  -- and never reaches before_set_vvar, so it re-types the variable
  -- permanently.  Recording that is the point; leaking it into every
  -- section below is not (it silently turned s7's E730 into E745).
  local ZERO = {
    [0] = '0',
    [1] = "''",
    [2] = "function('tr')",
    [3] = '[]',
    [4] = '{}',
    [5] = '0.0',
    [6] = 'v:false',
    [7] = 'v:null',
    [10] = '0z',
  }
  for _, name in ipairs(NAMES) do
    local okty, ty = pcall(vim.fn.eval, 'type(v:' .. name .. ')')
    for _, probe in ipairs(PROBES) do
      veval(
        string.format('vv %s %s', name, probe[1]),
        string.format(
          "execute('try | echo %s | catch | echo v:exception | endtry')",
          (probe[2]:gsub('%%s', name)):gsub("'", "''")
        )
      )
    end
    if okty and ZERO[ty] then
      quiet(string.format("silent! let v:['%s'] = %s", name, ZERO[ty]))
    end
  end
  -- Put back what the writes above may have changed.  v:testing in
  -- particular turns on assertions elsewhere in the process.
  quiet("let v:testing = 0\nlet v:errmsg = ''\nlet v:statusmsg = ''\nlet v:warningmsg = ''")
  quiet("let v:this_session = ''\nlet v:searchforward = 1\nlet v:oldfiles = []\nlet v:errors = []")

  -- Writing a v: variable *through the scope dictionary*, with every
  -- operator, against the plain spelling of the same write.  This is
  -- O-B14-10's ground: `let v:name = …` reaches before_set_vvar, which
  -- coerces into the declared type or raises E963 and runs the two side
  -- effects; `let v:['name'] = …` used to reach the dictionary item
  -- directly and replace the type outright.  b:changedtick is asked the
  -- same way because it is the other hand-written read-only entry, and
  -- the same question was open for it.
  local SCOPEDICT = {
    { 'errmsg-num', "let v:['errmsg'] = 4", 'string([type(v:errmsg), v:errmsg])' },
    { 'errmsg-list', "let v:['errmsg'] = [1]", 'string([type(v:errmsg), v:errmsg])' },
    { 'errmsg-concat', "let v:['errmsg'] .= 'z'", 'string([type(v:errmsg), v:errmsg])' },
    { 'errmsg-add', "let v:['errmsg'] += 3", 'string([type(v:errmsg), v:errmsg])' },
    { 'errmsg-plain-num', 'let v:errmsg = 4', 'string([type(v:errmsg), v:errmsg])' },
    { 'oldfiles-num', "let v:['oldfiles'] = 1", 'string(type(v:oldfiles))' },
    { 'oldfiles-str', "let v:['oldfiles'] = 'x'", 'string(type(v:oldfiles))' },
    { 'oldfiles-list', "let v:['oldfiles'] = ['a']", 'string(v:oldfiles)' },
    { 'oldfiles-plain-num', 'let v:oldfiles = 1', 'string(type(v:oldfiles))' },
    { 'searchforward-str', "let v:['searchforward'] = 'no'", 'string([type(v:searchforward), v:searchforward])' },
    { 'searchforward-concat', "let v:['searchforward'] .= 'z'", 'string([type(v:searchforward), v:searchforward])' },
    { 'searchforward-plain-concat', "let v:searchforward .= 'z'", 'string([type(v:searchforward), v:searchforward])' },
    { 'hlsearch-str', "let v:['hlsearch'] = 'x'", 'string([type(v:hlsearch), v:hlsearch])' },
    { 'errors-num', "let v:['errors'] = 5", 'string(type(v:errors))' },
    { 'errors-add', "let v:['errors'] += ['e']", 'string(v:errors)' },
    { 'completed-num', "let v:['completed_item'] = 1", 'string(type(v:completed_item))' },
    { 'char-num', "let v:['char'] = 1", 'string([type(v:char), v:char])' },
    { 'count-ro', "let v:['count'] = 9", 'string(v:count)' },
    { 'true-ro', "let v:['true'] = 9", 'string(v:true)' },
    { 'nosuch', "let v:['nosuchvvar'] = 1", "string(exists('v:nosuchvvar'))" },
    { 'changedtick', "let b:['changedtick'] = 1", 'string([type(b:changedtick), b:changedtick > 0])' },
    { 'changedtick-concat', "let b:['changedtick'] .= 'z'", 'string([type(b:changedtick), b:changedtick > 0])' },
    { 'changedtick-plain', 'let b:changedtick = 1', 'string([type(b:changedtick), b:changedtick > 0])' },
  }
  -- The reset goes through the scope dictionary on purpose: before the
  -- fix that is the only spelling that can put a re-typed variable back.
  local RESET = table.concat({
    "silent! let v:['errmsg'] = ''",
    "silent! let v:['searchforward'] = 1",
    "silent! let v:['hlsearch'] = 1",
    "silent! let v:['oldfiles'] = []",
    "silent! let v:['errors'] = []",
    "silent! let v:['char'] = ''",
    "silent! let v:['completed_item'] = {}",
  }, '\n')
  for _, case in ipairs(SCOPEDICT) do
    quiet(RESET)
    exec_then('vvdict ' .. case[1], case[2], case[3])
  end
  quiet(RESET)
end)

-- ---------------------------------------------------------------------
-- s7 -- error texts, one direct trigger each
-- ---------------------------------------------------------------------

section('s7-errors', function()
  -- The exact wording, punctuation and quoting of every message vars.c
  -- can produce.  A rewrite changes these silently and no type checker
  -- sees it; the sections above reach most of them incidentally, this
  -- one names them so a missing arm is a missing *line*.
  local CASES = {
    { 'undefined-read', 'echo g:NOSUCHVAR' },
    { 'undefined-let', 'let g:NOSUCHVAR .= 1' },
    { 'undefined-l', 'echo l:NOSUCHVAR' },
    { 'illegal-name', 'let 1bad = 1' },
    { 'illegal-name2', 'let g:1bad = 1' },
    { 'illegal-name-space', 'let g:a b = 1' },
    { 'illegal-scope', 'let q:x = 1' },
    { 'funcref-lower', "let g:lower = function('tr')" },
    { 'funcref-upper', "let g:Upper = function('tr')" },
    { 'funcref-scoped', "let b:lower = function('tr')" },
    { 'conflict-func', "function! Conflict()\nendfunction\nlet Conflict = 1" },
    { 'conflict-func-g', "function! Conflict2()\nendfunction\nlet g:Conflict2 = 1" },
    { 'unexpected', 'let g:x 1' },
    { 'no-equals', 'let g:x ==' },
    { 'trailing', 'let g:x = 1 trailing' },
    { 'listlast', 'let g:L = [1, 2] | let g:L[0:] = [1] | let g:L[:] = 0' },
    { 'slice-notlist', 'let g:L = [1, 2] | let g:L[0:1] = 5' },
    { 'slice-short', 'let g:L = [1, 2, 3] | let g:L[0:2] = [1]' },
    { 'slice-long', 'let g:L = [1, 2, 3] | let g:L[0:1] = [1, 2, 3]' },
    { 'number-required', "let &textwidth = 'abc'" },
    { 'unknown-option', 'let &nosuchoption = 1' },
    { 'option-list', 'let &textwidth = [1]' },
    { 'env-list', 'let $VARSP = [1]' },
    { 'reg-list', 'let @a = [1]' },
    { 'reg-bad', "let @\x01 = 'x'" },
    { 'lock-env', 'lockvar $VARSP' },
    { 'lock-option', 'lockvar &textwidth' },
    { 'lock-register', 'lockvar @a' },
    { 'lock-func', "lockvar tr" },
    { 'lock-missing', 'lockvar g:NOSUCHVAR' },
    { 'unlock-missing', 'unlockvar g:NOSUCHVAR' },
    { 'lock-noargs', 'lockvar' },
    { 'lock-depth-str', 'lockvar x g:L' },
    { 'sandbox-let', 'sandbox let g:x = 1' },
    { 'sandbox-vvar', 'sandbox let v:errmsg = "x"' },
    { 'vvar-type', 'let v:errmsg = [1]' },
    { 'vvar-ro', 'let v:count = 1' },
    { 'vvar-fixed', 'unlet v:true' },
    { 'dict-notdict', "let g:N = 5 | let g:N.a = 1" },
    { 'index-str-assign', "let g:S = 'abc' | let g:S[0] = 'z'" },
    { 'nested-missing', "let g:D = {} | let g:D.a.b = 1" },
    { 'func-as-var', 'let g:F = tr' },
    { 'let-empty', 'let =' },
    { 'let-eq-only', 'let g:x =' },
    { 'let-star', 'let g:x =* 1' },
    { 'unlet-func', "function! Delme()\nendfunction\nunlet Delme" },
    { 'redir-badname', 'redir => 1bad' },
    { 'redir-nested', 'redir => g:r1 | redir => g:r2 | redir END' },
    { 'setvar-toolong', 'let g:' .. string.rep('n', 300) .. ' = 1' },
    { 'autoload', 'echo g:nosuchpkg#nosuchvar' },
    { 'autoload-let', 'let g:nosuchpkg#var = 1' },
  }
  for _, case in ipairs(CASES) do
    quiet('silent! unlockvar! g:L')
    quiet('silent! unlet! g:x g:L g:N g:S g:D g:F g:lower g:Upper b:lower g:r1 g:r2')
    quiet('silent! unlet! $VARSP')
    quiet('set textwidth=0')
    quiet('silent! redir END')
    exec('er ' .. case[1], case[2])
    quiet('silent! redir END')
  end
  quiet('silent! delfunction! Conflict')
  quiet('silent! delfunction! Conflict2')
  quiet('silent! delfunction! Delme')
  quiet('silent! unlet! g:x g:L g:N g:S g:D g:F g:lower g:Upper b:lower g:r1 g:r2 g:Conflict2')
  quiet('silent! unlet! $VARSP')
  quiet('set textwidth=0')
  quiet("call setreg('a', '')")
end)

-- ---------------------------------------------------------------------
-- s8 -- heredoc assignment
-- ---------------------------------------------------------------------

section('s8-heredoc', function()
  -- heredoc_get() plus eval_all_expr_in_str/eval_one_expr_in_str.  The
  -- indentation rules ('trim'), the marker rules and the {expr}
  -- interpolation each have their own error, and the *value* of the
  -- resulting List is the only view of the trimming arithmetic.
  local CASES = {
    { 'plain', 'let g:H =<< END\nalpha\nbeta\nEND' },
    { 'empty', 'let g:H =<< END\nEND' },
    { 'indented', 'let g:H =<< END\n  alpha\n    beta\nEND' },
    { 'trim', '  let g:H =<< trim END\n    alpha\n      beta\n    END' },
    { 'trim-under', '    let g:H =<< trim END\n  alpha\n  END' },
    { 'trim-tabs', "\tlet g:H =<< trim END\n\t\talpha\n\tEND" },
    { 'eval', "let g:V = 'w' | let g:H =<< eval END\nx{g:V}y\nEND" },
    { 'eval-trim', "let g:V = 'w' | let g:H =<< trim eval END\n  x{g:V}y\n  END" },
    { 'eval-brace-escape', 'let g:H =<< eval END\na{{b}}c\nEND' },
    { 'eval-stray-close', 'let g:H =<< eval END\na}b\nEND' },
    { 'eval-unclosed', 'let g:H =<< eval END\na{1+1\nEND' },
    { 'eval-empty-expr', 'let g:H =<< eval END\na{}b\nEND' },
    { 'eval-bad-expr', 'let g:H =<< eval END\na{nosuchvar}b\nEND' },
    { 'eval-nested', 'let g:H =<< eval END\n{ "a" .. "b" }\nEND' },
    { 'noeval-brace', 'let g:H =<< END\na{1+1}b\nEND' },
    { 'marker-lower', 'let g:H =<< end\nalpha\nend' },
    { 'marker-missing', 'let g:H =<<\nalpha\n.' },
    { 'marker-quoted', 'let g:H =<< "END"\nalpha\nEND' },
    { 'marker-trailing', 'let g:H =<< END trailing\nalpha\nEND' },
    { 'no-end', 'let g:H =<< END\nalpha' },
    { 'end-with-space', 'let g:H =<< END\nalpha\n END' },
    { 'end-substring', 'let g:H =<< END\nalpha\nENDX\nEND' },
    { 'scoped', 'let b:H =<< END\nalpha\nEND' },
    { 'indexed', 'let g:L = [0] | let g:L[0] =<< END\nalpha\nEND' },
    { 'unpack', 'let [g:H, g:H2] =<< END\nalpha\nEND' },
    { 'const', 'const g:H =<< END\nalpha\nEND' },
    { 'op', 'let g:H = [] | let g:H +=<< END\nalpha\nEND' },
    { 'inline', 'let g:H =<< END | echo 1\nalpha\nEND' },
  }
  for _, case in ipairs(CASES) do
    quiet('silent! unlockvar! g:H')
    quiet('silent! unlet! g:H g:H2 g:L g:V b:H')
    exec_then(
      'hd ' .. case[1],
      case[2],
      "join(map(['g:H', 'g:H2', 'g:L', 'b:H'], "
        .. "'v:val . \"=\" . (exists(v:val) ? string(eval(v:val)) : \"GONE\")'), ' ')"
    )
  end
  quiet('silent! unlockvar! g:H')
  quiet('silent! unlet! g:H g:H2 g:L g:V b:H')
end)

-- ---------------------------------------------------------------------
-- s9 -- :redir into a variable
-- ---------------------------------------------------------------------

section('s9-redir', function()
  -- var_redir_start / var_redir_str / var_redir_stop.  The append form
  -- reads the old value first, the target may be a subscript, and the
  -- redirection has to survive the variable being deleted underneath
  -- it -- which is the arm nothing else in the tree exercises.
  local CASES = {
    { 'plain', 'redir => g:R', 'echo "one"', 'redir END' },
    { 'append', 'let g:R = "pre" | redir =>> g:R', 'echo "one"', 'redir END' },
    { 'append-missing', 'redir =>> g:R', 'echo "one"', 'redir END' },
    { 'multiple', 'redir => g:R', 'echo "one"\necho "two"', 'redir END' },
    { 'scoped-b', 'redir => b:R', 'echo "one"', 'redir END' },
    { 'index', 'let g:L = ["a"] | redir => g:L[0]', 'echo "one"', 'redir END' },
    { 'key', 'let g:D = {"k": "a"} | redir => g:D.k', 'echo "one"', 'redir END' },
    { 'locked', 'let g:R = "x" | lockvar g:R | redir => g:R', 'echo "one"', 'redir END' },
    { 'const', 'const g:R = "x" | redir => g:R', 'echo "one"', 'redir END' },
    { 'existing-list', 'let g:R = [1] | redir => g:R', 'echo "one"', 'redir END' },
    { 'vvar', 'redir => v:errmsg', 'echo "one"', 'redir END' },
    { 'vvar-ro', 'redir => v:count', 'echo "one"', 'redir END' },
    { 'badname', 'redir => 1bad', 'echo "one"', 'redir END' },
    { 'unlet-under', 'redir => g:R', 'echo "one"\nunlet g:R', 'redir END' },
    { 'no-end', 'redir => g:R', 'echo "one"', 'echo "no END"' },
    { 'env', 'redir => $VARSP', 'echo "one"', 'redir END' },
    { 'opt', 'redir => &textwidth', 'echo "one"', 'redir END' },
    { 'reg', 'redir => @a', 'echo "one"', 'redir END' },
  }
  for _, case in ipairs(CASES) do
    quiet('silent! redir END')
    quiet('silent! unlockvar! g:R')
    quiet('silent! unlet! g:R b:R g:L g:D')
    quiet("let v:errmsg = ''")
    exec('rd ' .. case[1] .. ' start', case[2])
    exec('rd ' .. case[1] .. ' body', case[3])
    exec('rd ' .. case[1] .. ' stop', case[4])
    quiet('silent! redir END')
    veval(
      'rd ' .. case[1] .. ' value',
      "join(map(['g:R', 'b:R', 'g:L', 'g:D'], "
        .. "'v:val . \"=\" . (exists(v:val) ? string(eval(v:val)) : \"GONE\")'), ' ')"
    )
  end
  quiet('silent! redir END')
  quiet('silent! unlockvar! g:R')
  quiet('silent! unlet! g:R b:R g:L g:D')
  quiet("let v:errmsg = ''")
end)

-- ---------------------------------------------------------------------
-- s10 -- exists()
-- ---------------------------------------------------------------------

section('s10-exists', function()
  -- var_exists(): the name is parsed, the leading part looked up, then
  -- handle_subscript() applied -- so `exists()` answers about a *path*,
  -- not a name, and every failure along it has to answer 0 rather than
  -- raise.
  quiet('silent! unlet! g:E g:L g:D g:F')
  quiet('let g:E = 1')
  quiet('let g:L = [1, [2]]')
  quiet("let g:D = {'a': {'b': 1}}")
  quiet("let g:F = function('tr')")
  quiet("let $VARSP = 'x'")
  local NAMES = {
    'g:E',
    'E',
    'g:NOSUCH',
    'g:L',
    'g:L[0]',
    'g:L[1][0]',
    'g:L[9]',
    'g:L[-1]',
    'g:D',
    'g:D.a',
    'g:D.a.b',
    'g:D.zz',
    "g:D['a']",
    'g:F',
    'g:F()',
    'g:',
    'b:',
    'w:',
    't:',
    'v:',
    'l:',
    'a:',
    's:',
    'v:count',
    'v:nosuch',
    'b:changedtick',
    '&textwidth',
    '&l:textwidth',
    '&g:textwidth',
    '&nosuchopt',
    '$VARSP',
    '$NOSUCHENV',
    '@a',
    '@\x01',
    '*tr',
    '*nosuchfunc',
    '*g:F',
    ':let',
    ':nosuchcmd',
    '#BufRead',
    '',
    '1bad',
    'nosuchscope:x',
    'g:E.a',
    'g:E[0]',
  }
  for _, name in ipairs(NAMES) do
    veval(
      'ex ' .. (name == '' and '<empty>' or esc(name)),
      string.format(
        "execute('try | echo exists(%s) | catch | echo v:exception | endtry')",
        vim.fn.string(name):gsub("'", "''")
      )
    )
  end
  quiet('silent! unlet! g:E g:L g:D g:F')
  quiet('silent! unlet! $VARSP')
end)

-- ---------------------------------------------------------------------
-- s11 -- the message column, uncaptured
-- ---------------------------------------------------------------------

section('s11-msgcolumn', function()
  -- Everything above routes its answer through `execute()`, which
  -- captures the message *after* msg_puts has already run.  This
  -- section deliberately does not: the listing and the errors go out
  -- the real message path and land on stderr, so the .stderr artifact
  -- carries msg_advance(22)'s padding, msg_outtrans's rendering of
  -- unprintable bytes and the `#`/`*`/`[`/`{` type prefix.  Without it
  -- that artifact is empty and gates nothing.
  local function say(cmd)
    -- A raw nvim_command raises instead of printing, so the try/catch
    -- lives in Vimscript and the `echo` is what reaches the prompt.
    pcall(vim.api.nvim_command, 'try | ' .. cmd .. ' | catch | echo v:exception | endtry')
  end
  quiet(table.concat({
    'silent! unlet! g:MMnum g:MMstr g:MMlist g:MMdict g:MMfn g:MMflt g:MMblob g:MMctrl g:MMutf',
    'let g:MMnum = -12345',
    "let g:MMstr = 'plain string'",
    'let g:MMlist = [1, [2], {}]',
    "let g:MMdict = {'k': 'v'}",
    "let g:MMfn = function('tr', ['a'])",
    'let g:MMflt = -0.125',
    'let g:MMblob = 0z0102ff',
    'let g:MMctrl = "a\x01b\x7fc"',
    'let g:MMutf = "a\xc3\xa9\xe4\xb8\xad"',
  }, '\n'))
  emit('msg see stderr')
  vim.api.nvim_command('echo "-- s11 begin"')
  for _, name in ipairs({
    'g:MMnum',
    'g:MMstr',
    'g:MMlist',
    'g:MMdict',
    'g:MMfn',
    'g:MMflt',
    'g:MMblob',
    'g:MMctrl',
    'g:MMutf',
  }) do
    say('let ' .. name)
  end
  say('let g:MMnum g:MMstr g:MMlist')
  say('let g:MMnosuch')
  say('let g:MMlist[1]')
  say('let g:MMdict.k')
  say('unlet g:MMnosuch')
  say('lockvar g:MMnum')
  say('let g:MMnum = 1')
  say('unlockvar g:MMnum')
  say('let v:count = 1')
  say('unlet v:true')
  say('let &nosuchoption = 1')
  say('let [g:MMa, g:MMb] = [1]')
  vim.api.nvim_command('echo "-- s11 end"')
  quiet('silent! unlockvar! g:MMnum')
  quiet('silent! unlet! g:MMnum g:MMstr g:MMlist g:MMdict g:MMfn g:MMflt g:MMblob g:MMctrl g:MMutf')
end)

-- ---------------------------------------------------------------------
-- s12 -- the four options that hold Vimscript the editor calls out to
-- ---------------------------------------------------------------------

section('s12-external', function()
  -- eval/vars/external.rs: 'charconvert', 'diffexpr', 'patchexpr' and
  -- 'spellsuggest' are options holding Vimscript that C code evaluates
  -- with the relevant v: variables in place.  Nothing else in this
  -- sweep reaches them, and neither does evalsweep -- before this
  -- section they were covered by the oldtest suite alone.
  --
  -- Each probe records three things: what the callback *saw* in the v:
  -- variables it was handed, that those variables were put back
  -- afterwards, and what the caller did with the answer.  No temporary
  -- path is ever printed: v:fname_in and friends name files under
  -- $TMPDIR whose names differ between two runs by construction, so
  -- only whether they are set is recorded.
  local one = work .. '/xone.txt'
  local two = work .. '/xtwo.txt'
  quiet("silent! unlet! g:XLOG")
  quiet("call writefile(['alpha', 'beta', 'gamma'], '" .. one .. "')")
  quiet("call writefile(['alpha', 'BETA', 'gamma'], '" .. two .. "')")
  quiet(table.concat({
    'let g:XLOG = []',
    'function! XCC() abort',
    "  call add(g:XLOG, printf('cc from=%s to=%s in=%d out=%d', v:charconvert_from, v:charconvert_to, v:fname_in != '', v:fname_out != ''))",
    "  call writefile(['converted'], v:fname_out)",
    '  return 0',
    'endfunction',
    'function! XCCfail() abort',
    "  call add(g:XLOG, 'ccfail')",
    '  return 1',
    'endfunction',
    'function! XDE() abort',
    "  call add(g:XLOG, printf('de in=%d new=%d out=%d', v:fname_in != '', v:fname_new != '', v:fname_out != ''))",
    "  call writefile(['2c2', '< beta', '---', '> BETA'], v:fname_out)",
    '  return 0',
    'endfunction',
    'function! XPE() abort',
    "  call add(g:XLOG, printf('pe in=%d diff=%d out=%d', v:fname_in != '', v:fname_diff != '', v:fname_out != ''))",
    "  call writefile(['patched'], v:fname_out)",
    'endfunction',
    'function! XSS(w) abort',
    "  call add(g:XLOG, printf('ss val=%s arg=%s', string(v:val), string(a:w)))",
    "  return [['fixed', 10], ['other', 20]]",
    'endfunction',
    'function! XSSarity(w) abort',
    "  return [['only']]",
    'endfunction',
    'function! XSSflat(w) abort',
    '  return 42',
    'endfunction',
    'function! XSSnostr(w) abort',
    '  return [[1, 2]]',
    'endfunction',
  }, '\n'))

  -- 'charconvert'.  A named encoding nvim cannot convert itself is what
  -- makes it run at all; the ++enc= spelling reaches the same path a
  -- file with a 'fileencodings' match would.
  quiet('set charconvert=XCC()')
  quiet('let g:XLOG = []')
  exec('cc ok', 'edit ++enc=zzz-unknown ' .. one)
  veval('cc ok log', 'string(g:XLOG)')
  veval('cc ok lines', "string(getline(1, '$'))")
  veval(
    'cc ok cleared',
    "string([v:charconvert_from, v:charconvert_to, v:fname_in, v:fname_out])"
  )
  quiet('set charconvert=XCCfail()')
  quiet('let g:XLOG = []')
  exec('cc fail', 'edit! ++enc=zzz-unknown ' .. one)
  veval('cc fail log', 'string(g:XLOG)')
  veval('cc fail lines', "string(getline(1, '$'))")
  quiet('set charconvert=XNoSuchFunc()')
  exec('cc nofunc', 'edit! ++enc=zzz-unknown ' .. one)
  veval('cc nofunc lines', "string(getline(1, '$'))")
  quiet('set charconvert=')
  exec('cc unset', 'edit! ++enc=zzz-unknown ' .. one)

  -- 'diffexpr' and 'patchexpr'.  The diff is bogus on purpose: what is
  -- being recorded is that the expression ran with the three file names
  -- in place, and what the caller made of the answer.
  quiet('set diffexpr=XDE()')
  quiet('let g:XLOG = []')
  exec('de', 'edit! ' .. one .. ' | diffthis | vsplit ' .. two .. ' | diffthis')
  veval('de log', 'string(g:XLOG)')
  veval('de cleared', "string([v:fname_in, v:fname_new, v:fname_out])")
  exec('de off', 'diffoff! | only')
  quiet('set diffexpr=')

  quiet('set patchexpr=XPE()')
  quiet('let g:XLOG = []')
  exec('pe', 'edit! ' .. one .. ' | diffpatch ' .. two)
  veval('pe log', 'string(g:XLOG)')
  veval('pe lines', "string(getline(1, '$'))")
  veval('pe cleared', "string([v:fname_in, v:fname_diff, v:fname_out])")
  exec('pe off', 'diffoff! | only')
  quiet('set patchexpr=')

  -- 'spellsuggest' as an expression: v:val is the bad word, and the
  -- answer must be a list of two-element lists.  get_spellword's two
  -- refusals (the wrong arity, a first element that is not a string)
  -- are the only place E5700 and its -1 answer are reachable from.
  quiet('edit! ' .. one)
  quiet('setlocal spell')
  for _, case in ipairs({
    { 'ok', 'XSS' },
    { 'arity', 'XSSarity' },
    { 'flat', 'XSSflat' },
    { 'nostr', 'XSSnostr' },
    { 'nofunc', 'XSSNoSuch' },
  }) do
    quiet('let g:XLOG = []')
    quiet('set spellsuggest=expr:' .. case[2] .. '(v:val)')
    veval('ss ' .. case[1], "execute('try | echo string(spellsuggest(''wrng'', 3)) | catch | echo v:exception | endtry')")
    veval('ss ' .. case[1] .. ' log', 'string(g:XLOG)')
    veval('ss ' .. case[1] .. ' val', "execute('try | echo string(v:val) | catch | echo v:exception | endtry')")
  end
  quiet('set spellsuggest& nospell')
  quiet('silent! unlet! g:XLOG')
  quiet('silent! delfunction! XCC')
  quiet('silent! delfunction! XCCfail')
  quiet('silent! delfunction! XDE')
  quiet('silent! delfunction! XPE')
  quiet('silent! delfunction! XSS')
  quiet('silent! delfunction! XSSarity')
  quiet('silent! delfunction! XSSflat')
  quiet('silent! delfunction! XSSnostr')
  quiet('enew!')
end)

-- ---------------------------------------------------------------------
-- s13 -- :function definitions, and what calling them answers
-- ---------------------------------------------------------------------

--- Ask a Vimscript expression through `execute()` so that an error is an
--- answer rather than a Lua exception.  Every expression below spells
--- its string literals with double quotes: the wrapper itself is a
--- single-quoted Vimscript string.
local function fcall(label, expr)
  veval(label, "execute('try | echo string(" .. expr .. ") | catch | echo v:exception | endtry')")
end

--- Run one ex command -- for `:call`, `:delfunction` and the
--- `:function` listings, whose answer is what they printed.  No
--- `try | ... | catch` here: `:function /pat` and `:delfunction` both
--- swallow the rest of the line, `|` included, so the wrapper would
--- become part of the pattern.  An error reaches the report through
--- `execute()` raising instead.
local function fcmd(label, cmd)
  veval(label, "execute('" .. cmd .. "')")
end

--- Evaluate an expression with no wrapper at all, for the cases whose
--- text is *not* parseable: an unbalanced lambda swallows whatever
--- follows it, so a `| catch |` tail would end up quoted in the answer.
local function fraw(label, expr)
  veval(label, 'string(' .. expr .. ')')
end

section('s13-funcdef', function()
  -- get_function_args / ex_function / get_function_body / call_user_func.
  -- Two kinds of report line per case: what the *definition* said, and
  -- what each call answers.  A rewrite that accepts a definition and
  -- then builds the wrong a: scope shows up only in the second.
  local many = {}
  for i = 1, 20 do
    many[i] = 'a' .. i
  end
  local DEFS = {
    {
      'noargs',
      { 'function! XFa()', '  return 1', 'endfunction' },
      { 'XFa()', 'XFa(1)' },
    },
    {
      'onearg',
      { 'function! XFb(x)', '  return a:x', 'endfunction' },
      { 'XFb(1)', 'XFb()', 'XFb(1, 2)', 'XFb("s")', 'XFb([1])' },
    },
    {
      'twoargs',
      { 'function! XFc(x, y)', '  return [a:x, a:y]', 'endfunction' },
      { 'XFc(1, 2)', 'XFc(1)', 'XFc()', 'XFc(1, 2, 3)' },
    },
    {
      'maxargs',
      {
        'function! XF20(' .. table.concat(many, ', ') .. ')',
        '  return [a:a1, a:a20, len(a:000)]',
        'endfunction',
      },
      { 'call("XF20", range(20))', 'call("XF20", range(19))' },
    },
    {
      'overmaxargs',
      {
        'function! XF21(' .. table.concat(many, ', ') .. ', a21)',
        '  return a:a21',
        'endfunction',
      },
      { 'call("XF21", range(21))' },
    },
    {
      'default1',
      { 'function! XFd1(x, y = 5)', '  return [a:x, a:y]', 'endfunction' },
      { 'XFd1(1)', 'XFd1(1, 2)', 'XFd1(1, 2, 3)', 'XFd1()' },
    },
    {
      -- Each default is evaluated at call time, in order, and can name
      -- the arguments to its left.
      'defaultorder',
      {
        'function! XFd2(x, y = a:x * 2, z = a:y + 1)',
        '  return [a:x, a:y, a:z]',
        'endfunction',
      },
      { 'XFd2(1)', 'XFd2(1, 9)', 'XFd2(1, 9, 99)' },
    },
    {
      'defaultside',
      {
        'let g:XFside = 0',
        'function! XFd3(x, y = XFbump())',
        '  return [a:x, a:y, g:XFside]',
        'endfunction',
        'function! XFbump()',
        '  let g:XFside += 1',
        '  return g:XFside',
        'endfunction',
      },
      { 'XFd3(1)', 'XFd3(1)', 'XFd3(1, 0)', 'g:XFside' },
    },
    {
      'defaultbad',
      { 'function! XFd4(x = g:XFnosuch)', '  return a:x', 'endfunction' },
      { 'XFd4()', 'XFd4(3)' },
    },
    {
      'defaultbeforereq',
      { 'function! XFd5(x = 1, y)', '  return [a:x, a:y]', 'endfunction' },
      { 'XFd5(1, 2)' },
    },
    {
      'varargs',
      { 'function! XFv1(...)', '  return [a:0, a:000]', 'endfunction' },
      { 'XFv1()', 'XFv1(1)', 'XFv1(1, 2, 3)', 'XFv1("a", [1], {})' },
    },
    {
      'argplusvarargs',
      { 'function! XFv2(a, ...)', '  return [a:a, a:0, a:000]', 'endfunction' },
      { 'XFv2(1)', 'XFv2(1, 2)', 'XFv2(1, 2, 3)', 'XFv2()' },
    },
    {
      'varargindex',
      { 'function! XFv3(...)', '  return [a:1, a:0]', 'endfunction' },
      { 'XFv3(7)', 'XFv3()', 'XFv3(7, 8)' },
    },
    {
      'varargindex2',
      { 'function! XFv4(...)', '  return [a:1, a:2, a:20]', 'endfunction' },
      { 'XFv4(1, 2)', 'call("XFv4", range(25))' },
    },
    {
      'defaultplusvarargs',
      {
        'function! XFv5(a, b = 2, ...)',
        '  return [a:a, a:b, a:0, a:000]',
        'endfunction',
      },
      { 'XFv5(1)', 'XFv5(1, 2)', 'XFv5(1, 2, 3)', 'XFv5(1, 2, 3, 4)' },
    },
    {
      -- a:000 is a locked list living inside the funccall_T, not a
      -- separately allocated one.
      'varargslock',
      {
        'function! XFv6(...)',
        '  let l:r = []',
        '  try',
        '    let a:000[0] = 99',
        '  catch',
        '    call add(l:r, v:exception)',
        '  endtry',
        '  try',
        '    call add(a:000, 5)',
        '  catch',
        '    call add(l:r, v:exception)',
        '  endtry',
        '  call add(l:r, [islocked("a:000"), a:000])',
        '  return l:r',
        'endfunction',
      },
      { 'XFv6(1, 2)' },
    },
    {
      -- FIXVAR_CNT is 12: the thirteenth local spills out of the
      -- funccall_T's embedded array and into the hashtab proper.
      'fixvarspill',
      {
        'function! XFfix()',
        '  let l:r = []',
        '  for i in range(20)',
        '    execute "let l:v" .. i .. " = " .. i',
        '  endfor',
        '  for i in range(20)',
        '    call add(l:r, eval("l:v" .. i))',
        '  endfor',
        '  return [l:r, len(keys(l:)), sort(keys(a:))]',
        'endfunction',
      },
      { 'XFfix()' },
    },
    {
      'rangeattr',
      {
        'function! XFrg() range',
        '  return [a:firstline, a:lastline, sort(keys(a:))]',
        'endfunction',
      },
      { 'XFrg()' },
    },
    {
      'abortattr',
      {
        'let g:XFafter = 0',
        'function! XFab() abort',
        '  call XFnosuchfn()',
        '  let g:XFafter = 1',
        '  return 2',
        'endfunction',
      },
      { 'XFab()', 'g:XFafter' },
    },
    {
      'noabort',
      {
        'let g:XFafter2 = 0',
        'function! XFna()',
        '  call XFnosuchfn()',
        '  let g:XFafter2 = 1',
        '  return 2',
        'endfunction',
      },
      { 'XFna()', 'g:XFafter2' },
    },
    {
      'dictattr',
      {
        'let g:XD = {"k": 41}',
        'function! g:XD.m() dict',
        '  return [self.k, sort(keys(l:)), exists("l:self")]',
        'endfunction',
      },
      { 'g:XD.m()', 'g:XD["m"]()' },
    },
    {
      'dictattrplain',
      { 'function! XFdt() dict', '  return self', 'endfunction' },
      { 'XFdt()' },
    },
    {
      'closureattr',
      {
        'function! XFmk(n)',
        '  let l:c = a:n',
        '  function! XFinner() closure',
        '    let l:c += 1',
        '    return l:c',
        '  endfunction',
        '  return function("XFinner")',
        'endfunction',
      },
      { 'XFmk(10)()', 'XFmk(20)()' },
    },
    {
      'badattr',
      { 'function! XFbadat() nosuchattr', '  return 1', 'endfunction' },
      { 'XFbadat()' },
    },
    {
      'lowername',
      { 'function! xflow()', '  return 1', 'endfunction' },
      { 'xflow()' },
    },
    {
      'scriptlocal',
      { 'function! s:Loc(x)', '  return a:x * 2', 'endfunction' },
      { 's:Loc(3)', 'call("s:Loc", [4])' },
    },
    {
      'curlyname',
      { 'function! {"XF" . "Curly"}()', '  return "curly"', 'endfunction' },
      { 'XFCurly()' },
    },
    {
      -- A curly-brace name is a fresh allocation, and upstream's E884
      -- guard compares a pointer into it against one into the command
      -- line (O-B14-12): before that was fixed this case answered
      -- differently depending on the *working directory*.
      'colonname',
      { 'function! {"XF:C"}()', '  return 1', 'endfunction' },
      { 'exists("*XF:C")' },
    },
    {
      'autoloadname',
      { 'function! xfau#x(n)', '  return a:n + 1', 'endfunction' },
      { 'xfau#x(1)', 'exists("*xfau#x")' },
    },
    {
      'numbered',
      { 'let g:XN = {}', 'function! g:XN.f(a)', '  return a:a', 'endfunction' },
      { 'g:XN.f(5)', 'type(g:XN.f)' },
    },
    {
      'redefine',
      { 'function XFa()', '  return 2', 'endfunction' },
      { 'XFa()' },
    },
    {
      'redefinebang',
      { 'function! XFa()', '  return 3', 'endfunction' },
      { 'XFa()' },
    },
    {
      'noend',
      { 'function! XFnoend()', '  return 1' },
      { 'XFnoend()' },
    },
    {
      'illegalarg',
      { 'function! XFbadarg(1x)', '  return 1', 'endfunction' },
      { 'XFbadarg(1)' },
    },
    {
      'dupearg',
      { 'function! XFdup(x, x)', '  return a:x', 'endfunction' },
      { 'XFdup(1, 2)' },
    },
    {
      'noname',
      { 'function! ()', '  return 1', 'endfunction' },
      {},
    },
    {
      'noparen',
      { 'function! XF:Y()', '  return 1', 'endfunction' },
      {},
    },
    {
      'nested',
      {
        'function! XFn1()',
        '  function! XFn2()',
        '    return 2',
        '  endfunction',
        '  return XFn2()',
        'endfunction',
      },
      { 'XFn1()', 'XFn2()' },
    },
    {
      -- The body is stored verbatim, comments and continuations
      -- included; get_function_body is what decides where it ends.
      'bodyshapes',
      {
        'function! XFbody(x)',
        '  " a comment',
        '  let l:r = [',
        '        \\ a:x,',
        '        \\ 2]',
        '  if a:x > 0 | call add(l:r, "pos") | endif',
        '  while 0 | endwhile',
        '  for i in [] | endfor',
        '  try | catch | endtry',
        '  return l:r',
        'endfunction',
      },
      { 'XFbody(1)', 'XFbody(-1)' },
    },
  }
  for _, case in ipairs(DEFS) do
    exec('def ' .. case[1], table.concat(case[2], '\n'))
    for i, expr in ipairs(case[3]) do
      fcall('def ' .. case[1] .. ' c' .. i, expr)
    end
  end

  -- :call, with and without a range, and the trailing-argument arms.
  fcmd('call plain', 'call XFa()')
  fcmd('call args', 'call XFc(1, 2)')
  fcmd('call unknown', 'call XFnosuchfn()')
  fcmd('call range', '1,1call XFrg()')
  fcmd('call rangeplain', '1,1call XFa()')
  fcmd('call trailing', 'call XFa() zz')
  fcmd('call noparen', 'call XFa')
  fcmd('call bang', 'call! XFa()')

  -- :return.  do_return / get_return_cmd / ex_return.
  local RETS = {
    { 'bare', { 'return' } },
    { 'value', { 'return 7' } },
    { 'expr', { 'return 1 + 2' } },
    { 'list', { 'return [1, {"a": 2}]' } },
    { 'bar', { 'return 1 | echo "unreached"' } },
    { 'junk', { 'return 1 2' } },
    { 'trycatch', { 'try', '  return 1', 'catch', '  return 2', 'endtry' } },
    {
      'tryfinally',
      { 'let g:XFfin = 0', 'try', '  return 1', 'finally', '  let g:XFfin = 1', 'endtry' },
    },
    {
      'finallyoverride',
      { 'try', '  return 1', 'finally', '  return 2', 'endtry' },
    },
    { 'inwhile', { 'while 1', '  return 3', 'endwhile', 'return 4' } },
    { 'infor', { 'for i in [5, 6]', '  return i', 'endfor' } },
    { 'afterreturn', { 'return 1', 'let g:XFdead = 1' } },
    { 'conditional', { 'if 1', '  return "then"', 'else', '  return "else"', 'endif' } },
  }
  for _, case in ipairs(RETS) do
    local src = { 'function! XFret()' }
    for _, line in ipairs(case[2]) do
      src[#src + 1] = '  ' .. line
    end
    src[#src + 1] = 'endfunction'
    quiet('silent! delfunction! XFret')
    quiet('silent! unlet! g:XFdead g:XFfin')
    exec('ret ' .. case[1] .. ' def', table.concat(src, '\n'))
    fcall('ret ' .. case[1], 'XFret()')
    fcall('ret ' .. case[1] .. ' side', '[get(g:, "XFfin", "U"), get(g:, "XFdead", "U")]')
  end
  fcmd('ret outside', 'return 1')
  fcall('ret outside eval', 'XFa()')
end)

-- ---------------------------------------------------------------------
-- s14 -- the :function listings and :delfunction
-- ---------------------------------------------------------------------

section('s14-funclist', function()
  -- list_functions / list_functions_matching_pat / list_one_function /
  -- list_func_head, and ex_delfunction.  The listing is the only view of
  -- the stored body and of the argument list as `list_func_head` prints
  -- it back -- defaults and `...` included.
  -- One `exec` per definition, not one block: a block that fails at its
  -- second line leaves every later fixture undefined and turns the whole
  -- section into a wall of E123, which reads as coverage.
  local FIXTURES = {
    { 'a', { 'function! XLa(x)', '  return a:x', 'endfunction' } },
    {
      'b',
      {
        'function! XLb(x, y = 1, ...) range abort',
        '  " kept comment',
        '  return [a:x, a:y, a:000]',
        'endfunction',
      },
    },
    { 'c', { 'function! XLc() dict', '  return 1', 'endfunction' } },
    { 'closure', { 'function! XLcl() closure', '  return 1', 'endfunction' } },
    { 's', { 'function! s:XLs()', '  return 2', 'endfunction' } },
    { 'dict', { 'let g:XLD = {}', 'function! g:XLD.m(a)', '  return a:a', 'endfunction' } },
  }
  for _, fixture in ipairs(FIXTURES) do
    exec('fixture ' .. fixture[1], table.concat(fixture[2], '\n'))
  end
  fcmd('list one', 'function XLa')
  fcmd('list attrs', 'function XLb')
  fcmd('list attrs2', 'function XLc')
  fcmd('list closure', 'function XLcl')
  fcmd('list scriptlocal', 'function s:XLs')
  fcmd('list unknown', 'function XLnope')
  fcmd('list curly', 'function {"XL" . "a"}')
  fcmd('list pattern', 'function /^XL')
  fcmd('list pattern2', 'function /^XL[ab]$')
  fcmd('list pattern3', 'function /nosuchpattern')
  fcmd('list pattern4', 'function /^xl')
  -- The bare listing walks the whole table, so record only the rows this
  -- section owns: everything else is a function of every section above.
  veval(
    'list bare filtered',
    'string(sort(filter(split(execute("function"), "\\n"), {_, v -> v =~# "XL"})))'
  )
  veval('list bare count', 'string(len(split(execute("function"), "\\n")) > 0)')
  -- A dict function is stored under a monotonic number, so the number
  -- itself is a function of every section above; the shape is not.
  veval(
    'list numbered',
    'string(substitute(execute("function g:XLD.m"), "function \\\\d\\\\+(", "function <NR>(", "g"))'
  )

  fcmd('del one', 'delfunction XLa')
  fcall('del one after', 'exists("*XLa")')
  fcmd('del again', 'delfunction XLa')
  fcmd('del again bang', 'delfunction! XLa')
  fcmd('del noarg', 'delfunction')
  fcmd('del noarg bang', 'delfunction!')
  fcmd('del builtin', 'delfunction strlen')
  fcmd('del curly', 'delfunction {"XL" . "c"}')
  fcall('del curly after', 'exists("*XLc")')
  fcmd('del dictfn', 'delfunction g:XLD.m')
  fcall('del dictfn after', 'string(g:XLD)')
  fcmd('del scriptlocal', 'delfunction s:XLs')
  fcmd('del trailing', 'delfunction XLb zz')
  fcall('del trailing after', 'exists("*XLb")')
  -- Deleting a function that is running, and redefining one: both reach
  -- func_remove()'s uf_calls guard rather than the hashtab.
  quiet(table.concat({
    'function! XLdeler()',
    '  delfunction XLuse',
    '  return 0',
    'endfunction',
    'function! XLredefiner()',
    '  function! XLuse()',
    '    return 9',
    '  endfunction',
    '  return 0',
    'endfunction',
    'function! XLuse(which)',
    '  if a:which == 0',
    '    call XLdeler()',
    '  else',
    '    call XLredefiner()',
    '  endif',
    '  return 1',
    'endfunction',
  }, '\n'))
  fcall('del inuse', 'XLuse(0)')
  fcall('redef inuse', 'XLuse(1)')
  fcall('del inuse after', 'exists("*XLuse")')
  -- A funcref keeps a deleted function alive; a name does not.
  quiet('silent! delfunction! XLref')
  quiet(table.concat({
    'function! XLref()',
    '  return "alive"',
    'endfunction',
    'let g:XLF = function("XLref")',
    'let g:XLN = "XLref"',
  }, '\n'))
  fcmd('del held', 'delfunction XLref')
  fcall('del held ref', 'g:XLF()')
  fcall('del held name', 'call(g:XLN, [])')
  fcall('del held exists', 'exists("*XLref")')
  quiet('silent! unlet! g:XLF g:XLN g:XLD')
  -- `:delfunction`'s `uf_refcount > 2` test (docket O-B14-13).  A
  -- funccall that outlived its call -- one that returned `a:000` --
  -- holds a reference to its own function until the garbage collector
  -- frees it, so the function cannot be deleted meanwhile.  That is
  -- `create_funccal`'s `func_ptr_ref` paired with `free_funccal`'s
  -- `func_ptr_unref`, and it has nothing to do with autoload: the
  -- scalar arm below is the same function shape that does *not* keep
  -- its funccall, and deletes cleanly after three calls.  Each arm has
  -- a function of its own, called an exact number of times.
  for _, name in ipairs({ 'XLrcA', 'XLrcB', 'XLrcC', 'XLrcD' }) do
    quiet('silent! delfunction! ' .. name)
  end
  quiet(table.concat({
    'function! XLrcA(...)',
    '  return a:000',
    'endfunction',
    'function! XLrcB(...)',
    '  return a:000',
    'endfunction',
    'function! XLrcC(...)',
    '  return a:000',
    'endfunction',
    'function! XLrcD(...)',
    '  return "scalar"',
    'endfunction',
  }, '\n'))
  fcall('del refcnt one call', 'XLrcA(1)')
  fcmd('del refcnt one', 'delfunction XLrcA')
  fcall('del refcnt two calls', '[XLrcB(1), XLrcB(2)]')
  fcmd('del refcnt two', 'delfunction XLrcB')
  fcall('del refcnt scalar calls', '[XLrcD(1), XLrcD(2), XLrcD(3)]')
  fcmd('del refcnt scalar', 'delfunction XLrcD')
  fcall('del refcnt gc calls', '[XLrcC(1), XLrcC(2)]')
  -- `garbagecollect()` only *asks* for a collection at the next
  -- main-loop turn, which a headless `-l` run never reaches;
  -- `test_garbagecollect_now()` runs one here, and is the only probe
  -- that shows the reference coming back.  `v:testing` is restored at
  -- once: `get_func_tv` reads it on every call.
  quiet('let v:testing = 1')
  quiet('call test_garbagecollect_now()')
  quiet('let v:testing = 0')
  fcmd('del refcnt gc', 'delfunction XLrcC')
  fcall('del refcnt exists',
    '[exists("*XLrcA"), exists("*XLrcB"), exists("*XLrcC"), exists("*XLrcD")]')
  -- `:function` under `eap->skip`, which nothing above reaches: a
  -- definition inside a *false* `:if` is still parsed, name and body
  -- and all, with `skip` set.  `get_lval` in skip mode leaves
  -- `ll_name_len` 0, so `trans_function_name`'s "strip the s:" arm
  -- subtracts 2 from 0 -- benign upstream, an abort in a checked
  -- build.  Every name spelling, because each takes a different arm.
  exec('skipdef s', 'if 0\n  func s:ZQs(a, b)\n    return 1\n  endfunc\nendif\necho "done"')
  exec('skipdef sid', 'if 0\n  func <SID>ZQi()\n    return 1\n  endfunc\nendif\necho "done"')
  exec('skipdef g', 'if 0\n  func g:ZQg()\n    return 1\n  endfunc\nendif\necho "done"')
  exec('skipdef plain', 'if 0\n  func ZQp()\n    return 1\n  endfunc\nendif\necho "done"')
  exec('skipdef dict', 'if 0\n  func d.ZQd()\n    return 1\n  endfunc\nendif\necho "done"')
  exec('skipdef curly', 'if 0\n  func {"ZQ" .. "c"}()\n    return 1\n  endfunc\nendif\necho "done"')
  exec('skipdef del', 'if 0\n  delfunction s:ZQs\nendif\necho "done"')
  fcall('skipdef exists', '[exists("*ZQp"), exists("*ZQc")]')
  -- A dictionary function named with *bracket* notation: the key is a
  -- dictionary key, not an identifier, so `ex_function`'s name check
  -- has to be skipped for it.  s13 only ever spelled this `d.m`.
  quiet('silent! unlet! g:ZB')
  quiet('let g:ZB = {}')
  exec('brkey define', 'func g:ZB["foo-bar"]() dict\n  return "hy"\nendfunc\necho "done"')
  fcall('brkey call', 'g:ZB["foo-bar"]()')
  exec('brkey plain', 'func g:ZB["plain"]() dict\n  return "p"\nendfunc\necho "done"')
  fcall('brkey plain call', 'g:ZB.plain()')
  exec('brkey dot', 'func g:ZB.dot-key() dict\n  return "d"\nendfunc\necho "done"')
  fcall('brkey keys', 'sort(keys(g:ZB))')
  quiet('silent! unlet! g:ZB')
  for _, name in ipairs({ 'XLa', 'XLb', 'XLc', 'XLuse', 'XLdeler', 'XLredefiner', 'XLref',
                          'XLrcA', 'XLrcB', 'XLrcC', 'XLrcD' }) do
    quiet('silent! delfunction! ' .. name)
  end
end)

-- ---------------------------------------------------------------------
-- s15 -- funcrefs, partials, closures and lambdas
-- ---------------------------------------------------------------------

section('s15-funcref', function()
  -- function() / funcref() / make_partial / call_func's partial arms,
  -- register_closure, and the lambda parser in get_lambda_tv.
  quiet(table.concat({
    'function! XRa(...)',
    '  return ["XRa", a:000, exists("l:self")]',
    'endfunction',
    'function! XRb(x, y)',
    '  return [a:x, a:y]',
    'endfunction',
    'function! XRd() dict',
    '  return self.k',
    'endfunction',
  }, '\n'))
  local REFS = {
    { 'function', 'function("XRa")' },
    { 'funcref', 'funcref("XRa")' },
    { 'function unknown', 'function("XRnope")' },
    { 'funcref unknown', 'funcref("XRnope")' },
    { 'function empty', 'function("")' },
    { 'function number', 'function(1)' },
    { 'function list', 'function([1])' },
    { 'function builtin', 'function("strlen")' },
    { 'function autoloadish', 'function("xr#nope")' },
    { 'function colon', 'function("g:a:b")' },
    { 'function args', 'function("XRa", [1, 2])' },
    { 'function emptyargs', 'function("XRa", [])' },
    { 'function badargs', 'function("XRa", 1)' },
    { 'function dict', 'function("XRd", {"k": 7})' },
    { 'function argsdict', 'function("XRa", [1], {"k": 7})' },
    { 'function dictonly', 'function("XRa", {"k": 7})' },
    { 'funcref args', 'funcref("XRb", [1])' },
    { 'function of partial', 'function(function("XRa", [1]), [2])' },
    { 'function of partial dict', 'function(function("XRa", [1]), {"k": 8})' },
    { 'funcref of funcref', 'funcref(funcref("XRa"))' },
  }
  for _, case in ipairs(REFS) do
    fcall('ref ' .. case[1], case[2])
    fcall('ref ' .. case[1] .. ' type', 'type(' .. case[2] .. ')')
  end
  -- Applying them.
  fcall('apply plain', 'function("XRa")(1, 2)')
  fcall('apply partial', 'function("XRa", [1])(2)')
  fcall('apply partial none', 'function("XRa", [1])()')
  fcall('apply over', 'function("XRb", [1])(2, 3)')
  fcall('apply under', 'function("XRb", [1])()')
  fcall('apply dict', 'function("XRd", {"k": 7})()')
  fcall('apply dict extra', 'function("XRd", [9], {"k": 7})()')
  fcall('apply call', 'call(function("XRa", [1]), [2])')
  fcall('apply call dict', 'call(function("XRd"), [], {"k": 3})')
  fcall('apply call name', 'call("XRb", [1, 2])')
  fcall('apply call unknown', 'call("XRnope", [])')
  fcall('apply call badargs', 'call("XRa", 1)')
  fcall('apply call toomany', 'call("XRb", [1, 2, 3])')
  -- Funcref-as-something-else.
  fcall('coerce number', '0 + function("XRa")')
  fcall('coerce string', '"" . function("XRa")')
  fcall('coerce bool', 'empty(function("XRa"))')
  fcall('coerce len', 'len(function("XRa"))')
  fcall('coerce compare', 'function("XRa") == function("XRa")')
  fcall('coerce compare2', 'function("XRa", [1]) == function("XRa", [1])')
  -- funcref() binds the function, function() binds the name.
  quiet(table.concat({
    'function! XRv()',
    '  return "one"',
    'endfunction',
    'let g:XRfr = funcref("XRv")',
    'let g:XRfn = function("XRv")',
    'function! XRv()',
    '  return "two"',
    'endfunction',
  }, '\n'))
  fcall('rebind funcref', 'g:XRfr()')
  fcall('rebind function', 'g:XRfn()')
  fcall('rebind string', '[string(g:XRfr), string(g:XRfn)]')
  -- Closures: the captured l: has to outlive the call that made it.
  quiet(table.concat({
    'function! XRmk(start)',
    '  let l:n = a:start',
    '  let l:Inc = {d -> [execute("let l:n += d"), l:n][1]}',
    '  return l:Inc',
    'endfunction',
    'function! XRpair()',
    '  let l:n = 0',
    '  return [{-> l:n}, {v -> [execute("let l:n = v"), l:n][1]}]',
    'endfunction',
    'function! XRnest()',
    '  let l:a = 1',
    '  return {-> {-> l:a + 1}}',
    'endfunction',
    'function! XRcaparg(x)',
    '  return {-> a:x * 3}',
    'endfunction',
  }, '\n'))
  fcall('closure counter', 'XRmk(10)(1)')
  fcall('closure repeat', '[XRmk(10)(1), XRmk(10)(2)]')
  quiet('let g:XRc = XRmk(100)')
  fcall('closure kept', '[g:XRc(1), g:XRc(1), g:XRc(1)]')
  quiet('let g:XRp = XRpair()')
  fcall('closure shared', '[g:XRp[0](), g:XRp[1](5), g:XRp[0]()]')
  fcall('closure nested', 'XRnest()()()')
  fcall('closure arg', 'XRcaparg(4)()')
  fcall('closure gc', '[garbagecollect(), g:XRc(1)][1]')
  -- Lambdas.
  local LAMBDAS = {
    { 'const', '{-> 7}()' },
    { 'one', '{x -> x + 1}(1)' },
    { 'two', '{x, y -> x . y}("a", "b")' },
    { 'underarity', '{x -> x}()' },
    { 'overarity', '{x -> x}(1, 2)' },
    { 'nested', '{x -> {y -> x + y}}(1)(2)' },
    { 'string', 'string({x -> x})' },
    { 'type', 'type({-> 1})' },
    { 'inlist', 'map([1, 2], {i, v -> v * 2})' },
    { 'sort', 'sort([3, 1, 2], {a, b -> b - a})' },
    { 'partialof', 'function({x, y -> [x, y]}, [1])(2)' },
    { 'callof', 'call({x -> x * 2}, [4])' },
    { 'defaultarg', '{x, y = 2 -> x + y}(1)' },
    { 'varargs', '{x, ... -> [x, a:000]}(1, 2, 3)' },
    { 'selfref', '{-> l:nosuch}()' },
  }
  for _, case in ipairs(LAMBDAS) do
    fcall('lambda ' .. case[1], case[2])
  end
  -- These four do not parse, so the `| catch |` tail of the wrapper
  -- would be swallowed into the error text.
  for _, case in ipairs({
    { 'raw noarrow', '{x x}' },
    { 'raw unclosed', '{x -> x' },
    { 'raw emptybody', '{x -> }' },
    { 'raw badarg', '{1x -> 1}' },
  }) do
    fraw('lambda ' .. case[1], case[2])
  end
  -- Recursion and maxfuncdepth.  E132 is raised by call_user_func_check
  -- before the body runs, so the depth is the number of *entries*.
  quiet(table.concat({
    'function! XRrec(n)',
    '  if a:n <= 0',
    '    return 0',
    '  endif',
    '  return 1 + XRrec(a:n - 1)',
    'endfunction',
  }, '\n'))
  fcall('depth ok', 'XRrec(20)')
  quiet('set maxfuncdepth=5')
  fcall('depth limit', 'XRrec(20)')
  fcall('depth exact', 'XRrec(3)')
  quiet('set maxfuncdepth=100')
  fcall('depth restored', 'XRrec(20)')
  quiet('silent! unlet! g:XRfr g:XRfn g:XRc g:XRp')
  for _, name in ipairs({ 'XRa', 'XRb', 'XRd', 'XRv', 'XRmk', 'XRpair', 'XRnest', 'XRcaparg', 'XRrec' }) do
    quiet('silent! delfunction! ' .. name)
  end
end)

-- ---------------------------------------------------------------------
-- s16 -- autoload
-- ---------------------------------------------------------------------

section('s16-autoload', function()
  -- The `pkg#name` spelling: call_func fails to find the function, asks
  -- script_autoload to source `autoload/pkg.vim`, and retries once.
  -- The fixture lives under the work directory so its path scrubs.
  local root = work .. '/rtp'
  local function put(path, lines)
    vim.fn.mkdir(vim.fn.fnamemodify(root .. '/' .. path, ':h'), 'p')
    local fd = assert(io.open(root .. '/' .. path, 'w'))
    fd:write(table.concat(lines, '\n'), '\n')
    fd:close()
  end
  put('autoload/xas.vim', {
    'let g:xas_sourced = get(g:, "xas_sourced", 0) + 1',
    'function! xas#bar(...)',
    '  return ["bar", a:000]',
    'endfunction',
    'function! xas#Cap()',
    '  return "cap"',
    'endfunction',
    'function! xas#Once()',
    '  return "once"',
    'endfunction',
    'function! xas#Twice()',
    '  return "twice"',
    'endfunction',
  })
  put('autoload/xas/deep.vim', {
    'let g:xasdeep_sourced = get(g:, "xasdeep_sourced", 0) + 1',
    'function! xas#deep#f()',
    '  return "deep"',
    'endfunction',
  })
  -- Reached only through `call()`, whose name never goes through the
  -- variable lookup: that is the one spelling where `call_func`'s own
  -- `script_autoload` retry is what loads the package, rather than
  -- `find_var`'s having already done it.
  put('autoload/xcl.vim', {
    'let g:xcl_sourced = get(g:, "xcl_sourced", 0) + 1',
    'function! xcl#f(n)',
    '  return ["xcl", a:n]',
    'endfunction',
  })
  put('autoload/xnf.vim', {
    'let g:xnf_sourced = get(g:, "xnf_sourced", 0) + 1',
  })
  put('autoload/xmm.vim', {
    'let g:xmm_sourced = get(g:, "xmm_sourced", 0) + 1',
    'function! xmm#wrong#name()',
    '  return 1',
    'endfunction',
  })
  quiet('set runtimepath+=' .. root)
  fcall('auto pre exists', '[exists("*xas#bar"), get(g:, "xas_sourced", "NONE")]')
  fcall('auto call', 'xas#bar(1, 2)')
  fcall('auto sourced', 'g:xas_sourced')
  fcall('auto post exists', 'exists("*xas#bar")')
  fcall('auto call again', '[xas#bar(), g:xas_sourced]')
  fcall('auto sibling', '[xas#Cap(), g:xas_sourced]')
  fcall('auto deep', '[xas#deep#f(), g:xasdeep_sourced]')
  fcall('auto missing fn', '[xas#nope(), 0]')
  fcall('auto missing file', 'xzz#nope()')
  fcall('auto empty file', '[xnf#nope(), get(g:, "xnf_sourced", "NONE")]')
  fcall('auto wrong name', '[xmm#thing(), get(g:, "xmm_sourced", "NONE")]')
  fcall('auto via call', 'call("xcl#f", [1])')
  fcall('auto via call again', '[xcl#f(2), g:xcl_sourced]')
  fcall('auto funcref', 'string(function("xas#bar"))')
  fcall('auto exists missing', 'exists("*xzz#nope")')
  fcall('auto exists loaded', 'exists("*xas#deep#f")')
  fcmd('auto list', 'function xas#bar')
  fcmd('auto call cmd', 'call xas#bar(3)')
  -- Deleting an autoloaded function.  Each arm gets a function of its
  -- own, called an exact number of times: `:delfunction` compares
  -- uf_refcount against 2, and every call to a `pkg#name` function
  -- leaks one (O-B14-12), so an arm that shared a function with the
  -- probes above would answer differently whenever one was added.
  fcall('auto del once call', 'xas#Once()')
  fcmd('auto del once', 'delfunction xas#Once')
  fcall('auto del once after', 'exists("*xas#Once")')
  fcall('auto del twice call', '[xas#Twice(), xas#Twice()]')
  fcmd('auto del twice', 'delfunction xas#Twice')
  fcall('auto del twice after', 'exists("*xas#Twice")')
  fcall('auto del bar call', 'xas#bar()')
  fcmd('auto del bar', 'delfunction xas#bar')
  fcall('auto del bar after', '[exists("*xas#bar"), g:xas_sourced]')
  fcall('auto sourced final', '[g:xas_sourced, g:xasdeep_sourced]')
  quiet('set runtimepath-=' .. root)
  fcall('auto rtp removed', 'exists("*xas#Cap")')
  fcall('auto rtp removed miss', 'xzz#nope()')
  quiet('silent! unlet! g:xas_sourced g:xasdeep_sourced g:xnf_sourced g:xmm_sourced g:xcl_sourced')
  for _, name in ipairs({ 'xas#bar', 'xas#Cap', 'xas#Once', 'xas#Twice', 'xas#deep#f', 'xcl#f', 'xmm#wrong#name' }) do
    quiet('silent! delfunction! ' .. name)
  end
end)

-- ---------------------------------------------------------------------
-- s17 -- one direct trigger per function-layer error code
-- ---------------------------------------------------------------------

section('s17-funcerr', function()
  -- The exact text of every error the function layer raises, one direct
  -- trigger each.  A definition is always given its whole body: in
  -- `-l` script mode a `:function Name(...)` with no `endfunction` in
  -- the same string asks the real input stream and nvim exits(0)
  -- mid-run, silently truncating the report.
  quiet(table.concat({
    'function! XEa(x, y)',
    '  return [a:x, a:y]',
    'endfunction',
    'function! XEd() dict',
    '  return self.k',
    'endfunction',
  }, '\n'))
  local ERRS = {
    { 'E117 eval', 'XEnope()' },
    { 'E117 call', 'call("XEnope", [])' },
    { 'E118', 'XEa(1, 2, 3)' },
    { 'E119', 'XEa(1)' },
    { 'E121 arg', 'XEa(g:XEnosuchvar, 1)' },
    { 'E129 empty', 'function("")' },
    { 'E700', 'function("XEnope")' },
    { 'E703', 'function("XEa") + 0' },
    { 'E729', 'function("XEa") . ""' },
    { 'E725', 'XEd()' },
    { 'E923', 'function("XEa", 1)' },
    { 'E725 number', 'call(1, [])' },
    { 'E1206 calldict', 'call("XEa", [1, 2], 1)' },
    { 'E1211 calllist', 'call("XEa", 1)' },
  }
  for _, case in ipairs(ERRS) do
    fcall('err ' .. case[1], case[2])
  end
  -- Definitions that are rejected.  Each one is a complete source.
  local ERRDEFS = {
    { 'E122', { 'function XEa(x, y)', '  return 0', 'endfunction' } },
    { 'E124', { 'function! XE:a()', '  return 1', 'endfunction' } },
    { 'E125', { 'function! XEbad(1x)', '  return 1', 'endfunction' } },
    { 'E126', { 'function! XEnoend()', '  return 1' } },
    { 'E128', { 'function! xelower()', '  return 1', 'endfunction' } },
    { 'E129 noname', { 'function! ()', '  return 1', 'endfunction' } },
    { 'E853', { 'function! XEdup(x, x)', '  return 1', 'endfunction' } },
    { 'E884', { 'function! {"XE:c"}()', '  return 1', 'endfunction' } },
    { 'E989', { 'function! XEorder(x = 1, y)', '  return 1', 'endfunction' } },
    { 'E699', { 'function! XEwide(' .. (function()
      local names = {}
      for i = 1, 21 do
        names[i] = 'a' .. i
      end
      return table.concat(names, ', ')
    end)() .. ')', '  return 1', 'endfunction' } },
  }
  for _, case in ipairs(ERRDEFS) do
    exec('errdef ' .. case[1], table.concat(case[2], '\n'))
  end
  -- Single-line commands.
  local ERRCMDS = {
    { 'E123', 'function XEnope' },
    { 'E130', 'delfunction XEnope' },
    { 'E133', 'return 1' },
    { 'E471', 'delfunction' },
    { 'E107', 'call XEa' },
    { 'E477', 'call! XEa(1, 2)' },
  }
  for _, case in ipairs(ERRCMDS) do
    fcmd('errcmd ' .. case[1], case[2])
  end
  fcall('errdef E699 call', 'call("XEwide", range(21))')
  -- Three different depth limits, from three different places:
  -- get_function_body's MAX_FUNC_NESTING (50 nested `:function`
  -- definitions in one body), ex_eval's `:if` nesting, and
  -- call_user_func_check's 'maxfuncdepth'.
  local nest = {}
  for i = 1, 52 do
    nest[#nest + 1] = string.rep(' ', i) .. 'function! XEn' .. i .. '()'
  end
  for i = 52, 1, -1 do
    nest[#nest + 1] = string.rep(' ', i) .. 'endfunction'
  end
  exec('errdef E1058', table.concat(nest, '\n'))
  local deep = { 'function! XEdeep()' }
  for i = 1, 55 do
    deep[#deep + 1] = string.rep(' ', i) .. 'if 1'
  end
  deep[#deep + 1] = string.rep(' ', 56) .. 'return 1'
  for i = 55, 1, -1 do
    deep[#deep + 1] = string.rep(' ', i) .. 'endif'
  end
  deep[#deep + 1] = 'endfunction'
  exec('errdef E579', table.concat(deep, '\n'))
  fcall('errdef E579 call', 'XEdeep()')
  for i = 1, 52 do
    quiet('silent! delfunction! XEn' .. i)
  end
  for _, name in ipairs({ 'XEa', 'XEd', 'XEdeep', 'XEwide', 'XEnoend' }) do
    quiet('silent! delfunction! ' .. name)
  end
end)

-- ---------------------------------------------------------------------
-- s18 -- the function messages, uncaptured
-- ---------------------------------------------------------------------

section('s18-funcmsg', function()
  -- s11's trick for the function layer: everything above routes its
  -- answer through `execute()`, so `list_func_head`'s leading three
  -- spaces, `list_one_function`'s line numbers and the numbered-function
  -- header only reach the real message path from here.
  local function say(cmd)
    pcall(vim.api.nvim_command, 'try | ' .. cmd .. ' | catch | echo v:exception | endtry')
  end
  -- The fixtures come from a *file*, not from this script: `:function`
  -- prints "Last set from <path> line N", stderr is the one artifact the
  -- scrub never sees, and the line it would otherwise name is wherever
  -- `exec`'s nvim_exec2 call happens to sit in this file.
  local fixture = work .. '/s18.vim'
  local fd = assert(io.open(fixture, 'w'))
  fd:write(table.concat({
    'function! XMa(x, y = 2, ...) range abort',
    '  " a comment line',
    '  let l:v = a:x',
    '  return l:v',
    'endfunction',
    'function! XMb()',
    '  return 1',
    'endfunction',
  }, '\n'), '\n')
  fd:close()
  quiet('source ' .. fixture)
  emit('funcmsg see stderr')
  vim.api.nvim_command('echo "-- s18 begin"')
  say('function XMa')
  say('function XMb')
  say('function /^XM')
  say('function XMnope')
  say('delfunction XMnope')
  say('call XMa()')
  say('call XMa(1, 2, 3, 4)')
  say('call XMnope()')
  say('delfunction XMb')
  say('function XMb')
  say('return 1')
  vim.api.nvim_command('echo "-- s18 end"')
  quiet('silent! delfunction! XMa')
  quiet('silent! delfunction! XMb')
end)

-- ---------------------------------------------------------------------
-- s19 -- reference counting: dict watchers, and what survives a GC
--
-- The one part of the variable layer no differential reached before
-- P23-14: `dictwatcheradd()`/`dictwatcherdel()` appeared in no oracle
-- at all, and `test_garbagecollect_now()` was called exactly once in
-- the whole battery (s14's `:delfunction` refcount case).  Both are
-- refcount surfaces -- a watcher owns a callback, the `change` dict a
-- callback is handed outlives the call if the callback keeps it, and
-- the mark-and-sweep is the only thing that frees a reference cycle --
-- so a rewrite of the ref/unref arithmetic is invisible to every other
-- section here.
--
-- Everything is asked TWICE where it matters: once with the graph
-- live, once after a forced collection.  An over-release shows as a
-- changed answer, a crash or a poisoned value; a missed release shows
-- as a live cycle the collector never reclaims, which `id()` sameness
-- across the sweep does not see but `:let` listing counts do not
-- either -- so this section gates the over-release direction, which is
-- the one that corrupts.
-- ---------------------------------------------------------------------
section('s19-refs', function()
  local NAMES = {
    'g:W', 'g:WLOG', 'g:WKEEP', 'g:GRl', 'g:GRd', 'g:GRi', 'g:GRs',
    'g:GRp', 'g:GRc', 'g:GRb', 'g:GRbd', 'g:GRfd', 'g:GRll', 'g:GRcp',
  }
  local function wipe()
    for _, name in ipairs(NAMES) do
      quiet('silent! unlockvar! ' .. name)
      quiet('silent! unlet! ' .. name)
    end
  end
  wipe()

  quiet(table.concat({
    'function! XWa(dict, key, change)',
    '  call add(g:WLOG, "a " . a:key . " " . string(sort(keys(a:change))))',
    'endfunction',
    'function! XWb(dict, key, change) dict',
    '  call add(g:WLOG, "b " . a:key . " " . string(self))',
    'endfunction',
    'function! XWkeep(dict, key, change)',
    '  let g:WKEEP = a:change',
    'endfunction',
    'function! XWmk(n)',
    '  let l:n = a:n',
    '  return {d -> [execute("let l:n += d"), l:n][1]}',
    'endfunction',
  }, '\n'))

  -- The watcher alphabet.  Each arm registers, drives the dict through
  -- the three change kinds (new key, updated key, deleted key) and
  -- deregisters; the log is the answer.
  local WATCHERS = {
    { 'name', "'XWa'" },
    { 'funcref', "function('XWa')" },
    { 'partial', "function('XWb', {'tag': 'p'})" },
    { 'lambda', "{d, k, c -> add(g:WLOG, 'l ' . k)}" },
    { 'lua', "luaeval('function(d, k, c) end')" },
  }
  for _, w in ipairs(WATCHERS) do
    local tag, cb = w[1], w[2]
    quiet("let g:WLOG = [] | let g:W = {'a': 1}")
    exec('watch add ' .. tag, string.format("call dictwatcheradd(g:W, '*', %s)", cb))
    quiet("let g:W.b = 2 | let g:W.a = 9 | unlet g:W.b")
    veval('watch log ' .. tag, 'string(g:WLOG)')
    exec('watch del ' .. tag, string.format("call dictwatcherdel(g:W, '*', %s)", cb))
    quiet('let g:W.c = 3')
    veval('watch after ' .. tag, 'string(g:WLOG)')
  end

  -- Key patterns.  `*` is the only wildcard, and it may be a prefix,
  -- a suffix or the whole pattern.
  for _, pat in ipairs({ '*', 'a', 'a*', '*a', 'no*such', 'a.b' }) do
    quiet("let g:WLOG = [] | let g:W = {}")
    exec('watch pat ' .. pat,
      string.format("call dictwatcheradd(g:W, '%s', function('XWa'))", pat))
    quiet("let g:W.a = 1 | let g:W.ab = 2 | let g:W.ba = 3 | let g:W['a.b'] = 4")
    veval('watch pat log ' .. pat, 'string(sort(copy(g:WLOG)))')
    quiet(string.format("silent! call dictwatcherdel(g:W, '%s', function('XWa'))", pat))
  end

  -- The rejected forms, one per documented arm.
  quiet("let g:W = {'a': 1}")
  for _, bad in ipairs({
    { 'notdict', "call dictwatcheradd(1, '*', function('XWa'))" },
    { 'nopat', 'call dictwatcheradd(g:W, 1, function("XWa"))' },
    { 'nocb', "call dictwatcheradd(g:W, '*', 1)" },
    { 'nofunc', "call dictwatcheradd(g:W, '*', function('XWnope'))" },
    { 'delmissing', "call dictwatcherdel(g:W, '*', function('XWa'))" },
    { 'delnotdict', "call dictwatcherdel(1, '*', function('XWa'))" },
    { 'delnocb', "call dictwatcherdel(g:W, '*', 1)" },
  }) do
    exec('watch bad ' .. bad[1], bad[2])
  end

  -- Two watchers on one dict, and the same watcher twice: both are
  -- separate registrations, each holding its own reference.
  quiet("let g:WLOG = [] | let g:W = {}")
  quiet("call dictwatcheradd(g:W, '*', function('XWa'))")
  quiet("call dictwatcheradd(g:W, '*', function('XWa'))")
  quiet("call dictwatcheradd(g:W, 'a', function('XWa'))")
  quiet('let g:W.a = 1')
  veval('watch dup log', 'string(g:WLOG)')
  quiet("call dictwatcherdel(g:W, '*', function('XWa'))")
  quiet("let g:WLOG = [] | let g:W.a = 2")
  veval('watch dup log2', 'string(g:WLOG)')
  quiet("silent! call dictwatcherdel(g:W, '*', function('XWa'))")
  quiet("silent! call dictwatcherdel(g:W, 'a', function('XWa'))")

  -- The `change` dict a callback is handed: it is built for the call
  -- and freed after it unless the callback keeps a reference, which
  -- this one does.  Read back after a forced collection.
  quiet('let v:testing = 1')
  quiet("let g:W = {} | call dictwatcheradd(g:W, '*', function('XWkeep'))")
  quiet("let g:W.k = 'v'")
  veval('watch keep new', 'string(g:WKEEP)')
  quiet('call test_garbagecollect_now()')
  veval('watch keep new gc', 'string(g:WKEEP)')
  quiet("let g:W.k = 'w'")
  veval('watch keep upd', 'string(g:WKEEP)')
  quiet('unlet g:W.k')
  veval('watch keep del', 'string(g:WKEEP)')
  quiet('call test_garbagecollect_now()')
  veval('watch keep del gc', 'string(g:WKEEP)')
  quiet("silent! call dictwatcherdel(g:W, '*', function('XWkeep'))")

  -- A dict that is freed while it still carries watchers: the queue
  -- and every callback on it have to be released with it.
  quiet("let g:W = {} | call dictwatcheradd(g:W, '*', function('XWa'))")
  quiet("call dictwatcheradd(g:W, '*', function('XWb', {'tag': 'q'}))")
  quiet('unlet g:W')
  quiet('call test_garbagecollect_now()')
  veval('watch freed', "string(exists('g:W'))")

  -- The graph.  Every refcounted kind, wired to itself or shared, then
  -- read before and after two collections.
  quiet(table.concat({
    'let g:GRl = [1, 2]',
    'call add(g:GRl, g:GRl)',
    "let g:GRd = {'a': 1}",
    'let g:GRd.self = g:GRd',
    'let g:GRi = [1, 2]',
    'let g:GRs = [g:GRi, g:GRi]',
    "let g:GRp = function('XWb', [1], {'tag': 'g'})",
    'let g:GRc = XWmk(100)',
    'let g:GRb = 0z00112233',
    "let g:GRbd = {'b': g:GRb, 'l': g:GRl}",
    "let g:GRfd = {'f': function('XWa'), 'p': g:GRp, 'c': g:GRc}",
    'let g:GRll = [{-> 1}, {x -> x}, g:GRc]',
    'let g:GRcp = deepcopy(g:GRd)',
  }, '\n'))
  local READS = {
    { 'l', 'string(g:GRl)' },
    { 'l inner', 'string(g:GRl[2][0])' },
    { 'd', 'string(sort(keys(g:GRd)))' },
    { 'd self', 'string(g:GRd.self.self.a)' },
    { 'shared', 'string([g:GRs[0] is g:GRs[1], g:GRs[0] is g:GRi])' },
    { 'p', 'string(g:GRp)' },
    { 'c', 'string(g:GRc(1))' },
    { 'b', 'string(g:GRb)' },
    { 'bd', 'string(sort(keys(g:GRbd))) . string(g:GRbd.b)' },
    { 'fd', 'string(sort(keys(g:GRfd)))' },
    { 'll', 'string([g:GRll[0](), g:GRll[1](7)])' },
    { 'cp', 'string([sort(keys(g:GRcp)), g:GRcp.self is g:GRcp, g:GRcp is g:GRd])' },
  }
  local function read_all(when)
    for _, r in ipairs(READS) do
      veval('gr ' .. when .. ' ' .. r[1], r[2])
    end
  end
  read_all('live')
  quiet('call test_garbagecollect_now()')
  read_all('gc1')
  quiet('call test_garbagecollect_now()')
  read_all('gc2')

  -- Locking a graph the collector has already walked, then walking it
  -- again: `lockvar` writes through every reference the sweep marked.
  quiet('lockvar! g:GRd')
  quiet('call test_garbagecollect_now()')
  veval('gr locked', "string([islocked('g:GRd'), islocked('g:GRd.a')])")
  quiet('silent! unlockvar! g:GRd')
  veval('gr unlocked', "string([islocked('g:GRd'), islocked('g:GRd.a')])")

  -- Dropping the roots one at a time, collecting between each: the
  -- cycles are unreachable from the first `unlet` and only the sweep
  -- can free them.
  for _, name in ipairs({ 'g:GRl', 'g:GRi', 'g:GRp', 'g:GRc', 'g:GRb' }) do
    quiet('silent! unlet! ' .. name)
    quiet('call test_garbagecollect_now()')
    veval('gr drop ' .. name, "string([exists('" .. name .. "'), "
      .. "string(g:GRbd is# g:GRbd), sort(keys(g:GRfd))])")
  end
  veval('gr survivors', 'string([sort(keys(g:GRbd)), sort(keys(g:GRfd)), len(g:GRll)])')
  veval('gr survivor call', 'string([g:GRfd.c(1), g:GRll[2](1)])')
  quiet('call test_garbagecollect_now()')
  veval('gr survivor call gc', 'string([g:GRfd.c(1), g:GRll[2](1)])')

  -- A funcref whose function is deleted while the reference is held,
  -- across a collection: the ufunc lives until the last reference goes.
  quiet('silent! delfunction! XWdel')
  quiet("function! XWdel()\n  return 'alive'\nendfunction")
  quiet("let g:GRfd.d = function('XWdel')")
  exec('gr delfunction', 'delfunction XWdel')
  veval('gr del held', 'string(g:GRfd.d())')
  quiet('call test_garbagecollect_now()')
  veval('gr del held gc', 'string(g:GRfd.d())')
  quiet('unlet g:GRfd.d')
  quiet('call test_garbagecollect_now()')
  veval('gr del gone', 'string(exists("*XWdel"))')

  quiet('let v:testing = 0')
  wipe()
  for _, name in ipairs({ 'XWa', 'XWb', 'XWkeep', 'XWmk', 'XWdel' }) do
    quiet('silent! delfunction! ' .. name)
  end
end)

-- ---------------------------------------------------------------------
-- Run.
-- ---------------------------------------------------------------------

-- Every option any section reads is set explicitly: a sweep that
-- inherits one is a sweep whose baseline moves when a default does.
quiet('set noswapfile nomore shortmess=filnxtToOF report=9999')
quiet('set encoding=utf-8 fileencoding= isprint=@,161-255')
quiet('set ignorecase& smartcase& maxfuncdepth=100 textwidth=0 spelllang=en')
quiet('set columns=80 lines=24 cmdheight=1')
quiet('language C')

for _, entry in ipairs(SECTIONS) do
  if not only or entry.name:match(only) then
    if trace then
      io.stderr:write('== ', entry.name, '\n')
    end
    emit('')
    emit('== ' .. entry.name .. ' ==')
    local ok, err = pcall(entry.fn)
    if not ok then
      emit('== ' .. entry.name .. ' ABORTED:', esc(errtext(err)))
    end
  end
end

emit('')
emit('== done ==')
structfd:close()
