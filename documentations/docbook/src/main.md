# Compartment Modeling Tool (CMTool)

The *CMTool* project aims to develop a set of tool to perform simulation using a compartment modelling approach.
Compartment modeling is an approach used to reduce computational cost in simulation while maintaining a balance between efficiency and precision.

The core part of the crate is focused on process and simplify computational fluid dynamics (CFD) results obtained from simulations on fine computational grids into coarser grids, known as compartments. Additonal features includes generation of Compartment Model drectly from models (namely Plug Flow Reactor Model)


## Objectives

This crate is a portage of exisiting C++ tool. The aim of the Rust crate is to evaluated the efficiency of this language to perform this kind of operation especially to keep the code maintable and performant.
In the long term, the goal is to phase out the dependency on the C++ tool, especially for new developments.

# Authors

[Casale Benjamin](mailto:casale@insa-toulouse)

Original idea of the c++ implementation of CFD-to-CMA tool comes from:
- [Morchain Jérome](mailto:morchain@insa-toulouse.fr)
- [Pigou Maxie](mailto:)

# License

This work is under GNU General Public License v3.0 or later (GPL-3.0-or-later)
