-- fssweep -- the fourteenth baselined differential.  Driven by
-- fssweep.sh, which builds the sandbox, pins $PATH/$HOME/
-- $TMPDIR and does the two scrubs only the shell can see.  Read that
-- header first.
--
-- The subsystem is eval/fs.rs plus fs/{name,path,find,dir,read,write}.rs
-- -- the thirty-three Vimscript builtins that ask the filesystem a
-- question or change it.  Before this oracle the family had NO
-- differential coverage at all (B18 survey Section 3).
--
-- Its characteristic regression is a SILENTLY WRONG PATH: `:h` that eats
-- one component too many, a `resolve()` that stops one link short, a
-- `glob()` that drops the dotfile, a `mkdir('p')` that leaves the
-- intermediate directory 0700.  So every case reports the ANSWER, and
-- every mutating case reports the resulting TREE -- mode, size, symlink
-- target and all -- rather than "did it succeed".
--
-- Sections:
--   s1  fnamemodify   name.rs -- modify_fname, 331 lines and the batch's
--                     single biggest item: 28 paths x 27 modifiers
--   s2  pathcalc      path.rs -- simplify / resolve / pathshorten /
--                     glob2regpat / isabsolutepath
--   s3  glob          find.rs -- glob / globpath / expand x wildcards x
--                     'wildignore' x 'suffixes' x backslash escapes
--   s4  readdir       find.rs -- readdir + its filter callback, and
--                     finddir/findfile across 'path'
--   s5  readwrite     read.rs + write.rs -- readfile/readblob <-> and
--                     writefile round trips, every flag
--   s6  mutate        dir.rs -- delete/rename/mkdir/filecopy, EACH IN
--                     ITS OWN NUMBERED SUBDIRECTORY with a fresh fixture
--   s7  cwd           dir.rs -- getcwd/chdir/haslocaldir across the
--                     global, tab and window scopes
--   s8  stat          fs.rs -- the getf* family + executable/exepath
--   s9  errors        every builtin with wrong arity and wrong types
--   s90 messages      the uncaptured block that fills .stderr
--   s91 crashprobe    the inputs that may kill the editor, one child each
--
-- Every section ends with a `## <name> rows=N` line.  That is not
-- decoration: a sweep that goes silently empty -- `verbose=0` under
-- `nvim -l`, or a pcall swallowing the whole section -- otherwise looks
-- exactly like a healthy one, and this is the cheapest tell there is.
--
-- NOTHING here is sorted that the editor sorts itself.  `readdir_core`
-- calls `sort_strings` and `gen_expand_wildcards` sorts its result, so
-- the order of `readdir()` and `glob()` is behaviour; sorting it in the
-- harness would blind the oracle to the regression it exists to catch.

local uv = vim.uv or vim.loop

local work = assert(os.getenv('FS_WORK'), 'FS_WORK unset')
local script = debug.getinfo(1, 'S').source:sub(2)

local argv = _G.arg or {}
local child_mode = argv[1]

local only = os.getenv('FSSWEEP_ONLY')
if only == '' then
  only = nil
end
local trace = os.getenv('FSSWEEP_TRACE') == '1'

io.stdout:setvbuf(child_mode and 'no' or 'line')

local rows = 0
local function emit(...)
  rows = rows + 1
  io.write(table.concat({ ... }, ' '), '\n')
end

-- --------------------------------------------------------------- scrub

