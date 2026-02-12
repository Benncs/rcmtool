# RCMTool

The specific goal of this repository is to develop a Compartment Modelling Tool. 

Compartmental Modeling Approach (CMA) is a method that simplifies spatial dimensions by dividing the domain into smaller, uniform "compartments." Each compartment behaves as a homogeneous unit, connected to its neighboring compartments. The domain is represented by a scalar field, and the interactions between compartments are governed by a flowmap.

This approach significantly reduces computational complexity while maintaining an acceptable level of model accuracy.
## Overview

This crates aims to provides different feature to perform complete simulation using CMA. 

- Unified data type that can easily be written and read 
- A Modeler to generate predefined simple model 
- A CFD-to-CMA generator to use CFD results exported in Ensight-Gold Format 
- Assemble flowmaps seamlessly

## Crate organization 
 - cmtool: main cli 
 - cmtool-data: utilities for high and low level manipulation on flowmaps,compartment data
 - cmtool-core: algorithm to transform results from fine-mesh transient CFD simuation into Compartment Models.
 - cmtool-assemble: Generate flowmaps from from models and assemble them
 - cmtool-python: Python bindings for cmtool-core,FlowmapTransitioner and case reading
 - cmtool-cxx: Bindings for C++ use, expose FlowMapTranstioner API

## Objectives 

This crate is a port of an existing C++ tool. The objective of the Rust implementation is to evaluate the efficiency of the Rust language to  perform this type of operation, with a particular focus on maintaining code readability, maintainability, and performance.

## Getting started 



## Authors
- **CASALE Benjamin**

## LICENCE

This work is under GNU Lesser General Public License v3.0 or later (LGPL-3.0-or-later)
