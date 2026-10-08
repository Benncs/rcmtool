use std::path::Path;

use cmtool::CmtoolError;
use cmtool_core::Reduction;
use cmtool_data::{
    CMAExportType, CMCase, CMCaseJson, CMCaseWriter, DEFAULT_CASE_FILE_NAME, RawData, RawDataScalar,
};

use crate::{
    args::{CommonArgs, ManualArgs},
    balance_settings, out_or_default,
};

///Flow map of one phase, out of what the export provides: a vector variable holds the velocity
///directly, otherwise it comes as its three components and they have to be merged.
fn dump_phase_flow(
    handle: &cmtool_core::CMHandle,
    root: &str,
    files: &[String],
    res_name: String,
    fraction: Option<cmtool_core::model::Scalar>,
    phase: Option<cmtool_core::model::OpenPhase>,
) -> Result<Option<cmtool_core::model::BalanceReport>, CmtoolError> {
    let path = |name: &String| Path::new(root).join(name);

    let report = match files {
        [vector] => match fraction {
            Some(fraction) => {
                handle.dump_vector_phase_fraction(res_name, path(vector), fraction, phase)?
            }
            None => {
                handle.dump_vector(res_name, path(vector))?;
                None
            }
        },
        [i, j, k] => {
            handle
                .dump_vector_from_scalar(res_name, path(i), path(j), path(k), fraction, phase)?
                .1
        }
        _ => {
            return Err(CmtoolError::Custom(format!(
                "A phase is either one vector file or its three components, {} were given",
                files.len()
            )));
        }
    };

    Ok(report)
}