--- Strip the bits of an answer that name where -- or when -- the run
--- happened.  Sorting happens after this, never before.
local function scrub(text)
  text = tostring(text)
  text = text:gsub(vim.pesc(work), '<WORK>')
  text = text:gsub(vim.pesc(script), '<SCRIPT>')
  -- `tempname()` is $TMPDIR/nvim.<user>/<six random>/nvim.<pid>.<n>.
  -- The SHAPE is the answer -- that it is under the sandbox's $TMPDIR,
  -- that it has the two levels, that the counter advances -- and every
  -- component of it differs per run by construction.  s91's
  -- `glob-40-stars` walks the whole sandbox and so sees the temp tree
  -- too, which is why all three levels are scrubbed and not just the
  -- one `tempname()` returns.  ORDER MATTERS: most specific first.
  text = text:gsub('nvim%.%d+%.%d+', 'nvim.<PID>.<SEQ>')
  text = text:gsub('nvim%.[%w_.-]+/[%w]+', 'nvim.<U>/<T>')
  text = text:gsub('nvim%.[%w_%-]+', 'nvim.<U>')
  -- A long run of one byte is the *length* of an s91 input, not the run.
  if #text >= 40 then
    local out, i, n = {}, 1, #text
    while i <= n do
      local c = text:sub(i, i)
      local j = i
      while j < n and text:sub(j + 1, j + 1) == c do
        j = j + 1
      end
      out[#out + 1] = (j - i + 1 >= 20) and string.format('%s<x%d>', c, j - i + 1)
        or text:sub(i, j)
      i = j + 1
    end
    text = table.concat(out)
  end
  return text
end

--- Cap a message.  A capped length is as diffable as an uncapped one and
--- this corpus' bloat is all in errors that quote a 4k path back.
local function cap(text, limit)
  limit = limit or 240
  if #text <= limit then
    return text
  end
  return text:sub(1, limit) .. string.format('...<+%d>', #text - limit)
end

--- Escape to one printable line.  The fixture carries a UTF-8 name and
--- s5 round-trips NUL bytes; a report line has to stay a report line.
local function esc(bytes)
  return (tostring(bytes):gsub('[%c\128-\255\\]', function(c)
    return string.format('\\x%02x', c:byte())
  end))
end

--- Case labels are names, not inputs: an input here is a path and a
--- path-shaped label would be scrubbed along with everything else.
local SEEN = {}
local function label_once(label)
  if SEEN[label] then
    emit('!!', 'DUPLICATE', 'LABEL', label)
  end
  SEEN[label] = true
  return label
end

-- -------------------------------------------------------------- struct

local structfd
if not child_mode then
  structfd = assert(io.open(assert(os.getenv('FS_STRUCT'), 'FS_STRUCT unset'), 'w'))
end

--- One-line quoting.  `string.format('%q', s)` writes a real newline for
--- \n in LuaJIT, which would split a struct row in two.
local function q(str)
  return '"'
    .. (str:gsub('[%c"\\\128-\255]', function(c)
      return string.format('\\x%02x', c:byte())
    end))
    .. '"'
end

local canon
function canon(value)
  local t = type(value)
  if t == 'string' then
    return q(scrub(value))
  elseif t == 'number' then
    if value == math.floor(value) and math.abs(value) < 2 ^ 53 then
      return string.format('%d', value)
    end
    return string.format('%.17g', value)
  elseif t ~= 'table' then
    return q(tostring(value))
  end
  local n = 0
  for _ in pairs(value) do
    n = n + 1
  end
  if n == #value then
    local parts = {}
    for i, item in ipairs(value) do
      parts[i] = canon(item)
    end
    return '[' .. table.concat(parts, ',') .. ']'
  end
  local keys = {}
  for k in pairs(value) do
    keys[#keys + 1] = tostring(k)
  end
  table.sort(keys)
  local parts = {}
  for _, k in ipairs(keys) do
    parts[#parts + 1] = q(k) .. ':' .. canon(value[k] == nil and value[tonumber(k)] or value[k])
  end
  return '{' .. table.concat(parts, ',') .. '}'
end

local function struct(label, value)
  if structfd then
    structfd:write(label, '\t', canon(value), '\n')
  end
end

--- Normalise an error to its message.  A pcall against nvim_eval prefixes
--- the Lua source position, which is a line number in THIS file and would
--- re-baseline the whole artifact on any edit above the case.
local function errtext(res)
  local s = tostring(res)
  s = s:gsub('\nstack traceback:.*$', '')
  s = s:gsub('^[^\n]-fssweep%.lua:%d+: ', '')
  s = s:gsub('^%[string "[^"]*"%]:%d+: ', '')
  s = s:gsub('\r?\n', ' | ')
  return cap(scrub(s))
end

local function ins(value)
  return (vim.inspect(value, { newline = ' ', indent = '' }))
end

-- ------------------------------------------------------------ sections

local secrows = 0
local function section(name, fn)
  if only and not name:match(only) then
    return
  end
  if trace then
    io.stderr:write('== ' .. name .. '\n')
  end
  -- Reset the world.  s7 leaves window- and tab-local directories
  -- behind, and a later section reading a relative path would then be
  -- asking a different question than it asks on a clean run.
  pcall(vim.cmd, 'silent! tabonly')
  pcall(vim.cmd, 'silent! only')
  pcall(vim.cmd, 'cd ' .. vim.fn.fnameescape(work))
  pcall(function()
    vim.o.wildignore = ''
    vim.o.suffixes = ''
    vim.o.path = '.,,'
    vim.o.wildignorecase = false
  end)
  secrows = rows
  emit('##', name)
  local ok, err = pcall(fn)
  if not ok then
    emit('##', name, 'RAISED', esc(scrub(errtext(err))))
  end
  emit('##', name, string.format('rows=%d', rows - secrows - 1))
end

-- ------------------------------------------------------------- fixture

--- mkdir -p, with `vim.uv` rather than the builtin under test.
local function mkdirp(path, mode)
  local parts = {}
  for part in path:gmatch('[^/]+') do
    parts[#parts + 1] = part
  end
  local at = ''
  for _, part in ipairs(parts) do
    at = at .. '/' .. part
    uv.fs_mkdir(at, mode or 493) -- 0755
  end
end

local function writebytes(path, bytes, mode)
  -- Parenthesised: `gsub` returns (string, count) and an unwrapped call
  -- would hand the count to `mkdirp` as its MODE.  It read 1, every
  -- fixture directory came out `--------x`, and every section died with
  -- one EACCES -- which is exactly what the `rows=` line exists to make
  -- visible.
  mkdirp((path:gsub('/[^/]*$', '')))
  -- Re-openable: the fixture carries a mode-000 file, and s91's children
  -- rebuild the same root once per case.  Without this, every child but
  -- the first died EACCES and read as 53 ABORTED rows.
  uv.fs_chmod(path, 420)
  local fd = assert(uv.fs_open(path, 'w', 420)) -- 0644
  if #bytes > 0 then
    uv.fs_write(fd, bytes)
  end
  uv.fs_close(fd)
  uv.fs_chmod(path, mode or 420)
end

local ROOT0 = uv.getuid and uv.getuid() == 0 or false

--- Build a fresh fixture at $WORK/f/<name> and return its absolute path.
---
--- A fresh directory per section rather than a rebuild in place: nothing
--- has to be deleted, so the fixture never depends on `delete()`, which
--- is one of the functions under test.
local function fixture(name)
  local root = work .. '/f/' .. name
  mkdirp(root)

  -- nested directories and files of known size
  writebytes(root .. '/top.txt', 'top1234')             -- 7
  writebytes(root .. '/a/one.txt', 'one\n')             -- 4
  writebytes(root .. '/a/b/two.txt', '0123456789')      -- 10
  writebytes(root .. '/a/b/c/deep.txt', 'deep\n')       -- 5
  writebytes(root .. '/empty.txt', '')                  -- 0
  writebytes(root .. '/README', 'readme\n')
  writebytes(root .. '/arch.tar.gz', 'gz')
  writebytes(root .. '/.hidden', 'hidden\n')
  writebytes(root .. '/note.txt~', 'backup\n')
  writebytes(root .. '/note.txt', 'note\n')
  mkdirp(root .. '/dir1/sub')
  mkdirp(root .. '/dir2')

  -- names the wildcard and escape layers have to survive
  writebytes(root .. '/spa ced.txt', 'spaced\n')
  writebytes(root .. '/br[ack].txt', 'bracket\n')
  writebytes(root .. '/st*ar.txt', 'star\n')
  writebytes(root .. '/ha#sh.txt', 'hash\n')
  writebytes(root .. '/per%ent.txt', 'percent\n')
  writebytes(root .. '/\195\188n\195\175c\195\184d\195\169.txt', 'utf8\n')

  -- permissions.  Explicit on every one, because getfperm is an answer.
  writebytes(root .. '/ro.txt', 'readonly\n', 292) -- 0444
  writebytes(root .. '/exec.sh', '#!/bin/sh\nexit 0\n', 493) -- 0755
  if not ROOT0 then
    -- Mode 000 is only a question when the process cannot override it.
    writebytes(root .. '/noperm.txt', 'noperm\n', 0)
  end

  -- a symlink CHAIN, a directory symlink, and a broken one
  uv.fs_symlink(root .. '/top.txt', root .. '/link2')
  uv.fs_symlink(root .. '/link2', root .. '/link1')
  uv.fs_symlink('a', root .. '/dlink')
  uv.fs_symlink(root .. '/nosuchtarget', root .. '/dangle')
  uv.fs_symlink('./relto.txt', root .. '/rellink')
  writebytes(root .. '/relto.txt', 'relto\n')
  -- A loop: resolve() has to give up rather than spin.
  uv.fs_symlink(root .. '/loopb', root .. '/loopa')
  uv.fs_symlink(root .. '/loopa', root .. '/loopb')

  return root
end

--- A canonical listing of a tree: the answer for every mutating case.
--- Mode, size and symlink target are all in it; mtime deliberately is
--- not.
local function tree_lines(root)
  local out = {}
  local function walk(dir, rel)
    local names = {}
    local h = uv.fs_scandir(dir)
    if not h then
      out[#out + 1] = (rel == '' and '.' or rel) .. ' <unreadable>'
      return
    end
    while true do
      local n = uv.fs_scandir_next(h)
      if not n then
        break
      end
      names[#names + 1] = n
    end
    table.sort(names)
    for _, n in ipairs(names) do
      local p = dir .. '/' .. n
      local r = rel == '' and n or (rel .. '/' .. n)
      local st = uv.fs_lstat(p)
      if not st then
        out[#out + 1] = r .. ' ?'
      elseif st.type == 'link' then
        out[#out + 1] = string.format('%s link -> %s', r, scrub(tostring(uv.fs_readlink(p))))
      elseif st.type == 'directory' then
        out[#out + 1] = string.format('%s dir %04o', r, st.mode % 4096)
        walk(p, r)
      else
        out[#out + 1] = string.format('%s %s %d %04o', r, st.type, st.size, st.mode % 4096)
      end
    end
  end
  walk(root, '')
  return out
end

local function report_tree(label, root)
  local lines = tree_lines(root)
  for _, line in ipairs(lines) do
    emit(label, 'T', esc(line))
  end
  if #lines == 0 then
    emit(label, 'T', '<empty>')
  end
  struct(label .. '#T', lines)
end

-- ---------------------------------------------------------------- ask

--- A Vimscript single-quoted literal.  Every path in this corpus goes
--- through here; a double-quoted one would re-interpret the backslash
--- escapes that half the cases are *about*.
local function vq(str)
  return "'" .. tostring(str):gsub("'", "''") .. "'"
end

--- Evaluate one Vimscript expression and report its answer.
local function ans(label, expr)
  label_once(label)
  local ok, value = pcall(vim.api.nvim_eval, expr)
  if ok then
    emit(label, '=', esc(scrub(ins(value))))
    struct(label .. '#=', value)
  else
    emit(label, '!', esc(errtext(value)))
    struct(label .. '#!', errtext(value))
  end
  return ok, value
end

-- =====================================================================
-- s91's corpus.  Inputs whose failure mode is "the editor stops
-- existing" rather than "an error is reported".
-- =====================================================================

local CRASH = {}
do
  local function add(name, expr)
    CRASH[#CRASH + 1] = { name, expr }
  end
  local big = string.rep('x', 8192)
  add('glob-deep-star', "glob('" .. string.rep('*/', 60) .. "*')")
  add('glob-40-stars', "glob('" .. string.rep('*', 40) .. "')")
  add('glob-doublestar-24', "glob('" .. string.rep('**/', 24) .. "*')")
  add('glob-brace-blowup', "glob('{a,b}{a,b}{a,b}{a,b}{a,b}{a,b}{a,b}{a,b}{a,b}{a,b}')")
  add('glob-name-8k', "glob('" .. big .. "')")
  add('glob-open-bracket', "glob('" .. string.rep('[', 4096) .. "')")
  add('glob-backslash-run', "glob('" .. string.rep('\\\\', 2048) .. "')")
  add('globpath-many-dirs', "globpath('" .. string.rep(work .. ',', 2000) .. "', '*')")
  add('globpath-deep-pat', "globpath('" .. work .. "', '" .. string.rep('*/', 40) .. "*')")
  add('expand-deep-star', "expand('" .. string.rep('*/', 60) .. "*')")
  add('mkdir-deep-p', "mkdir('" .. work .. '/f/s91/' .. string.rep('d/', 400) .. "', 'p')")
  add('mkdir-name-8k', "mkdir('" .. work .. '/f/s91/' .. big .. "', 'p')")
  add('mkdir-empty-p', "mkdir('', 'p')")
  add('mkdir-slash-p', "mkdir('/', 'p')")
  add('mkdir-dots-p', "mkdir('" .. string.rep('../', 200) .. "x', 'p')")
  add('mkdir-prot-huge', "mkdir('" .. work .. "/f/s91/pm', 'p', 9223372036854775807)")
  add('mkdir-prot-neg', "mkdir('" .. work .. "/f/s91/pn', 'p', -9223372036854775807-1)")
  add('delete-empty-rf', "delete('', 'rf')")
  add('delete-slash-d', "delete('/', 'd')")
  add('delete-name-8k', "delete('" .. big .. "', 'rf')")
  add('rename-empty', "rename('', '')")
  add('rename-name-8k', "rename('" .. big .. "', '" .. big .. "')")
  add('filecopy-self', "filecopy('" .. work .. "/empty', '" .. work .. "/empty')")
  add('fnamemodify-8k', "fnamemodify('" .. big .. "', ':p:h:h:t:r:e')")
  add('fnamemodify-mods-8k', "fnamemodify('a/b', '" .. string.rep(':h', 4096) .. "')")
  add('fnamemodify-gsub-empty', "fnamemodify('aaa', ':gs??X?')")
  add('fnamemodify-many-dots', "fnamemodify('" .. string.rep('../', 2000) .. "x', ':p')")
  add('resolve-loop', "resolve('" .. work .. "/f/s91fx/loopa')")
  add('resolve-8k', "resolve('" .. big .. "')")
  add('resolve-deep', "resolve('" .. string.rep('a/', 2000) .. "b')")
  add('simplify-8k-dots', "simplify('" .. string.rep('a/../', 2000) .. "b')")
  add('pathshorten-huge-len', "pathshorten('/a/b/c/d', 2147483647)")
  add('pathshorten-neg-len', "pathshorten('/a/b/c/d', -2147483648)")
  add('glob2regpat-8k', "glob2regpat('" .. big .. "')")
  add('glob2regpat-braces', "glob2regpat('" .. string.rep('{a,', 2000) .. "}')")
  add('readdir-proc-self', "len(readdir('/proc/self'))")
  add('readdir-filter-error', "readdir('" .. work .. "', {n -> nosuchfunc(n)})")
  add('readfile-proc-kcore-max', "len(readfile('/proc/self/maps', '', 1))")
  add('readfile-neg-max', "len(readfile('" .. work .. "/empty', '', -2147483648))")
  add('readblob-huge-off', "readblob('" .. work .. "/empty', 9223372036854775807)")
  add('writefile-empty-name', "writefile(['x'], '')")
  add('writefile-bad-flags', "writefile(['x'], '" .. work .. "/f/s91/w', 'zzzz')")
  add('findfile-count-max', "findfile('top.txt', '" .. work .. "/**', 2147483647)")
  add('finddir-upward-root', "finddir('nosuch', '" .. work .. ";/')")
  add('getcwd-huge', 'getcwd(2147483647, 2147483647)')
  add('haslocaldir-huge', 'haslocaldir(2147483647, 2147483647)')
  add('chdir-empty', "chdir('')")
  add('tempname-1k', "len(map(range(1000), {-> tempname()}))")
  -- The viml parser's own two sharp inputs, per the B18 plan: the
  -- lambda arrow and the dict-literal brace are what `parse.rs` and
  -- `lexer.rs` disagree about, and nothing else here reaches them.
  add('parse-arrow-run', "len(nvim_parse_expression('" .. string.rep('->', 2000) .. "', '', v:true))")
  add('parse-brace-run', "len(nvim_parse_expression('" .. string.rep('{', 2000) .. "', '', v:true))")
  add('parse-arrow-lambda', "len(nvim_parse_expression('{a->{b->{c->d}}}', '', v:true))")
  add('parse-brace-curly', "len(nvim_parse_expression('a{b{c{d}}}', 'm', v:true))")
  add('parse-hash-brace', "len(nvim_parse_expression('#{a: #{b: #{c: 1}}}', '', v:true))")
  add('eval-arrow-run', "'" .. string.rep('->', 500) .. "'")
end

--- One crash case, printed unbuffered as `<idx> <label> X <answer>`.
--- Exactly ONE line per index, deliberately: a two-line shape makes a
--- death during execution blame the next case and skip it.
local function crashline(i)
  local c = CRASH[i]
  local ok, value = pcall(vim.api.nvim_eval, c[2])
  io.write(
    i,
    ' k91/',
    c[1],
    ' X ',
    esc(scrub(ok and ('= ' .. ins(value)) or ('! ' .. errtext(value)))),
    '\n'
  )
end

if child_mode == '--crash' then
  fixture('s91fx')
  crashline(assert(tonumber(argv[2]), 'crash index'))
  os.exit(0)
end

-- =====================================================================
-- s1 -- fnamemodify.  name.rs / modify_fname, 331 lines.
-- =====================================================================

section('s1-fnamemodify', function()
  local root = fixture('s1')
  local home = work .. '/home'
  mkdirp(home)
  writebytes(home .. '/hfile.txt', 'home\n')

  local PATHS = {
    { 'empty', '' },
    { 'rel', 'top.txt' },
    { 'reldot', './top.txt' },
    { 'reldeep', 'a/b/c/deep.txt' },
    { 'reltrail', 'a/b/c/' },
    { 'reldotdot', 'a/b/../one.txt' },
    { 'absnosuch', '/no/such/file.tar.gz' },
    -- `nosub`: the substitute modifiers REWRITE THE WORK DIRECTORY
    -- ITSELF (`:gs#/#|#` turns every separator into a bar, `:s?a?Z?`
    -- eats an `a` out of the `mktemp` suffix), and a scrub that looks
    -- for `$WORK` no longer recognises what comes back.  Two runs from
    -- two work directories then differ for a reason that is not
    -- behaviour -- which is the whole point of running it twice.
    -- The substitute arm is measured on relative inputs instead, where
    -- the string is entirely the harness's own.
    { 'absarch', root .. '/arch.tar.gz', nosub = true },
    { 'absdeep', root .. '/a/b/c/deep.txt', nosub = true },
    { 'absdirtrail', root .. '/dir1/', nosub = true },
    { 'tildefile', '~/hfile.txt' },
    { 'tilde', '~' },
    { 'dot', '.' },
    { 'dotdot', '..' },
    { 'dotdotslash', '../' },
    { 'slash', '/' },
    { 'noext', 'README' },
    { 'hidden', '.hidden' },
    { 'spaced', 'spa ced.txt' },
    { 'bracket', 'br[ack].txt' },
    { 'star', 'st*ar.txt' },
    { 'hash', 'ha#sh.txt' },
    { 'utf8', '\195\188n\195\175c\195\184d\195\169.txt' },
    { 'dblslash', 'a//b///c.txt' },
    { 'twoext', 'x.tar.gz' },
    { 'link', 'link1' },
    { 'dangle', 'dangle' },
    { 'viadlink', 'dlink/one.txt' },
  }

  local MODS = {
    { 'p', ':p' },
    { 'ph', ':p:h' },
    { 'phh', ':p:h:h' },
    { 'pt', ':p:t' },
    { 'h', ':h' },
    { 'hh', ':h:h' },
    { 't', ':t' },
    { 'tr', ':t:r' },
    { 'r', ':r' },
    { 'rr', ':r:r' },
    { 'e', ':e' },
    { 'ee', ':e:e' },
    { 'tilde', ':~' },
    { 'dot', ':.' },
    { 'ptilde', ':p:~' },
    { 'pdot', ':p:.' },
    { 'S', ':S' },
    { 'eight', ':8' },
    { 'sub', ':s?a?Z?', sub = true },
    { 'gsub', ':gs?a?Z?', sub = true },
    { 'subplus', ':s+txt+TXT+', sub = true },
    { 'gsubslash', ':gs#/#|#', sub = true },
    { 'pht', ':p:h:t' },
    { 'te', ':t:e' },
    { 're', ':r:e' },
    { 'tildeh', ':~:h' },
    { 'dott', ':.:t' },
  }

  -- The cwd is the fixture root for the whole section: every relative
  -- path above resolves against it and `:.` is measured from it.
  vim.cmd('cd ' .. vim.fn.fnameescape(root))
  for _, p in ipairs(PATHS) do
    for _, m in ipairs(MODS) do
      if not (p.nosub and m.sub) then
        ans('s1/' .. p[1] .. '/' .. m[1], 'fnamemodify(' .. vq(p[2]) .. ', ' .. vq(m[2]) .. ')')
      end
    end
  end

  -- Modifier-string edge cases that are not a cross product.
  local EDGE = {
    { 'nomods', 'a/b.txt', '' },
    { 'unknown', 'a/b.txt', ':q' },
    { 'trailingcolon', 'a/b.txt', ':' },
    { 'doublecolon', 'a/b.txt', '::h' },
    { 'subnoclose', 'a/b.txt', ':s?a?Z' },
    { 'subempty', 'aaa', ':s???' },
    { 'subemptypat', 'aaa', ':s??X?' },
    { 'gsubemptypat', 'aaa', ':gs??X?' },
    { 'submeta', 'a.b.c', ':s?\\.?-?' },
    { 'gsubmeta', 'a.b.c', ':gs?\\.?-?' },
    { 'subamp', 'abc', ':s?b?[&]?' },
    { 'subbackref', 'abc', ':s?\\(b\\)?<\\1>?' },
    { 'subtilde', 'abc', ':s?b?X?:s?X?~?' },
    { 'esixteen', 'a.b.c.d.e', ':e:e:e:e' },
    { 'rtimes4', 'a.b.c.d.e', ':r:r:r:r' },
    { 'hafterp', 'top.txt', ':h:p' },
    { 'tafterh', 'a/b/c.txt', ':h:t' },
    { 'Safterp', 'spa ced.txt', ':p:S' },
    { 'Ssub', "it's", ':S' },
    { 'onlydots', '...', ':e' },
    { 'onlydotsr', '...', ':r' },
    { 'dotfileext', '.hidden', ':e' },
    { 'dotfiler', '.hidden', ':r' },
    { 'trailslashh', 'a/b/', ':h' },
    { 'trailslasht', 'a/b/', ':t' },
    { 'rootp', '/', ':p' },
    { 'rooth', '/', ':h' },
    { 'homeonly', '~', ':p' },
    { 'homeuser', '~root/x', ':p' },
    { 'envvar', '$HOME/x', ':p' },
  }
  for _, e in ipairs(EDGE) do
    ans('s1/edge/' .. e[1], 'fnamemodify(' .. vq(e[2]) .. ', ' .. vq(e[3]) .. ')')
  end
end)

-- =====================================================================
-- s2 -- simplify / resolve / pathshorten / glob2regpat / isabsolutepath.
-- =====================================================================

section('s2-pathcalc', function()
  local root = fixture('s2')
  vim.cmd('cd ' .. vim.fn.fnameescape(root))

  local SIMPLIFY = {
    { 'plain', 'a/b/c' },
    { 'dot', 'a/./b' },
    { 'dots', './a/./b/./' },
    { 'dotdot', 'a/../b' },
    { 'dotdot2', 'a/b/../../c' },
    { 'leadingdotdot', '../a' },
    { 'onlydotdot', '..' },
    { 'dotdotroot', '/../a' },
    { 'dblslash', 'a//b' },
    { 'trislash', 'a///b' },
    { 'leaddbl', '//a/b' },
    { 'leadtri', '///a/b' },
    { 'trailslash', 'a/b/' },
    { 'trailslashes', 'a/b///' },
    { 'empty', '' },
    { 'justdot', '.' },
    { 'justslash', '/' },
    { 'dotslash', './' },
    { 'mixed', './a/../b/./c/../d' },
    { 'dotdotpastroot', '/a/../../b' },
    { 'namedotdot', 'a..b/c' },
    { 'dotdotname', '..a/b' },
    { 'space', 'a b/../c' },
    { 'utf8', '\195\188/../x' },
    { 'abs', root .. '/a/../top.txt' },
  }
  for _, c in ipairs(SIMPLIFY) do
    ans('s2/simplify/' .. c[1], 'simplify(' .. vq(c[2]) .. ')')
  end

  local RESOLVE = {
    { 'chain', root .. '/link1' },
    { 'mid', root .. '/link2' },
    { 'dirlink', root .. '/dlink' },
    { 'viadirlink', root .. '/dlink/one.txt' },
    { 'viadirlinkdots', root .. '/dlink/../top.txt' },
    { 'broken', root .. '/dangle' },
    { 'rellink', root .. '/rellink' },
    { 'loop', root .. '/loopa' },
    { 'plainfile', root .. '/top.txt' },
    { 'plaindir', root .. '/dir1' },
    { 'trailslash', root .. '/dir1/' },
    { 'nosuch', root .. '/nosuchfile' },
    { 'relative', 'link1' },
    { 'reldot', './link1' },
    { 'dot', '.' },
    { 'dotdot', '..' },
    { 'root', '/' },
    { 'empty', '' },
    { 'tilde', '~' },
    { 'linktrailslash', root .. '/dlink/' },
  }
  for _, c in ipairs(RESOLVE) do
    ans('s2/resolve/' .. c[1], 'resolve(' .. vq(c[2]) .. ')')
  end

  local SHORTEN = {
    '/a/bb/ccc/dddd/e.txt',
    'a/bb/ccc',
    '~/a/bb/ccc',
    './a/bb',
    '/',
    '',
    'noslash',
    '/a//bb///ccc',
    '/.a/.bb/ccc',
    '/\195\188x/\195\169y/z',
    '/a b/c d/e',
    '/a/bb/ccc/',
  }
  for i, p in ipairs(SHORTEN) do
    ans(string.format('s2/pathshorten/%02d', i), 'pathshorten(' .. vq(p) .. ')')
    for _, n in ipairs({ 0, 1, 2, 3, 8 }) do
      ans(
        string.format('s2/pathshorten/%02d/n%d', i, n),
        'pathshorten(' .. vq(p) .. ', ' .. n .. ')'
      )
    end
  end

  local G2R = {
    '*',
    '*.c',
    'a?c',
    '[abc]',
    '[!abc]',
    '[^abc]',
    '{a,b}',
    '{a,b}{c,d}',
    '**',
    '**/*.c',
    'a\\*b',
    'a\\?b',
    'a.b',
    'a+b',
    'a$b',
    'a^b',
    'a~b',
    'a|b',
    'a(b)c',
    '',
    '/',
    'a\\\\b',
    '\195\188*',
    'a**b',
  }
  for i, p in ipairs(G2R) do
    ans(string.format('s2/glob2regpat/%02d', i), 'glob2regpat(' .. vq(p) .. ')')
  end

  local ABS = {
    '/a',
    'a',
    './a',
    '../a',
    '~',
    '~/a',
    '',
    '/',
    '//a',
    'C:/a',
    '\\a',
    ' /a',
    '$HOME/a',
  }
  for i, p in ipairs(ABS) do
    ans(string.format('s2/isabs/%02d', i), 'isabsolutepath(' .. vq(p) .. ')')
  end
end)

-- =====================================================================
-- s3 -- glob / globpath / expand.
-- =====================================================================

section('s3-glob', function()
  local root = fixture('s3')
  fixture('s3b')
  vim.cmd('cd ' .. vim.fn.fnameescape(root))

  local PATS = {
    { 'star', '*' },
    { 'startxt', '*.txt' },
    { 'dotstar', '.*' },
    { 'question', 'a?ch.tar.gz' },
    { 'classhit', '[an]ote.txt' },
    { 'classneg', '[!.]*.txt' },
    { 'brace', '{top,note}.txt' },
    { 'bracenest', '{top,{note,arch}}*' },
    { 'dstar', '**' },
    { 'dstartxt', '**/*.txt' },
    { 'dirstar', 'a/*' },
    { 'dirdstar', 'a/**' },
    { 'twolevel', '*/*' },
    { 'threelevel', '*/*/*' },
    { 'trailslash', 'dir*/' },
    { 'escbracket', 'br\\[ack\\].txt' },
    { 'rawbracket', 'br[ack].txt' },
    { 'escstar', 'st\\*ar.txt' },
    { 'spaced', 'spa ced.txt' },
    { 'escspace', 'spa\\ ced.txt' },
    { 'hash', 'ha#sh.txt' },
    { 'percent', 'per%ent.txt' },
    { 'utf8', '\195\188*' },
    { 'nomatch', 'nosuchthing*' },
    { 'exact', 'top.txt' },
    { 'exactmissing', 'nosuch.txt' },
    { 'abs', root .. '/*.txt' },
    { 'tilde', '~/*' },
    { 'tildeonly', '~' },
    { 'dotdot', '../s3/*.txt' },
    { 'linkstar', 'l*' },
    { 'dangle', 'dangle' },
    { 'empty', '' },
    { 'slash', '/' },
    { 'env', '$HOME/*' },
  }

  for _, p in ipairs(PATS) do
    ans('s3/glob/' .. p[1], 'glob(' .. vq(p[2]) .. ')')
    ans('s3/globlist/' .. p[1], 'glob(' .. vq(p[2]) .. ', v:false, v:true)')
  end

  -- 'wildignore' and 'suffixes' -- the two options that filter and
  -- reorder a glob result, and the `nosuf` argument that turns the first
  -- of them off.
  for _, wig in ipairs({ '', '*.txt', '*.txt,*.gz', 'note*', '*/a/*' }) do
    vim.o.wildignore = wig
    local tag = wig == '' and 'none' or wig:gsub('[^%w]', '_')
    ans('s3/wig/' .. tag .. '/star', "glob('*', v:false, v:true)")
    ans('s3/wig/' .. tag .. '/nosuf', "glob('*', v:true, v:true)")
    ans('s3/wig/' .. tag .. '/dstar', "glob('**', v:false, v:true)")
  end
  vim.o.wildignore = ''

  for _, suf in ipairs({ '', '.txt', '.gz,.txt', '~' }) do
    vim.o.suffixes = suf
    local tag = suf == '' and 'none' or suf:gsub('[^%w]', '_')
    ans('s3/suf/' .. tag .. '/note', "glob('note*', v:false, v:true)")
    ans('s3/suf/' .. tag .. '/expand', "expand('note*', v:false, v:true)")
  end
  vim.o.suffixes = ''

  -- alllinks (the fourth argument) decides whether a broken symlink is
  -- a match.
  for _, p in ipairs({ 'dangle', 'l*', '*' }) do
    ans(
      's3/alllinks/' .. p:gsub('[^%w]', '_'),
      'glob(' .. vq(p) .. ', v:false, v:true, v:true)'
    )
  end

  local GLOBPATH = {
    { 'two', work .. '/f/s3,' .. work .. '/f/s3b', '*.txt' },
    { 'twolist', work .. '/f/s3,' .. work .. '/f/s3b', '*.txt' },
    { 'missingdir', work .. '/f/nosuch,' .. work .. '/f/s3', '*.txt' },
    { 'emptydirs', '', '*.txt' },
    { 'trailcomma', work .. '/f/s3,', '*.txt' },
    { 'dstar', work .. '/f/s3', '**/*.txt' },
    { 'nomatch', work .. '/f/s3,' .. work .. '/f/s3b', 'nosuch*' },
    { 'spacedir', work .. '/f/s3', 'spa ced.txt' },
    { 'escapedcomma', work .. '/f/s3\\,x,' .. work .. '/f/s3', '*.txt' },
  }
  for _, c in ipairs(GLOBPATH) do
    ans('s3/globpath/' .. c[1], 'globpath(' .. vq(c[2]) .. ', ' .. vq(c[3]) .. ')')
    ans(
      's3/globpathlist/' .. c[1],
      'globpath(' .. vq(c[2]) .. ', ' .. vq(c[3]) .. ', v:false, v:true)'
    )
  end

  local EXPAND = {
    'top.txt',
    '*.txt',
    '~',
    '~/',
    '$HOME',
    '$NOSUCHVAR',
    '$HOME/nosuch',
    '%',
    '#',
    '<cfile>',
    '<afile>',
    'nosuch*',
    'spa ced.txt',
    'br[ack].txt',
    '\\*',
  }
  for i, p in ipairs(EXPAND) do
    ans(string.format('s3/expand/%02d', i), 'expand(' .. vq(p) .. ')')
    ans(string.format('s3/expandlist/%02d', i), 'expand(' .. vq(p) .. ', v:false, v:true)')
  end
end)

-- =====================================================================
-- s4 -- readdir and its filter callback; finddir/findfile over 'path'.
-- =====================================================================

section('s4-readdir', function()
  local root = fixture('s4')
  vim.cmd('cd ' .. vim.fn.fnameescape(root))

  -- NOT sorted here: readdir_core sorts its own result.
  ans('s4/readdir/plain', 'readdir(' .. vq(root) .. ')')
  ans('s4/readdir/rel', "readdir('.')")
  ans('s4/readdir/sub', 'readdir(' .. vq(root .. '/a/b') .. ')')
  ans('s4/readdir/empty', 'readdir(' .. vq(root .. '/dir2') .. ')')
  ans('s4/readdir/dirlink', 'readdir(' .. vq(root .. '/dlink') .. ')')
  ans('s4/readdir/trailslash', 'readdir(' .. vq(root .. '/a/') .. ')')
  ans('s4/readdir/nosuch', 'readdir(' .. vq(root .. '/nosuch') .. ')')
  ans('s4/readdir/isfile', 'readdir(' .. vq(root .. '/top.txt') .. ')')
  ans('s4/readdir/emptyarg', "readdir('')")

  -- The filter callback: 1 keeps, 0 drops, 2 keeps and stops, -1 aborts.
  ans('s4/readdir/f-all', 'readdir(' .. vq(root) .. ", {n -> 1})")
  ans('s4/readdir/f-none', 'readdir(' .. vq(root) .. ", {n -> 0})")
  ans('s4/readdir/f-stop', 'readdir(' .. vq(root) .. ", {n -> n =~ '^a' ? 2 : 1})")
  ans('s4/readdir/f-abort', 'readdir(' .. vq(root) .. ", {n -> n =~ '^b' ? -1 : 1})")
  ans('s4/readdir/f-txt', 'readdir(' .. vq(root) .. ", {n -> n =~ '\\\\.txt$'})")
  ans('s4/readdir/f-vval', 'readdir(' .. vq(root) .. ", 'v:val =~ \"^d\"')")
  ans('s4/readdir/f-string', 'readdir(' .. vq(root) .. ", '1')")
  ans('s4/readdir/f-badtype', 'readdir(' .. vq(root) .. ", {n -> [1]})")
  ans('s4/readdir/f-throws', 'readdir(' .. vq(root) .. ", {n -> nosuchfn(n)})")

  -- finddir / findfile over 'path'.  The answers are a path each, and
  -- the count argument is where the off-by-one lives.
  local PATHS = {
    { 'dot', '.' },
    { 'dotcomma', '.,,' },
    { 'abs', root },
    { 'dstar', root .. '/**' },
    { 'dstar2', root .. '/**2' },
    { 'two', root .. ',' .. root .. '/a' },
    { 'upward', root .. '/a/b/c;' },
    { 'upwardstop', root .. '/a/b/c;' .. root },
    { 'empty', '' },
    { 'nosuch', root .. '/nosuch' },
  }
  for _, p in ipairs(PATHS) do
    vim.o.path = p[2]
    ans('s4/findfile/' .. p[1] .. '/one', "findfile('one.txt')")
    ans('s4/findfile/' .. p[1] .. '/deep', "findfile('deep.txt')")
    ans('s4/findfile/' .. p[1] .. '/missing', "findfile('nosuch.txt')")
    ans('s4/findfile/' .. p[1] .. '/all', "findfile('one.txt', '', -1)")
    ans('s4/findfile/' .. p[1] .. '/second', "findfile('one.txt', '', 2)")
    ans('s4/finddir/' .. p[1] .. '/sub', "finddir('sub')")
    ans('s4/finddir/' .. p[1] .. '/all', "finddir('sub', '', -1)")
  end
  vim.o.path = '.,,'

  -- The explicit `path` argument, which overrides the option.
  ans('s4/findfile/argpath', "findfile('deep.txt', " .. vq(root .. '/**') .. ')')
  ans('s4/findfile/argpathall', "findfile('deep.txt', " .. vq(root .. '/**') .. ', -1)')
  ans('s4/finddir/argpath', "finddir('c', " .. vq(root .. '/**') .. ')')
  ans('s4/findfile/argupward', "findfile('top.txt', " .. vq(root .. '/a/b/c;') .. ')')
  ans('s4/findfile/emptyname', "findfile('', " .. vq(root) .. ')')
  ans('s4/finddir/emptyname', "finddir('', " .. vq(root) .. ')')
  ans('s4/findfile/count0', "findfile('one.txt', " .. vq(root .. '/**') .. ', 0)')
end)

-- =====================================================================
-- s5 -- readfile / readblob / writefile round trips.
-- =====================================================================

section('s5-readwrite', function()
  local root = fixture('s5')
  vim.cmd('cd ' .. vim.fn.fnameescape(root))

  -- Corpus written with uv, not with writefile: the read half has to be
  -- measurable independently of the write half.
  writebytes(root .. '/r_lf.txt', 'a\nb\nc\n')
  writebytes(root .. '/r_noeol.txt', 'a\nb\nc')
  writebytes(root .. '/r_crlf.txt', 'a\r\nb\r\nc\r\n')
  writebytes(root .. '/r_cr.txt', 'a\rb\rc')
  writebytes(root .. '/r_nul.txt', 'a\0b\nc\0\n')
  writebytes(root .. '/r_blank.txt', '\n\n\n')
  writebytes(root .. '/r_empty.txt', '')
  writebytes(root .. '/r_bom.txt', '\239\187\191a\nb\n')
  writebytes(root .. '/r_utf8.txt', '\195\188\195\169\n\195\184\n')
  writebytes(root .. '/r_long.txt', string.rep('x', 5000) .. '\n')
  writebytes(root .. '/r_bin.bin', '\0\1\2\255\254\n\r')

  local FILES = {
    'r_lf.txt',
    'r_noeol.txt',
    'r_crlf.txt',
    'r_cr.txt',
    'r_nul.txt',
    'r_blank.txt',
    'r_empty.txt',
    'r_bom.txt',
    'r_utf8.txt',
    'r_long.txt',
    'r_bin.bin',
    'noperm.txt',
    'nosuch.txt',
    'dir1',
  }
  for _, f in ipairs(FILES) do
    local tag = f:gsub('[^%w]', '_')
    ans('s5/readfile/' .. tag, 'readfile(' .. vq(f) .. ')')
    ans('s5/readfileb/' .. tag, 'readfile(' .. vq(f) .. ", 'b')")
    ans('s5/readfileB/' .. tag, 'readfile(' .. vq(f) .. ", 'B')")
    ans('s5/readblob/' .. tag, 'readblob(' .. vq(f) .. ')')
  end

  -- The `max` argument, positive and negative, and readblob's offset and
  -- size arguments.
  for _, n in ipairs({ 0, 1, 2, 3, 99, -1, -2, -99 }) do
    ans('s5/readmax/' .. tostring(n), "readfile('r_lf.txt', '', " .. n .. ')')
    ans('s5/readmaxb/' .. tostring(n), "readfile('r_lf.txt', 'b', " .. n .. ')')
  end
  for _, off in ipairs({ 0, 1, 5, -1, -3, 99 }) do
    ans('s5/blobo/' .. tostring(off), "readblob('r_lf.txt', " .. off .. ')')
    for _, sz in ipairs({ 0, 2, 99, -1 }) do
      ans(
        's5/blobos/' .. tostring(off) .. '/' .. tostring(sz),
        "readblob('r_lf.txt', " .. off .. ', ' .. sz .. ')'
      )
    end
  end

  -- writefile: every flag, then read the bytes back with uv so the
  -- assertion does not depend on the read half.
  local function wcase(tag, expr, file)
    ans('s5/write/' .. tag, expr)
    local st = uv.fs_stat(root .. '/' .. file)
    if st then
      local fd = uv.fs_open(root .. '/' .. file, 'r', 420)
      local body = fd and uv.fs_read(fd, st.size, 0) or ''
      if fd then
        uv.fs_close(fd)
      end
      emit('s5/write/' .. tag, 'A', esc(string.format('size=%d %s', st.size, body)))
      struct('s5/write/' .. tag .. '#A', { size = st.size, body = body })
    else
      emit('s5/write/' .. tag, 'A', '<absent>')
      struct('s5/write/' .. tag .. '#A', '<absent>')
    end
  end

  wcase('plain', "writefile(['a','b'], 'w_plain')", 'w_plain')
  wcase('empty', "writefile([], 'w_empty')", 'w_empty')
  wcase('emptystr', "writefile([''], 'w_emptystr')", 'w_emptystr')
  wcase('b', "writefile(['a','b'], 'w_b', 'b')", 'w_b')
  wcase('s', "writefile(['a'], 'w_s', 's')", 'w_s')
  wcase('S', "writefile(['a'], 'w_S', 'S')", 'w_S')
  wcase('a1', "writefile(['a'], 'w_a')", 'w_a')
  wcase('a2', "writefile(['b'], 'w_a', 'a')", 'w_a')
  wcase('ab', "writefile(['c'], 'w_a', 'ab')", 'w_a')
  wcase('nul', 'writefile(["x\\ny"], \'w_nul\')', 'w_nul')
  wcase('blob', "writefile(0z00010203, 'w_blob')", 'w_blob')
  wcase('blobb', "writefile(0z00010203, 'w_blobb', 'b')", 'w_blobb')
  wcase('utf8', "writefile(['\195\188\195\169'], 'w_utf8')", 'w_utf8')
  wcase('p', "writefile(['x'], 'wp/sub/w_p', 'p')", 'wp/sub/w_p')
  wcase('nop', "writefile(['x'], 'wq/sub/w_q')", 'wq/sub/w_q')
  wcase('overwrite-ro', "writefile(['x'], 'ro.txt')", 'ro.txt')
  wcase('todir', "writefile(['x'], 'dir1')", 'top.txt')
  wcase('stdout', "writefile(['x'], '/dev/null')", 'top.txt')

  -- The `D` flag registers a DEFERRED delete, which fires when the
  -- *function* that called writefile returns.  Ordering is observable
  -- and B18-13 has to preserve it, so the case is a real function.
  vim.cmd([[
    func! FsDeferred() abort
      call writefile(['x'], 'w_D', 'D')
      let g:fs_inside = filereadable('w_D')
    endfunc
  ]])
  ans('s5/write/D-call', 'FsDeferred()')
  ans('s5/write/D-inside', 'g:fs_inside')
  ans('s5/write/D-after', "filereadable('w_D')")
  vim.cmd([[
    func! FsDeferredErr() abort
      call writefile(['x'], 'w_D2', 'D')
      throw 'boom'
    endfunc
  ]])
  ans('s5/write/D2-call', 'FsDeferredErr()')
  ans('s5/write/D2-after', "filereadable('w_D2')")
  ans('s5/write/D-toplevel', "writefile(['x'], 'w_D3', 'D')")
  ans('s5/write/D-toplevel-after', "filereadable('w_D3')")

  -- Round trip: every read mode over what every write mode produced.
  for _, f in ipairs({ 'w_plain', 'w_b', 'w_s', 'w_nul', 'w_blob' }) do
    ans('s5/rt/' .. f, 'readfile(' .. vq(f) .. ')')
    ans('s5/rtb/' .. f, 'readfile(' .. vq(f) .. ", 'b')")
    ans('s5/rtblob/' .. f, 'readblob(' .. vq(f) .. ')')
  end

  report_tree('s5/tree', root)
end)

-- =====================================================================
-- s6 -- the mutating half.  dir.rs.
--
-- EACH CASE GETS ITS OWN NUMBERED SUBDIRECTORY with a freshly built
-- fixture, so that no case can observe another's leftovers and the
-- numbering cannot drift when one is inserted in the middle.
-- =====================================================================

section('s6-mutate', function()
  local n = 0
  --- One mutating case: fresh fixture, run the expressions in order,
  --- report each answer, then the resulting tree.
  local function mut(name, exprs)
    n = n + 1
    local tag = string.format('s6/%02d-%s', n, name)
    local root = fixture(string.format('s6/%02d', n))
    vim.cmd('cd ' .. vim.fn.fnameescape(root))
    for i, e in ipairs(exprs) do
      ans(string.format('%s/e%d', tag, i), (e:gsub('@', vq(root):sub(2, -2))))
    end
    report_tree(tag, root)
  end

  mut('delete-file', { "delete('top.txt')", "filereadable('top.txt')" })
  mut('delete-missing', { "delete('nosuch.txt')" })
  mut('delete-empty-dir', { "delete('dir2')", "delete('dir2', 'd')" })
  mut('delete-full-dir-d', { "delete('a', 'd')" })
  mut('delete-full-dir-rf', { "delete('a', 'rf')" })
  mut('delete-symlink', { "delete('link1')" })
  mut('delete-dirlink', { "delete('dlink')" })
  mut('delete-dirlink-rf', { "delete('dlink', 'rf')" })
  mut('delete-broken', { "delete('dangle')" })
  mut('delete-noperm', { "delete('noperm.txt')" })
  mut('delete-rf-file', { "delete('top.txt', 'rf')" })
  mut('delete-trailslash', { "delete('dir2/', 'd')" })
  mut('delete-dot-rf', { "delete('dir1/.', 'rf')" })
  mut('delete-relative-up', { "delete('a/b/../b/two.txt')" })

  mut('rename-file', { "rename('top.txt', 'renamed.txt')" })
  mut('rename-onto', { "rename('top.txt', 'note.txt')" })
  mut('rename-dir', { "rename('dir1', 'dir1x')" })
  mut('rename-into-dir', { "rename('top.txt', 'dir1/top.txt')" })
  mut('rename-missing', { "rename('nosuch', 'x')" })
  mut('rename-same', { "rename('top.txt', 'top.txt')" })
  mut('rename-to-missing-dir', { "rename('top.txt', 'nosuch/x')" })
  mut('rename-symlink', { "rename('link1', 'link1x')" })
  mut('rename-across', { "rename('a/b/two.txt', 'two.txt')" })

  mut('mkdir-plain', { "mkdir('new')" })
  mut('mkdir-existing', { "mkdir('dir1')" })
  mut('mkdir-existing-p', { "mkdir('dir1', 'p')" })
  mut('mkdir-nested-nop', { "mkdir('x/y/z')" })
  mut('mkdir-nested-p', { "mkdir('x/y/z', 'p')" })
  mut('mkdir-p-trailslash', { "mkdir('x/y/z/', 'p')" })
  mut('mkdir-prot-0700', { "mkdir('m700', '', 0700)" })
  mut('mkdir-prot-0777', { "mkdir('m777', '', 0777)" })
  mut('mkdir-p-prot', { "mkdir('p1/p2/p3', 'p', 0700)" })
  mut('mkdir-onto-file', { "mkdir('top.txt')" })
  mut('mkdir-onto-file-p', { "mkdir('top.txt', 'p')" })
  mut('mkdir-dots-p', { "mkdir('a/../mm', 'p')" })
  mut('mkdir-R', {
    "mkdir('rr', 'R')",
    "isdirectory('rr')",
  })
  mut('mkdir-D-file', { "mkdir('dd', 'D')", "isdirectory('dd')" })

  mut('filecopy-plain', { "filecopy('top.txt', 'copy.txt')" })
  mut('filecopy-onto', { "filecopy('top.txt', 'note.txt')" })
  mut('filecopy-perm', { "filecopy('exec.sh', 'exec2.sh')" })
  mut('filecopy-ro', { "filecopy('ro.txt', 'ro2.txt')" })
  mut('filecopy-missing', { "filecopy('nosuch', 'x')" })
  mut('filecopy-to-dir', { "filecopy('top.txt', 'dir1')" })
  mut('filecopy-dir-src', { "filecopy('dir1', 'dirc')" })
  mut('filecopy-symlink', { "filecopy('link1', 'linkc')" })
  mut('filecopy-broken', { "filecopy('dangle', 'danglec')" })
  mut('filecopy-empty', { "filecopy('empty.txt', 'emptyc.txt')" })
  mut('filecopy-noperm', { "filecopy('noperm.txt', 'npc.txt')" })
  mut('filecopy-self', { "filecopy('top.txt', 'top.txt')" })
  mut('filecopy-to-missing-dir', { "filecopy('top.txt', 'nosuch/x')" })

  -- Sequences: the ordering between two mutations is its own answer.
  mut('seq-mkdir-write-delete', {
    "mkdir('s/t', 'p')",
    "writefile(['x'], 's/t/f')",
    "delete('s', 'rf')",
    "isdirectory('s')",
  })
  mut('seq-copy-rename-delete', {
    "filecopy('top.txt', 'c1.txt')",
    "rename('c1.txt', 'c2.txt')",
    "delete('c2.txt')",
  })
  mut('seq-write-p-delete-rf', {
    "writefile(['x'], 'wd/e/f/g.txt', 'p')",
    "delete('wd', 'rf')",
  })

  -- setfperm/getfperm round trip lives here because it mutates.
  mut('setfperm', {
    "setfperm('top.txt', 'rwx------')",
    "getfperm('top.txt')",
    "setfperm('top.txt', 'r--r--r--')",
    "getfperm('top.txt')",
    "setfperm('top.txt', 'bogus')",
    "setfperm('nosuch', 'rwxrwxrwx')",
  })
end)

-- =====================================================================
-- s7 -- getcwd / chdir / haslocaldir across the three scopes.
-- =====================================================================

section('s7-cwd', function()
  local root = fixture('s7')
  mkdirp(root .. '/g1')
  mkdirp(root .. '/g2')
  mkdirp(root .. '/g3')
  vim.cmd('cd ' .. vim.fn.fnameescape(root))

  --- The whole scope matrix at one moment: the answer is nine numbers'
  --- worth of state, and reporting fewer hides exactly the bug where a
  --- window-local directory leaks into the tab.
  local function snap(tag)
    local parts = {}
    for _, args in ipairs({
      '',
      '0',
      '-1',
      '0, 0',
      '1, 0',
      '0, 1',
      '1, 1',
      '-1, 0',
      '0, -1',
      '-1, -1',
      '2, 1',
      '1, 2',
    }) do
      local ok, v = pcall(vim.api.nvim_eval, 'getcwd(' .. args .. ')')
      parts[#parts + 1] = 'getcwd(' .. args .. ')=' .. (ok and tostring(v) or ('!' .. errtext(v)))
      local ok2, v2 = pcall(vim.api.nvim_eval, 'haslocaldir(' .. args .. ')')
      parts[#parts + 1] = 'hld(' .. args .. ')=' .. (ok2 and tostring(v2) or ('!' .. errtext(v2)))
    end
    parts[#parts + 1] = 'tabs=' .. tostring(vim.fn.tabpagenr('$'))
    parts[#parts + 1] = 'wins=' .. tostring(vim.fn.winnr('$'))
    emit(tag, 'A', esc(scrub(table.concat(parts, ' '))))
    struct(tag .. '#A', scrub(table.concat(parts, ' ')))
  end

  snap('s7/00-start')
  ans('s7/chdir-g1', 'chdir(' .. vq(root .. '/g1') .. ')')
  snap('s7/01-after-chdir')
  ans('s7/chdir-back', 'chdir(' .. vq(root) .. ')')
  ans('s7/chdir-rel', "chdir('g2')")
  snap('s7/02-after-rel')
  ans('s7/chdir-missing', "chdir('nosuchdir')")
  snap('s7/03-after-missing')
  ans('s7/chdir-empty', "chdir('')")
  ans('s7/chdir-file', 'chdir(' .. vq(root .. '/top.txt') .. ')')
  snap('s7/04-after-bad')
  ans('s7/chdir-tilde', "chdir('~')")
  snap('s7/05-after-tilde')

  vim.cmd('cd ' .. vim.fn.fnameescape(root))
  vim.cmd('split')
  ans('s7/lcd', 'execute("lcd ' .. root .. '/g1")')
  snap('s7/06-window-local')
  ans('s7/lcd-chdir', "chdir('" .. root .. "/g2')")
  snap('s7/07-window-chdir')
  vim.cmd('wincmd w')
  snap('s7/08-other-window')
  vim.cmd('only')
  snap('s7/09-after-only')

  vim.cmd('cd ' .. vim.fn.fnameescape(root))
  vim.cmd('tabnew')
  ans('s7/tcd', 'execute("tcd ' .. root .. '/g3")')
  snap('s7/10-tab-local')
  vim.cmd('split')
  snap('s7/11-tab-split')
  ans('s7/tab-lcd', 'execute("lcd ' .. root .. '/g1")')
  snap('s7/12-tab-window-local')
  vim.cmd('tabnext')
  snap('s7/13-first-tab')
  vim.cmd('tabnext')
  ans('s7/tab-cd-global', 'execute("cd ' .. root .. '/g2")')
  snap('s7/14-after-global-cd')
  vim.cmd('tabonly')
  vim.cmd('only')
  snap('s7/15-collapsed')

  -- getcwd/haslocaldir with out-of-range and negative window/tab numbers
  -- -- the arithmetic the rewrite must not simplify away.
  for _, args in ipairs({
    '99',
    '-99',
    '0, 99',
    '99, 0',
    '-1, 99',
    '2147483647',
    '-2147483648',
    '2147483647, 2147483647',
  }) do
    ans('s7/getcwd/' .. args:gsub('[^%w-]', '_'), 'getcwd(' .. args .. ')')
    ans('s7/hld/' .. args:gsub('[^%w-]', '_'), 'haslocaldir(' .. args .. ')')
  end

  -- tempname(): shape only, and the fact that it advances.
  ans('s7/tempname/1', 'tempname()')
  ans('s7/tempname/2', 'tempname()')
  ans('s7/tempname/differ', 'tempname() !=# tempname()')
  ans('s7/tempname/under-tmpdir', "tempname()[0:len($TMPDIR)-1] ==# $TMPDIR")
end)

-- =====================================================================
-- s8 -- the stat family + executable/exepath.
-- =====================================================================

section('s8-stat', function()
  local root = fixture('s8')
  vim.cmd('cd ' .. vim.fn.fnameescape(root))

  local TARGETS = {
    { 'top', 'top.txt' },
    { 'empty', 'empty.txt' },
    { 'two', 'a/b/two.txt' },
    { 'deep', 'a/b/c/deep.txt' },
    { 'dir', 'dir1' },
    { 'dirtrail', 'dir1/' },
    { 'emptydir', 'dir2' },
    { 'link', 'link1' },
    { 'dirlink', 'dlink' },
    { 'dirlinktrail', 'dlink/' },
    { 'broken', 'dangle' },
    { 'loop', 'loopa' },
    { 'ro', 'ro.txt' },
    { 'exec', 'exec.sh' },
    { 'noperm', 'noperm.txt' },
    { 'nosuch', 'nosuch.txt' },
    { 'spaced', 'spa ced.txt' },
    { 'utf8', '\195\188n\195\175c\195\184d\195\169.txt' },
    { 'abs', root .. '/top.txt' },
    { 'dot', '.' },
    { 'dotdot', '..' },
    { 'root', '/' },
    { 'empty-arg', '' },
    { 'devnull', '/dev/null' },
    { 'proc', '/proc/self/status' },
  }
  for _, t in ipairs(TARGETS) do
    local p = vq(t[2])
    ans('s8/getfsize/' .. t[1], 'getfsize(' .. p .. ')')
    ans('s8/getftype/' .. t[1], 'getftype(' .. p .. ')')
    ans('s8/getfperm/' .. t[1], 'getfperm(' .. p .. ')')
    ans('s8/filereadable/' .. t[1], 'filereadable(' .. p .. ')')
    ans('s8/filewritable/' .. t[1], 'filewritable(' .. p .. ')')
    ans('s8/isdirectory/' .. t[1], 'isdirectory(' .. p .. ')')
    -- The VALUE of getftime is the clock.  Only its sign is an answer
    -- here; the ordering question is asked below against known mtimes.
    ans('s8/getftime-sign/' .. t[1], 'getftime(' .. p .. ') > 0')
  end

  -- getftime ORDERING, against three explicitly stamped mtimes.  Never
  -- the clock: the values differ per run by construction.
  writebytes(root .. '/t1', 'a')
  writebytes(root .. '/t2', 'b')
  writebytes(root .. '/t3', 'c')
  uv.fs_utime(root .. '/t1', 1000000000, 1000000000)
  uv.fs_utime(root .. '/t2', 1000000060, 1000000060)
  uv.fs_utime(root .. '/t3', 1000000120, 1000000120)
  ans('s8/getftime/order12', "getftime('t1') < getftime('t2')")
  ans('s8/getftime/order23', "getftime('t2') < getftime('t3')")
  ans('s8/getftime/delta12', "getftime('t2') - getftime('t1')")
  ans('s8/getftime/delta23', "getftime('t3') - getftime('t2')")
  ans('s8/getftime/missing', "getftime('nosuch')")
  ans('s8/getftime/empty', "getftime('')")
  ans('s8/getftime/dir-positive', "getftime('.') > 0")

  -- executable / exepath, against the pinned $PATH fixture.
  local EXE = {
    'fixexe',
    'fixnoexe',
    'fixdir',
    'fixlink',
    'nvim',
    'nosuchcommand',
    '',
    './fixexe',
    'exec.sh',
    './exec.sh',
    'a/one.txt',
    './top.txt',
    '/bin/sh',
    '/nosuch/sh',
    'sh',
    'ls',
    '~/nosuch',
    'fix exe',
  }
  for _, e in ipairs(EXE) do
    local tag = e == '' and 'empty' or e:gsub('[^%w]', '_')
    ans('s8/executable/' .. tag, 'executable(' .. vq(e) .. ')')
    ans('s8/exepath/' .. tag, 'exepath(' .. vq(e) .. ')')
  end

  -- browse/browsedir have no GUI here; that they answer '' rather than
  -- raising is the property.
  ans('s8/browse/save', "browse(1, 'title', " .. vq(root) .. ", 'name')")
  ans('s8/browse/open', "browse(0, 'title', " .. vq(root) .. ", '')")
  ans('s8/browsedir', "browsedir('title', " .. vq(root) .. ')')
end)

-- =====================================================================
-- s9 -- the error-text sweep.  Every builtin, wrong arity and wrong
-- types.  The error TEXT is the artifact: a rewrite that changes an
-- E-number or a message is a user-visible change.
-- =====================================================================

section('s9-errors', function()
  local root = fixture('s9')
  vim.cmd('cd ' .. vim.fn.fnameescape(root))

  local FNS = {
    'browse',
    'browsedir',
    'chdir',
    'delete',
    'executable',
    'exepath',
    'filecopy',
    'filereadable',
    'filewritable',
    'finddir',
    'findfile',
    'fnamemodify',
    'getcwd',
    'getfperm',
    'getfsize',
    'getftime',
    'getftype',
    'glob',
    'glob2regpat',
    'globpath',
    'haslocaldir',
    'isabsolutepath',
    'isdirectory',
    'mkdir',
    'pathshorten',
    'readblob',
    'readdir',
    'readfile',
    'rename',
    'resolve',
    'simplify',
    'tempname',
    'writefile',
  }

  -- Arity.  Zero args, and eight -- both ends of the check.
  for _, fn in ipairs(FNS) do
    ans('s9/arity0/' .. fn, fn .. '()')
    ans('s9/arity8/' .. fn, fn .. "(1,2,3,4,5,6,7,8)")
  end

  -- Types.  Every argument slot that takes a string, given each of the
  -- non-string types the evaluator can produce.
  local BAD = {
    { 'list', '[1,2]' },
    { 'dict', "{'a': 1}" },
    { 'func', "function('tr')" },
    { 'blob', '0z00' },
    { 'float', '1.5' },
    { 'null', 'v:null' },
    { 'bool', 'v:true' },
    { 'number', '42' },
  }
  local ONEARG = {
    'chdir',
    'executable',
    'exepath',
    'filereadable',
    'filewritable',
    'getfperm',
    'getfsize',
    'getftime',
    'getftype',
    'glob',
    'glob2regpat',
    'isabsolutepath',
    'isdirectory',
    'mkdir',
    'pathshorten',
    'readblob',
    'readdir',
    'readfile',
    'resolve',
    'simplify',
  }
  for _, fn in ipairs(ONEARG) do
    for _, b in ipairs(BAD) do
      ans('s9/type1/' .. fn .. '/' .. b[1], fn .. '(' .. b[2] .. ')')
    end
  end

  local TWOARG = {
    { 'delete', "'x'" },
    { 'filecopy', "'x'" },
    { 'finddir', "'x'" },
    { 'findfile', "'x'" },
    { 'fnamemodify', "'x'" },
    { 'globpath', "'x'" },
    { 'rename', "'x'" },
    { 'writefile', "['x']" },
    { 'pathshorten', "'x'" },
    { 'readdir', "'x'" },
    { 'readfile', "'x'" },
    { 'readblob', "'x'" },
    { 'glob', "'x'" },
    { 'mkdir', "'x'" },
  }
  for _, c in ipairs(TWOARG) do
    for _, b in ipairs(BAD) do
      ans('s9/type2/' .. c[1] .. '/' .. b[1], c[1] .. '(' .. c[2] .. ', ' .. b[2] .. ')')
      ans('s9/typeboth/' .. c[1] .. '/' .. b[1], c[1] .. '(' .. b[2] .. ', ' .. b[2] .. ')')
    end
  end

  -- writefile's first argument is a List or Blob, not a String, and its
  -- items have to be Strings: a different check on a different arm.
  for _, b in ipairs(BAD) do
    ans('s9/writefile/items/' .. b[1], "writefile([" .. b[2] .. "], 'w9')")
  end
  ans('s9/writefile/nested', "writefile([['a']], 'w9')")
  ans('s9/writefile/dictarg', "writefile({'a': 'b'}, 'w9')")
  ans('s9/writefile/stringarg', "writefile('ab', 'w9')")

  -- Flag strings that are not flags.
  for _, f in ipairs({ 'z', 'bz', 'ba', 'aa', 'bb', 'DD', '', ' ', 'B' }) do
    ans('s9/writeflag/' .. (f == '' and 'empty' or f:gsub('%s', '_sp')), "writefile(['x'], 'w9f', " .. vq(f) .. ')')
    ans('s9/deleteflag/' .. (f == '' and 'empty' or f:gsub('%s', '_sp')), "delete('dir2', " .. vq(f) .. ')')
    ans('s9/mkdirflag/' .. (f == '' and 'empty' or f:gsub('%s', '_sp')), "mkdir('m9' . " .. vq(f) .. ', ' .. vq(f) .. ')')
  end

  -- Numeric arguments out of range.
  ans('s9/mkdir/prot-string', "mkdir('m9s', 'p', 'rwx')")
  ans('s9/readfile/max-float', "readfile('top.txt', '', 1.5)")
  ans('s9/readblob/off-float', "readblob('top.txt', 1.5)")
  ans('s9/findfile/count-float', "findfile('top.txt', '', 1.5)")
  ans('s9/getcwd/float', 'getcwd(1.5)')
  ans('s9/haslocaldir/float', 'haslocaldir(1.5)')
  ans('s9/glob/nosuf-string', "glob('*', 'yes')")
  ans('s9/glob/list-string', "glob('*', v:false, 'yes')")
end)

-- =====================================================================
-- s90 -- the uncaptured block that fills .stderr.
--
-- NOT `verbose=0`: batch mode starts 'verbose' at 1 and that is what
-- routes nvim's own messages to stderr at all.  Zero silences the lot
-- and leaves this artifact empty while every other one looks healthy.
-- =====================================================================

section('s90-messages', function()
  local root = fixture('s90')
  vim.cmd('cd ' .. vim.fn.fnameescape(root))

  -- THE ERRORS HAVE TO RUN IN A CHILD.  In-process, a Vimscript error
  -- inside `vim.cmd` is converted to a Lua error, so `pcall` swallows it
  -- and nvim never *displays* anything: the whole artifact comes out
  -- empty while every other one looks healthy.  `-c` in a child is the
  -- only spelling under which nvim prints the message and keeps going.
  local CMDS = {
    'call delete("nosuchfile-message")',
    'call rename("nosuchfile-message", "x")',
    'call mkdir("top.txt")',
    'call mkdir("")',
    'call mkdir("dir1/../top.txt/sub", "p")',
    'call filecopy("nosuch", "x")',
    'call readfile("nosuchfile-message")',
    'call readfile("dir1")',
    'call readblob("nosuchfile-message")',
    'call writefile(["x"], "dir1")',
    'call writefile(["x"], "nosuchdir/x")',
    'call readdir("nosuchdir-message")',
    'call chdir("nosuchdir-message")',
    'cd nosuchdir-message',
    'lcd nosuchdir-message',
    'tcd nosuchdir-message',
    'echo fnamemodify("x", ":s?a?")',
    'echo glob2regpat([])',
    'echo pathshorten([])',
    'echo resolve([])',
    'echo simplify([])',
    'echo getcwd(1, 1, 1, 1)',
    'echo writefile(["x"], "w90", "zz")',
    'echo delete("dir1", "zz")',
    'echo mkdir("m90", "zz")',
    'echo finddir([])',
    'echo findfile([])',
    'echo globpath([], [])',
    'echo executable([])',
    'echo exepath([])',
    'echo getfperm([])',
    'echo isdirectory([])',
    'echo readdir([])',
    'echo writefile("notalist", "w90b")',
    'echo tempname(1)',
  }
  if not ROOT0 then
    CMDS[#CMDS + 1] = 'call readfile("noperm.txt")'
    CMDS[#CMDS + 1] = 'call writefile(["x"], "noperm.txt")'
    CMDS[#CMDS + 1] = 'echo getfperm("noperm.txt")'
  end

  -- CHUNKED AT SIX.  nvim caps `+command`/`-c`/`--cmd` at TEN in total
  -- and answers `Too many "+command" ... arguments` on the eleventh --
  -- which it writes to stderr, so a single over-long child looks like a
  -- healthy artifact with one odd line in it.
  local codes = {}
  local at = 1
  while at <= #CMDS do
    local args = { work .. '/bin/nvim', '--headless', '-u', 'NONE', '-i', 'NONE' }
    args[#args + 1] = '--cmd'
    args[#args + 1] = 'cd ' .. root
    local upto = math.min(at + 5, #CMDS)
    for i = at, upto do
      args[#args + 1] = '-c'
      args[#args + 1] = CMDS[i]
    end
    args[#args + 1] = '-c'
    args[#args + 1] = 'qa!'
    local res = vim
      .system(args, {
        text = true,
        cwd = root,
        env = {
          HOME = work .. '/home',
          PATH = work .. '/bin',
          TMPDIR = work .. '/tmp',
          TERM = 'dumb',
          SHELL = '/bin/sh',
          LANG = 'C.UTF-8',
          VIMRUNTIME = os.getenv('VIMRUNTIME') or '',
          NVIM_TEST = '1',
        },
        clear_env = true,
        timeout = 60000,
      })
      :wait()
    -- The leading newline is not cosmetic: the parent's own emsg output
    -- is written without a trailing one, and the child's first line
    -- would otherwise be glued to it.
    io.stderr:write('\n-- s90 chunk ' .. at .. '\n' .. scrub(res.stderr or ''))
    codes[#codes + 1] = tostring(res.code)
    at = upto + 1
  end

  -- s90's own stdout row.  Without one this section reads `rows=0`,
  -- which is indistinguishable from a section that died.
  emit('s90', 'A', string.format('cmds=%d codes=%s', #CMDS, table.concat(codes, ',')))
  struct('s90#A', { cmds = #CMDS, codes = codes })
end)

-- =====================================================================
-- s91 -- CRASHPROBE.  One child per input.
-- =====================================================================

section('s91-crashprobe', function()
  local progpath = work .. '/bin/nvim'
  local env = {
    HOME = work .. '/home',
    PATH = work .. '/bin',
    TMPDIR = work .. '/tmp',
    TERM = 'dumb',
    SHELL = '/bin/sh',
    LANG = 'C.UTF-8',
    VIMRUNTIME = os.getenv('VIMRUNTIME') or '',
    NVIM_TEST = '1',
    -- The children re-run this same file, and it asserts on this.
    FS_WORK = work,
  }
  local i, guard = 1, 0
  local aborted = 0
  while i <= #CRASH and guard < #CRASH + 40 do
    guard = guard + 1
    local res = vim
      .system({
        progpath,
        '--headless',
        '-u',
        'NONE',
        '-i',
        'NONE',
        '-l',
        script,
        '--crash',
        tostring(i),
        -- A safety net, not the assertion: every input here is meant to
        -- finish or die.  A child that hits the timeout is killed and
        -- reads as ABORTED, so anything that *wedges* has to be trimmed
        -- out of the corpus instead.
      }, { text = true, cwd = work, env = env, clear_env = true, timeout = 120000 })
      :wait()
    local last = i - 1
    for line in (res.stdout or ''):gmatch('[^\n]+') do
      local idx = tonumber(line:match('^(%d+) '))
      if idx then
        last = idx
        emit((line:gsub('^%d+ ', '')))
      end
    end
    -- vim.system reports an abort as a SIGNAL and leaves `code` at 0, so
    -- a code-only check calls a SIGABRT a success.
    local died = (res.signal or 0) ~= 0 or (res.code or 0) ~= 0
    if not died and last >= #CRASH then
      break
    end
    if died then
      local victim = CRASH[last + 1]
      if victim then
        aborted = aborted + 1
        emit(
          'k91/' .. victim[1],
          'X',
          string.format('ABORTED code=%s signal=%s', tostring(res.code), tostring(res.signal))
        )
      end
      last = last + 1
    end
    i = last + 1
  end
  emit('k91', 'groups', string.format('cases=%d aborted=%d', #CRASH, aborted))
  struct('k91#groups', { cases = #CRASH, aborted = aborted })
end)

if structfd then
  structfd:close()
end
emit('## done')
