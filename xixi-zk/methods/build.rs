use std::env;
use risc0_build::{DockerOptionsBuilder, GuestOptionsBuilder};

fn main() {
    // use docker to build the binary deterministically
    let build_mode = env::var("RISCV_BUILD_MODE").unwrap_or_else(|_| "docker".to_string());
    
    let guest_options = match build_mode.as_str() {
        "local" => GuestOptionsBuilder::default()
            .build()
            .expect("Failed to create guest options"),
        _ => {
            let docker_options = DockerOptionsBuilder::default()
                .root_dir("../../")
                .build()
                .expect("Failed to create docker options");
                
            GuestOptionsBuilder::default()
                .use_docker(docker_options)
                .build()
                .expect("Failed to create guest options")
        }
    };

    risc0_build::embed_methods_with_options(
        std::iter::once(("xixi_verifier", guest_options)).collect()
    );
}
