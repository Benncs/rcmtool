"""
Simple Example Script: Reactive Simulation with pycmtool
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
**Date:** 2025-12-08
**Version:** 1.0
"""

import os

import matplotlib.pyplot as plt
import numpy as np
import pycmtool
from scipy.integrate import solve_ivp
from typing import Callable


def integration(
    two_phase_flow: bool,
    fmt: "pycmtool.Transitioner",
    mass_0: np.ndarray,
    duration: float,
    n_species,
    f_reaction: Callable,
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
    n_p = 2 if two_phase_flow else 1

    def phase_dt(_mass, iphase, hydro_state, n_v):
        transition = pycmtool.get_sparse_transition_matrix(hydro_state)
        vol = hydro_state.volumes
        # C = _mass/vol
        # print(C.shape)
        # C = np.zeros((dims[0],dims[1],dims[-1]))
        C = _mass[:, :, iphase, :] / vol[:, None]
        mm = np.zeros_like(C)
        for i in range(n_v):
            mm[:, :, i] = C[:, :, i] @ transition
        return C, vol, mm

    def wrap_tpf(t: float, x: np.ndarray) -> np.ndarray:
        """Wrapper function for ODE integration."""
        d_t = 0  # Not used for this transitioner
        it = fmt.advance(t, d_t)
        n_c = it.n_compartments
        _mass = x.reshape((n_species, n_c, n_p, -1))
        n_v = _mass.shape[-1]

        C_l, vl, lflows = phase_dt(_mass, 0, it.liquid, n_v)
        Cg, vg, gflows = phase_dt(_mass, 1, it.gas, n_v)

        R_l = f_reaction(C_l) * vl[:, np.newaxis]

        T = np.zeros_like(R_l)
        T[2, :, :] = 0.2 * (0.03 * Cg[2, :, :] - C_l[2, :, :]) * vl[:, np.newaxis]

        dmdt = np.zeros_like(_mass)
        dmdt[:, :, 0, :] = lflows + R_l + T
        dmdt[:, :, 1, :] = gflows - T
        return dmdt.reshape(-1, n_v)

    def wrap_l(t: float, x: np.ndarray) -> np.ndarray:
        """Wrapper function for ODE integration."""
        d_t = 0  # Not used for this transitioner
        it = fmt.advance(t, d_t)
        n_c = it.n_compartments
        _mass = x.reshape((n_species, n_c, n_p, -1))
        n_v = _mass.shape[-1]
        C_l, vl, lflows = phase_dt(_mass, 0, it.liquid, n_v)
        R_l = f_reaction(C_l) * vl[:, np.newaxis]
        return (lflows + R_l).reshape(-1, n_v)

    wrap = wrap_tpf if two_phase_flow else wrap_l
    return solve_ivp(
        wrap,
        (0, duration),
        mass_0.reshape(-1),
        method="BDF",
        vectorized=True,
    )


def initial_c_distribution(n_c, n_p):
    """
    Generate a random initial concentration distribution
    for a simulation with `n_c` compartments.
    """
    n = np.random.random((n_c, n_p))
    return n


def gm0(two_phase_flow: bool, C, it):
    if two_phase_flow:
        m = np.zeros_like(C)
        m[:, :, 0] = C[:, :, 0] * it.liquid.volumes
        m[:, :, 1] = C[:, :, 1] * it.gas.volumes
        return m
    else:
        return C[:, :, 0] * it.liquid.volumes


def check_mixing(fmt, n_s, final_time: float, reaction_rate):
    it = fmt.get_current()
    n_c = it.n_compartments

    two_phase_flow = it.has_gas()
    n_p = 2 if two_phase_flow else 1
    C = np.zeros((n_s, n_c, n_p))
    C[0, :, :] = 0.7 * initial_c_distribution(n_c, n_p)
    C[1, :, :] = 0.2 * initial_c_distribution(n_c, n_p)
    C[2, :, 1] = 300e-3
    m0 = gm0(two_phase_flow, C, it)

    sol = integration(two_phase_flow, fmt, m0, final_time, n_s, reaction_rate)
    y = sol.y.reshape((n_s, n_c, n_p, -1))
    it = fmt.get_current()
    plt.figure()

    plt.plot(y[0, :, 0, 0] / it.liquid.volumes, label="glucose")
    plt.plot(y[1, :, 0, 0] / it.liquid.volumes, "--")
    plt.title("concentration in all compartments init")
    plt.legend()

    it = fmt.get_at(fmt.n_flowmaps - 1)

    plt.figure()
    plt.plot(y[0, :, 0, -1] / it.liquid.volumes, label="glucose")
    plt.plot(y[1, :, 0, -1] / it.liquid.volumes, "--")
    plt.title("concentration in all compartments final")
    plt.legend()

    plt.figure()
    plt.plot(sol.t, y[0, n_c // 3, 0, :] / it.liquid.volumes[n_c // 3], label="glucose")
    plt.plot(sol.t, y[1, n_c // 3, 0, :] / it.liquid.volumes[n_c // 3], "--")
    plt.title("concentration = f(t)")
    plt.legend()

    plt.figure()
    plt.plot(y[2, n_c // 3, 0, :] / it.liquid.volumes[n_c // 3])
    # plt.plot(y[2, n_c//3,1,:]/it.gas.volumes[n_c//3],'--')
    plt.legend()

    # plt.figure()
    # plt.plot(y[2, :, 0, -1] / it.liquid.volumes)
    # plt.plot(y[2, :, 1, -1] / it.liquid.volumes, "--")
    # plt.title("concentration in all compartments final2")
    # plt.legend()

    plt.show()


if __name__ == "__main__":
    final_time = 15 * 3600
    root =  os.environ["EXAMPLE_ROOT"]

    N_SPECIES = 3
    def _reaction_rate(Cl):
        mum = 0.8 / 3600
        K_S = 0.1
        r = np.zeros_like(Cl)
        mu = (
            mum * Cl[0, :, :] / (Cl[0, :, :] + K_S) * Cl[2, :, :] / (Cl[2, :, :] + 1e-6)
        )
        dx = Cl[1, :, :] * mu
        r[0, :, :] = -dx
        r[1, :, :] = dx / 2
        r[2, :, :] = -dx / 20
        # for i in range(Cl.shape[-1]):
        # mu = mum * Cl[0, :,i] / (Cl[0, :,i] + K_S)
        # dx = Cl[1, :,i] * mu
        # r[0,:,i]=-dx
        # r[1,:,i]=dx/2
        return r

    # Let CMTool read and load the full case automatically, ready to iterate
    fmt = pycmtool.data.get_transitioner(root)
    check_mixing(fmt, N_SPECIES, final_time, _reaction_rate)
