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
-- no C headers of its own; the upstream v0.12.4 tree (headers + generated
-- declaration headers) is reconstructed under target/upstream by
-- scripts/prep-unit-headers.sh. The unit tests then call the matching
-- transpiled symbols exported by the running nvim binary (ffi.C).
local deps_prefix = os.getenv('NVIM_DEPS_PREFIX')
local upstream = root .. '/target/upstream'
M.include_paths = {} --- @type string[]
if deps_prefix then
  table.insert(M.include_paths, deps_prefix .. '/include/luajit-2.1')
  table.insert(M.include_paths, deps_prefix .. '/include')
end
table.insert(M.include_paths, upstream .. '/build/src/nvim/auto')
table.insert(M.include_paths, upstream .. '/build/include')
table.insert(M.include_paths, upstream .. '/build/cmake.config')
table.insert(M.include_paths, upstream .. '/src/src')
-- Specs cimport paths like "./src/nvim/foo.h"; relative to the extracted
-- upstream root, since the port's own src/ holds the transpiled Rust.
table.insert(M.include_paths, upstream .. '/src')
M.apple_sysroot = ''

M.translations_enabled = false
M.is_asan = false
M.is_zig_build = false
-- Must match VTERM_TEST_FILE in the upstream build's auto/config.h; the
-- vterm fixture (unit-fixtures.so) writes there.
M.vterm_test_file = root .. '/target/upstream/build/test/vterm_test_output'
M.test_lua_prg = deps_prefix and (deps_prefix .. '/bin/luajit') or nil
M.test_luajit_prg = M.test_lua_prg
-- target/ is dressed up as a CMake-style build dir (runtime/, lib/nvim/) by
-- scripts/prep-test-tree.sh.
M.test_build_dir = root .. '/target'
M.test_source_path = root

return M
