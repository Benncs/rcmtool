use std::path::PathBuf;

pub fn generate(reactor_input_file_name: &str) -> bool {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../out/examples/");
    let contents = std::fs::read_to_string(reactor_input_file_name).expect("getcontent");
    let root_dir = path.to_str().unwrap().to_owned();
    std::fs::create_dir_all(&root_dir).expect("mkdir");

    if let Ok(mut domain) = cmtool_assemble::generate_domain(&root_dir, &contents) {
        println!("OK");
        return true;
    }

    eprintln!("Error domain");
    false
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
