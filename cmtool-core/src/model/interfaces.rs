// SPDX-License-Identifier: GPL-3.0-or-later

use crate::coordinates::*;
use crate::grid::NeighborDirection;
use crate::model::CMGeometry;
use crate::utils::{compute_intersection_area, is_curved_face, tangent_plane_at};

const REL_TOLERANCE_AREA: f64 = 0.1;

///The interface areas of a face should add up to the surface of the cell, a face with no
///geometric surface should carry no interface either
fn is_area_mismatch(total_area: f64, theoretical_area: f64) -> bool {
    if theoretical_area.abs() < f64::EPSILON {
        return total_area.abs() > f64::EPSILON;
    }

    (total_area - theoretical_area).abs() / theoretical_area > REL_TOLERANCE_AREA
}

#[derive(Default, Clone)]
pub struct InterfaceInfo {
    pub source_id: usize,
    pub target_id: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct InterfaceFlow {
    pub source_flow: f64,
    pub target_flow: f64,
}

pub struct AInterfacesInfo {
    n_facet: Vec<usize>,
    pub ids: Vec<InterfaceInfo>,
    pub area: Vec<Vec<f64>>,
    pub normal_axis: Vec<usize>,
    pub global_id_from_interface: Vec<Vec<usize>>,
    pub interface_theta: Vec<f64>,
    // pub plane_coordinates: Vec<f64>,
    // pub planes: Vec<BoundedPlane>,
}

impl AInterfacesInfo {
    pub fn new(at_interface: Vec<usize>) -> Self {
        let n_interfaces = at_interface.len();

        Self {
            n_facet: at_interface,
            ids: vec![Default::default(); n_interfaces],
            area: vec![Default::default(); n_interfaces],
            normal_axis: vec![Default::default(); n_interfaces],
            global_id_from_interface: vec![Default::default(); n_interfaces],
            interface_theta: vec![Default::default(); n_interfaces],
            // plane_coordinates: vec![0.; n_interfaces * 3 * 2], //Extent geometry
            // planes: Vec::new(),
        }
    }
}

//use std::{iter::Sum, ops::Add};
//impl Add for InterfaceFlow {
//type Output = Self;
//
//fn add(self, other: Self) -> Self {
//InterfaceFlow {
//source_flow: self.source_flow + other.source_flow,
//target_flow: self.target_flow + other.target_flow,
//}
//}
//}

//impl Sum for InterfaceFlow {
//fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
//iter.fold(Self::default(), Add::add)
//}
//}

impl Default for InterfaceFlow {
    fn default() -> Self {
        InterfaceFlow {
            source_flow: 0.0,
            target_flow: 0.0,
        }
    }
}

impl AInterfacesInfo {
    pub fn n_interfaces(&self) -> usize {
        self.n_facet.len()
    }

    pub fn fill(&mut self, geometry: &CMGeometry, interface_count_raw: &[usize]) {
        let mut planes: Vec<BoundedPlane> = Vec::with_capacity(self.n_interfaces());

        let grid = geometry.get_grid().unwrap();
        let n_zones = geometry.n_zone();
        let mut interfaces_id_from_cells = vec![0; n_zones * n_zones];
        let mut interface_counter = 0;

        for source_id in 0..n_zones {
            for target_id in 0..n_zones {
                if interface_count_raw[source_id * n_zones + target_id] == 0 {
                    continue;
                }

                let interface_id = interface_counter;
                interface_counter += 1;
                self.ids[interface_id] = InterfaceInfo {
                    source_id,
                    target_id,
                };
                interfaces_id_from_cells[source_id * n_zones + target_id] = interface_id;
                interfaces_id_from_cells[target_id * n_zones + source_id] = interface_id;

                let (plane, direction_neighbors) = grid.get_interface_plane(source_id, target_id);

                let origin = plane.origin.0;
                let theta = origin[1].atan2(origin[0]);
                self.interface_theta[interface_id] = theta;
                planes.push(plane);
                self.normal_axis[interface_id] = direction_neighbors;
            }
        }
        self.global_id_from_interface =
            self.count_interfaces_second_pass(geometry, &interfaces_id_from_cells);
        self.fill_area(geometry, &planes);

        // for i_interface in 0..self.n_facet.len() {
        //     let total: f64 = self.area[i_interface].iter().sum();
        //     if total > 0.0 {
        //         let source = self.ids[i_interface].source_id;
        //         let axis = self.normal_axis[i_interface];
        //         let theoretical = grid.cell_surface(source, index_to_oriented(axis));
        //         let factor = theoretical / total;
        //         for a in self.area[i_interface].iter_mut() {
        //             *a *= factor;
        //         }
        //     }
        // }

        // self.check_areas(geometry);
    }
    #[allow(unused)]
    fn check_areas(&self, geometry: &CMGeometry) {
        let grid = geometry.get_grid().unwrap();
        let n_zones = geometry.n_zone();
        for cell_id in 0..n_zones {
            for axis_idx in 0..3 {
                let axis = crate::grid::index_to_oriented(axis_idx);
                let theoretical_area = grid.cell_surface(cell_id, axis);
                let total_area: f64 = self
                    .ids
                    .iter()
                    .enumerate()
                    .filter(|(i, id)| {
                        (id.source_id == cell_id || id.target_id == cell_id)
                            && self.normal_axis[*i] == axis_idx
                    })
                    .map(|(i, _)| self.area[i].iter().sum::<f64>())
                    .sum();

                if is_area_mismatch(total_area, theoretical_area) {
                    println!(
                        "(areas): area incorrect : axis: {}\r\n -cell_id:{}\r\n -total_area: {}\r\n -theoretical: {}",
                        axis_idx, cell_id, total_area, theoretical_area
                    );
                }
            }
        }
    }

