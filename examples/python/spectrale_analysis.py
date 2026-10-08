"""
Spectral analysis of the compartment model transition matrix
============================================================

Pure mixing (no source term) is governed by

    dM/dt = A(t) M,   A(t) = P(t) V(t)^-1

with P the compartment-to-compartment flow matrix and V the compartment volumes,
both read from the CFD flowmaps.
The script also runs the numerical check from a pulse injection


The case folder (the one holding `cma_case`) is read from the CMA_CASE
environment variable.

Run with:
    CMA_CASE=/path/to/case uv run --no-sync python examples/python/spectrale_analysis.py

**Author:** CASALE Benjamin
**Version:** 2.0
"""

import argparse
import json
import os
import pathlib

import matplotlib.pyplot as plt
import numpy as np
import pycmtool
from scipy.linalg import expm

ENV_CASE = "CMA_CASE"

OKABE_ITO = [
    "#0072B2",  # blue
    "#E69F00",  # orange
    "#009E73",  # bluish green
    "#D55E00",  # vermillion
    "#56B4E9",  # sky blue
    "#CC79A7",  # reddish purple
    "#F0E442",  # yellow
    "#000000",  # black
]


def case_root() -> str:
    root = os.environ.get(ENV_CASE)
    if not root:
        raise SystemExit(f"set {ENV_CASE} to the folder containing the `cma_case` file")
    return root


def read_case_file(root: str) -> dict:
    try:
        with open(pathlib.Path(root) / "cma_case") as stream:
            return json.load(stream)
    except (OSError, ValueError):
        return {}


def state_matrix(it) -> np.ndarray:
    liquid = it.liquid
    transition = pycmtool.get_sparse_transition_matrix(liquid).toarray()
    volumes = np.asarray(liquid.volumes)
    return transition.T / volumes[np.newaxis, :]


def load_case(root: str):
    fmt = pycmtool.data.get_transitioner(root)
    matrices = [state_matrix(fmt.get_at(k)) for k in range(fmt.n_flowmaps)]
    volumes = np.asarray(fmt.get_at(0).liquid.volumes)
    return matrices, volumes


def spectrum(matrices, period: float):
    """Eigenvalues and eigenvectors of the mixing operator

    For a T-periodic case ->  Floquet
    """
    if len(matrices) == 1:
        values, vectors = np.linalg.eig(matrices[0])
    else:
        # https://fr.wikipedia.org/wiki/Th%C3%A9or%C3%A8me_de_Floquet#Stabilit%C3%A9_des_syst%C3%A8mes_de_Floquet
        # T-periodic flow: Floquet exponents/modes from the monodromy matrix
        d_t = period / len(matrices)
        monodromy = np.eye(matrices[0].shape[0])
        # https://www.sciencedirect.com/science/article/abs/pii/S0375960102017693 (Eq 7 and 8)

        # We can multiply matricex because we have linear operation
        for a in matrices:
            monodromy = expm(a * d_t) @ monodromy
        multipliers, vectors = np.linalg.eig(monodromy)
        values = np.log(multipliers.astype(complex)) / period

    order = np.argsort(-values.real)
    return values[order], vectors[:, order]


def decay_times(values: np.ndarray) -> np.ndarray:
    return -1.0 / values[1:].real


def variation_coefficient(concentration, volumes):
    """Volume-weighted coefficient of variation of a concentration field."""
    mean = np.average(concentration, weights=volumes)
    return np.sqrt(np.average((concentration - mean) ** 2, weights=volumes)) / mean


def propagate(matrices, volumes, period, duration, injected, vectors):
    """pulse injection in one compartment.

    Returns the times, the volume-weighted coefficient of variation I(t) on C = M / V
    """
    n_maps = len(matrices)
    d_t = period / n_maps if n_maps > 1 else duration / 400.0
    steps = [expm(a * d_t) for a in matrices]
    n_steps = int(np.ceil(duration / d_t))

    mass = np.zeros(volumes.size)
    mass[injected] = 1.0
    weights = np.linalg.solve(vectors, mass.astype(complex))
    attractor = (weights[0] * vectors[:, 0]).real

    variation = np.empty(n_steps + 1)
    deviation = np.empty(n_steps + 1)
    for k in range(n_steps + 1):
        concentration = mass / volumes
        bulk = np.average(concentration, weights=volumes)
        variation[k] = variation_coefficient(concentration, volumes)
        deviation[k] = (
            np.sqrt(np.average(((mass - attractor) / volumes) ** 2, weights=volumes))
            / bulk
        )
        mass = steps[k % n_maps] @ mass
        attractor = steps[k % n_maps] @ attractor

    return np.arange(n_steps + 1) * d_t, variation, deviation


