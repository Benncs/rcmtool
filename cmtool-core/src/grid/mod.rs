mod collections;
use std::{cell, default, f64};

use collections::*;
pub use collections::{cylindrical_index, AxisDescriptor, CylindricalAxis};

use crate::utils::{AxisPoints, Coords3, Coords3Ext};

#[derive(PartialEq, Clone, Copy)]
pub enum MeshType {
    Cylindrical,
    MeshRectangular,
}

#[derive(Debug, PartialEq)]
pub enum NeighborDirection {
    NotNeighbors = 0,
    XMinus = 1,
    XPlus = 2,
    YMinus = 3,
    YPlus = 4,
    ZMinus = 5,
    ZPlus = 6,
}

impl NeighborDirection {
    pub const fn is_positive(self) -> bool {
        matches!(
            self,
            NeighborDirection::XPlus | NeighborDirection::YPlus | NeighborDirection::ZPlus
        )
    }

    pub const fn is_negative(self) -> bool {
        matches!(
            self,
            NeighborDirection::XMinus | NeighborDirection::YMinus | NeighborDirection::ZMinus
        )
    }

    pub fn ordered_pair<T: Copy>(self, a: T, b: T) -> (T, T) {
        if self.is_positive() {
            (a, b)
        } else {
            (b, a)
        }
    }

