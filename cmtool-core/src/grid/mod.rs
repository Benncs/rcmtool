// SPDX-License-Identifier: GPL-3.0-or-later

mod collections;
use collections::*;
pub use collections::{
    AxisDescriptor, CylindricalAxis, OrientedAxis, cylindrical_index, index_to_oriented,
};
use enum_dispatch::enum_dispatch;
use std::f64;

#[cfg(feature = "use_vtk")]
pub(crate) mod vtk;

use crate::coordinates::*;
use crate::utils::AxisPoints;

// fn get_tangent_plane_at_r(
//     axis: usize,
//     r0: f64,
//     theta: f64,
//     z: f64,
//     extent_u: [f64; 2],
//     extent_v: [f64; 2],
// ) -> BoundedPlane {
//     let x0 = r0 * theta.cos();
//     let y0 = r0 * theta.sin();
//     let z0 = z;

//     let normal = CartesianVec3([x0 / r0, y0 / r0, 0.0]);

//     let origin = CartesianCoordinates([x0, y0, z0]);

//     BoundedPlane {
//         normal,
//         origin,
//         extent_u,
//         extent_v,
//         axis,
//     }
// }
//
// fn get_tangent_plane_at_r(
//     axis: usize,
//     r0: f64,
//     theta: f64,
//     z: f64,
//     extent_u: [f64; 2], // [theta0, theta1] — will be converted to arc length
//     extent_v: [f64; 2], // [z0, z1] — already metric
// ) -> BoundedPlane {
//     let origin = CartesianCoordinates([r0 * theta.cos(), r0 * theta.sin(), z]);
//     let normal = CartesianVec3([theta.cos(), theta.sin(), 0.0]);

//     let extent_u_metric = [r0 * extent_u[0], r0 * extent_u[1]];

//     BoundedPlane {
//         normal,
//         origin,
//         extent_u: extent_u_metric,
//         extent_v, // z is already metric
//         axis,
//     }
// }

/// Represents the type of mesh geometry.
#[derive(PartialEq, Clone, Copy)]
pub enum MeshType {
    /// A cylindrical mesh type.
    Cylindrical,
    /// A rectangular mesh type.
    Rectangular,
}

/// Represents the direction of a neighboring cell relative to a given cell in a 3D grid.
///
/// This enum is used to indicate the spatial relationship between cells in a grid
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NeighborDirection {
    /// Indicates that the cells are not neighbors.
    NotNeighbors = 0,
    /// Indicates the neighbor is in the negative X direction.
    XMinus = 1,
    /// Indicates the neighbor is in the positive X direction.
    XPlus = 2,
    /// Indicates the neighbor is in the negative Y direction.
    YMinus = 3,
    /// Indicates the neighbor is in the positive Y direction.
    YPlus = 4,
    /// Indicates the neighbor is in the negative Z direction.
    ZMinus = 5,
    /// Indicates the neighbor is in the positive Z direction.
    ZPlus = 6,
}

impl NeighborDirection {
    /// Checks if the direction is positive.
    ///
    /// Returns `true` if the direction is positive (XPlus, YPlus, ZPlus),
    /// otherwise returns `false`.
    pub const fn is_positive(self) -> bool {
        matches!(
            self,
            NeighborDirection::XPlus | NeighborDirection::YPlus | NeighborDirection::ZPlus
        )
    }

    /// Checks if the direction is negative.
    ///
    /// Returns `true` if the direction is negative (XMinus, YMinus, ZMinus),
    /// otherwise returns `false`.
    pub const fn is_negative(self) -> bool {
        matches!(
            self,
            NeighborDirection::XMinus | NeighborDirection::YMinus | NeighborDirection::ZMinus
        )
    }

    /// Returns an ordered pair of values based on the direction's positivity.
    ///
    /// If the direction is positive, returns (a, b). Otherwise, returns (b, a).
    pub fn ordered_pair<T: Copy>(self, a: T, b: T) -> (T, T) {
        if self.is_positive() { (a, b) } else { (b, a) }
    }