def asymptotic_slope(times, deviation, start):
    alive = deviation > 1e-10 * deviation[0]
    usable = alive & (times >= start)
    if usable.sum() < 3:
        usable = alive & (times >= 0.4 * times[-1])
    if usable.sum() < 3:
        return None, usable
    slope, _ = np.polyfit(times[usable], np.log(deviation[usable]), 1)
    return slope, usable


def plot_spectrum(ax, values):
    tau = decay_times(values)
    ax.plot(
        np.arange(values.size), values.real, "o", markersize=3.5, color=OKABE_ITO[0]
    )
    ax.plot([1], [values[1].real], "o", markersize=8, color=OKABE_ITO[2], zorder=5)
    ax.annotate(
        rf"$\tau_1$ = {tau[0]:.3g} s",
        xy=(1, values[1].real),
        xytext=(0.10 * values.size, values[1].real),
        va="center",
        arrowprops={"arrowstyle": "<-", "lw": 1.0},
    )
    ax.set_yscale("symlog", linthresh=max(1e-3, 0.5 * abs(values[1].real)))
    ax.axhline(0.0, lw=0.8, color="0.6")
    ax.set_xlabel("mode $n$")
    ax.set_ylabel(r"Re $\lambda_n$, 1/s")
    ax.set_title(f"{values.size} compartments")
    ax.grid(alpha=0.3)


def conjugate(values, index, tol=1e-8):
    scale = abs(values[index])
    if abs(values[index].imag) <= tol * scale:
        return None
    distance = np.abs(values - np.conj(values[index]))
    distance[index] = np.inf
    partner = int(np.argmin(distance))
    return partner if distance[partner] <= tol * scale else None


def plot_complex_plane(ax, values):
    ax.plot(values.real, values.imag, "o", markersize=3.5, color=OKABE_ITO[0])
    partner = conjugate(values, 1)
    marked = [1] if partner is None else [1, partner]
    label = (
        rf"$\lambda_1$ = {values[1].real:.4g} $\pm$ {abs(values[1].imag):.4g}$i$"
        if partner is not None
        else rf"$\lambda_1$ = {values[1].real:.4g}"
    )
    ax.plot(
        values[marked].real,
        values[marked].imag,
        "o",
        markersize=8,
        color=OKABE_ITO[2],
        zorder=5,
        label=label,
    )
    ax.axhline(0.0, lw=0.8, color="0.6")
    ax.axvline(0.0, lw=0.8, color="0.6")
    ax.set_xlabel(r"Re $\lambda$, 1/s")
    ax.set_ylabel(r"Im $\lambda$, 1/s")
    ax.legend(fontsize=9, frameon=False, loc="upper left")
    ax.grid(alpha=0.3)


def plot_decay_times(ax, values):
    tau = decay_times(values)
    bins = np.logspace(np.log10(tau.min()), np.log10(tau.max()), 40)
    ax.hist(tau, bins=bins, color=OKABE_ITO[0])
    ax.axvline(tau[0], color=OKABE_ITO[2], lw=2.5)
    if tau[0] > 1.2 * tau[1]:  # spectral gap isolating the mixing mode
        ax.axvspan(tau[1], tau[0], color=OKABE_ITO[4], alpha=0.25)
        ax.text(
            tau[1],
            0.8 * ax.get_ylim()[1],
            f"no modes\n{tau[1]:.3g}-{tau[0]:.3g} s",
            color=OKABE_ITO[0],
            ha="right",
            fontsize=9,
        )
    ax.set_xscale("log")
    ax.set_xlabel(r"decay time $\tau$, s")
    ax.set_ylabel("modes")
    ax.grid(alpha=0.3)