///Builds a case out of velocities given as separate scalar components, which is how a CFD export
///stores them when it holds no vector variable
pub fn manual_main(common: CommonArgs, args: ManualArgs) -> Result<(), CmtoolError> {
    if args.liquid.is_empty()
        && args.gas.is_empty()
        && args.vectors.is_empty()
        && args.scalars.is_empty()
        && args.scalar_means.is_empty()
        && args.epsilon.is_none()
    {
        return Err(CmtoolError::Custom(String::from(
            "Nothing to generate: give a scalar, a vector, or the three velocity components of a phase",
        )));
    }

    let root_dir = out_or_default(common.out.clone());
    std::fs::create_dir_all(&root_dir).map_err(cmtool_data::DataError::IO)?;

    let mut vtk_scalars = Vec::new();
    let mut handle = cmtool_core::CMHandle::init(
        [common.n_i, common.n_j, common.n_k],
        &args.root,
        &args.geo_file,
        cmtool_core::grid::MeshType::Cylindrical,
    )?;
    handle.set_balance_settings(balance_settings(&common, &handle));

    let scalar_path = |name: &str| Path::new(&args.root).join(name);
    let out_path = |name: &str| format!("{}/{}", root_dir, name);

    let mut case = CMCase::new(
        [common.n_i as u32, common.n_j as u32, common.n_k as u32],
        0.,
        Some(String::from("generated from scalar components")),
        false,
    );

    //The gas fraction splits the phases: a phase carries its share of the flow and of the volume
    let gas_fraction = args
        .gas_fraction
        .as_ref()
        .map(|name| handle.get_scalar(scalar_path(name)))
        .transpose()?;

    let mut has_flow_map = false;

    if !args.liquid.is_empty() {
        //Liquid takes what the gas leaves
        let fraction = args
            .gas_fraction
            .as_ref()
            .map(|name| {
                handle
                    .get_scalar(scalar_path(name))
                    .map(|s| s.scalar_shift(1.))
            })
            .transpose()?;
        dump_phase_flow(
            &handle,
            &args.root,
            &args.liquid,
            out_path("flowL"),
            fraction,
            None,
        )?;
        case.add(CMAExportType::LiquidFlow, "flowL.raw");
        has_flow_map = true;
    }

    if !args.gas.is_empty() {
        //TODO: quick and dirty, gas is open and liquid closed. Sparger of cfd_test_3 hardcoded as
        //a ring (disk r<=0.0147 at z=0, ring at its half area radius), Q = alpha*v*A of the Fluent
        //inlet

        // let phase = cmtool_core::model::OpenPhase(
        //     cmtool_core::model::InletPosition::Ring(
        //         [0., 0., 0.],
        //         0.0147 / std::f64::consts::SQRT_2,
        //         1e-4,
        //     ),
        //     0.049789912 * 0.25890034 * std::f64::consts::PI * 0.0147 * 0.0147,
        // );

        let q_sanofi = 2700.; //m3/h
        //TODO: quick and dirty, sparger dimensions unknown: the whole bottom layer gets the gas
        let grid = handle.grid();
        let bottom = (0..grid.number_cell())
            .filter(|&cell| grid.cell_points(cell)[2] == 0)
            .collect();
        let phase_sanofi = cmtool_core::model::OpenPhase(
            cmtool_core::model::InletPosition::Positions(bottom),
            q_sanofi / 3600.,
        );

        // let mut b = *handle.balance_settings();
        // b.do_balance = false;
        // handle.set_balance_settings(b);
        let report = dump_phase_flow(
            &handle,
            &args.root,
            &args.gas,
            out_path("flowG"),
            gas_fraction,
            Some(phase_sanofi),
        )?;
        case.add(CMAExportType::GasFlow, "flowG.raw");

        //Compartments the gas went in and out through, dense: 0 outside of them
        if let Some(report) = report {
            let n_zone = handle.grid().number_cell();
            for (boundary, file, stype) in [
                (&report.inlet, "inletG.raw", CMAExportType::GasInlet),
                (&report.outlet, "outletG.raw", CMAExportType::GasOutlet),
            ] {
                let mut dense = vec![0.; n_zone];
                boundary.iter().for_each(|&(zone, f)| dense[zone] = f);
                RawDataScalar::from(dense.as_slice()).write_raw(&out_path(file))?;
                case.add(stype, file);
            }
        }
    }

    //Integrating the gas fraction over a compartment gives the volume the gas occupies in it
    let total_volume = handle.real_volume();
    let gas_volume = match &args.gas_fraction {
        Some(name) => handle
            .dump_scalar(out_path("vofG"), scalar_path(name), Reduction::Integral)?
            .values
            .iter()
            .map(|v| v.value)
            .collect(),
        None => vec![0.; total_volume.len()],
    };

    let liquid_volume: Vec<f64> = total_volume
        .iter()
        .zip(&gas_volume)
        .map(|(total, gas)| total - gas)
        .collect();

    RawDataScalar::from(liquid_volume.as_slice()).write_raw(&out_path("vofL.raw"))?;
    case.add(CMAExportType::LiquidVolume, "vofL.raw");
    if args.gas_fraction.is_some() {
        case.add(CMAExportType::GasVolume, "vofG.raw");
    }

    //Epsilon is the one of the liquid, averaged over the liquid volume of the compartment
    if let Some(name) = &args.epsilon {
        let tmp_sc = match &args.gas_fraction {
            Some(gas) => handle.dump_scalar_fraction(
                out_path("epsturb"),
                scalar_path(name),
                handle.get_scalar(scalar_path(gas))?.scalar_shift(1.),
                Reduction::Mean,
            )?,
            None => handle.dump_scalar(out_path("epsturb"), scalar_path(name), Reduction::Mean)?,
        };
        case.add(CMAExportType::EnergyDissipation, "epsturb.raw");
        vtk_scalars.push((tmp_sc, name.to_owned()));
    }

    //A scalar is integrated over each compartment, a vector becomes a flow map of its own
    let file_name = |path: &str| {
        Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(path)
            .to_owned()
    };
    for scalar in &args.scalars {
        let tmp_sc = handle.dump_scalar(
            out_path(&file_name(scalar)),
            scalar_path(scalar),
            Reduction::Integral,
        )?;
        vtk_scalars.push((tmp_sc, scalar.to_owned()));
    }

    for scalar in &args.scalar_means {
        let tmp_sc = handle.dump_scalar(
            out_path(&file_name(scalar)),
            scalar_path(scalar),
            Reduction::Mean,
        )?;
        vtk_scalars.push((tmp_sc, scalar.to_owned()));
    }

    for vector in &args.vectors {
        handle.dump_vector(out_path(&file_name(vector)), scalar_path(vector))?;
    }

    //A case needs a flow map to go with its volumes, dumping scalars alone does not make one
    if has_flow_map {
        CMCaseJson::write_case(case, &Path::new(&root_dir).join(DEFAULT_CASE_FILE_NAME))?;
        println!("Case written in {}", root_dir);
    } else {
        println!(
            "Files written in {}, no case: it holds no flow map",
            root_dir
        );
    }

    #[cfg(feature = "use_vtk")]
    handle.export_vtk(format!("{}/cma_case.vtu", root_dir), vtk_scalars)?;

    Ok(())
}
