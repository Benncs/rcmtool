// SPDX-License-Identifier: GPL-3.0-or-later

use crate::CMError;
use cmtool_data::PhaseCM;

pub struct Reactor0DDescriptor {
    liquid_volume: f64,
    gas_volume: f64,
}

impl Reactor0DDescriptor {
    pub fn liquid_volume(&self) -> f64 {
        self.liquid_volume
    }

    pub fn gas_volume(&self) -> f64 {
        self.gas_volume
    }

    pub fn is_valid(&self) -> Result<(), CMError> {
        if !self.liquid_volume.is_finite() || self.liquid_volume <= 0.0 {
            return Err(CMError::Descriptor(
                "Liquid volume must be a finite positive number".to_owned(),
            ));
        }
        //TODO
        if self.gas_volume < 0. {
            return Err(CMError::Descriptor(
                "Gas volume must be a finite positive number".to_owned(),
            ));
        }

        Ok(())
    }

    pub fn new<L, G>(liquid_volume: L, gas_volume: G) -> Result<Self, CMError>
    where
        L: Into<f64>,
        G: Into<f64>,
    {
        let reactor = Self {
            liquid_volume: liquid_volume.into(),
            gas_volume: gas_volume.into(),
        };

        reactor.is_valid()?;
        Ok(reactor)
    }

    pub fn from_fraction(total_volume: f64, gas_fraction: f64) -> Result<Self, CMError> {
        if !(0. ..=1.).contains(&gas_fraction) {
            return Err(CMError::Descriptor(format!(
                "Gas fraction must be between 0 and 1, got {}",
                gas_fraction
            )));
        }
        let gas_volume = gas_fraction * total_volume;
        let liquid_volume = total_volume - gas_volume;

        Self::new(liquid_volume, gas_volume)
    }
}

pub struct PFRDescription {
    pub(super) n_compartment: usize,
    pub(super) length: f64,
    pub(super) diameter: f64,
    pub(super) liquid_flow: f64,
    pub(super) gas_flow: f64,
    pub(super) gas_fraction: f64,
    pub(super) axial_dispersion: f64,
}

impl PFRDescription {
    pub fn new(
        n_compartment: usize,
        length: impl Into<f64>,
        diameter: impl Into<f64>,
        liquid_flow: impl Into<f64>,
        gas_flow: impl Into<f64>,
        gas_fraction: impl Into<f64>,
        axial_dispersion: impl Into<f64>,
    ) -> Result<PFRDescription, CMError> {
        let pfr = Self {
            n_compartment,
            length: length.into(),
            diameter: diameter.into(),
            liquid_flow: liquid_flow.into(),
            gas_flow: gas_flow.into(),
            gas_fraction: gas_fraction.into(),
            axial_dispersion: axial_dispersion.into(),
        };

        pfr.is_valid()?;
        Ok(pfr)
    }

    pub fn get_liquid_flow(&self) -> f64 {
        self.liquid_flow
    }

    pub fn get_gas_flow(&self) -> f64 {
        self.gas_flow
    }
    pub fn get_gas_fraction(&self) -> f64 {
        self.gas_fraction
    }
    pub fn get_liquid_fraction(&self) -> f64 {
        1. - self.gas_fraction
    }
    //volume is h*pi*d^2/4
    #[allow(unused)]
    pub fn geometrical_volume(&self) -> f64 {
        self.length * (self.diameter * self.diameter) * std::f64::consts::PI / 4.
    }

    pub fn extract_volume_flow(&self, phase: PhaseCM) -> (f64, f64) {
        match phase {
            PhaseCM::Liquid => (self.get_liquid_fraction(), self.get_liquid_flow()),
            PhaseCM::Gas => (self.get_gas_fraction(), self.get_gas_flow()),
        }
    }

    pub fn is_valid(&self) -> Result<(), CMError> {
        if self.n_compartment == 0 {
            return Err(CMError::Descriptor("n_compartment must be >= 1".to_owned()));
        }
        if !self.length.is_finite() || self.length <= 0.0 {
            return Err(CMError::Descriptor(
                "length must be a finite positive number".to_owned(),
            ));
        }
        if !self.diameter.is_finite() || self.diameter <= 0.0 {
            return Err(CMError::Descriptor(
                "diameter must be a finite positive number".to_owned(),
            ));
        }
        if !self.liquid_flow.is_finite() || self.liquid_flow < 0.0 {
            return Err(CMError::Descriptor(
                "liquid_flow must be a finite non-negative number".to_owned(),
            ));
        }
        if !self.gas_flow.is_finite() || self.gas_flow < 0.0 {
            return Err(CMError::Descriptor(
                "gas_flow must be a finite non-negative number".to_owned(),
            ));
        }
        if !self.gas_fraction.is_finite() || self.gas_fraction < 0.0 || self.gas_fraction > 1.0 {
            return Err(CMError::Descriptor(
                "gas_fraction must be between 0.0 and 1.0".to_owned(),
            ));
        }
        if !self.axial_dispersion.is_finite() || self.axial_dispersion < 0.0 {
            return Err(CMError::Descriptor(
                "axial_dispersion must be a finite non-negative number".to_owned(),
            ));
        }

        if self.liquid_flow == 0.0 && self.gas_flow == 0.0 {
            return Err(CMError::Descriptor(
                "At least one of liquid_flow or gas_flow must be positive".to_owned(),
            ));
        }
        if (self.gas_flow > 0.0 && self.liquid_flow == 0.0)
            && (self.gas_fraction < 0.0 || self.gas_fraction > 1.0)
        {
            return Err(CMError::Descriptor(
                "Invalid gas_fraction given flows".to_owned(),
            ));
        }

        // geometric length should be >= diameter
        if self.length < self.diameter {
            return Err(CMError::Descriptor(
                "length should be greater than or equal to diameter".to_owned(),
            ));
        }

        Ok(())
    }
}
