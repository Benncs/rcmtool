mod collections;
use std::{cell, default, f64};

use collections::*;
pub use collections::{AxisDescriptor, CylindricalAxis, cylindrical_index};

use crate::coordinates::*;
use crate::utils::AxisPoints;

#[derive(PartialEq, Clone, Copy)]
pub enum MeshType {
    Cylindrical,
    MeshRectangular,
}

#[derive(Debug, PartialEq, Clone, Copy)]
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
        if self.is_positive() { (a, b) } else { (b, a) }
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
    fn n_maximum_interface(&self) -> usize;
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
        #[cfg(debug_assertions)]
        {
            // Debug mode: safe indexing with bounds check
            self.axes[i_axis].edges[i_point]
        }

        #[cfg(not(debug_assertions))]
        unsafe {
            *self.axes.get_unchecked(i_axis).edges.get_unchecked(i_point)
        }
    }

    fn get_cell_center(&self, i_axis: usize, i_point: usize) -> f64 {
        self.axes[i_axis].centers[i_point]
    }
}

pub type MeshCylindrical = BaseCompartmentMesh<CylindricalMarker>;
pub type MeshRectangular = BaseCompartmentMesh<RectangularMarker>;

impl CompartmentMeshManip for MeshCylindrical {
    fn n_maximum_interface(&self) -> usize {
        let nr = self.axes[0].descriptor.n_range ;
        let ntheta = self.axes[1].descriptor.n_range ;
        let nz = self.axes[2].descriptor.n_range ;

        let interfaces_r = (nr-1) * ntheta * nz;
        let interfaces_theta = nr * (ntheta - 1) * nz;
        let interfaces_z = nr * ntheta * (nz - 1);

        let wrap = nr * nz; //Conexion between theta=-pi and theta=pi 


        interfaces_r + interfaces_theta + interfaces_z + wrap
    }

    fn are_cell_neighbor(&self, cell1_id: usize, cell2_id: usize) -> NeighborDirection {
        let [r1, theta1, z1] = self.cell_points(cell1_id);
        let [r2, theta2, z2] = self.cell_points(cell2_id);

        let theta_divs = self.axes[1].descriptor.n_range as isize;

        let r1 = r1 as isize;
        let r2 = r2 as isize;
        let z1 = z1 as isize;
        let z2 = z2 as isize;
        let t1 = theta1 as isize;
        let t2 = theta2 as isize;

        let dr = r2 - r1;
        let dz = z2 - z1;

        let delta_theta = (t2 - t1 + theta_divs) % theta_divs;

        let dtheta: isize = if delta_theta == 1 {
            1
        } else if delta_theta == theta_divs - 1 {
            -1
        } else if delta_theta == 0 {
            0
        } else {
            2
        };

        let distance = dr.abs() + dtheta.abs() + dz.abs();

        if distance != 1 {
            return NeighborDirection::NotNeighbors;
        }

        match (dr, dtheta, dz) {
            (1, 0, 0) => NeighborDirection::XPlus,
            (-1, 0, 0) => NeighborDirection::XMinus,
            (0, 1, 0) => NeighborDirection::YPlus,
            (0, -1, 0) => NeighborDirection::YMinus,
            (0, 0, 1) => NeighborDirection::ZPlus,
            (0, 0, -1) => NeighborDirection::ZMinus,
            _ => NeighborDirection::NotNeighbors,
        }
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
    fn t_identification() {
        let mesh = ref_mesh_cyclindrical();

        let assert_id = |a: Coords3, expect: usize| {
            let aa = a.cylindrical_to_cartesian();

            let id1 = mesh
                .cell_from_coordinates(&aa)
                .expect("Test neighbors: coordinates for cell a are outside the mesh.");

            assert!(
                id1 == expect,
                "Assertion failed: expected {:?}, got {:?}",
                expect,
                id1
            );
        };

        assert_id([0., -std::f64::consts::PI, 0.], 0);

        assert_id([0., -std::f64::consts::PI, max_ax3], number_point_ax3 - 1);
        let theta = -std::f64::consts::PI + mesh.mesh_step_axis(1) * 1.1;
        //R!=0 because with cartesian conversion is x=rcos(theta) if theta changes but no r its the same compartment
        assert_id([0.01, theta, 0.], number_point_ax3);
        //-1 because we consider cell ID for 0 to n-1
        assert_id(
            [max_ax1, std::f64::consts::PI, max_ax3],
            (number_point_ax3 * number_point_ax1 * number_point_ax2) - 1,
        );
    }

    #[test]
    fn t_neighbors() {
        use NeighborDirection::*;

        let mesh = ref_mesh_cyclindrical();

        let assert_neighbors =
            |a: CylindricalCoordinates, b: CylindricalCoordinates, expected: NeighborDirection| {
                let CartesianCoordinates(aa) = a.into();

                let CartesianCoordinates(bb) = b.into();

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
            CylindricalCoordinates([fix_i, fix_j, fix_k]),
            CylindricalCoordinates([fix_i + offset_i, fix_j, fix_k]),
            XPlus,
        );
        assert_neighbors(
            CylindricalCoordinates([fix_i + offset_i, fix_j, fix_k]),
            CylindricalCoordinates([fix_i, fix_j, fix_k]),
            XMinus,
        );

        // Testing in Y direction
        assert_neighbors(
            CylindricalCoordinates([fix_i, fix_j + offset_j, fix_k]),
            CylindricalCoordinates([fix_i, fix_j, fix_k]),
            YMinus,
        );
        assert_neighbors(
            CylindricalCoordinates([fix_i, fix_j, fix_k]),
            CylindricalCoordinates([fix_i, fix_j + offset_j, fix_k]),
            YPlus,
        );

        // Testing in Z direction
        assert_neighbors(
            CylindricalCoordinates([fix_i, fix_j, fix_k + offset_k]),
            CylindricalCoordinates([fix_i, fix_j, fix_k]),
            ZMinus,
        );
        assert_neighbors(
            CylindricalCoordinates([fix_i, fix_j, fix_k]),
            CylindricalCoordinates([fix_i, fix_j, fix_k + offset_k]),
            ZPlus,
        );

        // Testing non-neighbor cases
        assert_neighbors(
            CylindricalCoordinates([0., fix_j, fix_k + offset_k]),
            CylindricalCoordinates([fix_i, fix_j, fix_k]),
            NotNeighbors,
        );
        assert_neighbors(
            CylindricalCoordinates([fix_i, fix_j, fix_k]),
            CylindricalCoordinates([0., fix_j, fix_k + offset_k]),
            NotNeighbors,
        );
        let little_offset = mesh.mesh_step_axis(0) * 1.005;
        let c1: f64 = mesh.get_cell_edge(0, 2);
        assert_neighbors(
            CylindricalCoordinates([c1, fix_j, fix_k]),
            CylindricalCoordinates([c1 + little_offset, fix_j, fix_k]),
            NotNeighbors,
        );
    }
}
