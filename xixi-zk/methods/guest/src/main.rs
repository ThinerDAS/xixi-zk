//#![no_std]

/*
#![no_main]
risc0_zkvm::guest::entry!(main);
*/

use risc0_zkvm::guest::env;
use xixi_verifier::verifier;
use anyhow::{bail, Result};
use sha2::{Sha256, Digest};
use xixi_verifier::model::Output;

// Configurable buffer size in MB - adjust based on actual config requirements
const BUFFER_SIZE_MB: usize = 1; // Reduce from 16MB to 1MB by default
const BUFFER_SIZE: usize = BUFFER_SIZE_MB * 1024 * 1024;

// Static global buffer for configuration data
//
// Size: CONFIG_BUFFER_SIZE_MB * 1024 * 1024 bytes
#[repr(align(4096))]
struct AlignedBuffer {
    data: [u8; BUFFER_SIZE]
}

static mut CONFIG_BUFFER_BUF: AlignedBuffer = AlignedBuffer {
    data: [0u8; BUFFER_SIZE]
};

static mut ROUTE_BUFFER_BUF: AlignedBuffer = AlignedBuffer {
    data: [0u8; BUFFER_SIZE]
};

fn read_input() -> Result<(&'static [u8], [u8; 32], [u8; 32], &'static [u8])> {
    // Read config bytes with length prefix into static buffer
    let config_len: u32 = env::read();
    if config_len > BUFFER_SIZE.try_into().unwrap() {
        bail!("Config length exceeds size limit");
    }
    
    // Use static buffer instead of dynamic AlignedVec
    let config_len = config_len as usize;
    let config_bytes = unsafe {
        env::read_slice(&mut CONFIG_BUFFER_BUF.data[..config_len]);
        &CONFIG_BUFFER_BUF.data[..config_len]
    };

    // Calculate config hash immediately after reading config bytes
    let config_hash = Sha256::digest(config_bytes).into();

    // Read user credential hash (fixed 32 bytes)
    let mut user_cred_hash = [0u8; 32];
    env::read_slice(&mut user_cred_hash);

    // Read route bytes with length prefix
    let route_bytes_len: u32 = env::read();
    if route_bytes_len > BUFFER_SIZE.try_into().unwrap() { // 16MB limit
        bail!("Route length exceeds size limit");
    }
    let route_bytes_len = route_bytes_len as usize;
    let route_bytes = unsafe {
        env::read_slice(&mut ROUTE_BUFFER_BUF.data[..route_bytes_len]);
        &ROUTE_BUFFER_BUF.data[..route_bytes_len]
    };
    
    Ok((config_bytes, config_hash, user_cred_hash, route_bytes))
}

fn main() {
    let (config_bytes, config_hash, user_cred_hash, route_bytes) = read_input()
        .expect("Failed to read input");

    // Call verifier to parse route, simulate game and get scores
    let scores = verifier::do_main(config_bytes, route_bytes)
        .expect("Verification failed");

    // Assemble final output structure here
    let output = Output {
        config_hash,
        user_cred_hash,
        scores,
    };

    // Commit the full output structure
    env::commit(&output);
}
