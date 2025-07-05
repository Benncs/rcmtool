use std::fmt::Debug;
mod collections;
use collections::*;
pub use collections::{cylindrical_index, AxisDescriptor, Coords3, CylindricalAxis};

#[derive(PartialEq)]
pub enum MeshType {
    Cylindrical,
    MeshRectangular,
}

pub enum NeighborDirection {
    NotNeighbors = 0,
    XMinus = 1,
    XPlus = 2,
    YMinus = 3,
    YPlus = 4,
    ZMinus = 5,
    ZPlus = 6,
}

impl TryFrom<i32> for NeighborDirection {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NeighborDirection::NotNeighbors),
            1 => Ok(NeighborDirection::XMinus),
            2 => Ok(NeighborDirection::XPlus),
            3 => Ok(NeighborDirection::YMinus),
            4 => Ok(NeighborDirection::YPlus),
            5 => Ok(NeighborDirection::ZMinus),
            6 => Ok(NeighborDirection::ZPlus),
            _ => Err("Invalid integer value for NeighborDirection"),
        }
    }
}

pub trait CompartmentMeshAccessor {
    fn min_axis(&self, i_axis: usize) -> f64;

    fn max_axis(&self, i_axis: usize) -> f64;

    fn mesh_step_axis(&self, i_axis: usize) -> f64;

    fn n_points_axis(&self, i_axis: usize) -> usize;

    fn number_cell(&self) -> usize;

    fn get_cell_edge(&self, i_axis: usize, i_point: usize) -> f64;

    fn get_cell_center(&self, i_axis: usize, i_point: usize) -> f64;
}

pub trait CompartmentMeshManip {
    fn are_cell_neighbor(&self, cell1_id: usize, cell2_id: usize) -> NeighborDirection;
    fn cell_surface(&self, cell_id: usize) -> f64;
    fn cell_volume(&self, cell_id: usize) -> f64;
    fn cell_from_coordinates(&self, coords: &Coords3) -> Option<usize>;
    fn is_point_inside(&self, cell_id: usize, point_coords: &Coords3) -> bool;
    fn cell_points(&self, cell_1d: usize) -> AxisPoints;
}

pub trait CompartmentMesh:  CompartmentMeshAccessor + CompartmentMeshManip {}
impl<T: CompartmentMeshAccessor + CompartmentMeshManip> CompartmentMesh for T {}

pub struct CylindricalMarker;
pub struct RectangularMarker;


pub struct BaseCompartmentMesh<T> {
    axes: [CoordAxis; 3],
    n_cells: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T> CompartmentMeshAccessor for BaseCompartmentMesh<T> {
    fn min_axis(&self, i_axis: usize) -> f64 {
        self.axes[i_axis].descriptor.min_range
    }

    fn max_axis(&self, i_axis: usize) -> f64 {
        self.axes[i_axis].descriptor.max_range
    }

    fn mesh_step_axis(&self, i_axis: usize) -> f64 {
        self.axes[i_axis].descriptor.step
    }

    fn n_points_axis(&self, i_axis: usize) -> usize {
        self.axes[i_axis].descriptor.n_range
    }

    fn number_cell(&self) -> usize {
        self.n_cells
    }

    fn get_cell_edge(&self, i_axis: usize, i_point: usize) -> f64 {
        self.axes[i_axis].edges[i_point]
    }

    fn get_cell_center(&self, i_axis: usize, i_point: usize) -> f64 {
        self.axes[i_axis].centers[i_point]
    }
}

pub type MeshCylindrical = BaseCompartmentMesh<CylindricalMarker>;
pub type MeshRectangular = BaseCompartmentMesh<RectangularMarker>;

impl MeshCylindrical {
    fn new() -> Self {
        todo!()
    }
}

impl MeshRectangular {
    fn new() -> Self {
        todo!()
    }
}

impl CompartmentMeshManip for MeshCylindrical {
    fn are_cell_neighbor(&self, cell1_id: usize, cell2_id: usize) -> NeighborDirection {
        let indices_points_cell1 = self.cell_points(cell1_id);
        let indices_points_cell2 = self.cell_points(cell2_id);

        let mut max_diff = 0;
        let diff: Vec<i64> = indices_points_cell1
            .iter()
            .zip(indices_points_cell2)
            .map(|(i_p_c_1, i_p_c_2)| {
                let di = *i_p_c_1 as i64 - i_p_c_2 as i64;
                if i64::abs(di) > i64::abs(max_diff) {
                    max_diff = di;
                }
                di
            })
            .collect();

        let rmax = self.axes[cylindrical_index(CylindricalAxis::R)]
            .descriptor
            .n_range;

        let theta_cell1 = indices_points_cell1[cylindrical_index(CylindricalAxis::Theta)];
        let theta_cell2 = indices_points_cell2[cylindrical_index(CylindricalAxis::Theta)];
        let r_cell1 = indices_points_cell1[cylindrical_index(CylindricalAxis::R)];
        let r_cell2 = indices_points_cell2[cylindrical_index(CylindricalAxis::R)];

        if theta_cell1 == theta_cell2 {
            if r_cell1 == 0 && r_cell2 == rmax {
                return NeighborDirection::YMinus;
            }
            if r_cell1 == rmax && r_cell2 == 0 {
                return NeighborDirection::YPlus;
            }
        }

        if max_diff.abs() > 1 {
            return NeighborDirection::NotNeighbors;
        }

        if diff.iter().filter(|&&i| i.abs() > 1).count() > 1 {
            return NeighborDirection::NotNeighbors;
        }

        for (i, cd) in diff.iter().enumerate() {
            if *cd == 1 {
                return NeighborDirection::try_from((i as i32) * 2 + 1).unwrap();
            }
            if *cd == -1 {
                return NeighborDirection::try_from((i as i32) * 2 + 2).unwrap();
            }
        }

        NeighborDirection::NotNeighbors
    }

    fn cell_surface(&self, _cell_id: usize) -> f64 {
        todo!()
    }

    fn cell_volume(&self, cell_id: usize) -> f64 {
        todo!()
    }

    fn cell_from_coordinates(&self, coords: &Coords3) -> Option<usize> {
        let mut mesh_id = 0;
        let mut cumulative_product = 1;

        for (i, axe) in self.axes.as_ref().iter().enumerate() {
            let current_index = axe.index_from_edge_value(coords[i])?;
            mesh_id += current_index * cumulative_product;
            cumulative_product *= axe.descriptor.n_range;
        }
        Some(mesh_id)
    }

    fn is_point_inside(&self, cell_id: usize, point_coords: &Coords3) -> bool {
        todo!()
    }

    fn cell_points(&self, cell_1d: usize) -> AxisPoints {
        let mut axis_points = AxisPoints::default();

        let mut p_coeff_up = cell_1d;

        for (current_point, current_axis) in axis_points.iter_mut().zip(&self.axes) {
            let array_size = current_axis.descriptor.n_range;
            *current_point = p_coeff_up % array_size;
            p_coeff_up /= array_size;
        }

        axis_points
    }
}

pub fn get_mesh(meshtype: MeshType,axis:[AxisDescriptor;3]) -> Box<dyn CompartmentMesh> {
    match meshtype {
        MeshType::Cylindrical => Box::new(MeshCylindrical::new()),
        MeshType::MeshRectangular => unimplemented!("Manip for Rectangular impl"),
    }
}