    /// Converts the direction into a coordinate index.
    ///
    /// Returns `Some(usize)` representing the index of the coordinate (0 for X axis,
    /// 1 for Y axis, 2 for Z axis), or `None` if the variant is `NotNeighbors`.
    pub fn to_coord_index(&self) -> Option<usize> {
        match self {
            Self::XMinus | Self::XPlus => Some(0),
            Self::YMinus | Self::YPlus => Some(1),
            Self::ZMinus | Self::ZPlus => Some(2),
            Self::NotNeighbors => None,
        }
    }
}

// impl TryFrom<i32> for NeighborDirection {
//     type Error = &'static str;

//     fn try_from(value: i32) -> Result<Self, Self::Error> {
//         match value {
//             0 => Ok(NeighborDirection::NotNeighbors),
//             1 => Ok(NeighborDirection::XMinus),
//             2 => Ok(NeighborDirection::XPlus),
//             3 => Ok(NeighborDirection::YMinus),
//             4 => Ok(NeighborDirection::YPlus),
//             5 => Ok(NeighborDirection::ZMinus),
//             6 => Ok(NeighborDirection::ZPlus),
//             _ => Err("Invalid integer value for NeighborDirection"),
//         }
//     }
// }

/// Trait for accessing mesh data in a coarsed-mesh model.
///
/// Defines methods to access various properties of a coarsed mesh (structured grid),
pub trait CompartmentMeshAccessor {
    /// Returns the minimum value along the specified axis.
    ///
    /// # Arguments
    ///
    /// * `i_axis` - The index of the axis
    ///
    /// # Returns
    ///
    /// The minimum value along the specified axis.
    fn min_axis(&self, i_axis: usize) -> f64;

    /// Returns the maximum value along the specified axis.
    ///
    /// # Arguments
    ///
    /// * `i_axis` - The index of the axis
    ///
    /// # Returns
    ///
    /// The maximum value along the specified axis.
    fn max_axis(&self, i_axis: usize) -> f64;

    /// Returns the step size between points along the specified axis.
    ///
    /// # Arguments
    ///
    /// * `i_axis` - The index of the axis
    ///
    /// # Returns
    ///
    /// The step size between points along the specified axis.
    fn mesh_step_axis(&self, i_axis: usize) -> f64;

    /// Returns the number of points along the specified axis.
    ///
    /// # Arguments
    ///
    /// * `i_axis` - The index of the axis
    ///
    /// # Returns
    ///
    /// The number of points along the specified axis.
    fn n_points_axis(&self, i_axis: usize) -> usize;

    /// Returns the total number of cells in the mesh.
    ///
    /// # Returns
    ///
    /// The total number of cells.
    fn number_cell(&self) -> usize;

    /// Returns the edge position of a specific cell along the specified axis and point index.
    ///
    /// # Arguments
    ///
    /// * `i_axis` - The index of the axis.
    /// * `i_point` - The index of the point along the specified axis.
    ///
    /// # Returns
    ///
    /// The edge position of the specified cell.
    fn get_cell_edge(&self, i_axis: usize, i_point: usize) -> f64;

    /// Returns the center position of a specific cell along the specified axis and point index.
    ///
    /// # Arguments
    ///
    /// * `i_axis` - The index of the axis.
    /// * `i_point` - The index of the point along the specified axis.
    ///
    /// # Returns
    ///
    /// The center position of the specified cell.
    fn get_cell_center(&self, i_axis: usize, i_point: usize) -> f64;
}

/// Trait for manipulating and querying properties of cells in a compartment mesh.
pub trait CompartmentMeshManip {
    /// Determines if two cells are neighbors and returns the direction of neighborhood.
    ///
    /// # Arguments
    ///
    /// * `cell1_id` - The ID of the first cell.
    /// * `cell2_id` - The ID of the second cell.
    ///
    /// # Returns
    ///
    /// A `NeighborDirection` indicating whether and how the cells are neighbors.
    fn are_cell_neighbor(&self, cell1_id: usize, cell2_id: usize) -> NeighborDirection;

