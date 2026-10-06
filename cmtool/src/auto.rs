use super::*;

pub fn auto_main(common: CommonArgs, autoargs: AutoArgs) -> Result<(), CmtoolError> {
    let stem = Path::new(&autoargs.case_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap();

    let root_dir = out_or_default(common.out);

    let case = cmtool_core::ensight_gold::case::Case::read(&autoargs.case_path)?;

    std::fs::create_dir_all(&root_dir).unwrap();

    let mut handle = cmtool_core::CMHandle::init(
        [common.n_i, common.n_j, common.n_k],
        &case.root,
        &case.geometry_file_path,
        cmtool_core::grid::MeshType::Cylindrical,
    )
    .unwrap();

    //Flow balancing keeps its defaults unless the command line says otherwise
    let mut balance = *handle.balance_settings();
    if let Some(tolerance) = common.balance_tolerance {
        balance.tolerance = tolerance;
    }
    if let Some(iterations) = common.balance_iterations {
        balance.max_iterations = iterations;
    }
    if let Some(max_divergence) = common.max_divergence {
        balance.max_divergence = max_divergence;
    }
    if common.no_balance {
        balance.do_balance = false;
    }
    handle.set_balance_settings(balance);

    handle
        .dump_all(format!("{}/{}", root_dir, stem), &case.root, &case.paths)
        .map_err(CmtoolError::Core)?;

    handle.dump_real_volume(format!("{}/{}/vofL", root_dir, stem))?;
    handle.dump_real_volume(format!("{}/{}/vtot", root_dir, stem))?;

    let _f = cmtool::check_flows(
        handle.grid(),
        &RawDataFlux::read_raw("./out/RESULTS/flowL.raw").unwrap(),
    )
    .unwrap();

    Ok(())
}