    pub fn to_coord_index(&self) -> Option<usize> {
        match self {
            Self::XMinus | Self::XPlus => Some(0),
            Self::YMinus | Self::YPlus => Some(1),
            Self::ZMinus | Self::ZPlus => Some(2),
            Self::NotNeighbors => None,
        }
    }
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

pub trait CompartmentMesh: Send + Sync + CompartmentMeshAccessor + CompartmentMeshManip {}
impl<T: CompartmentMeshAccessor + CompartmentMeshManip + Send + Sync> CompartmentMesh for T {}

pub struct CylindricalMarker;
pub struct RectangularMarker;

pub struct BaseCompartmentMesh<T> {
    axes: [CoordAxis; 3],
    n_cells: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T> BaseCompartmentMesh<T> {
    fn new(descriptors: [AxisDescriptor; 3]) -> Self {
        let axes: [CoordAxis; 3] = descriptors.map(CoordAxis::from);

        let n_cells = axes
            .iter()
            .map(|ax| ax.descriptor.n_range)
            .product::<usize>();
        Self {
            axes,
            n_cells,
            _marker: std::marker::PhantomData,
        }
    }
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

        //TODO impove this, namely order or condiions
        if max_diff.abs() > 1 {
            return NeighborDirection::NotNeighbors;
        }
        if diff.iter().filter(|&&i| i.abs() > 1).count() > 1 {
            return NeighborDirection::NotNeighbors;
        }

        if theta_cell1 == theta_cell2 {
            if r_cell1 == 0 && r_cell2 == rmax {
                return NeighborDirection::YMinus;
            }
            if r_cell1 == rmax && r_cell2 == 0 {
                return NeighborDirection::YPlus;
            }
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
        let cylindrical_coords = coords.cartesian_to_cylindrical();

        // for (i, axe) in self.axes.as_ref().iter().enumerate() {
        //     let current_index = axe.index_from_edge_value(cylindrical_coords[i])?;
        //     mesh_id += current_index * cumulative_product;
        //     cumulative_product *= axe.descriptor.n_range;
        // }

        for i in (0..self.axes.len()).rev() {
            let axe = &self.axes[i];
            let current_index = axe.index_from_edge_value(cylindrical_coords[i])?;
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

        // for (current_point, current_axis) in axis_points.iter_mut().zip(&self.axes) {
        //     let array_size = current_axis.descriptor.n_range;
        //     *current_point = p_coeff_up % array_size;
        //     p_coeff_up /= array_size;
        // }

        for i in (0..self.axes.len()).rev() {
            let axe = &self.axes[i];
            let array_size = axe.descriptor.n_range;
            axis_points[i] = p_coeff_up % array_size;
            p_coeff_up /= array_size;
        }

        axis_points
    }
}

pub fn get_mesh(
    meshtype: MeshType,
    mut ax_descriptor: [AxisDescriptor; 3],
) -> Box<dyn CompartmentMesh> {
    match meshtype {
        MeshType::Cylindrical => {
            ax_descriptor[0].min_range = 0.;
            ax_descriptor[1].min_range = -std::f64::consts::PI;
            ax_descriptor[1].max_range = std::f64::consts::PI;
            Box::new(MeshCylindrical::new(ax_descriptor))
        }
        MeshType::MeshRectangular => unimplemented!("Manip for Rectangular impl"),
    }
}

#[cfg(test)]
mod test {
    use std::process::id;

    use super::*;

    const number_point_ax1: usize = 8;
    const max_ax1: f64 = 4.;

    const number_point_ax2: usize = 5;
    const max_ax2: f64 = 2.;

    const number_point_ax3: usize = 10;
    const max_ax3: f64 = 10.;

    fn ref_mesh_cyclindrical() -> Box<dyn CompartmentMesh> {
        let ax1 = AxisDescriptor::new(0., max_ax1, number_point_ax1);
        let ax2 = AxisDescriptor::new(
            -std::f64::consts::PI,
            std::f64::consts::PI,
            number_point_ax2,
        );
        let ax3 = AxisDescriptor::new(0., max_ax3, number_point_ax3);
        get_mesh(MeshType::Cylindrical, [ax1, ax2, ax3])
    }

    #[test]
    fn t_get_mesh() {
        let ax1 = AxisDescriptor::new(0.5, 1., 10);
        let ax2 = AxisDescriptor::new(-std::f64::consts::PI, std::f64::consts::PI, 20);
        let ax3 = AxisDescriptor::new(0., 2., 15);
        let expected_step_x = 1. / 10.; //Cylindrical start ax from 0 to max_range
        let mesh = get_mesh(MeshType::Cylindrical, [ax1, ax2, ax3]);
        assert!(
            mesh.mesh_step_axis(0) == expected_step_x,
            "{} {}",
            mesh.mesh_step_axis(0),
            expected_step_x
        );
    }

    #[test]
    fn t_getter() {
        let mesh = ref_mesh_cyclindrical();
        assert!(mesh.max_axis(0) == max_ax1);
        assert!(mesh.max_axis(1) == std::f64::consts::PI);
        assert!(mesh.max_axis(2) == max_ax3);

        assert!(mesh.min_axis(0) == 0.);
        assert!(mesh.min_axis(1) == -std::f64::consts::PI);

        assert!(mesh.n_points_axis(0) == number_point_ax1);
        assert!(mesh.n_points_axis(1) == number_point_ax2);
        assert!(mesh.n_points_axis(2) == number_point_ax3);
        assert!(mesh.number_cell() == number_point_ax1 * number_point_ax2 * number_point_ax3);
    }

    #[test]
    fn t_neighbors() {
        use NeighborDirection::*;

        let mesh = ref_mesh_cyclindrical();

        let assert_neighbors = |a: Coords3, b: Coords3, expected: NeighborDirection| {

            let aa = a.cylindrical_to_cartesian();
            let bb = b.cylindrical_to_cartesian();

            let id1 = mesh
                .cell_from_coordinates(&aa)
                .expect("Test neighbors: coordinates for cell a are outside the mesh.");
            let id2 = mesh
                .cell_from_coordinates(&bb)
                .expect("Test neighbors: coordinates for cell b are outside the mesh.");

            let neighbors = mesh.are_cell_neighbor(id1, id2);
            assert!(
                neighbors == expected,
                "Assertion failed: expected {:?}, got {:?}",
                expected,
                neighbors
            );
        };

        // Fixed coordinates and offsets for testing
        let fix_i = 2.2;
        let offset_i = mesh.mesh_step_axis(0);
        let fix_j = 0.;
        let offset_j = 0.68;
        let fix_k = 4.0;
        let offset_k = 0.9;

        // Assert neighbor relationships in different directions

        // Testing in X direction
        assert_neighbors(
            [fix_i, fix_j, fix_k],
            [fix_i + offset_i, fix_j, fix_k],
            XPlus,
        );
        assert_neighbors(
            [fix_i + offset_i, fix_j, fix_k],
            [fix_i, fix_j, fix_k],
            XMinus,
        );

        // Testing in Y direction
        assert_neighbors(
            [fix_i, fix_j + offset_j, fix_k],
            [fix_i, fix_j, fix_k],
            YMinus,
        );
        assert_neighbors(
            [fix_i, fix_j, fix_k],
            [fix_i, fix_j + offset_j, fix_k],
            YPlus,
        );

        // Testing in Z direction
        assert_neighbors(
            [fix_i, fix_j, fix_k + offset_k],
            [fix_i, fix_j, fix_k],
            ZMinus,
        );
        assert_neighbors(
            [fix_i, fix_j, fix_k],
            [fix_i, fix_j, fix_k + offset_k],
            ZPlus,
        );

        // Testing non-neighbor cases
        assert_neighbors(
            [0., fix_j, fix_k + offset_k],
            [fix_i, fix_j, fix_k],
            NotNeighbors,
        );
        assert_neighbors(
            [fix_i, fix_j, fix_k],
            [0., fix_j, fix_k + offset_k],
            NotNeighbors,
        );
        let little_offset = mesh.mesh_step_axis(0)*1.005;
        let c1: f64 = mesh.get_cell_edge(0, 2);
        assert_neighbors(
            [c1, fix_j, fix_k],
            [c1+little_offset, fix_j, fix_k],
            NotNeighbors,
        );
    }
}
