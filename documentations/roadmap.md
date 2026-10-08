# RCMTool Roadmap 

## Chore
- refractor cmtool-core/model to put cfd oriented into separated mod and move "reactor model" such as 0d/pfr from cmtool-assemble to cmtool-core
- 

## Core
- Open phase balance (`balance_2`): remove the hardcoded cfd_test_3 sparger from `manual.rs`
  - CLI: specify for each phase if it is open or closed
    - Open phase injection limited to a point + flowrate
  - Python API: give the inlet (`OpenPhase`) directly when calling dump_vector (dump_vector_gas, dump_vector_from_scalar_gas, ...)
  - Merge `balance` and `balance_2` into a single balance 
  - check_flow: take inlet/outlet into account instead of being skipped for an open phase


## Assemble

- Add autofeed tag in xml datamodel
  - Auto calculate flowrate from given dilution rate 
    - If cfd-based reactor select different source (N) to spread flowrate (Q/N)
