use cmtool_data::CMAExportType;
use cmtool_data::CMCase;
use cmtool_data::RawData;
use std::path::Path;
pub struct Generator {
    dest: String,
}

struct Field0D {
    name: String,
    value: f64,
}

impl Generator {
    pub fn new(root: &str) -> Self {
        Self {
            dest: root.to_string(),
        }
    }

    fn generate_0d(
        &self,
        liquid_volume: f64,
        gas_volume: f64,
        fields: Option<&[Field0D]>,
    ) -> Result<CMCase, ()> {
        let mut case = CMCase::default();
        case.n_div = [1, 0, 0];

        let mut rd = cmtool_data::RawDataFlux::new(1, 1);
        let mut f = &mut rd.fluxes[0];
        f.id_source = 0;
        f.id_target = 0;
        f.flux_source_target = 0.;
        f.flux_target_source = 0.;

        let relative_path = "flowL.raw";

        rd.write_raw(&format!("{}/{}", self.dest, relative_path))?;
        case.add(
            cmtool_data::CMAExportType::LiquidFlow,
            &format!("./{}", relative_path),
        );

        let mut volume = cmtool_data::RawDataScalar::new(1);
        let relative_path = "volL.raw";
        volume.values.push(cmtool_data::RawScalar {
            value: liquid_volume,
        });
        volume.write_raw(&format!("{}/{}", self.dest, relative_path))?;
        case.add(
            cmtool_data::CMAExportType::LiquidVolume,
            &format!("./{}", relative_path),
        );
        if gas_volume != 0. {
            let relative_path = "flowG.raw";
            rd.write_raw(&format!("{}/{}", self.dest, relative_path))?;
            case.add(
                cmtool_data::CMAExportType::GasFlow,
                &format!("./{}", relative_path),
            );

            let relative_path = "volG.raw";
            let mut volume = cmtool_data::RawDataScalar::new(1);
            volume
                .values
                .push(cmtool_data::RawScalar { value: gas_volume });
            volume.write_raw(&format!("{}/{}", self.dest, relative_path))?;
            case.add(
                cmtool_data::CMAExportType::GasVolume,
                &format!("./{}", relative_path),
            );
        }
        if let Some(scalars) = fields {
            todo!("Scalar field")
        }

        Ok(case)
    }

    pub fn generate_0d_from_fraction(
        &self,
        total_volume: f64,
        gas_fraction: f64,
    ) -> Result<CMCase, ()> {
        if gas_fraction < 0. || gas_fraction > 1. {
            panic!("TODO: handle error gas fraction generation 0d");
        }

        let gas_volume = gas_fraction * total_volume;
        let liquid_volume = total_volume * (1. - gas_fraction);

        self.generate_0d(liquid_volume, gas_volume, None)
    }