    fn count_interfaces_second_pass(
        &mut self,
        geometry: &CMGeometry,
        interfaces_id_from_cells: &[usize],
    ) -> Vec<Vec<usize>> {
        let mut tmp_element_counter = vec![0; self.n_facet.len()];
        let mut global_id_from_interface: Vec<Vec<usize>> = vec![Vec::new(); self.n_facet.len()];
        for (element_id, n_element) in global_id_from_interface.iter_mut().zip(self.n_facet.iter())
        {
            *element_id = vec![0; *n_element];
        }
        let grid = geometry.get_grid().unwrap();
        let n_zones = geometry.n_zone();

        for (vol_element_global_id, _, interface_cid_k, k_vertex) in geometry.interface_iter() {
            // if k_vertex >= 1 {
            for i in 0..k_vertex {
                let cid_i = geometry
                    .volume_elements
                    .get_list_compartment_id(vol_element_global_id, i);
                let neighbors = grid.are_cell_neighbor(cid_i, interface_cid_k);
                if neighbors != NeighborDirection::NotNeighbors {
                    let interface_global_id =
                        interfaces_id_from_cells[cid_i * n_zones + interface_cid_k];
                    let k_element = tmp_element_counter[interface_global_id];
                    tmp_element_counter[interface_global_id] += 1;
                    global_id_from_interface[interface_global_id][k_element] =
                        vol_element_global_id;
                }
            }
            // }
        }

        global_id_from_interface
    }

    fn fill_area(&mut self, geometry: &CMGeometry, planes: &[BoundedPlane]) {
        //This is almost the same algorithm as fill for c_info struct (to compute volume of velem)
        for (i, n) in self.n_facet.iter().enumerate() {
            self.area[i].resize(*n, 0.);
        }

        let mut local_vertices: Vec<CartesianCoordinates> = Vec::new();

        for (interface_id, cn_facet) in self.n_facet.iter().enumerate() {
            let plane = &planes[interface_id];
            for i_facet in 0..*cn_facet {
                let volume_element_global_id = self.global_id_from_interface[interface_id][i_facet];

                let (elem_type, n_vertex) = geometry
                    .volume_elements
                    .get_element_and_nvertex(volume_element_global_id);

                geometry.fill_vertices(volume_element_global_id, n_vertex, &mut local_vertices);

                //A curved face gives every element the tangent plane of its own position
                let element_plane = is_curved_face(plane).then(|| {
                    tangent_plane_at(
                        plane,
                        geometry.volume_elements.xyz[volume_element_global_id],
                    )
                });

                let area = compute_intersection_area(
                    &local_vertices,
                    elem_type,
                    element_plane.as_ref().unwrap_or(plane),
                )
                .expect("Area between element");

                self.area[interface_id][i_facet] = area;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_matching_area_is_not_reported() {
        assert!(!is_area_mismatch(10., 10.));
        //Within the tolerance
        assert!(!is_area_mismatch(10.5, 10.));
    }

    #[test]
    fn test_mismatching_area_is_reported() {
        assert!(is_area_mismatch(5., 10.));
        assert!(is_area_mismatch(0., 10.));
        assert!(is_area_mismatch(20., 10.));
    }

    ///A degenerate face divides by zero, which used to hide the mismatch behind a NaN
    #[test]
    fn test_area_without_geometric_surface() {
        assert!(is_area_mismatch(1., 0.));
        assert!(!is_area_mismatch(0., 0.));
    }
}
