use ethers_solc::{Project, ProjectPathsConfig};

fn main() {
    if cfg!(feature = "solidity_compile") {
        // configure the project with all its paths, solc, cache etc.
        let project = Project::builder()
            .paths(
                ProjectPathsConfig::hardhat(env!("CARGO_MANIFEST_DIR"))
                    .expect("failed to create hardhat config"),
            )
            .build()
            .expect("failed to build project");
        let output = project.compile().expect("failed to compile project");

        if output.has_compiler_errors() {
            panic!("{}", format!("{:?}", output.output().errors));
        }

        // Tell Cargo that if a source file changes, to rerun this build script.
        project.rerun_if_sources_changed();
    }
}
