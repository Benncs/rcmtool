// SPDX-License-Identifier: GPL-3.0-or-later

use crate::fs::File;
use std::fs;
use std::io::Write;
use xsd_parser::{
    Config, Error,
    config::{GeneratorFlags, InterpreterFlags, OptimizerFlags, RenderStep, Schema},
    generate,
};

static ROOT: &str = "./datamodel";

fn domain_schema() -> Result<(), Error> {
    let files = [
        format!("{}/units.xsd", ROOT),
        format!("{}/reactors.xsd", ROOT),
        format!("{}/connections.xsd", ROOT),
        format!("{}/main.xsd", ROOT),
    ];

    let mut cfg = Config::default();
    cfg.parser.schemas = files
        .into_iter()
        .map(|f| {
            println!("cargo:rerun-if-changed={}", f);
            Schema::File(f.into())
        })
        .collect();
    cfg = cfg.with_render_steps([
        //RenderStep::Types,
        RenderStep::Defaults,
        RenderStep::TypesSerdeXmlRs {
            version: xsd_parser::config::SerdeXmlRsVersion::Version08AndAbove,
        },
        // RenderStep::NamespaceConstants,

        // RenderStep::QuickXmlDeserialize {
        //     boxed_deserializer: false,
        // },
        // RenderStep::TypesSerdeQuickXml,
    ]);

    cfg = cfg.with_derive(["Debug", "Clone"]);
    cfg.interpreter.flags = InterpreterFlags::all()
        - InterpreterFlags::WITH_NUM_BIG_INT
        - InterpreterFlags::WITH_XS_ANY_TYPE;
    cfg.optimizer.flags = OptimizerFlags::all();
    cfg.generator.flags.insert(GeneratorFlags::all());
    let code = generate(cfg).expect("Failed to generate Rust code from XSD");
    let mut file = File::create("src/parser/generated_domain.rs")?;
    file.write_all(code.to_string().as_bytes())?;
    Ok(())
}

fn main() -> Result<(), Error> {
    domain_schema()?;
    println!("cargo:rerun-if-changed=cmtool-assemble/build.rs");
    Ok(())
}