    /// Computes the surface area of a specified cell.
    ///
    /// # Arguments
    ///
    /// * `cell_id` - The ID of the cell.
    /// * `axis_project` - Axis index on which the surface is calculated
    ///
    /// # Returns
    ///
    /// The surface area of the cell as a floating-point number.
    fn cell_surface(&self, cell_id: usize, i_axis: OrientedAxis) -> f64;

    /// Computes the volume of a specified cell.
    ///
    /// # Arguments
    ///
    /// * `cell_id` - The ID of the cell.
    ///
    /// # Returns
    ///
    /// The volume of the cell as a floating-point number.
    fn cell_volume(&self, cell_id: usize) -> f64;

    /// Finds the cell ID corresponding to specific 3D coordinates.
    ///
    /// # Arguments
    ///
    /// * `coords` - A reference to a `Coords3` representing the 3D coordinates.
    ///
    /// # Returns
    ///
    /// An `Option<usize>` containing the cell ID if found, or `None` otherwise.
    fn cell_from_coordinates(&self, coords: &Coords3) -> Option<usize>;

    /// Checks if a point is inside a specified cell.
    ///
    /// # Arguments
    ///
    /// * `cell_id` - The ID of the cell to check.
    /// * `point_coords` - A reference to a `Coords3` representing the point's coordinates.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the point is inside the cell.
    fn is_point_inside(&self, cell_id: usize, point_coords: &Coords3) -> bool;

    /// Retrieves the points index defining a specified cell.
    ///
    /// # Arguments
    ///
    /// * `cell_1d` - The ID of the cell.
    ///
    /// # Returns
    ///
    /// An `AxisPoints` object containing the indices of points of the cell.
    fn cell_points(&self, cell_1d: usize) -> AxisPoints;

    /// Returns the maximum number of interfaces a cell can have in this mesh.
    ///
    /// # Returns
    ///
    /// The maximum number of interfaces as a `usize`.
    fn n_maximum_interface(&self) -> usize;

    fn get_interface_plane(&self, cell1_id: usize, cell2_id: usize) -> (BoundedPlane, usize);

    fn cell_from_ax_points(&self, axis_points: &AxisPoints) -> Option<usize>;
    fn get_boundary(&self) -> Vec<usize>;
}
/// A compartment mesh grid.
///
/// This trait is automatically implemented for any type that implements both
/// `CompartmentMeshAccessor` and `CompartmentMeshManip`, and is thread-safe (`Send` + `Sync`).
/// It provides a unified interface for operations on compartment meshes, ensuring that such types
/// can be used in concurrent programming contexts safely.
#[enum_dispatch]
pub trait CompartmentMesh: Send + Sync + CompartmentMeshAccessor + CompartmentMeshManip {}

/// Automatically implements `CompartmentMesh` for any type `T` that implements both
/// `CompartmentMeshAccessor` and `CompartmentMeshManip`, and is thread-safe.
///
/// This blanket implementation ensures that any type meeting these criteria can be used
/// wherever a `CompartmentMesh` is required, without explicit implementation.
impl<T: CompartmentMeshAccessor + CompartmentMeshManip + Send + Sync> CompartmentMesh for T {}

pub struct CylindricalMarker;
pub struct RectangularMarker;

/// Base Compartment mesh structure for spatial modeling.
///
/// This struct provides the base components data for compartmental meshes
///
/// # Type Parameters
///
/// * `T`: A marker type used to distinguish different mesh configurations
pub struct BaseCompartmentMesh<T> {
    /// The axes of the coordinate system for the mesh.
    ///
    /// This array contains three `CoordAxis` instances, each representing one of the
    /// principal axes
    axes: [CoordAxis; 3],

    /// The total number of cells in the mesh.
    ///
    /// This field stores the count of cells within the compartmental mesh.
    n_cells: usize,

