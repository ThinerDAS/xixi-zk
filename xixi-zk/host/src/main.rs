use std::fs;

use anyhow::{bail, Context, Result};
use bincode;
use methods::{XIXI_VERIFIER_ELF, XIXI_VERIFIER_ID};
use risc0_zkvm::{default_prover, ExecutorEnv, ProverOpts, Receipt};
use serde_json;
use sha2::{Sha256, Digest};
use xixi_core::{GameConfig, Output};

/// Wrapped receipt containing original user credential
#[derive(serde::Serialize, serde::Deserialize)]
struct WrappedReceipt {
    receipt: Receipt,
    user_cred: Vec<u8>,
}

/// Convert route JSON to compact byte format (little-endian u32)
fn route_to_bytes(path: &str) -> Result<Vec<u8>> {
    let bytes = fs::read(path)?;
    let route: Vec<u32> = serde_json::from_slice(&bytes)
        .or_else(|_| {
            std::str::from_utf8(&bytes)?
                .split_whitespace()
                .map(|s| s.parse().map_err(|e| anyhow::anyhow!("Invalid route value: {}", e)))
                .collect()
        })?;
    
    Ok(route.iter()
        .flat_map(|n| n.to_le_bytes().to_vec())
        .collect())
}

/// Convert JSON config to rkyv binary format
fn json_to_rkyv(json_path: &str, output_path: &str) -> Result<()> {
    let json_str = fs::read_to_string(json_path)?;
    let config = GameConfig::from_json(&json_str)?;
    let rkyv_bytes = config.to_rkyv();
    fs::write(output_path, rkyv_bytes)?;
    println!("Successfully converted {} to {}", json_path, output_path);
    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  Convert JSON to rkyv: {} convert <input.json> <output.rkyv>", args[0]);
        eprintln!("  Generate proof: {} prove <config.rkyv> <user_cred.txt> <route.json> <output.bin>", args[0]);
        eprintln!("  Verify proof:   {} verify <input.bin>", args[0]);
        std::process::exit(1);
    }

    match args[1].as_str() {
        "convert" => {
            if args.len() != 4 {
                eprintln!("Usage: {} convert <input.json> <output.rkyv>", args[0]);
                std::process::exit(1);
            }
            json_to_rkyv(&args[2], &args[3])?;
        }
        "prove" => {
            if args.len() != 6 {
                eprintln!("Usage: {} prove <config.rkyv> <user_cred.txt> <route.json> <output.bin>", args[0]);
                std::process::exit(1);
            }

            // Validate input sizes
            let config_bytes = fs::read(&args[2]).context("Failed to read config")?;
            if config_bytes.len() > 10_000_000 {
                bail!("Config file too large");
            }
            
            let user_cred = fs::read(&args[3]).context("Failed to read user cred")?;
            if user_cred.len() > 1_000_000 {
                bail!("User credential file too large");
            }
            let user_cred_hash = Sha256::digest(&user_cred);
            
            let route_bytes = route_to_bytes(&args[4])?;
            if route_bytes.len() > 1_000_000 {
                bail!("Route data too large");
            }

            let env = ExecutorEnv::builder()
                // Send config bytes with length prefix
                .write(&(config_bytes.len() as u32))?
                .write_slice(&config_bytes)
                // Send user cred hash (fixed size 32 bytes, no length prefix)
                .write_slice(&user_cred_hash)
                // Send route bytes with length prefix
                .write(&(route_bytes.len() as u32))?
                .write_slice(&route_bytes)
                .build()?;

            let prover = default_prover();
            let receipt = prover.prove_with_opts(env, XIXI_VERIFIER_ELF, &ProverOpts::succinct())?.receipt;

            // Decode and print guest output
            let output: Output = receipt.journal.decode()?;
            let output_json = serde_json::json!({
                "config_hash": hex::encode(output.config_hash),
                "user_cred_hash": hex::encode(output.user_cred_hash),
                "scores": output.scores
            });
            println!("Guest output:");
            println!("{}", serde_json::to_string_pretty(&output_json)?);

            // Wrap receipt with user credential
            let wrapped = WrappedReceipt { receipt, user_cred };
            fs::write(&args[5], bincode::serialize(&wrapped)?)?;
            println!("Proof written to: {}", args[5]);
        }
        "verify" => {
            if args.len() != 3 {
                eprintln!("Usage: {} verify <input.bin>", args[0]);
                std::process::exit(1);
            }

            let wrapped: WrappedReceipt = bincode::deserialize(&fs::read(&args[2])?)?;
            
            // 1. Verify receipt
            wrapped.receipt.verify(XIXI_VERIFIER_ID)?;
            
            // 2. Verify output contains expected hashes
            let output: Output = wrapped.receipt.journal.decode()?;
            
            // 3. Verify user credential hash matches
            let computed_user_hash = Sha256::digest(&wrapped.user_cred);
            if computed_user_hash != output.user_cred_hash.into() {
                anyhow::bail!("User credential hash mismatch");
            }
            
            // Output verification result as JSON
            let user_cred_str = String::from_utf8(wrapped.user_cred)
                .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in user credential: {}", e))?;
            
            let result = serde_json::json!({
                "status": "verified",
                "game": hex::encode(output.config_hash),
                "usercred": user_cred_str,
                "scores": output.scores
            });
            println!("{}", result.to_string());
        }
        _ => {
            eprintln!("Invalid command. Use 'convert', 'prove' or 'verify'");
            std::process::exit(1);
        }
    }

    Ok(())
}
