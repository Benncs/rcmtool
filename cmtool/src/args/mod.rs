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
}

#[derive(Parser, Default, Clone)]
pub struct ManualArgs {
    pub root: String,
    pub geo_file: String,
    pub scalars: Vec<String>,
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