    /// PhantomData marker for generic type `T`.
    ///
    /// This field is used to mark the generic type `T` in the struct without actually storing data.
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

/// Cylindrical Compartment mesh structure for spatial modeling.
pub type MeshCylindrical = BaseCompartmentMesh<CylindricalMarker>;

/// Cuboid Compartment mesh structure for spatial modeling.
pub type MeshRectangular = BaseCompartmentMesh<RectangularMarker>;

impl CompartmentMeshManip for MeshCylindrical {
    fn n_maximum_interface(&self) -> usize {
        //Trivial numbering of interfaces in a structured grid
        let nr = self.axes[0].descriptor.n_range;
        let ntheta = self.axes[1].descriptor.n_range;
        let nz = self.axes[2].descriptor.n_range;

        let interfaces_r = (nr - 1) * ntheta * nz;
        let interfaces_theta = nr * (ntheta - 1) * nz;
        let interfaces_z = nr * ntheta * (nz - 1);

        let wrap = nr * nz; //Wrap-in for connection between theta=-pi and theta=pi

        interfaces_r + interfaces_theta + interfaces_z + wrap
    }

    fn get_interface_plane(&self, cell1_id: usize, cell2_id: usize) -> (BoundedPlane, usize) {
        let neighbors = self.are_cell_neighbor(cell1_id, cell2_id);
        let axis = neighbors
            .to_coord_index()
            .expect("RMTOOL(get_interface_plane): Cells must be neighbors to get interface plane");

        let sign = if neighbors.is_negative() { -1.0 } else { 1.0 };
        let normal_dir = get_normal(axis, sign == -1.);

        let indices_cell = self.cell_points(cell1_id);

        // Cell edges
        let r0 = self.get_cell_edge(0, indices_cell[0]);
        let r1 = self.get_cell_edge(0, indices_cell[0] + 1);
        let theta0 = self.get_cell_edge(1, indices_cell[1]);
        let theta1 = self.get_cell_edge(1, indices_cell[1] + 1);
        let z0 = self.get_cell_edge(2, indices_cell[2]);
        let z1 = self.get_cell_edge(2, indices_cell[2] + 1);
        let pi = std::f64::consts::PI;
        let normalize = |a: f64| -> f64 {
            let mut x = a % (2.0 * pi);
            if x > pi {
                x -= 2.0 * pi;
            }
            if x < -pi {
                x += 2.0 * pi;
            }
            x
        };

        // Centers
        let r_center = 0.5 * (r0 + r1);

        let z_center = 0.5 * (z0 + z1);
        // let theta_center = 0.5 * (theta0 + theta1);
        let theta_center = {
            let mut dtheta = theta1 - theta0;
            if dtheta > pi {
                dtheta -= 2.0 * pi;
            }
            if dtheta < -pi {
                dtheta += 2.0 * pi;
            }
            normalize(theta0 + 0.5 * dtheta)
        };

        let (r, theta, z) = match axis {
            0 => (if sign < 0.0 { r0 } else { r1 }, theta_center, z_center),
            1 => (r_center, if sign < 0.0 { theta0 } else { theta1 }, z_center),
            2 => (r_center, theta_center, if sign < 0.0 { z0 } else { z1 }),
            _ => unreachable!(),
        };

        let (extent_u, extent_v) = match axis {
            0 => ([theta0, theta1], [z0, z1]),
            1 => ([r0, r1], [z0, z1]),
            2 => ([r0, r1], [theta0, theta1]),
            _ => unreachable!(),
        };

        if axis == 0 {
            let normal_cartesian = CartesianVec3([theta.cos(), theta.sin(), 0.0]);
            let origin = CartesianCoordinates([r * theta.cos(), r * theta.sin(), z]);
            let bounded_plane = BoundedPlane {
                normal: normal_cartesian,
                origin,
                extent_u,
                extent_v,
                axis,
            };
            (bounded_plane, axis)
        } else {
            let cyl_normal = CylindricalVec3(normal_dir, theta);
            let normal_cartesian = cyl_normal.to_cartesian_vec();
            let origin = CylindricalCoordinates([r, theta, z]).into();
            let bounded_plane = BoundedPlane {
                normal: normal_cartesian,
                origin,
                extent_u,
                extent_v,
                axis,
            };
            (bounded_plane, axis)
        }
    }

