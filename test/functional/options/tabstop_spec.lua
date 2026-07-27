local t = require('test.testutil')
local n = require('test.functional.testnvim')()

local assert_alive = n.assert_alive
local clear = n.clear
local feed = n.feed

describe("'tabstop' option", function()
  before_each(function()
    clear()
  end)

  -- NOTE: Setting 'tabstop' to a big number reproduces crash #2838.
  -- Disallowing big 'tabstop' would not fix #2838, only hide it.
  it('tabstop=<big-number> does not crash #2838', function()
    -- Insert a <Tab> character for 'tabstop' to work with.
    feed('i<Tab><Esc>')
    -- Set 'tabstop' to a very high value.
    -- Use feed(), not command(), to provoke crash.
    feed(':set tabstop=3000000000<CR>')
    assert_alive()
  end)
end)

describe("'vartabstop' option", function()
  local api = n.api
  local command = n.command
  local eq = t.eq

  before_each(function()
    clear()
  end)

  -- Upstream freed and cleared the buffer's tabstop array before validating
  -- each stop, so a rejected value left 'vartabstop' reading its old text
  -- while the buffer measured tabs with 'tabstop' instead.
  it('keeps the previous stops when a new value is rejected', function()
    command('set tabstop=8 vartabstop=4')
    api.nvim_buf_set_lines(0, 0, -1, true, { '\tx' })
    eq(5, n.fn.virtcol({ 1, 2 }))

    -- 10000 is past TABSTOP_MAX, so this is rejected.
    eq(false, pcall(command, 'set vartabstop=10000'))
    eq('4', api.nvim_get_option_value('vartabstop', {}))
    eq(5, n.fn.virtcol({ 1, 2 }))
  end)
end)
