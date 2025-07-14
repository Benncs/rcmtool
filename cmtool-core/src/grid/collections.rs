///Represent axis absolution direction in a direct-orientied coordinate system
pub enum OrientedAxis {
    I = 0,
    J = 1,
    K = 2,
}

///Represent axis direction in cylindrical coordinates
pub enum CylindricalAxis {
    R = OrientedAxis::I as isize,
    Theta = OrientedAxis::J as isize,
    Z = OrientedAxis::K as isize,
}

///Represent axis direction in cartesian coordinates
pub enum CartesianAxis {
    X = OrientedAxis::I as isize,
    Y = OrientedAxis::J as isize,
    Z = OrientedAxis::K as isize,
}

impl From<CartesianAxis> for usize {
    fn from(axis: CartesianAxis) -> usize {
        axis as usize
    }
}
impl From<CylindricalAxis> for usize {
    fn from(axis: CylindricalAxis) -> usize {
        axis as usize
    }
}
impl From<OrientedAxis> for usize {
    fn from(axis: OrientedAxis) -> usize {
        axis as usize
    }
}

#[inline(always)]
pub const fn cylindrical_index(axis: CylindricalAxis) -> usize {
    match axis {
        CylindricalAxis::R => 0,
        CylindricalAxis::Theta => 1,
        CylindricalAxis::Z => 2,
    }
}

#[inline(always)]
pub const fn index_to_oriented(axis: usize) -> OrientedAxis {
    match axis {
        0 => OrientedAxis::I,
        1 => OrientedAxis::J,
        2 => OrientedAxis::K,
        _ => panic!("index_to_oriented needs index between 0 and 3"),
    }
}

#[derive(Default)]
pub struct AxisDescriptor {
    pub min_range: f64,
    pub max_range: f64,
    pub n_range: usize,
    pub step: f64,
}

impl AxisDescriptor {
    pub fn new(min_range: f64, max_range: f64, n_range: usize) -> Self {
        Self {
            min_range,
            max_range,
            n_range,
            step: 0.,
        }
    }
}

pub struct CoordAxis {
    pub edges: Vec<f64>,
    pub centers: Vec<f64>,
    pub descriptor: AxisDescriptor,
    // pub i_axis: usize,
}

impl From<AxisDescriptor> for CoordAxis {
    fn from(mut descriptor: AxisDescriptor) -> Self {
        const STEP_OFFSET: f64 = 0.5;
        let mut edges = Vec::with_capacity(descriptor.n_range + 1);
        let mut centers = Vec::with_capacity(descriptor.n_range);

        descriptor.step =
            (descriptor.max_range - descriptor.min_range) / (descriptor.n_range as f64);

        let mut i_point = 0;
        while i_point < descriptor.n_range {
            let f_i_point = i_point as f64;
            centers.push(descriptor.min_range + descriptor.step * (f_i_point + STEP_OFFSET));

            edges.push(descriptor.min_range + descriptor.step * f_i_point);
            i_point += 1;
        }
        let f_i_point = i_point as f64;
        edges.push(descriptor.min_range + descriptor.step * f_i_point);

        Self {
            edges,
            centers,
            descriptor,
        }
    }
}

impl CoordAxis {
    pub fn index_from_edge_value(&self, axis_value: f64) -> Option<usize> {
        let mut cell_id = 0;
        let n_point = self.descriptor.n_range;
        while (cell_id < n_point) && (self.edges[cell_id + 1] < axis_value) {
            cell_id += 1;
        }
        if cell_id == n_point {
            return None;
        }

        Some(cell_id)
    }
}
