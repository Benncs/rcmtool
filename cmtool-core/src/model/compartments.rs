
#[derive(Default,Clone, Copy)]
pub struct ElementVolumeInfo
{
    pub global_id:usize,
    pub volume:f64,
}

pub struct CompartmentInfo {
    pub n_volumes: Vec<usize>,
    pub volumes: Vec<Vec<ElementVolumeInfo>>,
}

impl CompartmentInfo {
    fn new(per_compartment: Vec<usize>) -> Self {
        let size = per_compartment.len();

        let mut volumes = vec![Vec::new(); size];

        for i in 0..size
        {
            volumes[i].resize(per_compartment[i], ElementVolumeInfo { global_id: 0, volume:0. });
        }

        Self {
            n_volumes: per_compartment,
            volumes,
        }
    }
}

pub struct InterfacesInfo {
    n_facet: Vec<usize>,
}

impl InterfacesInfo {
    fn new(at_interface: Vec<usize>) -> Self {
        Self {
            n_facet: at_interface,
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

    pub fn into_reduce(self) -> (CompartmentInfo, InterfacesInfo) {
        let at_interface: Vec<usize> = self.at_interface.into_iter().filter(|&v| v > 0).collect();
        let per_compartment: Vec<usize> = self
            .per_compartment
            .into_iter()
            .filter(|&v| v != 0)
            .collect();

        (
            CompartmentInfo::new(per_compartment),
            InterfacesInfo::new(at_interface),
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
