use std::fmt::Write;

///Symmetric mean absolute percentage error
///
/// Details : https://en.wikipedia.org/wiki/Symmetric_mean_absolute_percentage_error
fn smape(a: f64, b: f64) -> f64 {
    if a == 0. && b == 0. {
        return 0.;
    }
    (a - b).abs() / (a.abs() + b.abs())
}

pub fn check_flows(raw_flows: &cmtool_data::RawDataFlux) -> Option<String> {
    let mut mass_balance: Vec<(f64, f64)> = vec![(0.0, 0.0); raw_flows.header.n_zone as usize];

    for flow in raw_flows.fluxes.iter() {
        mass_balance[flow.id_target as usize].0 += flow.flux_source_target;
        mass_balance[flow.id_source as usize].1 += flow.flux_source_target;
    }

    let mut zone_relative_errors = Vec::with_capacity(raw_flows.header.n_zone as usize);
    let mut total_relative_error: f64 = 0.0;
    let mut max_relative_error: f64 = 0.0;

    let mut total_inflow = 0.0;
    let mut total_outflow = 0.0;
    let mut f = String::new();

    writeln!(&mut f, "zone_id,int,out,relative_error_percent").unwrap();
    for (i, (inflow, outflow)) in mass_balance.iter().enumerate() {
        let relative_error = smape(*inflow, *outflow);

        zone_relative_errors.push(relative_error);
        total_relative_error += relative_error;
        max_relative_error = max_relative_error.max(relative_error);

        total_inflow += inflow;
        total_outflow += outflow;

        writeln!(
            &mut f,
            "{},{},{},{}",
            i + 1,
            inflow,
            outflow,
            relative_error * 100.0
        )
        .unwrap();
    }
    writeln!(&mut f).unwrap();
    let mean_relative_error = total_relative_error / raw_flows.header.n_zone as f64;

    writeln!(&mut f, "metric,value").unwrap();
    writeln!(&mut f, "total_inflow,{:.6}", total_inflow).unwrap();
    writeln!(&mut f, "total_outflow,{:.6}", total_outflow).unwrap();
    writeln!(
        &mut f,
        "global_net_error_percent,{:.6}",
        smape(total_inflow, total_outflow) * 100.0
    )
    .unwrap();
    writeln!(
        &mut f,
        "mean_relative_error_percent,{:.6}",
        mean_relative_error * 100.0
    )
    .unwrap();
    writeln!(
        &mut f,
        "max_relative_error_percent,{:.6}",
        max_relative_error * 100.0
    )
    .unwrap();

    Some(f)
}
