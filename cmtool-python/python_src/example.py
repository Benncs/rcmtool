import pycmtool

# scalar = pycmtool.read_rawscalar("/home-local/casale/Documents/thesis/cfd-cma/cma_data/sanofi/raw/vofL.raw")

# print(f"Read scalar with {scalar.n_zone} compartment")



import gc

def get():
    fm = pycmtool.read_flowmap("/home/benjamin/Documents/code/rust/rcmtool/out/flowL.raw")  
    fm.flowmap[0,0] = 1    # mutates data via NumPy view in Python (no Rust checks)
    
    f = fm.flowmap          # borrow again, Python still holds the array (data mut)
    del fm                  # Rust object drops but Python still references underlying buffer -> undefined behavior possible!
      
    return f                        

f = get()
print(f)  
print(f.base)   



   

