import pycmtool
import numpy as np
from scipy.integrate import solve_ivp
import matplotlib.pyplot as plt


def get(path: str, pathvol: str):
    # fm = pycmtool.read_flowmap(
    #    "/home/benjamin/Documents/code/rust/rcmtool/out/cuve_sldmsh_initmrf/velocity.raw"
    # )
    fm = pycmtool.read_flowmap(path, pathvol)
    f = fm.flowmap  # borrow again, Python still holds the array (data mut)
    return f, fm.volumes


cpp_ref = "/home/benjamin/Documents/code/cpp/compartment-modelling-tool/cuve_sldmsh_initmrf.vel.vector-raw"
# rust = "/home/benjamin/Documents/code/rust/rcmtool/out/cuve_sldmsh_initmrf/velocity.raw"
# rust_volume = "/home/benjamin/Documents/code/rust/rcmtool/out/cuve_sldmsh_initmrf/total_volume.raw"
#
rust = "/home/benjamin/Documents/thesis/cfd-cma/sanofi/raw/flowL.raw"
rust_volume = "/home/benjamin/Documents/thesis/cfd-cma/sanofi/raw/vofL.raw"


def get_transition_matrix(flows):
    transition = flows.copy()

    row_sums = transition.sum(axis=1)

    np.fill_diagonal(transition, 0.0)

    np.fill_diagonal(transition, -row_sums)

    return transition


def check_mixing(path, path2):
    f, vol = get(path, path2)
    n_c = f.shape[0]
    M = get_transition_matrix(f)
    print(M)
    C = np.zeros((1, n_c))
    C[0, 0] = 1

    def wrap(t, x):
        _mass = x.reshape((1, n_c))
        C = _mass / vol
        return C @ M

    m0 = C * vol

    sol = solve_ivp(wrap, (0, 5000), m0.reshape(-1), method="LSODA")
    y = sol.y.reshape((n_c, -1))
    y = sol.y.reshape((n_c, -1))

    m0_c = y[:, 0]

    mt_c = y[:, -1]

    print("Mass init :", np.sum(m0_c))
    print("Mass final :", np.sum(mt_c))

    c_init = (m0_c / vol) / np.mean(m0_c / vol)
    c_final = (mt_c / vol) / np.mean(mt_c / vol)

    print("Profil initial:", c_init[:5])
    print("Profil final:", c_final[:5])
    print(np.var(c_final))

    plt.plot(c_final)
    plt.show()


check_mixing(rust, rust_volume)
