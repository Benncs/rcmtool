mod case;
mod rawdata;

pub use case::{CCMCaseInfo, CMAExportType, CMCase, CMCaseReader, CMCaseWriter,CMCaseJson};
pub use rawdata::{
    FluxFileHeader, RawData, RawDataFlux, RawDataScalar, RawFlux, RawScalar, ScalarFileHeader,
    ScalarValueType,
};

#[inline(always)]
fn linear_index_row_major(_n_row: usize, n_col: usize, i: usize, j: usize) -> usize {
    i * n_col + j
}

#[inline(always)]
fn linear_index_col_major(n_row: usize, _n_col: usize, i: usize, j: usize) -> usize {
    j * n_row + i
}

#[derive(Clone, Copy)]
pub enum Layout {
    RowMajor,
    ColMajor,
}

pub struct LightView2D<'a> {
    non_owning_data: &'a mut [f64],
    n_row: usize,
    n_col: usize,
    layout: Layout,
    f: fn(n_row: usize, n_col: usize, i: usize, j: usize) -> usize,
}

impl<'a> LightView2D<'a> {
    pub fn new(data: &'a mut [f64], n_row: usize, n_col: usize, layout: Layout) -> Self {
        assert_eq!(
            data.len(),
            n_row * n_col,
            "Data length does not match the specified dimensions"
        );

        let f = match layout {
            Layout::RowMajor => linear_index_row_major,
            Layout::ColMajor => linear_index_col_major,
        };

        LightView2D {
            non_owning_data: data,
            n_row,
            n_col,
            layout,
            f,
        }
    }

    pub fn get_layout(&self) -> Layout {
        self.layout
    }

    pub fn get(&self, i: usize, j: usize) -> Option<f64> {
        if i >= self.n_row || j >= self.n_col {
            return None;
        }

        let index = (self.f)(self.n_row, self.n_col, i, j);

        Some(self.non_owning_data[index])
    }

    pub fn get_mut(&mut self, i: usize, j: usize) -> Option<&mut f64> {
        if i >= self.n_row || j >= self.n_col {
            return None;
        }

        let index = (self.f)(self.n_row, self.n_col, i, j);

        Some(&mut self.non_owning_data[index])
    }

    pub fn set(&mut self, i: usize, j: usize, value: f64) -> Result<(), &'static str> {
        if i >= self.n_row || j >= self.n_col {
            return Err("Index out of bounds");
        }

        let index = (self.f)(self.n_row, self.n_col, i, j);

        self.non_owning_data[index] = value;
        Ok(())
    }
}

struct FlowMap {
    internal_data: Vec<f64>,
    n_row: usize,
}

impl FlowMap {
    fn new(n_row: usize) -> Self {
        Self {
            internal_data: Vec::with_capacity(n_row * n_row),
            n_row,
        }
    }

    fn get_view(&mut self) -> LightView2D<'_> {
        LightView2D::new(
            &mut self.internal_data,
            self.n_row,
            self.n_row,
            Layout::RowMajor,
        )
    }
}

struct FlowMapDescriptor {
    flowmap: FlowMap,
    neighbors: Vec<Vec<usize>>, //TODO
}

impl FlowMapDescriptor {
    fn from_raw_data(data: &rawdata::RawDataFlux) -> Result<Self, ()> {
        let mut flowmap = FlowMap::new(data.header.n_zone as usize);

        let mut neighbors: Vec<Vec<usize>> = Vec::with_capacity(data.header.n_zone as usize);

        //Scope to drop the view
        {
            let mut view = flowmap.get_view();

            for rawflux in data.fluxes.iter() {
                let id_source = rawflux.id_source as usize;
                let id_target = rawflux.id_target as usize;

                if let Some(g) = view.get_mut(id_source, id_target) {
                    *g += rawflux.flux_source_target;
                }

                if let Some(g) = view.get_mut(id_target, id_source) {
                    *g += rawflux.flux_target_source;
                }

                neighbors[id_source].push(id_target);
                neighbors[id_target].push(id_source);
                //todo
            }
        }

        Ok(FlowMapDescriptor { flowmap, neighbors })
    }
}