def plot_homogeneity(ax, times, variation, usable, slope, lambda_1):
    """Segregation index against the exponential set by the slowest mode."""
    ax.plot(times, variation, color=OKABE_ITO[0], label="$I(t)$")
    start = times[usable][0] if slope is not None else 0.0
    reference = np.interp(start, times, variation) * np.exp(lambda_1 * (times - start))
    ax.plot(
        times,
        reference,
        "--",
        color=OKABE_ITO[3],
        label=rf"$\exp(\lambda_1 t)$, $\lambda_1$ = {lambda_1:.4g} 1/s",
    )
    ax.set_ylim(0.5 * variation.min(), 2.0 * variation.max())
    ax.set_yscale("log")
    ax.set_xlabel("$t$, s")
    ax.set_ylabel("$I(t)$, segregation index")
    ax.legend(fontsize=9, frameon=False)
    ax.grid(alpha=0.3)


def eigenvector(values, vectors, index):
    """Eigenvector of mode `index`, real and normalized to a unit peak.

    A complex conjugate pair spans two real vectors sharing the same decay time:
    the real part for the first of the pair, the imaginary part for the second.
    """
    paired = index > 0 and abs(values[index] - np.conj(values[index - 1])) < 1e-9
    vector = vectors[:, index]
    vector = vector.imag if paired else vector.real
    return vector / np.abs(vector).max()


