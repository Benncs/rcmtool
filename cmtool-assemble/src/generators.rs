// SPDX-License-Identifier: GPL-3.0-or-later


use crate::CMError;

use cmtool_data::{
    CMAExportType, CMCase, CMCaseJson, CMCaseReader, CMCaseWriter, CMExportType, PhaseCM, RawData,
    RawDataFlux, RawDataScalar, RawFlux, RawPhase,
};
use std::path::Path;

const LIQUID_PAIR: (CMAExportType, CMAExportType) =
    (CMAExportType::LiquidFlow, CMAExportType::LiquidVolume);

const GAS_PAIR: (CMAExportType, CMAExportType) = (CMAExportType::GasFlow, CMAExportType::GasVolume);

type PairType = (CMAExportType, CMAExportType);

const PAIRS: (PairType, PairType) = (LIQUID_PAIR, GAS_PAIR);

pub struct PFRDescription {
    pub n_compartment: usize,
    pub length: f64,
    pub diameter: f64,
    pub liquid_flow: f64,
    pub gas_flow: f64,
    pub gas_fraction: f64,
    pub axial_dispersion: f64,
}

pub struct Generator {
    raw_phase: Vec<RawPhase>,
}

struct Field0D {
    name: String,
    value: f64,
}

fn resolve_path(
    case: &CMCase,
    relative_path: &str,
    value: CMAExportType,
) -> Result<String, CMError> {
    case.resolve(relative_path, value)
        .ok_or(CMError::Custom("Error resolving path".to_string()))
}

fn get_raw_phase(
    flows: Vec<RawDataFlux>,
    vol: Vec<RawDataScalar>,
    phase: PhaseCM,
) -> Vec<RawPhase> {
    flows
        .into_iter()
        .zip(vol)
        .map(|(f, v)| RawPhase {
            flow: f,
            volume: v,
            identifier: phase,
        })
        .collect()
}

fn filter_phase(raw_phase: &[RawPhase], phase: PhaseCM) -> Vec<RawPhase> {
    raw_phase
        .iter()
        .filter(|p| p.identifier == phase)
        .cloned()
        .collect()
}

impl Generator {
    
    pub fn new() -> Self {
        Self {
            raw_phase: Default::default(),
        }
    }

    fn write_phase(
        dest: &str,
        case: &mut CMCase,
        phase: RawPhase,
        relative_path: Option<String>,
    ) -> Result<(), CMError> {
        let (flowp, volumep) = phase.write(dest)?;
        let flowp = match &relative_path {
            Some(rel) => format!("./{}/{}", rel, flowp),
            None => format!("./{}", flowp),
        };

        let volumep = match relative_path {
            Some(rel) => format!("./{}/{}", rel, volumep),
            None => format!("./{}", volumep),
        };
        case.add(CMExportType::Flow(phase.identifier).into(), &flowp);
        case.add(CMExportType::Volume(phase.identifier).into(), &volumep);

        Ok(())
    }

    fn generate_0d_phase(
        &mut self,
        case: &mut CMCase,
        volume: f64,
        phase: PhaseCM,
        dest: Option<String>,
    ) -> Result<(), CMError> {
        let mut phase = cmtool_data::RawPhase::new(1, 1, phase);
        phase.flow.fluxes[0] = Default::default(); //Not usefull because new already makes default
        phase.volume.values.push(volume.into());

        match dest {
            Some(s) => Self::write_phase(&s, case, phase, None),
            None => {
                self.raw_phase.push(phase);
                Ok(())
            }
        }
    }

    fn generate_0d(
        &mut self,
        liquid_volume: f64,
        gas_volume: f64,
        fields: Option<&[Field0D]>,
        dest: Option<String>,
    ) -> Result<CMCase, CMError> {
        let mut case = CMCase::default();
        case.n_div = [1, 0, 0];

        self.generate_0d_phase(&mut case, liquid_volume, PhaseCM::Liquid, dest.clone())?;

        if gas_volume != 0. {
            self.generate_0d_phase(&mut case, gas_volume, PhaseCM::Gas, dest)?;
        }

        if let Some(scalars) = fields {
            todo!("Scalar field")
        }

        Ok(case)
    }

    pub fn generate_0d_from_fraction(
        &mut self,
        total_volume: f64,
        gas_fraction: f64,
        dest: Option<String>,
    ) -> Result<CMCase, CMError> {
        if !(0. ..=1.).contains(&gas_fraction) {
            panic!("TODO: handle error gas fraction generation 0d");
        }

        let gas_volume = gas_fraction * total_volume;
        let liquid_volume = total_volume * (1. - gas_fraction);

        self.generate_0d(liquid_volume, gas_volume, None, dest)
    }

    #[allow(clippy::too_many_arguments)]
    fn generate_1d(
        &mut self,
        case: &mut CMCase,
        n_compartment: usize,
        length: f64,
        diameter: f64,
        flow: f64,
        volume_fraction: f64,
        axial_dispersion: f64,
        gas: bool,
        dest: Option<String>,
    ) -> Result<(), CMError> {
        let dx = length / (n_compartment as f64);
        let reactor_section_area = std::f64::consts::PI * diameter.powf(2.) / 4.;
        let compartment_volume = volume_fraction * dx * reactor_section_area;
        let flow_velocity = flow / reactor_section_area;
        let n_flow = n_compartment - 1;

        let flow_source_target = reactor_section_area / dx * (flow_velocity + axial_dispersion);
        let flow_target_source = reactor_section_area / dx * axial_dispersion;

        assert!(
            ((compartment_volume * n_compartment as f64)
                - (volume_fraction * reactor_section_area * length))
                .abs()
                < 1e-5
        );

        let cp = if gas { PhaseCM::Gas } else { PhaseCM::Liquid };
        let mut phase = RawPhase::new(n_compartment, n_flow, cp);

        //n_flow != n_compartment, need to set volume separately
        phase.volume = RawDataScalar::from(vec![compartment_volume; n_compartment]); //Scalar values are not preallo

        for (current_index, flow) in phase.flow.fluxes.iter_mut().enumerate() {
            flow.id_source = current_index as u32;
            flow.id_target = (current_index + 1) as u32;
            flow.flux_source_target = flow_source_target;
            flow.flux_target_source = flow_target_source;
        }

        if let Some(s) = dest {
            Self::write_phase(&s, case, phase, None)?;
        } else {
            self.raw_phase.push(phase);
        }

        Ok(())
    }

    pub fn generate_1d_from_fraction(
        &mut self,

        PFRDescription {
            n_compartment,
            length,
            diameter,
            liquid_flow,
            gas_flow,
            gas_fraction,
            axial_dispersion,
        }: PFRDescription,
        dest: Option<String>,
    ) -> Result<CMCase, CMError> {
        let mut case = CMCase::default();
        case.n_div = [0, 0, n_compartment as u32];

        if !(0. ..=1.).contains(&gas_fraction) {
            panic!("TODO: handle error gas fraction generation 0d");
        }

        self.generate_1d(
            &mut case,
            n_compartment,
            length,
            diameter,
            liquid_flow,
            1.0 - gas_fraction,
            axial_dispersion,
            false,
            dest.clone(),
        )?;
        if gas_fraction != 0. {
            self.generate_1d(
                &mut case,
                n_compartment,
                length,
                diameter,
                gas_flow,
                1.0 - gas_fraction,
                axial_dispersion,
                true,
                dest,
            )?;
        }

        Ok(case)
    }

    fn merge_phase(
        // flows: Vec<RawDataFlux>,
        // volumes: Vec<RawDataScalar>
        phases: Vec<RawPhase>,
        connections: Option<RawDataFlux>,
    ) -> Result<RawPhase, CMError> {
        let mut phase = RawPhase::new(0, 0, phases[0].identifier);

        // let mut merge_phase_flow = RawDataFlux::new(0, 0); //
        // let mut merge_phase_volume = RawDataScalar::new(0);
        let offset_compartment = std::cell::Cell::new(0u32);
        let incr_id = |mut flow: RawFlux| -> RawFlux {
            flow.id_source += offset_compartment.get();
            flow.id_target += offset_compartment.get();
            flow
        };

        for p in &phases {
            let rd = &p.flow;
            let v = &p.volume;
            phase.flow.header.n_zone += rd.header.n_zone;
            phase.volume.header.n_zone += rd.header.n_zone;
            phase.flow.header.n_fluxes += rd.header.n_fluxes;
            phase
                .flow
                .fluxes
                .extend(rd.fluxes.iter().map(|&flux| incr_id(flux)));
            phase.volume.values.extend(v.values.clone().into_iter());
            offset_compartment.set(offset_compartment.get() + rd.header.n_zone);
        }
        if let Some(connections) = connections {
            phase.flow.header.n_fluxes += connections.header.n_fluxes;
            phase.flow.fluxes.extend(connections.fluxes);
        }

        if phase.flow.fluxes.len() != phase.flow.header.n_fluxes as usize {
            panic!(
                "TODO: handle merge error {} {}",
                phase.flow.fluxes.len(),
                phase.flow.header.n_fluxes
            );
        }

        Ok(phase)
    }

    pub fn merge_from_memory(
        &self,
        dest: &str,
        connections: Option<[RawDataFlux; 2]>,
    ) -> Result<(), CMError> {
        
        const MERGE_FOLDER_NAME:&str = "merged";
        let liquid_phase = filter_phase(&self.raw_phase,PhaseCM::Liquid);
        let gasphase = filter_phase(&self.raw_phase,PhaseCM::Gas);
        
        let path = format!("{}/{}", dest,MERGE_FOLDER_NAME);
        std::fs::create_dir_all(&path)?; //FIXME
        let mut case = CMCase::default();
        // case.n_div = n_div;
        let relative = Some(String::from(MERGE_FOLDER_NAME));

        let liquid_connection = connections.as_ref().map(|c| c[0].clone());
        let gas_connection = connections.as_ref().map(|c| c[1].clone());

        let phase = Self::merge_phase(liquid_phase, liquid_connection)?;
        Self::write_phase(&path, &mut case, phase, relative.clone())?;

        if !gasphase.is_empty() {
            let phase = Self::merge_phase(gasphase, gas_connection)?;
            Self::write_phase(&path, &mut case, phase, relative)?;
        }

        CMCaseJson::write_case(case.clone(), Path::new(&format!("{}/cma_case", dest)))?;
        // cmtool_data::CCMCaseInfo::write_case(case, Path::new(&format!("{}/cma_case", dest)))?;
        Ok(())
    }

    pub fn merge(
        &self,
        dest: &str,
        ids: &[String],
        connections: Option<[RawDataFlux; 2]>,
    ) -> Result<(), CMError> {
        let mut liquid_flows = Vec::with_capacity(ids.len());
        let mut liquid_volumes = Vec::with_capacity(ids.len());
        let mut gas_flows = Vec::with_capacity(ids.len());
        let mut gas_volumes = Vec::with_capacity(ids.len());

        let mut n_div = [0, 0, 0];
        for id in ids.iter() {
            let case = CMCaseJson::read_case(Path::new(&format!("{}/{}/cma_case", dest, id)))?;
            let relative_path = format!("{}/{}", dest, id);
            let (liquid_flow, liquid_volume) = PAIRS.0;

            let path = resolve_path(&case, &relative_path, liquid_flow)?;
            let rf =
                RawDataFlux::read_raw(path).ok_or(CMError::Custom("Error reading".to_string()))?;

            let n_zone = rf.header.n_zone as usize;
            liquid_flows.push(rf);
            n_div[0] += case.n_div[0];
            n_div[1] += case.n_div[1];
            n_div[2] += case.n_div[2];

            let path = resolve_path(&case, &relative_path, liquid_volume)?;

            let sc = RawDataScalar::read_raw(path)
                .ok_or(CMError::Custom("Error reading".to_string()))?;
            liquid_volumes.push(sc);
            let (gas_flow, gas_volume) = PAIRS.1;
            let path = resolve_path(&case, &relative_path, gas_flow)?;

            let rf = match RawDataFlux::read_raw(path) {
                Some(rf) => rf,
                None => {
                    RawDataFlux::new(n_zone, 1) //Default implementation of RawFlux implies
                    //correct workaround
                }
            };
            gas_flows.push(rf);

            let path = resolve_path(&case, &relative_path, gas_volume)?;
            let rs = match RawDataScalar::read_raw(path) {
                Some(rs) => rs,
                None => {
                    let mut sc = RawDataScalar::new(n_zone);
                    sc.values.push((1e-9).into());
                    sc
                }
            };
            gas_volumes.push(rs);
        }

        let path = format!("{}/merged", dest);
        std::fs::create_dir_all(&path)?;
        let mut case = CMCase::default();
        case.n_div = n_div;

        let liquid_phases = get_raw_phase(liquid_flows, liquid_volumes, PhaseCM::Liquid);
        let gas_phases = get_raw_phase(gas_flows, gas_volumes, PhaseCM::Gas);

        let liquid_connection = connections.as_ref().map(|c| c[0].clone());
        let gas_connection = connections.as_ref().map(|c| c[1].clone());
        //TODO wont working without let relative = Some(String::from("merged"));
        let phase = Self::merge_phase(liquid_phases, liquid_connection)?;
        Self::write_phase(&path, &mut case, phase, None)?;

        if !gas_phases.is_empty() {
            let phase = Self::merge_phase(gas_phases, gas_connection)?;
            Self::write_phase(&path, &mut case, phase, None)?;
        }

        CMCaseJson::write_case(case.clone(), Path::new(&format!("{}/jcma_case", dest)))?;
        let _ =
            cmtool_data::CCMCaseInfo::write_case(case, Path::new(&format!("{}/cma_case", dest)));
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_0d() {
        let path = "/tmp/test_0d";
        std::fs::create_dir_all(path).unwrap();

        let case = Generator::new()
            .generate_0d_from_fraction(10., 0.2, Some(path.to_owned()))
            .expect("case");

        let liquid_volume_path = case
            .resolve(path, cmtool_data::CMAExportType::LiquidVolume)
            .expect("path");

        let liquid_volume =
            cmtool_data::RawDataScalar::read_raw(liquid_volume_path.clone()).expect("Liquid error");
        let gas_volume_path = case
            .resolve(path, cmtool_data::CMAExportType::GasVolume)
            .expect("path");

        let gas_volume =
            cmtool_data::RawDataScalar::read_raw(gas_volume_path.clone()).expect("gas_volume");
        std::fs::remove_dir_all(path).unwrap();

        assert_eq!(gas_volume.values.len(), 1);
        assert_eq!(gas_volume.values[0].value, 2.0);
        assert_eq!(liquid_volume.values[0].value, 8.0);
    }
    #[test]
    fn test_1d() {
        let path = "/tmp/test_1d";
        std::fs::create_dir_all(path).unwrap();
        let l = 1.;
        let d = 0.2;
        let alpha_g = 0.1;

        let desc = PFRDescription {
            n_compartment: 10,
            length: l,
            diameter: d,
            liquid_flow: 0.01,
            gas_flow: 0.01,
            gas_fraction: alpha_g,
            axial_dispersion: 1e-9,
        };

        let case = Generator::new()
            .generate_1d_from_fraction(desc, Some(path.to_owned()))
            .expect("case");

        let liquid_volume_path =
            resolve_path(&case, path, cmtool_data::CMAExportType::LiquidVolume).unwrap();

        let liquid_volume: f64 = cmtool_data::RawDataScalar::read_raw(liquid_volume_path.clone())
            .expect("Liquid error")
            .values
            .iter()
            .map(|v| v.value)
            .sum();
        std::fs::remove_dir_all(path).unwrap();
        //volume is h*pi*d^2/4
        let geo_volume = l * (d * d) * std::f64::consts::PI / 4.;

        assert!(liquid_volume - (1. - alpha_g) * geo_volume < 1e-9);
    }

    #[test]
    fn test_merge_phase() {
        let path = "/tmp/test_merge";
        std::fs::create_dir_all(path).unwrap();
        let l = 1.;
        let d = 0.2;
        let alpha_g = 0.1;

        let desc = PFRDescription {
            n_compartment: 10,
            length: l,
            diameter: d,
            liquid_flow: 0.01,
            gas_flow: 0.001,
            gas_fraction: alpha_g,
            axial_dispersion: 1e-9,
        };

        let case = Generator::new()
            .generate_1d_from_fraction(desc, Some(path.to_owned()))
            .expect("case");
        let liquid_volume_path = case
            .resolve(path, cmtool_data::CMAExportType::LiquidVolume)
            .expect("path");

        let liquid_volume: f64 = cmtool_data::RawDataScalar::read_raw(liquid_volume_path.clone())
            .expect("Liquid error")
            .values
            .iter()
            .map(|v| v.value)
            .sum();

        //volume is h*pi*d^2/4
        let geo_volume = l * (d * d) * std::f64::consts::PI / 4.;
        std::fs::remove_dir_all(path).unwrap();
        assert!(
            liquid_volume - (1. - alpha_g) * geo_volume < 1e-9,
            "liquid_volume {}, alpha {}, geo_volume {}",
            liquid_volume,
            alpha_g,
            geo_volume
        );
    }
}
