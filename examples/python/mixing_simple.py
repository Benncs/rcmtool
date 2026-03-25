"""
Simple Example Script: Mixing Simulation with pycmtool
======================================================

**Purpose:**
This script demonstrates how to use the `pycmtool` library to perform a simple mixing simulation.
It integrates a mass balance over time using a sparse matrix representation of the system
The core functionality is used for chemical or compartmental mixing simulations.

**Key Features:**
- Uses `pycmtool` to construct flow sparse matrices from system states.
- Integrates mass balance equations using `scipy.integrate.solve_ivp`.
- Visualizes results with `matplotlib`.

**Example Workflow:**
1. Define initial mass distribution.
2. Use `pycmtool` to advance the system state and construct sparse matrices.
3. Integrate the system over a specified duration.
4. Plot the results.

**Author:** CASALE Benjamin
**Date:** 2025-09-18
**Version:** 1.0
"""

import os

import matplotlib.pyplot as plt
import numpy as np
import pycmtool
from scipy.integrate import solve_ivp

N_SPECIES = 2  # We use 2 species to demonstrates that pycmtool can handle different dissolved species


def integration(
    fmt: "pycmtool.Transitioner", mass_0: np.ndarray, duration: float, n_species
):
    """
    Perform a mixing simulation by integrating the mass balance ODE over a specified duration.

    This function uses `pycmtool` to advance the system state and construct sparse matrices,
    then integrates the mass balance equations using `scipy.integrate.solve_ivp`.

    Args:
        fmt: A `pycmtool.Transitioner` object to advance the system state.
        mass_0: Initial mass distribution (1D array of shape `(n_species * n_compartments,)`).
        duration: Total simulation time (in seconds or any consistent time unit).
        n_species: Number of species in the simulation .
    """

    def wrap(t: float, x: np.ndarray) -> np.ndarray:
        """Wrapper function for ODE integration."""
        d_t = 0  # Not used for this transitioner
        # Advance iterator to the corresponding flowmap (according to t or dt)
        it = fmt.advance(t, d_t)
        n_c = it.n_compartments
        liquid_state = it.liquid
        # Get the transition matrix
        transition = pycmtool.get_sparse_transition_matrix(liquid_state)

        vol = liquid_state.volumes  # Volume of each compartment
        _mass = x.reshape((n_species, n_c))  # Reshape to (N_SPECIES, n_compartments)
        C = _mass / vol  # Concentration: mass / volume
        return (C @ transition).reshape(-1)  # Return flattened array for ODE solver

    return solve_ivp(
        wrap,
        (0, duration),
        mass_0.reshape(-1),
        method="BDF",
        vectorized=False,
    )


def initial_c_distribution(n_c):
    """
    Generate a random initial concentration distribution for a simulation with `n_c` compartments.
    """
    return np.random.random((1, n_c))
    # m = np.zeros((n_c,))
    # m[0] = 1
    # return m


def get_normalized(it, y, i):
    """
    Calculate the normalized concentration for a given species across all compartments.

    The normalization is performed by dividing each compartment's concentration by the mean concentration
    of the species across all compartments. This is useful for comparing relative concentrations.
    """
    vol = it.liquid.volumes
    m0_c = y[0, :, i]
    return (m0_c / vol) / np.mean(m0_c / vol, axis=0)


def check_mixing(fmt, final_time: float):
    it = fmt.get_current()
    n_c = it.n_compartments

    C = np.zeros((N_SPECIES, n_c))
    C[0, :] = initial_c_distribution(n_c)
    vol = it.liquid.volumes
    m0 = C * vol

    sol = integration(fmt, m0, final_time, N_SPECIES)
    y = sol.y.reshape((N_SPECIES, n_c, -1))
    it = fmt.get_current()
    m0_c = y[0, :, 0]
    mt_c = y[0, :, -1]
    c_init = get_normalized(fmt.get_at(0), y, 0)
    c_final = get_normalized(fmt.get_at(fmt.n_flowmaps - 1), y, -1)

    m0m = np.sum(m0_c, axis=0)
    mfm = np.sum(mt_c, axis=0)

    print("Total volume: ", np.sum(vol))
    print("Inital mass: ", m0m)
    print("Final mass: ", mfm)
    print("Initial normalized C: ", c_init[:5])
    print("Final normalized C: ", c_final[:5])
    print("Final variance: ", np.var(c_final, axis=0))

    plt.figure()
    plt.style.use("tableau-colorblind10")
    plt.plot(c_final, label="final")
    plt.plot(c_init, "x", markersize=2.5, label="init")
    plt.title("Normalized concentration in all compartments (1 is the target value)")
    plt.xlabel("Compartment ID")
    plt.ylabel("Normalized concentration")
    plt.legend()

    plt.figure()
    plt.style.use("tableau-colorblind10")
    plt.plot(sol.t, y[0, 0, :])
    plt.legend()

    plt.show()
    assert abs(m0m - mfm) < 1e-8


if __name__ == "__main__":
    final_time = 200
    root = os.environ["EXAMPLE_ROOT"]
    # Let CMTool read and load the full case automatically, ready to iterate

    fmt = pycmtool.data.get_transitioner(root)
    check_mixing(fmt, final_time)
