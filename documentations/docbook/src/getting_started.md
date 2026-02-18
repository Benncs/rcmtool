# Getting started 

Select the lastest available tag 
```sh
git clone --depth 1 --branch <tag_name> git@github.com:Benncs/rcmtool.git
```


## Rust 

To build the full cli application

```sh
cargo build --release 
```

Some example are available with : 
```sh
cargo run --example <example_name>
```

### VTK

The VTK features can be enabled if VTK dev dependencies are installed in the system. 
For rust build the feature 'use_vtk' is needed.
```sh
cargo build --release --features use_vtk
```

## Python package 

[uv](https://docs.astral.sh/uv/) package manager is highly recommended.

To enable use of python package, the following

```sh
uv venv 
uv pip install -r pyproject.toml --extra examples
```


Example can be run with 
```sh
export EXAMPLE_ROOT='/path/to/cma/case'
uv run examples/python/mixing_simple.py
```

> **Note:** The `export` command works only on POSIX-compliant systems (Linux, macOS, Unix).  
> Windows users should use `set` (Command Prompt) or `$env:` (PowerShell) instead.


### VTK

The VTK features can be enabled if VTK dev dependencies are installed in the system. 
Python with automatically detects

## C++

Shared library object can be compiled with [meson](https://mesonbuild.com/) build system. 
The best way is to use this folder as subproject and add the following to your 'meson.build'
```meson
cmtool = dependency('rcmtool', required: true, version: '>=0.1.0', static: true)
```
