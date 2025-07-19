import pycmtool

# scalar = pycmtool.read_rawscalar("/home-local/casale/Documents/thesis/cfd-cma/cma_data/sanofi/raw/vofL.raw")

# print(f"Read scalar with {scalar.n_zone} compartment")


import numpy as np


def get(path: str):
    # fm = pycmtool.read_flowmap(
    #    "/home/benjamin/Documents/code/rust/rcmtool/out/cuve_sldmsh_initmrf/velocity.raw"
    # )
    fm = pycmtool.read_flowmap(path)
    f = fm.flowmap  # borrow again, Python still holds the array (data mut)
    return f


cpp_ref = "/home/benjamin/Documents/code/cpp/compartment-modelling-tool/cuve_sldmsh_initmrf.vel.vector-raw"
rust = "/home/benjamin/Documents/code/rust/rcmtool/out/cuve_sldmsh_initmrf/velocity.raw"

f = get(cpp_ref)
inf = np.sum(f, axis=1)
out = np.sum(f, axis=0)

n = np.random.randint(0,f.shape[0],5)


print(inf[0:5])
print(out[0:5])
f = get(rust)
inf = np.sum(f, axis=1)
out = np.sum(f, axis=0)
print(inf[0:5])
print(out[0:5])
