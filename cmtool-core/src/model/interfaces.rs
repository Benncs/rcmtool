use std::{iter::Sum, ops::Add};

use crate::{
    ensight_gold::types::ElementsType,
    model::{geometry, CMGeometry},
    utils::{compute_intersection_area},
};
use crate::coordinates::*;


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
    pub n_facet: Vec<usize>,
    pub info: Vec<InterfaceInfo>,
    pub area: Vec<Vec<f64>>,
    pub axis: Vec<usize>,
}

impl AInterfacesInfo {
    pub fn new(at_interface: Vec<usize>) -> Self {
        let n_interfaces = at_interface.len();

        Self {
            n_facet: at_interface,
            info: vec![Default::default(); n_interfaces],
            area: vec![Default::default(); n_interfaces],
            axis: vec![Default::default(); n_interfaces],
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
    pub fn fill(
        &mut self,
        geometry: &CMGeometry,
        interface_count_raw: &[usize],
        global_id_from_interface: &[Vec<usize>],
    ) {
        let grid = geometry.get_grid().unwrap();
        let n_zones = geometry.n_zone();
        let mut interface_counter = 0;

        let mut interfaces_id_from_cells = vec![0; n_zones * n_zones];

        for source_id in 0..n_zones {
            for target_id in 0..n_zones {
                // let index = source_id*n_zones+target_id;
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

                self.axis[interface_id] = grid
                    .are_cell_neighbor(source_id, target_id)
                    .to_coord_index()
                    .expect("Unwrap because we already know they are neighbors");
            }
        }

        self.fill_area(geometry, global_id_from_interface);
    }

    fn fill_area(&mut self, geometry: &CMGeometry, global_id_from_interface: &[Vec<usize>]) {
        //This is almost the same algorithm as fill for c_info struct (to compute volume of velem)
        for (i, n) in self.n_facet.iter().enumerate() {
            self.area[i].resize(*n, 0.);
        }

        let mut local_vertices: Vec<Coords3> = Vec::new();

        for (interface_id, n_elem) in self.n_facet.iter().enumerate() {
            for i_element in 0..*n_elem {
                let axis_index = self.axis[interface_id];
                let point_on_axe = 0.;

                let volume_element_global_id = global_id_from_interface[interface_id][i_element]; //m_dbLimit_lvelem[interface][n_elem]

                let element = geometry.volume_elements.vtype[volume_element_global_id];
                let n_vertex = geometry
                    .volume_elements
                    .get_vertex_per_element(volume_element_global_id);
                local_vertices.clear();
                local_vertices.resize(n_vertex, Default::default());

                for (k_vertex, local_vertex) in local_vertices.iter_mut().enumerate() {
                    let vertex_global_id = geometry
                        .volume_elements
                        .get_vertex_from_vol_global_id(volume_element_global_id, k_vertex);
                    *local_vertex = geometry.vertices.get_slice_xyz(vertex_global_id).to_owned();
                }
                let area =
                    compute_intersection_area(&local_vertices, element, point_on_axe, axis_index)
                        .expect("Area between element");
                self.area[interface_id][i_element] += area;
            }
        }
    }
}
