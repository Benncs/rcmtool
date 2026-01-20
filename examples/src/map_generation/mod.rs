use std::path::PathBuf;

pub fn generate(reactor_input_file_name: &str) -> bool {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../out/examples/");
    if let Err(msg) = cmtool_assemble::headless_generate(reactor_input_file_name, path) {
        eprintln!("{}", msg);
        return false;
    }
    return true;
}

#[cfg(test)]
mod domain_integration_test {
    use super::*;

    fn common_test(name: &str) {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../examples/data/{}/reactors.xml", name));

        assert!(generate(path.to_str().unwrap()));
    }

    #[test]
    fn case_0d1d() {
        common_test("case_0d1d");
    }
    #[test]
    fn fcase_0d() {
        common_test("simple_0d");
    }
    #[test]
    fn simple_0d1d() {
        common_test("simple_0d1d");
    }
    #[test]
    fn case_0d1d_liq() {
        common_test("case_0d1d_liq");
    }
}
