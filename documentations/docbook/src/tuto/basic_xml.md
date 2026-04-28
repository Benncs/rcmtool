# XML

## How to use

- Rust API: `cmtool-assemble` exposes `Parser` struct  as well as `generate_domain` 
- App: rcmtool: 
  ```sh 
  cargo run --  xml --help
  ```
- Python API: TBD

- Example:
  - Names
    - case_0d1d
    - simple_0d
    - simple_1d
    - neubauer
  - Run with 
    ```sh
    cargo run --example <example_name>
    ```


For Rust and Python API, please refer to RustDoc [Here](TBD) 


## XML details 

### XML tags 

- Reactors:
  - Reactor0D: 
    - Volume fraction 
    - Diameter/Height or volume
  - Reactor1D:
  - 
  - ReactorFromFile
    - Path to file 
- Connections:
  - flux
- Fleeds
  - Flux

Different simple examples  [are available](../../../../examples/data) to assemble flowmaps and scalar fields such as this one :

```xml
{{#include ../../../../examples/data/case_0d1d_liq/reactors.xml}}
```
