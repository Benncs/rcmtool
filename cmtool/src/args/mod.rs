// SPDX-License-Identifier: GPL-3.0-or-later

use clap::{Parser, Subcommand};
#[derive(Parser, Clone)]
pub struct CommonArgs {
    pub n_i: usize,
    pub n_j: usize,
    pub n_k: usize,
    /// Verbosity level
    #[clap(short, long)]
    pub verbose: bool,

    /// Output directory
    #[clap(short, long)]
    pub out: Option<String>,

    /// Divergence the flow balancing aims for
    #[clap(long)]
    pub balance_tolerance: Option<f64>,

    /// Iterations the flow balancing may spend
    #[clap(long)]
    pub balance_iterations: Option<usize>,

    /// Divergence above which the generated flow map is rejected
    #[clap(long)]
    pub max_divergence: Option<f64>,
}

#[derive(Parser, Default, Clone)]
pub struct ManualArgs {
    /// Root directory of the CFD case
    pub root: String,

    /// Geometry file, relative to the root
    pub geo_file: String,

    /// Liquid velocity: either one vector file, or its three components as scalar files
    #[clap(long, value_delimiter = ',')]
    pub liquid: Vec<String>,

    /// Gas velocity: either one vector file, or its three components as scalar files
    #[clap(long, value_delimiter = ',')]
    pub gas: Vec<String>,

    /// Scalar file holding the gas volume fraction, used to split the phases
    #[clap(long)]
    pub gas_fraction: Option<String>,

    /// Scalar file to integrate over each compartment, repeatable
    #[clap(long = "scalar")]
    pub scalars: Vec<String>,

    /// Vector file to turn into a flow map, repeatable
    #[clap(long = "vector")]
    pub vectors: Vec<String>,
}

#[derive(Parser, Default, Clone)]
pub struct AutoArgs {
    pub case_path: String,
}

#[derive(Subcommand, Clone)]
pub enum Mode {
    Manual(ManualArgs),
    Auto(AutoArgs),
}

#[derive(Parser, Clone)]
pub struct CfdGenerate {
    #[clap(flatten)]
    pub common: CommonArgs,

    #[clap(subcommand)]
    pub mode: Mode,
}

#[derive(Parser, Clone)]
pub struct XMLGenerate {
    #[clap(short, long)]
    pub descriptor_path: String,
    #[clap(short, long)]
    pub out_dir: Option<String>,
}

#[derive(Subcommand, Clone)]
//Command line arguments, parsed once: the size of a variant does not matter here
#[allow(clippy::large_enum_variant)]
pub enum AllModes {
    Cfd(CfdGenerate),
    Xml(XMLGenerate),
}

#[derive(Parser, Clone)]
#[command(
    name = "CMTool",
    author,
    version,
    about = "Command line interface to generate Compartment Models",
    help_template = "\
{name} {version}

{about}

USAGE:
    {usage}

OPTIONS:
{options}

COMMANDS:
{subcommands}

By {author}
"
)]
pub struct GenArgs {
    #[clap(subcommand)]
    pub mode: AllModes,
}

impl GenArgs {
    // #[cfg(debug_assertions)]
    // pub fn get() -> Self {
    //     GenArgs {
    //         common: CommonArgs {
    //             n_i: 3,
    //             n_j: 3,
    //             n_k: 3,
    //             out: None,
    //             verbose: true,
    //         },
    //         mode: Mode::Auto(AutoArgs {
    //             case_path:
    //                 "/home/benjamin/Documents/thesis/cfd-cma/sanofi_cfd/inputs/RESULTS.encas"
    //                     .to_string(),
    //         }),
    //     }
    // }
    // #[cfg(not(debug_assertions))]
    pub fn get() -> Self {
        GenArgs::parse()
    }
}
