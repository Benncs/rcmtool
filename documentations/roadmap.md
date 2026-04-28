# RCMTool Roadmap 

## Chore
- refractor cmtool-core/model to put cfd oriented into separated mod and move "reactor model" such as 0d/pfr from cmtool-assemble to cmtool-core
- 

## Core

- CFD-to_CMA algo doesn't work because of bad interface area calculation
  - Scalar field ok 
  - vector field direction seems to be good but magnitude false because of area

## Assemble

- Add autofeed tag in xml datamodel
  - Auto calculate flowrate from given dilution rate 
    - If cfd-based reactor select different source (N) to spread flowrate (Q/N)
