-- The text of API errors that no other spec asserts. A client matches on
-- these strings, so each is pinned verbatim.

local t = require('test.testutil')
local n = require('test.functional.testnvim')()

local clear = n.clear
local eq = t.eq
local api = n.api
local exec_lua = n.exec_lua
local pcall_err = t.pcall_err

--- `pcall_err` for an error raised inside `exec_lua`, without the chunk's
--- `file:line:` prefix.
local function pcall_err_lua(...)
  return (pcall_err(...):gsub('^[^ ]*:%d+: ', ''))
end

describe('API error text', function()
  before_each(clear)

  it('nvim_input_mouse', function()
    local function mouse_err(button, action, modifier)
      return pcall_err(api.nvim_input_mouse, button, action, modifier, 0, 0, 0)
    end
    eq('invalid button or action', mouse_err('bogus', 'press', ''))
    eq('invalid button or action', mouse_err('wheel', 'sideways', ''))
    eq('invalid button or action', mouse_err('left', 'bogus', ''))
    eq('Invalid modifier: Z', mouse_err('left', 'press', 'Z'))
    eq('Invalid modifier: Z', mouse_err('left', 'press', 'C-Z'))
    -- A known modifier, and the wheel's directions, go through.
    api.nvim_input_mouse('wheel', 'left', 'C-S', 0, 0, 0)
    api.nvim_input_mouse('wheel', 'up', '', 0, 0, 0)
  end)

  it('nvim_parse_cmd', function()
    eq(
      'Parsing command-line: E492: Not an editor command: foobar',
      pcall_err(api.nvim_parse_cmd, 'foobar', {})
    )
  end)

  it('nvim_create_autocmd and nvim_del_autocmd', function()
    t.matches(
      "^Conflict: 'callback' not allowed with 'command'$",
      pcall_err_lua(exec_lua, function()
        vim.api.nvim_create_autocmd('BufEnter', { callback = function() end, command = 'echo' })
      end)
    )
    t.matches(
      "^Invalid 'callback': expected Lua function or Vim function name, got Integer$",
      pcall_err_lua(exec_lua, function()
        vim.api.nvim_create_autocmd('BufEnter', { callback = 1 })
      end)
    )
    eq(
      "Invalid 'event': 'NotAnEvent'",
      pcall_err(api.nvim_create_autocmd, 'NotAnEvent', { command = 'echo' })
    )
    eq('Invalid autocmd id: 0', pcall_err(api.nvim_del_autocmd, 0))
    eq('Failed to delete autocmd', pcall_err(api.nvim_del_autocmd, 99999))
    -- A refused call leaves no autocommand behind.
    eq(0, #api.nvim_get_autocmds({ event = 'BufEnter' }))
  end)

  it('nvim_call_dict_function', function()
    eq(
      'Invalid dict argument: expected String or Dict',
      pcall_err(api.nvim_call_dict_function, 1, 'f', {})
    )
  end)

  it('nvim_get_hl_by_name', function()
    eq(
      "Invalid highlight name: 'NoSuchGroup'",
      pcall_err(api.nvim_get_hl_by_name, 'NoSuchGroup', true)
    )
  end)

  it('nvim_buf_add_highlight', function()
    eq(
      'Invalid line number: out of range',
      pcall_err(api.nvim_buf_add_highlight, 0, -1, 'Error', -1, 0, -1)
    )
  end)

  it('nvim_win_set_hl_ns', function()
    eq("Invalid 'namespace'", pcall_err(api.nvim_win_set_hl_ns, 0, -2))
  end)

  it('nvim_get_proc', function()
    eq("Invalid 'pid': -1", pcall_err(api.nvim_get_proc, -1))
  end)

  it('nvim_buf_get_offset', function()
    eq('Index out of bounds', pcall_err(api.nvim_buf_get_offset, 0, 5))
  end)

  it('nvim_paste with a handler that throws', function()
    exec_lua(function()
      vim.paste = function()
        error('boom')
      end
    end)
    t.matches('boom', pcall_err(api.nvim_paste, 'a', false, 1))
    -- The rest of that paste is dropped without calling the handler again.
    eq(false, api.nvim_paste('b', false, 2))
  end)
end)
