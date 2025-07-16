import pycmtool

scalar = pycmtool.read_rawscalar("/home-local/casale/Documents/thesis/cfd-cma/cma_data/sanofi/raw/vofL.raw")

print(f"Read scalar with {scalar.n_zone} compartment")


fm = pycmtool.read_flowmap("/home-local/casale/Documents/thesis/cfd-cma/cma_data/sanofi/raw/flowL.raw")

print(fm)