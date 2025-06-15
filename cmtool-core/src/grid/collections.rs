pub type Coords3 = [f64; 3];
pub type AxisPoints = [usize; 3];

pub enum OrientedAxis {
    I = 0,
    J = 1,
    K = 2,
}

pub enum CylindricalAxis {
    R = 0,
    Theta = 1,
    Z = 2,
}

pub const fn index(axis: CylindricalAxis) -> usize {
    match axis {
        CylindricalAxis::R => 0,
        CylindricalAxis::Theta => 1,
        CylindricalAxis::Z => 2,
    }
}

enum ProjectCoordinates {
    CartesianToCyclindrical,
    CyclindricalToCartesian,
    None,
}

pub struct AxisDescriptor {
    pub min_range: f64,
    pub max_range: f64,
    pub n_range: usize,
    pub step: f64,
}

pub struct CoordAxis {
    pub edges: Vec<f64>,
    pub centers: Vec<f64>,
    pub descriptor: AxisDescriptor,
    pub i_axis: usize,
}

impl CoordAxis {
    pub fn index_from_edge_value(&self, axis_value: f64) -> Option<usize> {
        let mut cell_id = 0;
        let n_point = self.edges.len();
        while (cell_id < n_point && self.edges[cell_id + 1] < axis_value) {
            cell_id += 1;
        }
        if (cell_id == n_point) {
            return None;
        }

        Some(cell_id)
    }
}
