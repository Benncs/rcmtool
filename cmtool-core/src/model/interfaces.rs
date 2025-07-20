use std::{iter::Sum, ops::Add};

use crate::coordinates::*;
use crate::grid::NeighborDirection;
use crate::utils::compute_intersection_area;
use crate::{
    ensight_gold::types::ElementsType,
    model::{CMGeometry, geometry},
    // utils::compute_intersection_area,
};

#[derive(Default, Clone)]
pub struct InterfaceInfo {
    pub source_id: usize,
    pub target_id: usize,
}

#[derive(Clone)]
pub struct InterfaceFlow {
    pub source_flow: f64,
    pub target_flow: f64,
}

pub struct AInterfacesInfo {
    n_facet: Vec<usize>,
    pub info: Vec<InterfaceInfo>,
    pub area: Vec<Vec<f64>>,
    pub normal_axis: Vec<usize>,
    pub global_id_from_interface: Vec<Vec<usize>>,
    // pub plane_coordinates: Vec<f64>,
    // pub planes: Vec<BoundedPlane>,
}

impl AInterfacesInfo {
    pub fn new(at_interface: Vec<usize>) -> Self {
        let n_interfaces = at_interface.len();

        Self {
            n_facet: at_interface,
            info: vec![Default::default(); n_interfaces],
            area: vec![Default::default(); n_interfaces],
            normal_axis: vec![Default::default(); n_interfaces],
            global_id_from_interface: vec![Default::default(); n_interfaces],
            // plane_coordinates: vec![0.; n_interfaces * 3 * 2], //Extent geometry
            // planes: Vec::new(),
        }
    }
}

impl Add for InterfaceFlow {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        InterfaceFlow {
            source_flow: self.source_flow + other.source_flow,
            target_flow: self.target_flow + other.target_flow,
        }
    }
}

impl Sum for InterfaceFlow {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), Add::add)
    }
}

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
        let mut interface_counter = 0;
        let mut interfaces_id_from_cells = vec![0; n_zones * n_zones];

        for source_id in 0..n_zones {
            for target_id in 0..n_zones {
                if interface_count_raw[source_id * n_zones + target_id] == 0 {
                    continue;
                }

                let interface_id = interface_counter;
                interface_counter += 1;
                self.info[interface_id] = InterfaceInfo {
                    source_id,
                    target_id,
                };
                interfaces_id_from_cells[source_id * n_zones + target_id] = interface_id;
                interfaces_id_from_cells[target_id * n_zones + source_id] = interface_id;

                let (plane, direction_neighbors) = grid.get_interface_plane(source_id, target_id);
                planes.push(plane);
                self.normal_axis[interface_id] = direction_neighbors;
            }
        }
        self.global_id_from_interface =
            self.count_interfaces_second_pass(geometry, &interfaces_id_from_cells);
        self.fill_area(geometry, &planes);
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

        let n_zones = geometry.n_zone();
        let functor = |vol_element_global_id: usize,
                       interface_cid_0: usize,
                       interface_cid_k: usize,
                       k_vertex: usize| {
            if k_vertex >= 1
                && geometry
                    .get_grid()
                    .as_ref()
                    .unwrap()
                    .are_cell_neighbor(interface_cid_0, interface_cid_k)
                    != NeighborDirection::NotNeighbors
            {
                let interface_global_id =
                    interfaces_id_from_cells[interface_cid_0 * n_zones + interface_cid_k];
                let k_element = tmp_element_counter[interface_global_id];
                tmp_element_counter[interface_global_id] += 1;
                global_id_from_interface[interface_global_id][k_element] = vol_element_global_id;
            }
        };
        geometry.interface_iterator(functor);

        global_id_from_interface
    }

    fn fill_area(&mut self, geometry: &CMGeometry, planes: &[BoundedPlane]) {
        let fill_vertices =
            |volume_element_global_id, n_vertex, local_vertices: &mut Vec<CartesianCoordinates>| {
                local_vertices.clear();
                local_vertices.reserve(n_vertex);

                for k_vertex in 0..n_vertex {
                    let vertex_global_id = geometry
                        .volume_elements
                        .get_vertex_from_vol_global_id(volume_element_global_id, k_vertex);
                    local_vertices.push(CartesianCoordinates(
                        geometry.vertices.get_slice_xyz(vertex_global_id).to_owned(),
                    ));
                }
            };

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

                fill_vertices(volume_element_global_id, n_vertex, &mut local_vertices);

                let area = compute_intersection_area(&local_vertices, elem_type, plane)
                    .expect("Area between element");
                if area == 0. {
                    println!("{} {}", interface_id, i_facet);
                }
                self.area[interface_id][i_facet] = area;
            }
        }
    }
}
