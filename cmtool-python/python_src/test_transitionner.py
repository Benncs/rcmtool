import pycmtool
import numpy as np
from scipy.integrate import solve_ivp
import matplotlib.pyplot as plt
import scipy.sparse


def create_sparse_array(row_indices, col_indices, values, shape=None):
    if shape is None:
        max_row = max(row_indices) if row_indices else 0
        max_col = max(col_indices) if col_indices else 0
        shape = (max_row + 1, max_col + 1)

    sparse_array = scipy.sparse.coo_array(
        (values, (row_indices, col_indices)),
        shape=shape,
        copy=False,
    )
    return sparse_array


def check_mixing(fmt):
    it = fmt.get_at(0)
    M = create_sparse_array(*it.flowmap)
    n_c = M.shape[0]
    C = np.zeros((1, n_c))
    C[0, 0] = 1
    vol = it.volumes
    it = fmt.get_at(19)
    vol2 = it.volumes
    m0 = C * vol

    def wrap(t, x):
        d_t = 0  # Not used for this transitionner
        it = fmt.advance(t, d_t)
        M = create_sparse_array(*it.flowmap)
        vol = it.volumes
        _mass = x.reshape((1, n_c))
        C = _mass / vol
        return C @ M

    sol = solve_ivp(wrap, (0, 50), m0.reshape(-1), method="LSODA")
    y = sol.y.reshape((n_c, -1))
    y = sol.y.reshape((n_c, -1))

    m0_c = y[:, 0]

    mt_c = y[:, -1]

    print("Mass init :", np.sum(m0_c))
    print("Mass final :", np.sum(mt_c))

    c_init = (m0_c / vol) / np.mean(m0_c / vol)
    c_final = (mt_c / vol2) / np.mean(mt_c / vol2)

    print("Profil initial:", c_init[:5])
    print("Profil final:", c_final[:5])
    print(np.var(c_final))

    plt.plot(c_final)
    plt.show()


# root = "/home/benjamin/Documents/thesis/cfd-cma/cma_data/sanofi/"
root = "/home/benjamin/Documents/thesis/cfd-cma/cma_data/b20l/"
fmt = pycmtool.get_transitionner(root, f"{root}/cma_case")
check_mixing(fmt)