    // fn get_interface_plane(&self, cell1_id: usize, cell2_id: usize) -> (BoundedPlane, usize) {
    //     let neighbors = self.are_cell_neighbor(cell1_id, cell2_id);
    //     let axis = neighbors
    //         .to_coord_index()
    //         .expect("Cells must be neighbors to get interface plane");

    //     let sign = if neighbors.is_negative() { -1.0 } else { 1.0 };

    //     let indices_cell = self.cell_points(cell1_id);

    //     // Cell edges
    //     let r0 = self.get_cell_edge(0, indices_cell[0]);
    //     let r1 = self.get_cell_edge(0, indices_cell[0] + 1);
    //     let theta0 = self.get_cell_edge(1, indices_cell[1]);
    //     let theta1 = self.get_cell_edge(1, indices_cell[1] + 1);
    //     let z0 = self.get_cell_edge(2, indices_cell[2]);
    //     let z1 = self.get_cell_edge(2, indices_cell[2] + 1);

    //     // Midpoints
    //     let r_center = 0.5 * (r0 + r1);
    //     let z_center = 0.5 * (z0 + z1);
    //     let theta_center = 0.5 * (theta0 + theta1);

    //     // Extents
    //     let (extent_u, extent_v) = match axis {
    //         0 => ([r0, r1], [z0, z1]),         // Plane defined by (r, z)
    //         1 => ([theta0, theta1], [z0, z1]), // Plane defined by (theta, z)
    //         2 => ([r0, r1], [theta0, theta1]), // Plane defined by (r, theta)
    //         _ => unreachable!("Axis must be 0, 1, or 2"),
    //     };

    //     // Origin and normal
    //     let (origin, normal) = match axis {
    //         0 => {
    //             // Plane defined by (r, z)
    //             let origin = CartesianCoordinates([r_center, theta_center, z_center]).into();
    //             let normal = CartesianVec3([1.0, 0.0, 0.0]); // Radial direction
    //             (origin, normal)
    //         }
    //         1 => {
    //             // Plane defined by (theta, z)
    //             let origin = CartesianCoordinates([r_center, theta_center, z_center]).into();
    //             let normal = CartesianVec3([0.0, 1.0, 0.0]); // Angular direction
    //             (origin, normal)
    //         }
    //         2 => {
    //             // Plane defined by (r, theta)
    //             let origin = CartesianCoordinates([r_center, theta_center, z_center]).into();
    //             let normal = CartesianVec3([0.0, 0.0, 1.0]); // Axial direction
    //             (origin, normal)
    //         }
    //         _ => unreachable!("Axis must be 0, 1, or 2"),
    //     };

    //     let bounded_plane = BoundedPlane {
    //         normal,
    //         origin,
    //         extent_u,
    //         extent_v,
    //         axis,
    //     };

    //     (bounded_plane, axis)
    // }

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

        //Handle specific case for theta, which is mod pi
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

    fn cell_surface(&self, cell_id: usize, i_axis: OrientedAxis) -> f64 {
        let points_indices = self.cell_points(cell_id);

        let i_axis: CylindricalAxis = oriented_to_cylindrical(i_axis);

        let delta_ijk: Vec<f64> = self
            .axes
            .iter()
            .zip(points_indices.iter())
            .map(|(ax, cell_p)| ax.edges[cell_p + 1] - ax.edges[*cell_p])
            .collect();

        let min_max = if points_indices[0] == 0 { 1 } else { 0 };

        let r = self.axes[0].edges[points_indices[0] + min_max];
        match i_axis {
            CylindricalAxis::R => r * delta_ijk[1] * delta_ijk[2], // ds=r*dtheta*dz
            CylindricalAxis::Theta => delta_ijk[0] * delta_ijk[2], // ds = dr*dz
            CylindricalAxis::Z => {
                // ds  =r*dr*dtheta
                // rr here is not radius but (r-R)
                let rr = self.axes[0].edges[points_indices[0] + 1];
                let r2 = self.axes[0].edges[points_indices[0]];
                0.5 * (rr * rr - r2 * r2) * delta_ijk[1]
            }
        }
    }