    fn generate_1d(
        &self,
        case: &mut CMCase,
        n_compartment: usize,
        length: f64,
        diameter: f64,
        flow: f64,
        volume_fraction: f64,
        axial_dispersion: f64,
        gas: bool,
    ) -> Result<(), ()> {
        let dx = length / (n_compartment as f64);
        let reactor_section_area = std::f64::consts::PI * diameter.powf(2.) / 4.;
        let compartment_volume = volume_fraction * dx * reactor_section_area;
        let flow_velocity = flow / reactor_section_area;
        let n_flow = n_compartment - 1;

        let flow_source_target = reactor_section_area / dx * (flow_velocity + axial_dispersion);

        let flow_target_source = reactor_section_area / dx * axial_dispersion;

        let tolerance = 1e-5;
        assert!(
            ((compartment_volume * n_compartment as f64)
                - (volume_fraction * reactor_section_area * length))
                .abs()
                < 1e-5,
            "Volume mismatch: got {}, expected {}, diff {}",
            compartment_volume * n_compartment as f64,
            volume_fraction * reactor_section_area * length,
            ((compartment_volume * n_compartment as f64)
                - (volume_fraction * reactor_section_area * length))
                .abs()
        );
        let mut flow_rd = cmtool_data::RawDataFlux::new(n_compartment, n_flow);

        let mut vol_rd = cmtool_data::RawDataScalar::new(n_compartment);
        for (current_index, flow) in flow_rd.fluxes.iter_mut().enumerate() {
            flow.id_source = current_index as u32;
            flow.id_target = (current_index + 1) as u32;
            flow.flux_source_target = flow_source_target;
            flow.flux_target_source = flow_target_source;

            vol_rd.values.push(compartment_volume.into());
        }

        let phase_id = if gas { "G" } else { "L" };

        let relative_path = format!("flow{}.raw", phase_id);
        flow_rd.write_raw(&format!("{}/{}", self.dest, relative_path))?;

        let volrelative_path = format!("vol{}.raw", phase_id);
        vol_rd.write_raw(&format!("{}/{}", self.dest, volrelative_path))?;

        let mut f_addcase = |flowtype: cmtool_data::CMAExportType,
                             voltype: cmtool_data::CMAExportType| {
            case.add(flowtype, &format!("./{}", relative_path));
            case.add(voltype, &format!("./{}", volrelative_path));
        };

        if gas {
            f_addcase(CMAExportType::GasFlow, CMAExportType::GasVolume);
        } else {
            f_addcase(CMAExportType::LiquidFlow, CMAExportType::LiquidVolume);
        }

        Ok(())
    }

    pub fn generate_1d_from_fraction(
        &self,
        n_compartment: usize,
        length: f64,
        diameter: f64,
        flow: f64,
        gas_fraction: f64,
        axial_dispersion: f64
    ) -> Result<CMCase, ()> {
        let mut case = CMCase::default();
        case.n_div = [0, 0, n_compartment as u32];

        if gas_fraction < 0. || gas_fraction > 1. {
            panic!("TODO: handle error gas fraction generation 0d");
        }

        self.generate_1d(
            &mut case,
            n_compartment,
            length,
            diameter,
            flow,
            1.0 - gas_fraction,
            axial_dispersion,
            false,
        )?;
        if gas_fraction != 0. {
            self.generate_1d(
                &mut case,
                n_compartment,
                length,
                diameter,
                flow,
                1.0 - gas_fraction,
                axial_dispersion,
                true,
            )?;
        }

        Ok(case)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    fn test_0d() {
        let case = Generator::new("/tmp")
            .generate_0d_from_fraction(10., 0.2)
            .expect("AA");

        let path = case
            .resolve("/tmp", cmtool_data::CMAExportType::LiquidVolume)
            .expect("AAA");

        let liquid_volume = cmtool_data::RawDataScalar::read_raw(path).expect("Liquid error");
        let path = case
            .resolve("/tmp", cmtool_data::CMAExportType::GasVolume)
            .expect("AAA");

        let gas_volume = cmtool_data::RawDataScalar::read_raw(path).expect("AAA");
        assert_eq!(gas_volume.values.len(), 1);
        assert_eq!(gas_volume.values[0].value, 2.0);
        assert_eq!(liquid_volume.values[0].value, 8.0);

        //TODO clean
    }
    #[test]
    fn test_1d() {
        let l = 1.;
        let d = 0.2;
        let alpha_g = 0.1;
        let case = Generator::new("/tmp")
            .generate_1d_from_fraction(10, l, d, 0.01, alpha_g, 1e-9)
            .expect("AA");
        let path = case
            .resolve("/tmp", cmtool_data::CMAExportType::LiquidVolume)
            .expect("AAA");

        let liquid_volume: f64 = cmtool_data::RawDataScalar::read_raw(path)
            .expect("Liquid error")
            .values
            .iter()
            .map(|v| v.value)
            .sum();

        //volume is h*pi*d^2/4
        let geo_volume = l * (d * d) * std::f64::consts::PI / 4.;

        assert!(liquid_volume - (1.-alpha_g)*geo_volume < 1e-9);
        //TODO clean
    }
}
