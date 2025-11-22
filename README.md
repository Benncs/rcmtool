# RCMTool

The specific goal of this repository is to develop a Compartment Modelling Tool. 
A reactor compartment model is composed of two main parts:
- Flows between compartments (referred to as flowmaps)
- Scalar fields, with values within each compartment


The compartmentalization process aims to reduce computational costs while maintaining a balance between computational efficiency and accuracy.

The core feature of the crate focuses on processing and simplifying Computational Fluid Dynamics (CFD) results obtained from simulations on fine computational grids into coarser grids, referred to as compartments. The provided tools enable convenient manipulation of compartment data and the configuration of simulations.
Additional features include the capability to generate Compartment Models directly from predefined reactor models (e.g., Plug Flow RReactor Model)

## Objectives 

This crate is a port of an existing C++ tool. The objective of the Rust implementation is to evaluate the efficiency of the Rust language for performing this type of operation, with a particular focus on maintaining code readability, maintainability, and performance.


### References

- Ensight gold specifications : 
    - https://dav.lbl.gov/archive/NERSC/Software/ensight/doc/Manuals/UserManual.pdf

## Authors
- **CASALE Benjamin**

## LICENCE

This work is under GNU Lesser General Public License v3.0 or later (LGPL-3.0-or-later)