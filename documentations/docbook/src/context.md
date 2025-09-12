# Context

In the field of process engineering, processes are often driven over long periods to produce a valuable product within reactors. This is due to multiple factors, but two can be highlighted:
- The reaction rate is slow and requires a long time to reach a steady state for continuous processes or to complete for batch processes.
- The reaction and the entire reactor environment are very sensitive to initial conditions and intermediate states, leading to hardly predictable transient and steady behaviors.

For these reasons, simulating an entire process at an industrial scale is not straightforward if we want to maintain efficiency and precision. In most cases, usual numerical techniques (FEM, FVM, FDM) work well with very high spatial and temporal precision, but they are also known to be time-consuming. For transient simulations, all calculations are performed in a finely discretized space. In some cases, for specific studies, we do not always need the same amount of precision throughout the entire transient simulation. Sometimes, we do not even want to recompute the evolution of some processes (especially for very slow processes). Taking this into account, instead of using more and more computational power and more efficient methods, methods to reduce dimension and accuracy when needed were considered.


One method to reduce complexity is the Compartment Modeling Approach (CMA). 
## Compartment Modeling Approach


One method to reduce complexity is the **Compartment Modeling Approach** (CMA). 
The idea is simple: if we have a very slow and long-driven reaction that does not significantly affect the reactor's spatial properties (such as flow, temperature, pressure), we can first solve the main reactor's behavior with very high spatial and temporal precision but for a small time range, like a typical CFD study (covering minutes, or perhaps a few hours, of physical time). We then keep these results and reduce the spatial resolution as needed by projecting the results onto a coarser grid. These fixed results are used for long-term simulations.

The specificity of CMA is that we reduce the results such that we no longer consider individual "cell meshes" but rather compartments that represent large spatial regions in the reactor, which are considered homogeneous. Furthermore, while meshes can be unstructured with cells of any shape and size, compartments are arranged in a structured grid of regular tetrahedrons.
    

### Use cases

This obsviouly reduce results accuracy but for some specific application it can be very useful. Compartments keeps, the main average flow between reactors regions (very important for mixing study), the eulerian scalar fields (gas-fraction,temperature,turbulency) is kept but volume averaged. 

Follow some academic work 



