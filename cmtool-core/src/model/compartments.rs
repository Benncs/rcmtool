use crate::{
    model::{CMGeometry, interfaces::AInterfacesInfo},
    utils::compute_volume,
};

#[derive(Clone, Copy)]
pub struct ElementVolumeInfo {
    pub global_id: usize,
    pub volume: f64,
}

impl Default for ElementVolumeInfo {
    fn default() -> Self {
        ElementVolumeInfo {
            global_id: 0,
            volume: 0.,
        }
    }
}

pub struct CompartmentInfo {
    pub n_volumes: Vec<usize>,
    pub volumes: Vec<Vec<ElementVolumeInfo>>,
}

impl CompartmentInfo {
    /// Creates a new [`CompartmentInfo`].
    fn new(volume_per_compartment: Vec<usize>) -> Self {
        let volumes: Vec<Vec<ElementVolumeInfo>> = volume_per_compartment
            .iter()
            .map(|n_volume| vec![ElementVolumeInfo::default(); *n_volume])
            .collect();

        Self {
            n_volumes: volume_per_compartment,
            volumes,
        }
    }

    pub fn fill(&mut self, geometry: &CMGeometry) {
        let mut local_vertices = Vec::new();

        let mut tmp_count_k_element: Vec<usize> = vec![0; geometry.n_zone()];

        for (volume_element_global_id, n_compartment_in_velem) in
            geometry.volume_elements.enumerate_number_id()
        {
            let n_compartment_in_velem = *n_compartment_in_velem;
            let (elem_type, n_vertex) = geometry
                .volume_elements
                .get_element_and_nvertex(volume_element_global_id);

            local_vertices.clear();
            local_vertices.resize(n_vertex, Default::default());
            for i_compartment_in_velem in 0..n_compartment_in_velem {
                let compartment_id = geometry
                    .volume_elements
                    .get_list_compartment_id(volume_element_global_id, i_compartment_in_velem);

                let k_element = tmp_count_k_element[compartment_id];
                tmp_count_k_element[compartment_id] += 1;

                geometry.fill_vertices(volume_element_global_id, n_vertex, &mut local_vertices);
                let volume = compute_volume(&local_vertices, elem_type).unwrap()
                    / (n_compartment_in_velem as f64);

                assert!(volume >= 0.);
                self.volumes[compartment_id][k_element] = ElementVolumeInfo {
                    global_id: volume_element_global_id,
                    volume,
                };
            }
        }
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
        let at_interface: Vec<usize> = self.at_interface.into_iter().filter(|&v| v != 0).collect();

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
