# Components

This crates aims to provides different feature to perform complete simulation using CMA. 

- Unified data type that can easily be written and read 
- A Modeler to generate predefined simple model 
- A CFD-to-CMA generator to use CFD results exported in Ensight-Gold Format 
- Assemble flowmaps seamlessly

# Crate organization 

## cmtool

Simple CLI tool for fast use. 
Generation from CFD case or from xml descriptor

## cmtool-data

Utilities to manipulation Compartment Data such as Flowmaps and Scalar Fields. High level manipultation is done with FlowMap transitioner, low-level is used to directly write Compartment data. 

## cmtool-core
CFD-to-CMA: Algorithm to transform results from fine-mesh transient CFD simuation into Compartment Models. 

## cmtool-assemble

Generate flowmaps from from models and assemble them. Flowmaps descriptor as XML file can be used from high-level use. Direct generate and case writer is available for low-level use. 

## cmtool-python

Python bindings, namely binds cmtool-core generation from Ensight case, FlowmapTransitioner and case reading. 

## cmtool-cxx

Bindings for c++ use, expose FlowMapTranstioner API
