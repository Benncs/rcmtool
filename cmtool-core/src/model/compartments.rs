use std::{iter::Sum, ops::Add};

use crate::{ensight_gold::types::VolumeElementTypes, model::{interfaces::AInterfacesInfo, CMGeometry}, utils::compute_volume};

#[derive(Default, Clone, Copy)]
pub struct ElementVolumeInfo {
    pub global_id: usize,
    pub volume: f64,
}

pub struct CompartmentInfo {
    pub n_volumes: Vec<usize>,
    pub volumes: Vec<Vec<ElementVolumeInfo>>,
}

impl CompartmentInfo {
    fn new(per_compartment: Vec<usize>) -> Self {
        let size = per_compartment.len();

        let mut volumes = vec![Vec::new(); size];

        for i in 0..size {
            volumes[i].resize(
                per_compartment[i],
                ElementVolumeInfo {
                    global_id: 0,
                    volume: 0.,
                },
            );
        }

        Self {
            n_volumes: per_compartment,
            volumes,
        }
    }

    pub fn fill(&mut self,geometry:&CMGeometry) {
        let mut v_tot = 0.;
        let mut local_vertices = Vec::new();


        let mut tmp_count_k_element: Vec<usize> = vec![0; geometry.n_zone()];

        for volume_element_global_id in 0..geometry.volume_elements.n_element() {
            let n_compartment_in_velem = geometry
                .volume_elements
                .get_number_cid(volume_element_global_id);

            let n_vertex = geometry
                .volume_elements
                .get_vertex_per_element(volume_element_global_id);

            let vtype: VolumeElementTypes =
                geometry.volume_elements.vtype[volume_element_global_id];

            local_vertices.clear();
            local_vertices.resize(n_vertex, Default::default());
            for i_compartment_in_velem in 0..n_compartment_in_velem {
                let compartment_id = geometry
                    .volume_elements
                    .get_list_compartment_id(volume_element_global_id, i_compartment_in_velem);

                let k_element = tmp_count_k_element[compartment_id];
                tmp_count_k_element[compartment_id] += 1;

                for (k_vertex, local_vertex) in local_vertices.iter_mut().enumerate() {
                    let vertex_global_id = geometry
                        .volume_elements
                        .get_vertex_from_vol_global_id(volume_element_global_id, k_vertex);
                    *local_vertex = geometry.vertices.get_slice_xyz(vertex_global_id).to_owned();
                }

                let volume = compute_volume(&local_vertices, vtype).unwrap()
                    / (n_compartment_in_velem as f64);
                self.volumes[compartment_id][k_element] = ElementVolumeInfo {
                    global_id: volume_element_global_id,
                    volume,
                };
                assert!(volume >= 0.);
                v_tot += volume;
            }
        }
        println!("{}", v_tot);
    }
}





pub struct CountVolumeElement {
    pub per_compartment: Vec<usize>,
    pub at_interface: Vec<usize>,
}

impl CountVolumeElement {
    pub fn new(n_zone: usize) -> Self {
        CountVolumeElement {
            per_compartment: vec![0; n_zone],
            //Each compartment can technically have one interface with each other
            //Ahead of time, allocate more than needed. be resized later
            at_interface: vec![0; n_zone * n_zone],
        }
    }

    pub fn into_reduce(self) -> (CompartmentInfo, AInterfacesInfo) {
        let at_interface: Vec<usize> = self.at_interface.into_iter().filter(|&v| v > 0).collect();
        let per_compartment: Vec<usize> = self
            .per_compartment
            .into_iter()
            .filter(|&v| v != 0)
            .collect();

        (
            CompartmentInfo::new(per_compartment),
            AInterfacesInfo::new(at_interface),
        )
    }

    pub fn reduce(self) -> Self {
        let at_interface: Vec<usize> = self.at_interface.into_iter().filter(|&v| v > 0).collect();

        let per_compartment: Vec<usize> = self
            .per_compartment
            .into_iter()
            .filter(|&v| v != 0)
            .collect();

        Self {
            at_interface,
            per_compartment,
        }
    }

    pub fn incr_compartment(&mut self, compartment_id: usize) {
        self.per_compartment[compartment_id] += 1;
    }
    pub fn incr_interface(&mut self, compartment_id_0: usize, compartment_id_k: usize) {
        self.at_interface[compartment_id_0 * self.per_compartment.len() + compartment_id_k] += 1;
    }
    pub fn n_interfaces(&self) -> usize {
        self.at_interface.iter().filter(|&&v| v > 0).count()
    }

    pub fn n_zone_with_volume_element(&self) -> usize {
        self.per_compartment.iter().filter(|v| **v != 0).count()
    }
}
