-- The tree-sitter binding's own surface, case by case: what the Lua side of
-- `lua/treesitter/` hands back for an edited tree, a depth-limited query, a
-- language's metadata, a parse that runs out of time, a logger, and the exact
-- text of the errors it raises. The other specs here reach most of this only
-- through `LanguageTree`; these call the userdata methods directly so a
-- rewrite of the binding is checked against the values, not the highlighting
-- built on them.

local t = require('test.testutil')
local n = require('test.functional.testnvim')()

local clear = n.clear
local eq = t.eq
local exec_lua = n.exec_lua
local pcall_err = t.pcall_err
local assert_alive = n.assert_alive
local NIL = vim.NIL

before_each(function()
  clear()
  exec_lua(function()
    vim.treesitter.language.add('c')
  end)
end)

describe('treesitter tree:edit()', function()
  --- Parses `before`, applies `edit` (tree:edit()'s nine arguments) and
  --- re-parses `after` against the edited tree.
  local function reparse(before, after, edit)
    return exec_lua(function()
      local parser = vim._create_ts_parser('c')
      local old = parser:parse(nil, before, true)
      local edited = old:edit(unpack(edit))
      local new, changes = parser:parse(edited, after, true)
      local _, short = vim._create_ts_parser('c'):parse(edited, after, false)
      return {
        changes = changes,
        short = short,
        old = old:root():sexpr(),
        new = new:root():sexpr(),
      }
    end)
  end

  local src = 'int a = 1;\nint b = 2;\n'
  local decl = '(declaration type: (primitive_type) declarator: '
    .. '(init_declarator declarator: (identifier) value: (number_literal)))'

  it('reports the inserted declaration at a line start', function()
    local r = reparse(src, 'int a = 1;\nlong x;\nint b = 2;\n', { 11, 11, 19, 1, 0, 1, 0, 2, 0 })
    eq(('(translation_unit %s %s)'):format(decl, decl), r.old)
    eq(
      ('(translation_unit %s %s %s)'):format(
        decl,
        '(declaration type: (sized_type_specifier) declarator: (identifier))',
        decl
      ),
      r.new
    )
    eq({ { 1, 0, 11, 1, 7, 18 } }, r.changes)
    -- Without `include_bytes` the same range comes back as four points.
    eq({ { 1, 0, 1, 7 } }, r.short)
  end)

  it('reports a mid-line replacement', function()
    local r = reparse(src, 'int a = 12+3;\nint b = 2;\n', { 8, 9, 12, 0, 8, 0, 9, 0, 12 })
    eq({ { 0, 8, 8, 0, 12, 12 } }, r.changes)
    eq(
      '(translation_unit (declaration type: (primitive_type) declarator: '
        .. '(init_declarator declarator: (identifier) value: (binary_expression '
        .. 'left: (number_literal) right: (number_literal)))) '
        .. decl
        .. ')',
      r.new
    )
  end)

  it('reports a replacement across lines', function()
    -- `1;\nint b = 2` -> `f(x,\ny)`: one declaration becomes a call initialiser
    -- spanning two lines.
    local r = reparse(src, 'int a = f(x,\ny);\n', { 8, 20, 15, 0, 8, 1, 9, 1, 2 })
    eq({ { 0, 8, 8, 1, 2, 15 } }, r.changes)
    eq(
      '(translation_unit (declaration type: (primitive_type) declarator: '
        .. '(init_declarator declarator: (identifier) value: (call_expression '
        .. 'function: (identifier) arguments: (argument_list (identifier) (identifier))))))',
      r.new
    )
  end)

  it('reports no syntactic change for a deletion that leaves a well-formed tree', function()
    local r = reparse(src, 'int a = 2;\n', { 8, 19, 8, 0, 8, 1, 8, 0, 8 })
    eq({}, r.changes)
    eq(('(translation_unit %s)'):format(decl), r.new)
  end)

  it('reports nothing for a re-parse without an edit', function()
    eq(
      {},
      exec_lua(function()
        local parser = vim._create_ts_parser('c')
        local tree = parser:parse(nil, src, true)
        local _, changes = parser:parse(tree, src, true)
        return changes
      end)
    )
  end)

  it('leaves the edited tree a copy', function()
    eq(
      { true, 6, 7 },
      exec_lua(function()
        local parser = vim._create_ts_parser('c')
        local old = parser:parse(nil, 'int a;', true)
        local edited = old:edit(4, 5, 6, 0, 4, 0, 5, 0, 6)
        return { old ~= edited, select(3, old:root():end_()), select(3, edited:root():end_()) }
      end)
    )
  end)

  it('refuses too few arguments', function()
    eq(
      'not enough args to tree:edit()',
      exec_lua(function()
        local tree = vim._create_ts_parser('c'):parse(nil, 'int a;', true)
        local _, err = pcall(tree.edit, tree, 1, 2)
        return err
      end)
    )
  end)

  it('drives changedtree ranges from buffer edits', function()
    local changes = exec_lua(function()
      vim.api.nvim_buf_set_lines(0, 0, -1, false, { 'int a = 1;', 'int b = 2;' })
      local parser = vim.treesitter.get_parser(0, 'c')
      parser:parse()
      local seen = {}
      parser:register_cbs({
        on_changedtree = function(ranges)
          for _, r in ipairs(ranges) do
            seen[#seen + 1] = r
          end
        end,
      })
      vim.api.nvim_buf_set_text(0, 0, 8, 0, 9, { '12+3' })
      parser:parse()
      vim.api.nvim_buf_set_lines(0, 1, 1, false, { 'long x;' })
      parser:parse()
      return seen
    end)
    eq({ { 0, 8, 8, 0, 12, 12 }, { 1, 0, 14, 1, 7, 21 } }, changes)
  end)
end)

describe('treesitter query max_start_depth', function()
  local src = 'int f(void) {\n  int a = b + c;\n  if (x) {\n    y = z;\n  }\n}\n'

  --- The identifiers `iter_captures` and the match count `iter_matches`
  --- answer for `(identifier) @id` at each depth.
  local function at_depths(depths)
    return exec_lua(function()
      local parser = vim._create_ts_parser('c')
      local root = parser:parse(nil, src, true):root()
      local query = vim.treesitter.query.parse('c', '(identifier) @id')
      local out = {}
      for _, depth in ipairs(depths) do
        local opts = { max_start_depth = depth >= 0 and depth or nil }
        local names = {}
        for _, node in query:iter_captures(root, src, 0, -1, opts) do
          names[#names + 1] = vim.treesitter.get_node_text(node, src)
        end
        local matches = 0
        for _ in query:iter_matches(root, src, 0, -1, opts) do
          matches = matches + 1
        end
        out[#out + 1] = { table.concat(names, ','), matches }
      end
      return out
    end)
  end

  it('limits captures by the depth their pattern starts at', function()
    -- Depth counts from the node the query runs on: f is at 3
    -- (function_definition > function_declarator > identifier), a at 5, the
    -- operands b and c at 6, y and z inside the `if` at 7.
    eq({
      { '', 0 },
      { '', 0 },
      { '', 0 },
      { 'f', 1 },
      { 'f', 1 },
      { 'f,a,x', 3 },
      { 'f,a,b,c,x', 5 },
      { 'f,a,b,c,x,y,z', 7 },
      { 'f,a,b,c,x,y,z', 7 },
      { 'f,a,b,c,x,y,z', 7 },
    }, at_depths({ 0, 1, 2, 3, 4, 5, 6, 7, 8, 100 }))
  end)

  it('is unlimited when unset', function()
    eq({ { 'f,a,b,c,x,y,z', 7 } }, at_depths({ -1 }))
  end)

  it('passes 0 through rather than meaning unlimited', function()
    -- `max_start_depth = 0` reaches `ts_query_cursor_set_max_start_depth(0)`:
    -- only a pattern rooted at the start node itself can match.
    eq(
      { 'translation_unit', '' },
      exec_lua(function()
        local root = vim._create_ts_parser('c'):parse(nil, src, true):root()
        local out = {}
        for _, q in ipairs({ '(translation_unit) @t', '(function_definition) @f' }) do
          local query = vim.treesitter.query.parse('c', q)
          local types = {}
          for _, node in query:iter_captures(root, src, 0, -1, { max_start_depth = 0 }) do
            types[#types + 1] = node:type()
          end
          out[#out + 1] = table.concat(types, ',')
        end
        return out
      end)
    )
  end)

  it('counts from a nested start node', function()
    eq(
      { '', '', 'a,x' },
      exec_lua(function()
        local root = vim._create_ts_parser('c'):parse(nil, src, true):root()
        local body = root:child(0):field('body')[1]
        local query = vim.treesitter.query.parse('c', '(identifier) @id')
        local out = {}
        for _, depth in ipairs({ 1, 2, 3 }) do
          local names = {}
          for _, node in query:iter_captures(body, src, 0, -1, { max_start_depth = depth }) do
            names[#names + 1] = vim.treesitter.get_node_text(node, src)
          end
          out[#out + 1] = table.concat(names, ',')
        end
        return out
      end)
    )
  end)
end)

describe('treesitter language metadata', function()
  it('reports the version triple of an ABI 15 grammar', function()
    local res = exec_lua(function()
      local out = {}
      for _, lang in ipairs({ 'c', 'lua', 'vim', 'vimdoc', 'query', 'markdown' }) do
        local info = vim.treesitter.language.inspect(lang)
        local keys = vim.tbl_keys(info.metadata or {})
        table.sort(keys)
        local integral = true
        for _, k in ipairs(keys) do
          local v = info.metadata[k]
          integral = integral and type(v) == 'number' and v >= 0 and math.floor(v) == v
        end
        out[lang] = {
          abi = info.abi_version,
          keys = keys,
          integral = integral,
          states = info.state_count > 0,
          supertypes = type(info.supertypes),
        }
      end
      return out
    end)
    for lang, info in pairs(res) do
      -- `metadata` is present exactly when the grammar is ABI 15.
      eq(15, info.abi, lang)
      eq({ 'major_version', 'minor_version', 'patch_version' }, info.keys, lang)
      eq(true, info.integral, lang)
      eq(true, info.states, lang)
      eq('table', info.supertypes, lang)
    end
  end)

  it('matches the bundled C grammar', function()
    -- nix/deps.json pins tree-sitter-c v0.24.1; a bump changes this triple.
    eq(
      { major_version = 0, minor_version = 24, patch_version = 1 },
      exec_lua(function()
        return vim.treesitter.language.inspect('c').metadata
      end)
    )
  end)
end)

describe('treesitter parse timeout', function()
  before_each(function()
    exec_lua(function()
      local lines = {}
      for i = 1, 5000 do
        lines[i] = ('int v%d = %d + f(%d, "s");'):format(i, i, i)
      end
      _G.buf = vim.api.nvim_create_buf(false, true)
      vim.api.nvim_buf_set_lines(_G.buf, 0, -1, false, lines)
    end)
  end)

  it('cancels a parse whose progress callback sees the deadline passed', function()
    -- A 1 ns budget has always expired at the first progress check, so the
    -- parse stops there and answers nothing; each call resumes where the last
    -- stopped, and enough of them finish the tree.
    eq(
      { 0, true, 5000 },
      exec_lua(function()
        local parser = vim._create_ts_parser('c')
        local first = select('#', parser:parse(nil, _G.buf, true, 1))
        local calls, tree = 1, nil
        repeat
          calls = calls + 1
          tree = parser:parse(nil, _G.buf, true, 1)
        until tree or calls > 100000
        return { first, calls > 2, tree:root():named_child_count() }
      end)
    )
  end)

  it('finishes within a generous budget and returns the included ranges', function()
    -- With no old tree the second result is the tree's included ranges: the
    -- whole source, ending at UINT32_MAX.
    local max = 4294967295
    eq(
      { 5000, { { 0, 0, 0, max, max, max } } },
      exec_lua(function()
        local tree, changes = vim._create_ts_parser('c'):parse(nil, _G.buf, true, 1e12)
        return { tree:root():named_child_count(), changes }
      end)
    )
  end)

  it('treats an explicit nil timeout as none', function()
    eq(
      5000,
      exec_lua(function()
        return vim._create_ts_parser('c'):parse(nil, _G.buf, true, nil):root():named_child_count()
      end)
    )
  end)

  it('treats an absent timeout argument as zero', function()
    -- `lua_isnil` is false for a missing argument, so `lua_tointeger` reads 0
    -- and the parse is cancelled at once (upstream's binding does the same;
    -- LanguageTree always passes the fifth argument).
    eq(
      0,
      exec_lua(function()
        return select('#', vim._create_ts_parser('c'):parse(nil, _G.buf, true))
      end)
    )
  end)

  it('starts over after reset()', function()
    eq(
      { 0, 5000 },
      exec_lua(function()
        local parser = vim._create_ts_parser('c')
        local first = select('#', parser:parse(nil, _G.buf, true, 1))
        parser:reset()
        return { first, parser:parse(nil, _G.buf, true, nil):root():named_child_count() }
      end)
    )
  end)

  it('lets LanguageTree finish an async parse it had to yield', function()
    -- Five thousand lines take far more than the 3 ms async budget, so
    -- `parse()` returns nil and the callback gets the tree later.
    eq(
      { true, { NIL, 5000 } },
      exec_lua(function()
        vim.bo[_G.buf].filetype = 'c'
        local parser = vim.treesitter.get_parser(_G.buf, 'c')
        local got
        local sync = parser:parse(true, function(err, trees)
          got = { err, trees and trees[1]:root():named_child_count() }
        end)
        vim.wait(10000, function()
          return got ~= nil
        end)
        return { sync == nil, got }
      end)
    )
  end)

  it('parses synchronously under _ts_force_sync_parsing', function()
    eq(
      { 5000, { NIL, 5000 } },
      exec_lua(function()
        vim.g._ts_force_sync_parsing = true
        local parser = vim.treesitter.get_parser(_G.buf, 'c')
        local got
        local trees = parser:parse(true, function(err, trees)
          got = { err, trees[1]:root():named_child_count() }
        end)
        return { trees[1]:root():named_child_count(), got }
      end)
    )
  end)
end)

describe('treesitter parser logger', function()
  it('round-trips the callback and filters by log type', function()
    eq(
      { true, true, false, true, 'nil' },
      exec_lua(function()
        local parser = vim._create_ts_parser('c')
        local seen = {}
        local function cb(kind)
          seen[kind] = true
        end
        local before = parser:_logger()
        parser:_set_logger(false, true, cb)
        local same = parser:_logger() == cb
        parser:parse(nil, 'int a;', true)
        return { same, seen.parse == true, seen.lex == true, before == nil, type(before) }
      end)
    )
  end)

  it('logs lexing when asked', function()
    eq(
      { true, false },
      exec_lua(function()
        local parser = vim._create_ts_parser('c')
        local seen = {}
        parser:_set_logger(true, false, function(kind)
          seen[kind] = true
        end)
        parser:parse(nil, 'int a;', true)
        return { seen.lex == true, seen.parse == true }
      end)
    )
  end)

  it('checks its arguments', function()
    eq(
      {
        -- A method call: `luaL_argerror` numbers the arguments after `self`.
        "bad argument #1 to '_set_logger' (boolean expected)",
        "bad argument #2 to '_set_logger' (boolean expected)",
        "bad argument #3 to '_set_logger' (function expected)",
      },
      exec_lua(function()
        local parser = vim._create_ts_parser('c')
        local out = {}
        for _, args in ipairs({ { 1, true, print }, { true, 1, print }, { true, true, 1 } }) do
          local _, err = pcall(function()
            local r = parser:_set_logger(unpack(args))
            return r
          end)
          out[#out + 1] = err:gsub('^.-: ', '')
        end
        return out
      end)
    )
  end)

  it('raises a failing callback as a Lua error from parse()', function()
    eq(
      { false, 'treesitter logger callback failed', '(translation_unit (primitive_type))' },
      exec_lua(function()
        local parser = vim._create_ts_parser('c')
        parser:_set_logger(true, true, function()
          error('boom')
        end)
        local ok, err = pcall(parser.parse, parser, nil, 'int a;', true)
        return { ok, err, vim._create_ts_parser('c'):parse(nil, 'int;', true):root():sexpr() }
      end)
    )
    assert_alive()
  end)

  it('releases the callback with the parser', function()
    exec_lua(function()
      local parser = vim._create_ts_parser('c')
      parser:_set_logger(true, true, function() end)
      parser = nil
      collectgarbage()
      collectgarbage()
    end)
    assert_alive()
  end)
end)

describe('treesitter binding errors', function()
  -- The exact texts the binding raises from a C function (`luaL_argerror`,
  -- `luaL_error`): each is a longjmp out of the binding, and each must say
  -- the same thing when it is raised another way.

  it('names the invalid buffer handle', function()
    eq(
      "bad argument #2 to 'parse' (invalid buffer handle: 999)",
      exec_lua(function()
        local parser = vim._create_ts_parser('c')
        local _, err = pcall(function()
          local r = parser:parse(nil, 999, true)
          return r
        end)
        return (err:gsub('^.-: ', ''))
      end)
    )
  end)

  it('names a wiped buffer by its handle', function()
    eq(
      "bad argument #2 to 'parse' (invalid buffer handle: 7)",
      exec_lua(function()
        local buf = vim.api.nvim_create_buf(false, true)
        while buf < 7 do
          buf = vim.api.nvim_create_buf(false, true)
        end
        vim.api.nvim_buf_delete(buf, { force = true })
        local parser = vim._create_ts_parser('c')
        local _, err = pcall(function()
          local r = parser:parse(nil, buf, true)
          return r
        end)
        return (err:gsub('^.-: ', ''))
      end)
    )
  end)

  it('refuses a source that is neither string nor buffer', function()
    eq(
      "bad argument #2 to 'parse' (expected either string or buffer handle)",
      exec_lua(function()
        local parser = vim._create_ts_parser('c')
        local _, err = pcall(function()
          local r = parser:parse(nil, {}, true)
          return r
        end)
        return (err:gsub('^.-: ', ''))
      end)
    )
  end)

  it('reports an unknown language', function()
    eq('no such language: borklang', pcall_err(exec_lua, "vim._create_ts_parser('borklang')"))
  end)
end)
