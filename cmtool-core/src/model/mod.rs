use std::{ops::Index, sync::Arc};

use crate::{ensight_gold::types::VolumeElementTypes, model::scalar::Scalar, CoreError};
mod data;
pub mod scalar;
use cmtool_data::RawDataScalar;

use data::*;
mod geometry;
use geometry::*;

pub struct CMModel {
    geometry: Arc<CMGeometry>,
    reduce:CountVolumeElement,
}

pub struct CountVolumeElement {
    pub per_compartment: Vec<usize>,
    pub at_interface: Vec<usize>,
}

struct ModelCompartment
{
    
}

impl CountVolumeElement {
    fn new(n_zone: usize) -> Self {
        CountVolumeElement {
            per_compartment: vec![0; n_zone],
            //Each compartment can technically have one interface with each other
            //Ahead of time, allocate more than needed. be resized later
            at_interface: vec![0; n_zone * n_zone],
        }
    }

    fn reduce(self) -> Self {
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

    fn incr_compartment(&mut self, compartment_id: usize) {
        self.per_compartment[compartment_id] += 1;
    }
    fn incr_interface(&mut self, compartment_id_0: usize, compartment_id_k: usize) {
        self.at_interface[compartment_id_0 * self.per_compartment.len() + compartment_id_k] += 1;
    }
    pub fn n_interfaces(&self) -> usize {
        self.at_interface.iter().filter(|&&v| v > 0).count()
    }

    pub fn n_zone_with_volume_element(&self) -> usize {
        self.per_compartment.iter().filter(|v| **v != 0).count()
    }
}

pub use geometry::CMGeometry;



    


impl CMModel {
    pub fn init(geometry: Arc<CMGeometry>) -> Self {
        println!("Init model with {} compartment", geometry.n_zone());
        let volume_element_count = geometry.get_count_volume_element();

        let mut v_tot = 0.;

        let n_zone_with_volume_element = volume_element_count.n_zone_with_volume_element();

        if n_zone_with_volume_element != geometry.n_zone() {
            unimplemented!(
                "Detected compartment should be the same as given by user {} vs {}",
                n_zone_with_volume_element,
                geometry.n_zone()
            )
        }

        // let n_interfaces = volume_element_count.n_interfaces();

        for volume_element_global_id in 0..geometry.volume_elements.n_element()
        {
            let ncid = geometry.volume_elements.get_number_cid(volume_element_global_id);
            let n_vertex = geometry.volume_elements.get_vertex_per_element(volume_element_global_id);
            let vtype: VolumeElementTypes =geometry.volume_elements.vtype[volume_element_global_id];
            for k_c in 0..ncid
            {
                geometry.volume_elements.get_limit_cell_id(volume_element_global_id, k_c);
            }
            let volume =0.;
            v_tot+= volume;
        }


        Self { geometry,reduce:volume_element_count.reduce() }
    }

    fn compute_flux_through_limits() -> Vec<f64> {
        todo!()
    }

    fn compute_volume_integral_per_zone() -> Vec<f64> {
        todo!()
    }

    pub fn export_flux_through_limits(&self, flow: &mut cmtool_data::RawDataFlux) {
        todo!()
    }

    // pub fn export_volume_integral_per_zone(&self,scalar:&mut cmtool_data::RawDataScalar) {
    //     todo!()
    // }

    pub fn export_volume_integral_per_zone(
        &self,
        scalar: Scalar,
    ) -> Result<cmtool_data::RawDataScalar, CoreError> {
        println!(
            "Creating scalar with {} compartment",
            self.geometry.n_zone()
        );
        let mut scalar_field = RawDataScalar::new(self.geometry.n_zone());

        for (compartment_id,n_volume_element) in self.reduce.per_compartment.iter().enumerate()
        {
            scalar_field.values.push((0.).into());
            for  k_volume_element in 0..*n_volume_element
            {
                let volume_element_global_id = 0;
                let volume_of_element = 0.;
                let value = scalar[volume_element_global_id];
                scalar_field.values[compartment_id].value+=value*volume_of_element;
            }
        }

        todo!("export_volume_integral_per_zone")
    }

    pub fn compartments_volumes(&self) -> Vec<f64> {
        todo!()
    }

    pub fn get_real_volume(&self) -> &[f64] {
        todo!()
    }
}
