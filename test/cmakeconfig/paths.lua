-- Static replacement for the values CMake used to interpolate into
-- paths.lua.in (kept for reference). There is no CMake build anymore: the
-- Rust port builds with cargo, straight into the repo, so the "build dir"
-- and the source dir are both the repo root. The nvim binary itself is
-- located via $NVIM_PRG (set by scripts/run-tests.sh), not through here.
local M = {}

local function repo_root()
  local source = debug.getinfo(1, 'S').source
  assert(type(source) == 'string' and vim.startswith(source, '@'), 'failed to resolve paths.lua')
  local path = assert(vim.uv.fs_realpath(source:sub(2)), 'failed to resolve paths.lua')
  -- …/test/cmakeconfig/paths.lua -> repo root
  return vim.fs.dirname(vim.fs.dirname(vim.fs.dirname(path)))
end

local root = repo_root()

-- C header include paths for the unit tests' FFI preprocessor. The port has
-- no C headers to parse, so this is empty (unit tests cannot run).
M.include_paths = {} --- @type string[]
M.apple_sysroot = ''

M.translations_enabled = false
M.is_asan = false
M.is_zig_build = false
M.vterm_test_file = ''
-- target/ is dressed up as a CMake-style build dir (runtime/, lib/nvim/) by
-- scripts/prep-test-tree.sh.
M.test_build_dir = root .. '/target'
M.test_source_path = root

return M
