import numpy as np
import pycmtool


def generate_liq_volume():
    total_volume = pycmtool.data.read_rawscalar("./out/RESULTS/vtot.raw")
    vgas = pycmtool.data.read_rawscalar("./out/RESULTS/gas_vof.raw")
    v_liq = np.array(total_volume.data - vgas.data)
    sc = pycmtool.data.scalar_from_data(v_liq)
    sc.write("./out/RESULTS/liq_vof.raw")


if __name__ == "__main__":
    generate_liq_volume()
    # Check volumes
    total_volume = pycmtool.data.read_rawscalar("./out/RESULTS/vtot.raw")
    vgas = pycmtool.data.read_rawscalar("./out/RESULTS/gas_vof.raw")
    vliq = pycmtool.data.read_rawscalar("./out/RESULTS/liq_vof.raw")

    geometric_volume = np.pi * np.power(5.78824, 2) / 4 * 10.4736

    vtot = np.sum(vliq.data) + np.sum(vgas.data)
    print(
        geometric_volume,
        np.sum(vliq.data),
        np.sum(vgas.data),
        vtot,
        np.sum(total_volume.data),
    )
    assert np.abs(vtot - np.sum(total_volume.data)) < 1e-10

    vL = pycmtool.data.read_rawscalar("/tmp/sanofi/vofL.raw")
    vG = pycmtool.data.read_rawscalar("/tmp/sanofi/vofG.raw")
    vtot = np.sum(vL.data) + np.sum(vG.data)
    print(vtot, np.sum(vL.data), np.sum(vG.data))
