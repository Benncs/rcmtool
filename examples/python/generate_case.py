# SPDX-License-Identifier: GPL-3.0-or-later
"""
Generate a CMA case from a CFD export
"""

import os
import pathlib

import numpy as np
from pycmtool import case, data, generate

CFD_CASE = os.environ.get("CFD_CASE", "../two_phase_cfd_inputs/RESULTS.encas")
CASE_OUT = os.environ.get("CASE_OUT", "./out/z12py")
N_DIV = [6, 6, 12]

# A dumped file gets its `.raw` from the core, the manifest needs the full name
out = lambda name: str(pathlib.Path(CASE_OUT) / name)

pathlib.Path(CASE_OUT).mkdir(parents=True, exist_ok=True)

handle = generate.CMHandle(N_DIV, CFD_CASE)
handle.set_balance_settings(1000, 1e-6, 1e-2)

GAS_FRACTION = handle.variable("gas_vof")

# flow maps
handle.dump_vector_from_scalar_liquid(
    out("flowL"),
    handle.variable("liquid_x_velocity"),
    handle.variable("liquid_y_velocity"),
    handle.variable("liquid_z_velocity"),
    GAS_FRACTION,
)
handle.dump_vector_from_scalar_gas(
    out("flowG"),
    handle.variable("gas_x_velocity"),
    handle.variable("gas_y_velocity"),
    handle.variable("gas_z_velocity"),
    GAS_FRACTION,
)

# volumes
handle.dump_scalar(out("vofG"), GAS_FRACTION)

gas_volume = np.array(data.read_rawscalar(out("vofG.raw")).data)
liquid_volume = handle.real_volume() - gas_volume
data.scalar_from_data(liquid_volume).write(out("vofL.raw"))

# turbulence
handle.dump_scalar_liquid(
    out("epsturb"), handle.variable("liquid_turb_diss_rate"), GAS_FRACTION
)
handle.dump_scalar_liquid(
    out("kturb"), handle.variable("liquid_turb_kinetic_energy"), GAS_FRACTION
)

# case
cma = case.make_cm_case(N_DIV, 0.0, "generated from python", False)
cma.add(case.CMExportType.LiquidFlow, "flowL.raw")
cma.add(case.CMExportType.GasFlow, "flowG.raw")
cma.add(case.CMExportType.LiquidVolume, "vofL.raw")
cma.add(case.CMExportType.GasVolume, "vofG.raw")
cma.add(case.CMExportType.EnergyDissipation, "epsturb.raw")
cma.add(case.CMExportType.Other, "kturb.raw")
cma.write(out("cma_case"))


state = data.get_transitioner(CASE_OUT).get_current()
epsilon = np.array(state.misc("energy_dissipation"))

print(f"case written in {CASE_OUT}")
print(
    f"  {len(liquid_volume)} compartments, liquid {liquid_volume.sum():.4g} m3, "
    f"gas {gas_volume.sum():.4g} m3"
)
print(
    f"  epsilon integral {epsilon.sum():.5g}, "
    f"specific {epsilon.sum() / liquid_volume.sum():.5g} W/kg"
)
