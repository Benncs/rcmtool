// SPDX-License-Identifier: GPL-3.0-or-later

use crate::fs::File;
use std::io::Write;
use std::{fs, path::PathBuf};
use xsd_parser::{
    Config, Error,
    config::{GeneratorFlags, InterpreterFlags, OptimizerFlags, ParserFlags, RenderStep, Schema},
    generate,
};

static ROOT: &str = "datamodel";
// static ROOT: &str = "./datamodel";

fn domain_schema() -> Result<(), Box<Error>> {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR variable not found");
    let path = PathBuf::from(manifest_dir).join(ROOT).join("main.xsd");
    let mut cfg = Config::default().with_schema(Schema::File(path));
    cfg = cfg.set_parser_flags(ParserFlags::RESOLVE_INCLUDES | ParserFlags::DEFAULT_NAMESPACES);
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

    cfg.generator.flags = GeneratorFlags::all()
        - GeneratorFlags::USE_MODULES
        - GeneratorFlags::USE_NAMESPACE_MODULES
        - GeneratorFlags::USE_SCHEMA_MODULES;

    let code = generate(cfg).expect("Failed to generate Rust code from XSD");
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let generated = out.join("generated_domain.rs");
    let mut file = File::create(generated).unwrap();

    //     file.write_all(
    //         b"
    // #![allow(clippy::all)]
    // #![allow(dead_code)]
    // #![allow(unused_imports)]
    // ",
    //     )
    //     .unwrap();

    file.write_all(code.to_string().as_bytes()).unwrap();
    Ok(())
}

fn main() -> Result<(), Box<Error>> {
    domain_schema()?;

    println!("cargo:rerun-if-changed=cmtool-assemble/build.rs");
    Ok(())
}
