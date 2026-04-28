import pycmtool
import pycmtool.export_vtk

if __name__ == "__main__":
    vx = "out/cuve_sldmsh_initmrf/x_velocity.raw"
    vy = "out/cuve_sldmsh_initmrf/y_velocity.raw"
    vz = "out/cuve_sldmsh_initmrf/z_velocity.raw"
    scvx = pycmtool.data.read_rawscalar(vx)
    scvy = pycmtool.data.read_rawscalar(vy)
    scvz = pycmtool.data.read_rawscalar(vz)

    scvx_vtk = pycmtool.export_vtk.mk_scalar(scvz.data, "v_x")
    scvy_vtk = pycmtool.export_vtk.mk_scalar(scvz.data, "v_y")
    scvz_vtk = pycmtool.export_vtk.mk_scalar(scvz.data, "v_z")

    vtu_path = "out/cuve_sldmsh_initmrf/cma_case.vtu"
    pycmtool.export_vtk.append_scalar(
        vtu_path, "/tmp/test.vtu", scvx_vtk, scvx_vtk, scvy_vtk, scvz_vtk
    )
