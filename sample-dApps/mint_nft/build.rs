use aiken_project::{config::Config, Project};
use std::env;

const PROJECT: &str = "./mint_nft";

fn main() {
    let config = Config::load(PROJECT.into()).unwrap();
    let mut project = Project::new(config, PROJECT.into());
    let build_result = project.build(false);

    if let Err(err) = build_result {
        err.report();
        panic!("🍂 Failed to build Aiken code 🍂");
    }
}