def plateau_window(times, variation, min_points=64):
    """Index slice of the trailing plateau of I(t), or None while it still decays."""
    level = np.median(variation[-max(4, variation.size // 10) :])
    close = variation < 1.5 * level
    start = variation.size - 1
    while start > 0 and close[start - 1]:
        start -= 1
    if variation.size - start < min_points:
        return None
    head, tail = np.array_split(variation[start:], 2)
    return slice(start, None) if tail.mean() > 0.5 * head.mean() else None


def plateau_spectrum(times, variation, window):
    """Amplitude spectrum of the residual oscillation of I(t) on the plateau."""
    signal = variation[window]
    signal = signal - signal.mean()
    taper = np.hanning(signal.size)
    amplitude = 2.0 * np.abs(np.fft.rfft(signal * taper)) / taper.sum()
    return np.fft.rfftfreq(signal.size, times[1] - times[0]), amplitude


def plot_plateau_spectrum(ax, frequency, amplitude, period, level):
    ax.plot(frequency, amplitude, lw=1.0, color=OKABE_ITO[0])
    for harmonic in range(1, int(frequency[-1] * period) + 1):
        ax.axvline(
            harmonic / period,
            lw=0.8,
            ls=":",
            color=OKABE_ITO[3],
            label="flow map harmonics $k/T$" if harmonic == 1 else None,
        )
    ax.set_yscale("log")
    ax.set_xlabel("frequency, Hz")
    ax.set_ylabel("amplitude of $I(t)$")
    ax.set_title(f"plateau at $I$ = {level:.3g}, $T$ = {period:.4g} s")
    ax.legend(fontsize=9, frameon=False)
    ax.grid(alpha=0.3)


def plot_eigenvectors(ax, values, vectors, n_modes):
    for k in range(1, n_modes + 1):
        ax.plot(
            eigenvector(values, vectors, k),
            lw=1.0,
            color=OKABE_ITO[(k - 1) % len(OKABE_ITO)],
            label=f"eigenvector {k}, tau = {-1 / values[k].real:.3g} s",
        )
    ax.axhline(0.0, lw=0.8, color="0.6")
    ax.set_xlabel("compartment id")
    ax.set_ylabel("eigenvector component")
    ax.legend(fontsize=9, frameon=False)
    ax.grid(alpha=0.3)


def spectrum_figure(times, variation, window, period, root, args):
    spectrum_figure, spectrum_ax = plt.subplots(figsize=(7, 4.5))
    level = variation[window].mean()
    frequency, amplitude = plateau_spectrum(times, variation, window)
    print(
        f"  plateau from t = {times[window][0]:.3g} s at I = {level:.3g}, "
        f"peak to peak {np.ptp(variation[window]):.3g}"
    )
    peaks = 1 + np.flatnonzero(
        (amplitude[1:-1] > amplitude[:-2]) & (amplitude[1:-1] > amplitude[2:])
    )
    for i in peaks[np.argsort(amplitude[peaks])[::-1][:5]]:
        print(
            f"    f = {frequency[i]:8.4f} Hz, period = {1 / frequency[i]:7.4f} s, "
            f"amplitude = {amplitude[i]:.3e}, f/f_map = {frequency[i] * period:.4f}"
        )

    plot_plateau_spectrum(spectrum_ax, frequency, amplitude, period, level)
    spectrum_figure.suptitle(
        f"Residual oscillation of $I(t)$, {pathlib.Path(root).name}"
    )
    spectrum_figure.tight_layout()
    if args.save:
        stem = pathlib.Path(args.save)
        spectrum_figure.savefig(
            stem.with_name(f"{stem.stem}_spectrum{stem.suffix}"), dpi=150
        )


def parse():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--period", type=float, default=None, help="flow map period, s")
    parser.add_argument("--duration", type=float, default=None, help="run length, s")
    parser.add_argument("--inject", type=int, default=0, help="injection compartment")
    parser.add_argument("--n-modes", type=int, default=3, help="eigenvectors to draw")
    parser.add_argument("--save", default=None)
    return parser.parse_args()


def main():
    args = parse()
    root = case_root()
    case = read_case_file(root)
    matrices, volumes = load_case(root)

    period = args.period or len(matrices) * case.get("time_per_flow_map", 0.0) or 1.0
    values, vectors = spectrum(matrices, period)
    tau = decay_times(values)
    lambda_1 = values[1].real

    duration = args.duration if args.duration else 8.0 * tau[0]
    times, variation, deviation = propagate(
        matrices, volumes, period, duration, args.inject % volumes.size, vectors
    )

    # fit start after the modes faster than lambda_1 have decayed
    faster = tau[tau < 0.99 * tau[0]]
    slope, usable = asymptotic_slope(
        times, deviation, 5.0 * (faster[0] if faster.size else tau[0])
    )

    print(
        f"{root}: {values.size} compartments, {len(matrices)} flow map(s), T = {period:.4g} s"
    )
    print(f"  lambda_0 = {values[0].real:.3e} 1/s (mass conservation)")
    print(f"  lambda_1 = {lambda_1:.4e} 1/s -> tau_mix = {tau[0]:.3g} s")
    print(f"  next mode: tau = {tau[1]:.3g} s, fastest: tau = {tau[-1]:.3g} s")
    oscillating = np.abs(values.imag) > 1e-9 * np.abs(values.real).max()
    print(
        f"  oscillating modes: {oscillating.sum()} of {values.size}, "
        f"max |Im| = {np.abs(values.imag).max():.3g} 1/s"
    )
    print("  mode        Re, 1/s        Im, 1/s     tau, s   conjugate pair")
    for k in range(1, max(1, args.n_modes) + 1):
        partner = conjugate(values, k)
        if partner is None:
            check = "real mode"
        else:
            residual = abs(values[k] - np.conj(values[partner]))
            check = f"with mode {partner}, |lam_k - conj(lam_p)| = {residual:.1e}"
        print(
            f"  {k:<4d} {values[k].real:+14.7e} {values[k].imag:+14.7e} "
            f"{-1 / values[k].real:8.3g}   {check}"
        )
    print(
        f"  I(0) = {variation[0]:.3g}, floor = {variation.min():.3g} (mass balance residual)"
    )
    if slope is not None:
        window = times[usable]
        print(
            f"  ln I(t) slope over [{window[0]:.3g}, {window[-1]:.3g}] s = {slope:.4e} 1/s"
        )
        print(f"  -> {100 * slope / lambda_1:.1f} % of lambda_1")

    plt.rcParams["axes.prop_cycle"] = plt.cycler(color=OKABE_ITO)
    fig = plt.figure(figsize=(15, 8))
    grid = fig.add_gridspec(2, 6)
    plot_spectrum(fig.add_subplot(grid[0, 0:2]), values)
    plot_complex_plane(fig.add_subplot(grid[0, 2:4]), values)
    plot_decay_times(fig.add_subplot(grid[0, 4:6]), values)
    plot_eigenvectors(
        fig.add_subplot(grid[1, 0:3]), values, vectors, max(1, args.n_modes)
    )
    plot_homogeneity(
        fig.add_subplot(grid[1, 3:6]), times, variation, usable, slope, lambda_1
    )

    # Check if we have final plateau
    window = plateau_window(times, variation)
    if window is not None:
        spectrum_figure(times, variation, window, period, root, args)

    fig.tight_layout()

    if args.save:
        fig.savefig(args.save, dpi=150)
    plt.show()


if __name__ == "__main__":
    main()
