// SPDX-License-Identifier: GPL-3.0-or-later

use cmtool::CmtoolError;
use cmtool_core::model::BalanceSettings;
use cmtool_data::{RawData, RawDataFlux};
// use std::fmt::Write;

// use std::fs;
use std::path::PathBuf;
use std::{env, path::Path};
mod args;
use args::*;
mod auto;
mod manual;
use auto::auto_main;

use crate::manual::manual_main;

fn out_or_default(out: Option<String>) -> String {
    out.unwrap_or(format!("{}/../out/", env!("CARGO_MANIFEST_DIR")))
}

///Balancing knobs, the command line only overrides what it was given
fn balance_settings(common: &CommonArgs, handle: &cmtool_core::CMHandle) -> BalanceSettings {
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
    balance
}

fn main() -> Result<(), CmtoolError> {
    let args = GenArgs::get();
    let mode = args.mode;
    match mode {
        AllModes::Cfd(cfdargs) => match cfdargs.mode {
            Mode::Auto(autoargs) => {
                if let Err(e) = auto_main(cfdargs.common, autoargs) {
                    eprintln!("{}", e);
                    return Err(e);
                }
                Ok(())
            }

            Mode::Manual(manual_args) => manual_main(cfdargs.common, manual_args),
        },
        AllModes::Xml(xml) => {
            let path = PathBuf::from(out_or_default(xml.out_dir));
            cmtool_assemble::headless_generate(&xml.descriptor_path, path)?;
            Ok(())
        }
    }
}