    fn cell_volume(&self, cell_id: usize) -> f64 {
        let points_indices = self.cell_points(cell_id);
        let delta_ijk: Vec<f64> = self
            .axes
            .iter()
            .zip(points_indices.iter())
            .map(|(ax, cell_p)| ax.edges[cell_p + 1] - ax.edges[*cell_p])
            .collect();

        let height_axis: usize = CylindricalAxis::Z.into();
        let surface_ij = self.cell_surface(cell_id, height_axis.into());

        delta_ijk[height_axis] * surface_ij
    }

    fn cell_from_ax_points(&self, axis_points: &AxisPoints) -> Option<usize> {
        let mut cell_1d = 0;
        let mut multiplier = 1;

        for i in (0..self.axes.len()).rev() {
            let n = self.axes[i].descriptor.n_range;
            if axis_points[i] >= n {
                return None;
            }
            cell_1d += axis_points[i] * multiplier;

            multiplier *= n;
        }

        Some(cell_1d)
    }

    fn cell_from_coordinates(&self, coords: &Coords3) -> Option<usize> {
        let mut mesh_id = 0;
        let mut cumulative_product = 1;
        let CylindricalCoordinates(cylindrical_coords) = CartesianCoordinates(*coords).into();

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

    fn is_point_inside(&self, _cell_id: usize, _point_coords: &Coords3) -> bool {
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

        //Keep same logic as c++ code, would be better to start numbering from top to have forward loop
        for i in (0..self.axes.len()).rev() {
            let axe = &self.axes[i];
            let array_size = axe.descriptor.n_range;
            axis_points[i] = p_coeff_up % array_size;
            p_coeff_up /= array_size;
        }

        axis_points
    }

    fn get_boundary(&self) -> Vec<usize> {
        let (n_r, n_theta, n_z) = (
            self.n_points_axis(0),
            self.n_points_axis(1),
            self.n_points_axis(2),
        );

        let expected = n_theta * (n_z - 2) + 2 * n_r * n_z;
        let mut v = Vec::with_capacity(expected);

        for i in 0..n_r {
            for j in 0..n_theta {
                let p = self
                    .cell_from_ax_points(&[i, j, 0])
                    .expect("get_boundary: out of bound ");
                let p2 = self
                    .cell_from_ax_points(&[i, j, n_z - 1])
                    .expect("get_boundary: out of bound ");
                v.push(p);
                v.push(p2);
            }
        }

        for k in 1..n_z - 1 {
            for j in 0..n_theta {
                let p = self
                    .cell_from_ax_points(&[n_r - 1, j, k])
                    .expect("get_boundary: out of bound ");
                v.push(p)
            }
        }

        if expected != v.len() {
            panic!("Detected number is not correct {} {}", expected, v.len());
        }

        v
    }
}

pub fn get_mesh(
    meshtype: MeshType,
    mut ax_descriptor: [AxisDescriptor; 3],
) -> Box<dyn CompartmentMesh> {
    match meshtype {
        //TODO move this into geometryy module as just assert
        //Having this logic here, hides specific behaviour to user which may lead to errors
        MeshType::Cylindrical => {
            ax_descriptor[0].min_range = 0.;
            ax_descriptor[1].min_range = -std::f64::consts::PI;
            ax_descriptor[1].max_range = std::f64::consts::PI;
            Box::new(MeshCylindrical::new(ax_descriptor))
        }
        MeshType::Rectangular => unimplemented!("Manip for Rectangular impl"),
    }
}

#[cfg(test)]
mod tests;
