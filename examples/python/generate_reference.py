import numpy as np
import pycmtool

if __name__ == "__main__":
    NX, NY, NZ = 3, 3, 2
    n_zone = NX * NY * NZ

    vol_tot = 10.0  # L

    def zone_weight(x, y, z):
        bx = x == 0 or x == NX - 1
        by = y == 0 or y == NY - 1
        bz = z == 0 or z == NZ - 1
        boundary_count = bx + by + bz
        return {3: 0.5, 2: 0.8, 1: 1.2, 0: 2.0}[boundary_count]

    def zone_id(x, y, z):
        return x + NX * y + NX * NY * z

    weights = np.array(
        [zone_weight(x, y, z) for z in range(NZ) for y in range(NY) for x in range(NX)]
    )
    # vol = vol_tot * weights / weights.sum()
    vol = vol_tot / n_zone * np.ones((n_zone))
    vol_array = pycmtool.data.scalar_from_data(vol)

    Q_xy = 0.5
    Q_z = 0.3

    flow_map = []

    for z in range(NZ):
        for y in range(NY):
            for x in range(NX):
                src = zone_id(x, y, z)
                if x + 1 < NX:
                    tgt = zone_id(x + 1, y, z)
                    flow_map.append(pycmtool.data.new_raw_flux(src, tgt, Q_xy, Q_xy))
                if y + 1 < NY:
                    tgt = zone_id(x, y + 1, z)
                    flow_map.append(pycmtool.data.new_raw_flux(src, tgt, Q_xy, Q_xy))
                if z + 1 < NZ:
                    tgt = zone_id(x, y, z + 1)
                    flow_map.append(pycmtool.data.new_raw_flux(src, tgt, Q_z, Q_z))

    fm = pycmtool.data.vector_from_data(n_zone, flow_map)

    fm.write("/tmp/fm/flowL.raw")
    vol_array.write("/tmp/fm/vofL.raw")

    print(vol_array.data)

    print(fm.n_zone, vol_array.n_zone)

    v = pycmtool.data.read_flowmap("/tmp/fm/flowL.raw", "/tmp/fm/vofL.raw")
    print(v.flowmap)
