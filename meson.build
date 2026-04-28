project(
  'rcmtool-cxx',
  'cpp',
  version: '0.1.4',
  meson_version: '>= 1.3.0',
)
out_dir = meson.current_build_dir() + '/rust_target'
root = meson.project_source_root()
cargo_path = root + '/Cargo.toml'
cargo = find_program('cargo')
cpp = meson.get_compiler('cpp')

_r = run_command('sh', '-c', f'mkdir -p @out_dir@/cxxbridge/cmtool-cxx/src', check: true).stdout()

dummy = custom_target(
  'rust_build',
  output: ['dummy'],
  command: [
    cargo,
    'build',
    '--manifest-path', cargo_path,
    '--target-dir', out_dir,
    '--profile', 'release-performance',
    '-p', 'cmtool-cxx',
  ],
  build_always_stale: true,
  build_by_default: true,
  console: true,
)

dirs = include_directories('./rust_target/cxxbridge/cmtool-cxx/src')

libdir = out_dir + '/release-performance/'

rustlib = join_paths(libdir, 'libcmtool_cxx.a')

ext = declare_dependency(sources: [dummy], include_directories: dirs, link_args: [rustlib])
