# Basic Mixing Example

This example demonstrates how to use Cmtool as a Compartmental Modeling (CM) framework to perform realistic simulations. We focus on reactor mixing, a fundamental application of compartmental modeling.
<!-- toc -->
## Motivation

Mixing is one of the simplest applications of Compartmental Modeling, involving only the flows between compartments. As established in previous research (TODO: add references), mixing does not alter the total system mass.

The rate of change of mass in each compartment can be described by the following differential equation:

$$ \dfrac{dm_{i}}{dt}= \sum_{j\neq i} c_j F_{ji} - \sum_{i\neq j} c_{i}F_{ij} $$

*Note*: Keep mass because volume might not be constant.

This can be rewritten using matrix formulation as:

$$ \dot{M} = C \cdot F $$



This formulation, allows for efficient computation and integration over time, making it suitable for simulations with F is the transition matrix provided by **CMTool**.




The mixing experiment involves integrating this ordinary differential equation (ODE) over time. An important aspect is that the matrix F can be time-dependent, denoted as F(t). **CMTool** simplifies this process with the **FlowMapTransitioner** trait.This trait allows users to load their cases, whether transient (with multiple flow maps) or not. The flowmap transitioner (abbreviated *fmt* ) is responsible for handling transitions between different flow maps and will automatically update the flow map over time as needed.



## Python Code

Our Python wrapper offers an easy-to-use interface for out-of-the-box simulations and proofs of concept. With compatibility with the numpy API, performing calculations becomes straightforward. Currently, one type of transitioner is available, and the volumes and transitions matrix can be easily accessed from the iterator. Compatibility with sparse matrices ensures efficient integrations. Furthermore, this setup is compatible with the widely-used `solve_ivp` function for performant ODE integration, although more sophisticated methods can be envisioned.




```python
{{#include ../../../example/python/mixing_simple.py}}
```


## Rust Code

Here is an example of how to implement this in Rust. The **CMTool** components are compatible with popular linear algebra libraries like Nalgebra and Ndarray. Although the interface is currently low-level, it effectively demonstrates how to use CMtool for performing integrations. This example uses a first-order explicit scheme for simplicity.

*Note*: For mixing problems, an implicit method is generally recommended for better performance and stability.

Using Rust is advisable for building complete simulation tools. Rust allows developers to focus on writing performant code and provides more control over data, although it may require more development effort.

```rust
{{#include ../../../example/src/mixing_simple.rs}}
```
