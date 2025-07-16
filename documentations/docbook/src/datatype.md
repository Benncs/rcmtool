# Data Type

There are two main types of data needed to perform a simulation with a compartment model.

A reactor compartment model is composed of two main parts:
- Flows between compartments (Called flowmaps)
- Scalar fields inside compartments

## Case 

A CMA case file is needed for each reactor model. This fils basically contains the list of subsequent needed files (flows and scalar)

## Flows

Each flow is bidirectional and is described with:

| Nomenclature | Description | Type |
|--------------|-------------|------|
| source_id | The compartment ID of the flow origin | uint_32 |
| target_id | The compartment ID of the flow destination | uint_32 |
| flow_source_target | The value of flow source->destination | float64 |
| flow_target_source | The value of flow destination->source | float64 |

Flows are stored as binary data with a structure that can be described as "flow-oriented data". Having this file format is very modular because modelers have the ability to either add or remove flows independently from others.

The raw file structure is as follows:

- Header:
  - Number of compartments
  - Number of flows
- Body:
  - List of flows:

## Scalars

Scalar fields are more standard files, they contains the value of the considered sclar for each zone of our grid, id for each compartment. 
- Header:
  - Number of compartments
- Body:
  - List of values:
