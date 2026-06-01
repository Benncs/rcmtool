// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

use crate::CMError;
mod artefact;
pub use artefact::GenerateContract;
mod descriptors;

use cmtool_data::{
    CMAExportType, CMCase, CMCaseJson, CMCaseReader, DEFAULT_CASE_FILE_NAME, PhaseCM, RawData,
    RawDataFlux, RawDataScalar, RawFlux, RawPhase,
};
pub use descriptors::{PFRDescription, Reactor0DDescriptor};

const LIQUID_PAIR: (CMAExportType, CMAExportType) =
    (CMAExportType::LiquidFlow, CMAExportType::LiquidVolume);

const GAS_PAIR: (CMAExportType, CMAExportType) = (CMAExportType::GasFlow, CMAExportType::GasVolume);

type PairType = (CMAExportType, CMAExportType);

const PAIRS: (PairType, PairType) = (LIQUID_PAIR, GAS_PAIR);

//TODO improve and change name
pub struct Generator {
    raw_phase: Vec<RawPhase>,
}

struct Field0D {
    #[allow(unused)]
    name: String,
    #[allow(unused)]
    value: f64,
}
///wrapper Get absolute path from relative
fn resolve_path(
    case: &CMCase,
    relative_path: impl AsRef<std::path::Path>,
    value: CMAExportType,
) -> Result<String, CMError> {
    case.resolve(relative_path.as_ref().to_str().expect("UTF-8 path"), value)
        .ok_or(CMError::Custom("Error resolving path".to_string()))
}

///Create vector of raw phase from raw
fn raw_phase_from_flow_vol(
    flows: Vec<RawDataFlux>,
    vol: Vec<RawDataScalar>,
    phase: PhaseCM,
) -> Vec<RawPhase> {
    flows
        .into_iter()
        .zip(vol)
        .map(move |(f, v)| RawPhase {
            flow: f,
            volume: v,
            identifier: phase,
        })
        .collect()
}

// Select specific phase type in a slice of phases
// fn filter_phase(raw_phase: &[RawPhase], phase: PhaseCM) -> Vec<RawPhase> {
//     raw_phase
//         .iter()
//         .filter(|p| p.identifier == phase)
//         .cloned()
//         .collect()
// }
fn filter_phase(raw_phase: &[RawPhase], phase: PhaseCM) -> impl Iterator<Item = &RawPhase> {
    raw_phase.iter().filter(move |p| p.identifier == phase)
}

const MERGE_FOLDER_NAME: &str = "merged";

//public
impl Generator {
    pub fn new() -> Self {
        Self {
            raw_phase: Default::default(),
        }
    }

    pub fn generate_0d(
        &mut self,
        descriptor: Reactor0DDescriptor,
        dest: Option<String>,
    ) -> Result<CMCase, CMError> {
        self.impl_generate_0d(descriptor, None, dest)
    }

    pub fn generate_1d(
        &mut self,
        descriptor: PFRDescription,
        out_dir: Option<String>,
    ) -> Result<CMCase, CMError> {
        //Method expects descriptor has been validated with is_valid method

        let mut case = CMCase::default();
        case.n_div = [0, 0, descriptor.n_compartment as u32];

        self.impl_generate_1d(&mut case, &descriptor, PhaseCM::Liquid, out_dir.clone())?;
        if descriptor.gas_fraction != 0. {
            self.impl_generate_1d(&mut case, &descriptor, PhaseCM::Gas, out_dir)?;
        }

        Ok(case)
    }

    pub fn merge_from_memory(
        &self,
        connections: Option<[RawDataFlux; 2]>,
    ) -> Result<GenerateContract, CMError> {
        let liquid_phase = filter_phase(&self.raw_phase, PhaseCM::Liquid);
        let gas_phase = filter_phase(&self.raw_phase, PhaseCM::Gas);
        let case = CMCase::default();
        let relative = Some(String::from(MERGE_FOLDER_NAME));

        let (liquid_phase, gas_phase) = Self::impl_merge(liquid_phase, gas_phase, connections)?;

        Ok(GenerateContract::new(
            case,
            liquid_phase,
            gas_phase,
            relative,
        ))
    }

    pub fn merge(
        &self,
        root_dir: impl AsRef<std::path::Path>,
        ids: &[String],
        connections: Option<[RawDataFlux; 2]>,
    ) -> Result<GenerateContract, CMError> {
        //helpers
        let read_flux = |partial_case: &CMCase,
                         relative_path: &std::path::PathBuf,
                         key: CMAExportType|
         -> Result<Option<RawDataFlux>, CMError> {
            let p = resolve_path(partial_case, relative_path, key)?;
            Ok(RawDataFlux::read_raw(p))
        };

        let read_scalar = |partial_case: &CMCase,
                           relative_path: &std::path::PathBuf,
                           key: CMAExportType|
         -> Result<Option<RawDataScalar>, CMError> {
            let p = resolve_path(partial_case, relative_path, key)?;
            Ok(RawDataScalar::read_raw(p))
        };

        let mut n_div = [0, 0, 0];
        //helper
        let mut add_ndiv = |n: &[u32; 3]| {
            n_div[0] += n[0];
            n_div[1] += n[1];
            n_div[2] += n[2];
        };

        let read_partial_case = |relative_path: &PathBuf| {
            let case_path = relative_path.join(DEFAULT_CASE_FILE_NAME);
            CMCaseJson::read_case(&case_path)
        };

        //prealloc
        let mut liquid_flows = Vec::with_capacity(ids.len());
        let mut liquid_volumes = Vec::with_capacity(ids.len());
        let mut gas_flows = Vec::with_capacity(ids.len());
        let mut gas_volumes = Vec::with_capacity(ids.len());

        let mut impl_merge_reactor = |id: &str| -> Result<(), CMError> {
            let relative_path = root_dir.as_ref().join(id);
            let partial_case = read_partial_case(&relative_path)?;

            let (liquid_flow, liquid_volume) = PAIRS.0;
            let (gas_flow, gas_volume) = PAIRS.1;

            let raw_liquid_flow = read_flux(&partial_case, &relative_path, liquid_flow)?
                .ok_or_else(|| CMError::Custom("Error reading liquid flow".into()))?;

            let n_zone = raw_liquid_flow.header.n_zone as usize;
            liquid_flows.push(raw_liquid_flow);
            add_ndiv(&partial_case.n_div);

            let sc_liquid_volume = read_scalar(&partial_case, &relative_path, liquid_volume)?
                .ok_or_else(|| CMError::Custom("Error reading liquid volume".into()))?;
            liquid_volumes.push(sc_liquid_volume);

            // gas flow: fallback to default when missing
            // Default implementation of RawFlux implies correct workaround
            let raw_flow_gas = read_flux(&partial_case, &relative_path, gas_flow)?
                .unwrap_or_else(|| RawDataFlux::new(n_zone, 1));
            gas_flows.push(raw_flow_gas);

            // gas volume: fallback to default scalar when missing
            let sc_gas_volume = read_scalar(&partial_case, &relative_path, gas_volume)?
                .unwrap_or_else(|| {
                    let mut sc = RawDataScalar::new(n_zone);
                    sc.values.push((1e-9).into());
                    sc
                });
            gas_volumes.push(sc_gas_volume);

            Ok(())
        };

        ids.iter().try_for_each(|id| impl_merge_reactor(id))?;

        let merge_path = root_dir.as_ref().join(MERGE_FOLDER_NAME);

        std::fs::create_dir_all(&merge_path)?;
        let mut case = CMCase::default();
        case.n_div = n_div;

        let liquid_phases = raw_phase_from_flow_vol(liquid_flows, liquid_volumes, PhaseCM::Liquid);
        let gas_phases = raw_phase_from_flow_vol(gas_flows, gas_volumes, PhaseCM::Gas);

        let (liquid_phase, gas_phase) =
            Self::impl_merge(liquid_phases.iter(), gas_phases.iter(), connections)?;

        Ok(GenerateContract::new(
            // &path,
            case,
            liquid_phase,
            gas_phase,
            None,
        ))
    }
}

//private impl
impl Generator {
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
            Some(s) => {
                *case = GenerateContract::new_single_phase(case.clone(), phase, None).write(&s)?;
                Ok(())
            }
            None => {
                self.raw_phase.push(phase);
                Ok(())
            }
        }
    }

    fn impl_generate_0d(
        &mut self,
        descriptor: Reactor0DDescriptor,
        fields: Option<&[Field0D]>,
        dest: Option<String>,
    ) -> Result<CMCase, CMError> {
        let mut case = CMCase::default();
        case.n_div = [1, 0, 0];

        self.generate_0d_phase(
            &mut case,
            descriptor.liquid_volume(),
            PhaseCM::Liquid,
            dest.clone(),
        )?;
        let gas_volume = descriptor.gas_volume();
        if gas_volume != 0. {
            self.generate_0d_phase(&mut case, gas_volume, PhaseCM::Gas, dest)?;
        }

        if let Some(_scalars) = fields {
            todo!("Scalar field")
        }

        Ok(case)
    }

    fn impl_generate_1d(
        &mut self,
        case: &mut CMCase,
        descriptor: &PFRDescription,
        phase_d: PhaseCM,
        out_dir: Option<String>,
    ) -> Result<(), CMError> {
        //Get needed info
        let &PFRDescription {
            n_compartment,
            length,
            diameter,
            axial_dispersion,
            ..
        } = descriptor;

        let (volume_fraction, flow) = descriptor.extract_volume_flow(phase_d);

        debug_assert!(length > 0.);
        debug_assert!(diameter > 0.);
        debug_assert!(flow >= 0.);
        debug_assert!(axial_dispersion >= 0.);
        debug_assert!(volume_fraction <= 1. && volume_fraction > 0.);

        //Geometrical data

        let dx = length / (n_compartment as f64);
        let reactor_section_area = std::f64::consts::PI * diameter.powf(2.) / 4.;
        let compartment_volume = volume_fraction * dx * reactor_section_area;
        let n_flow = n_compartment - 1;

        //PreCalculate velocity
        let flow_velocity = flow / reactor_section_area;
        let flow_source_target = reactor_section_area / dx * (flow_velocity + axial_dispersion);
        let flow_target_source = reactor_section_area / dx * axial_dispersion;

        //Discretization should preserve geometrical volume
        assert!(
            ((compartment_volume * n_compartment as f64)
                - (volume_fraction * reactor_section_area * length))
                .abs()
                < 1e-5
        );

        let mut phase = RawPhase::new(n_compartment, n_flow, phase_d);

        //n_flow != n_compartment, need to set volume separately
        phase.volume = RawDataScalar::from(vec![compartment_volume; n_compartment]); //Scalar values are not preallo

        //Set flow for each compartment
        for (current_index, flow) in phase.flow.fluxes.iter_mut().enumerate() {
            flow.id_source = current_index as u32;
            flow.id_target = (current_index + 1) as u32;
            flow.flux_source_target = flow_source_target;
            flow.flux_target_source = flow_target_source;
        }

        //Select whether generate in place or defered
        if let Some(s) = out_dir {
            *case = GenerateContract::new_single_phase(case.clone(), phase, None).write(&s)?;
            // GenerateContract::write_phase(&s, case, phase, None)?;
        } else {
            self.raw_phase.push(phase);
        }

        Ok(())
    }

    fn merge_phase<'a, I>(
        // flows: Vec<RawDataFlux>,
        // volumes: Vec<RawDataScalar>
        phases: I,
        connections: Option<RawDataFlux>,
    ) -> Result<RawPhase, CMError>
    where
        I: Iterator<Item = &'a RawPhase>,
    {
        let phases = &mut phases.into_iter().peekable();

        if phases.peek().is_none() {
            return Err(CMError::Custom("Empty phases".to_owned()));
        }

        let next = phases.peek().unwrap();

        let mut phase = RawPhase::new(0, 0, next.identifier);

        let offset_compartment = std::cell::Cell::new(0u32);
        let incr_id = |mut flow: RawFlux| -> RawFlux {
            flow.id_source += offset_compartment.get();
            flow.id_target += offset_compartment.get();
            flow
        };
        phases.for_each(|p| {
            let rd = &p.flow;
            let v = &p.volume;
            phase.flow.header.n_zone += rd.header.n_zone;
            phase.volume.header.n_zone += rd.header.n_zone;
            phase.flow.header.n_fluxes += rd.header.n_fluxes;
            phase
                .flow
                .fluxes
                .extend(rd.fluxes.iter().map(|&flux| incr_id(flux)));
            phase.volume.values.extend(v.values.clone());
            offset_compartment.set(offset_compartment.get() + rd.header.n_zone);
        });

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

    fn impl_merge<'a>(
        liquid_phase: impl Iterator<Item = &'a RawPhase>,
        gas_phase: impl Iterator<Item = &'a RawPhase>,
        connections: Option<[RawDataFlux; 2]>,
    ) -> Result<(RawPhase, Option<RawPhase>), CMError> {
        let liquid_connection = connections.as_ref().map(|c| c[0].clone());
        let gas_connection = connections.as_ref().map(|c| c[1].clone());

        let merged_liquid_phase = Self::merge_phase(liquid_phase, liquid_connection)?;
        let gas_phase = &mut gas_phase.peekable();
        let merged_gas_phase = if gas_phase.peek().is_some() {
            Some(Self::merge_phase(gas_phase, gas_connection)?)
        } else {
            None
        };
        Ok((merged_liquid_phase, merged_gas_phase))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn g_descriptor_pfr() -> PFRDescription {
        let l = 1.;
        let d = 0.2;
        let alpha_g = 0.1;

        PFRDescription::new(10, l, d, 0.01, 0.01, alpha_g, 1e-9).unwrap()
    }

    #[test]
    fn test_0d() {
        let path = "/tmp/test_0d";
        std::fs::create_dir_all(path).unwrap();

        let case = Generator::new()
            .generate_0d(
                Reactor0DDescriptor::from_fraction(10., 0.2),
                Some(path.to_owned()),
            )
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
    fn test_merge_lazy() {
        let path = "/tmp/test_merge_lazy";
        std::fs::create_dir_all(path).unwrap();
        //same value in g_descriptor_pfr
        let l = 1.;
        let d = 0.2;

        let desc = g_descriptor_pfr();
        let alpha_g = desc.get_gas_fraction();
        //volume is h*pi*d^2/4
        let geo_volume = l * (d * d) * std::f64::consts::PI / 4.;
        assert!(geo_volume == desc.geometrical_volume());

        let mut generator = Generator::new();
        let _case = generator.generate_1d(desc, None).expect("case");

        let c = generator.merge_from_memory(None).expect("merge");

        let case = c.write(path).expect("write");
        let liquid_volume_path =
            resolve_path(&case, path, cmtool_data::CMAExportType::LiquidVolume).unwrap();

        let liquid_volume: f64 = cmtool_data::RawDataScalar::read_raw(liquid_volume_path.clone())
            .expect("Liquid error")
            .values
            .iter()
            .map(|v| v.value)
            .sum();
        std::fs::remove_dir_all(path).unwrap();

        assert!(liquid_volume - (1. - alpha_g) * geo_volume < 1e-9);
    }

    #[test]
    fn test_merge_phase_2() {
        let path = "/tmp/test_merge_phase_2";
        std::fs::create_dir_all(path).unwrap();

        let v_0d = 10.;
        let n_c = 10;

        let desc_pfr = g_descriptor_pfr();
        let alpha_g = desc_pfr.get_gas_fraction();
        let geo_volume = desc_pfr.geometrical_volume();
        let desc_0d = Reactor0DDescriptor::from_fraction(v_0d, alpha_g);

        let mut gene = Generator::new();

        gene.generate_1d(desc_pfr, None).unwrap();
        gene.generate_0d(desc_0d, None).unwrap();

        let gc = gene.merge_from_memory(None).unwrap();

        let case = gc.write(path).unwrap();

        let liquid_volume_path = case
            .resolve(path, cmtool_data::CMAExportType::LiquidVolume)
            .expect("path");

        let liquid_volume =
            cmtool_data::RawDataScalar::read_raw(liquid_volume_path.clone()).expect("Liquid error");
        assert!(liquid_volume.header.n_zone as usize == n_c + 1);

        let total_volume: f64 = liquid_volume.values.iter().map(|v| v.value).sum();
        let expected_volume = (1. - alpha_g) * (geo_volume + v_0d);
        assert!((total_volume - expected_volume).abs() < 1e-9);

        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn test_merge_phase_1() {
        let path = "/tmp/test_merge";
        std::fs::create_dir_all(path).unwrap();
        let desc_pfr = g_descriptor_pfr();
        let alpha_g = desc_pfr.get_gas_fraction();
        let geo_volume = desc_pfr.geometrical_volume();

        let case = Generator::new()
            .generate_1d(desc_pfr, Some(path.to_owned()))
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
        std::fs::remove_dir_all(path).unwrap();
        assert!(
            liquid_volume - (1. - alpha_g) * geo_volume < 1e-9,
            "liquid_volume {}, alpha {}, geo_volume {}",
            liquid_volume,
            alpha_g,
            geo_volume
        );
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
            gas_flow: 0.001,
            gas_fraction: alpha_g,
            axial_dispersion: 1e-9,
        };

        let case = Generator::new()
            .generate_1d(desc, Some(path.to_owned()))
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

        let gas_volume_path = case
            .resolve(path, cmtool_data::CMAExportType::GasVolume)
            .expect("path");
        let gas_volume: f64 = cmtool_data::RawDataScalar::read_raw(gas_volume_path.clone())
            .expect("gas error")
            .values
            .iter()
            .map(|v| v.value)
            .sum();

        //volume is h*pi*d^2/4
        let geo_volume = l * (d * d) * std::f64::consts::PI / 4.;
        std::fs::remove_dir_all(path).unwrap();
        assert!(
            (liquid_volume - (1. - alpha_g) * geo_volume).abs() < 1e-9,
            "liquid_volume {}, alpha {}, geo_volume {}",
            liquid_volume,
            alpha_g,
            geo_volume
        );

        assert!(
            ((liquid_volume + gas_volume) - geo_volume).abs() < 1e-9,
            "liquid_volume {}, gas_volume {}, geo_volume {}",
            liquid_volume,
            gas_volume,
            geo_volume
        );
    }
}
