# RCMTool

The specific goal of this repository is to develop a Compartment Modelling Tool. 
A reactor compartment model is composed of two main parts:
- Flows between compartment (Called flowmaps)
- Scalar fields inside of compartment 

The compartmentalization process is crucial for reducing computational costs in subsequent simulation steps while maintaining a balance between computational efficiency and precision.

The core part of the crate is focused on process and simplify computational fluid dynamics (CFD) results obtained from simulations on fine computational grids into coarser grids, known as compartments. 

Additonal features includes generation of Compartment Model drectly from models (namely Plug Flow Reactor Model)


## Objectives 

This crate is a portage of exisiting c++ tool. The aim of the Rust crate is to evaluated the efficiency of this language to perform this kind of operation especially to keep the code maintable and performant.  

### References

- Ensight gold specifications : 
    - https://dav.lbl.gov/archive/NERSC/Software/ensight/doc/Manuals/UserManual.pdf

## Authors
- **CASALE Benjamin**

## LICENCE

This work is under GNU Lesser General Public License v3.0 or later (LGPL-3.0-or-later)